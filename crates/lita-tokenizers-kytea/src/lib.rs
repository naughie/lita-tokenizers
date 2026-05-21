//! The implementation of [`lita-tokenizers-core`](`tokenizers_core`) for [`kytea-sys`](`kytea_sys`).
//!
//! Note that this is very the fundamental and low-level APIs: in most cases you shold prefer the `lita-tokenizers` crate instead.

use tokenizers_core::delim;
use tokenizers_core::{Tag, Token, TokenPosition, TokenStream, TokenStreamBuilder, Tokenizer};

pub use kytea_sys as sys;
use kytea_sys::DebugLevel;
use kytea_sys::TrainConfigError;
use kytea_sys::{CorpusFormat, KyTea as Model, StringStream};

use escaped_delimiter::Cursor as EscapedCursor;
use escaped_delimiter::CursorPosition;
use escaped_delimiter::Iter as EscapedIter;

use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::iter::Peekable;
use std::slice::Iter as SliceIter;
use std::str::Lines;

#[derive(Debug)]
pub enum Error {
    InvalidConfig(TrainConfigError),
    Predict(IoError),
    InvalidUtf8Output(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(TrainConfigError::DoNothing) => {
                write!(
                    f,
                    "invalid config: neither word segmentation nor tag prediction are enabled"
                )
            }
            Self::InvalidConfig(TrainConfigError::RawWithoutWordSegmentation) => {
                write!(f, "invalid config: word segmentation is disabled")
            }
            Self::InvalidConfig(TrainConfigError::NoWordSegmentationModel) => {
                write!(f, "invalid config: no model has been loaded")
            }
            Self::Predict(e) => {
                write!(f, "error on prediction: {e}")
            }
            Self::InvalidUtf8Output(e) => {
                write!(f, "output is invalid UTF-8: {e:?}")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Predict(e) => Some(e),
            _ => None,
        }
    }
}

pub struct KyTea<O = Noop> {
    model: Model,
    ord: O,
}

impl<O> KyTea<O> {
    pub fn new(ord: O) -> Self {
        Self {
            model: Model::new(),
            ord,
        }
    }

    pub fn model(&mut self) -> &mut Model {
        &mut self.model
    }
}

impl<O: TagOrder> Tokenizer for KyTea<O> {
    type StreamBuilder<'t>
        = KyTeaStreamBuilder<'t, O>
    where
        Self: 't;
    type Error = Error;

    fn builder<'t>(&'t mut self) -> Result<Self::StreamBuilder<'t>, Self::Error> {
        self.model
            .config()
            .set_debug(DebugLevel::Silent)
            .set_training(false)
            .set_word_bound(delim::TOKEN_CSTR)
            .set_tag_bound(delim::TAG_CSTR)
            .set_escape(delim::ESCAPE_CSTR)
            // elem bound will be escaped too, so we have to redirect to delim::*
            .set_elem_bound(delim::TOKEN_CSTR)
            .set_input_format(CorpusFormat::Raw);

        Ok(KyTeaStreamBuilder {
            model: &mut self.model,
            ord: &self.ord,
        })
    }
}

pub struct KyTeaStreamBuilder<'t, O = Noop> {
    model: &'t mut Model,
    ord: &'t O,
}

impl<'a, O: TagOrder> TokenStreamBuilder for KyTeaStreamBuilder<'a, O> {
    type Output<'t, 's>
        = KyTeaStream<'t, O>
    where
        'a: 't;
    type Error = Error;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error> {
        let mut input_stream = StringStream::new();
        input_stream.push(input);
        if !input.ends_with('\n') {
            input_stream.push("\n");
        }

        let mut output = StringStream::new();

        let mut ctx = self
            .model
            .context(&mut input_stream, &mut output)
            .map_err(Error::InvalidConfig)?;
        while ctx.predict().map_err(Error::Predict)?.is_continue() {}

        let bytes = output.as_bytes();
        if simdutf8::basic::from_utf8(bytes).is_ok() {
            Ok(KyTeaStream {
                output,
                ord: self.ord,
            })
        } else {
            Err(Error::InvalidUtf8Output(
                String::from_utf8_lossy(bytes).into_owned(),
            ))
        }
    }
}

pub struct KyTeaStream<'t, O = Noop> {
    output: StringStream,
    ord: &'t O,
}

impl<'t, O: TagOrder> TokenStream for KyTeaStream<'t, O> {
    type IntoIter<'o>
        = KyTeaTokenIter<'o, O>
    where
        Self: 'o;
    type Token<'o>
        = KyTeaToken<'o, O>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o> {
        // SAFETY: validated in `KyTeaStreamBuilder::tokenize()`
        let output = unsafe { std::str::from_utf8_unchecked(self.output.as_bytes()) };
        let mut lines = output.lines();

        let curr_line = first_non_whitespace_line(&mut lines)
            .map(|s| escaped_delimiter::iter(s.as_bytes(), delim::TOKEN, delim::ESCAPE).peekable());

        KyTeaTokenIter {
            curr_line,
            rest_lines: lines,
            ord: self.ord,
        }
    }
}

fn first_non_whitespace_line<'s>(lines: &mut Lines<'s>) -> Option<&'s str> {
    lines.find(|line| !line.trim_start().is_empty())
}

pub struct KyTeaTokenIter<'o, O = Noop> {
    curr_line: Option<Peekable<EscapedIter<'o>>>,
    rest_lines: Lines<'o>,
    ord: &'o O,
}

impl<'o, O: TagOrder> KyTeaTokenIter<'o, O> {
    fn next_token_on_curr_line(&mut self) -> Option<(TokenPosition, KyTeaToken<'o, O>)> {
        use tokenizers_utils::str::matches_eos_marker;

        let curr_line = self.curr_line.as_mut()?;

        curr_line.next().map(|token| {
            let token = unsafe { std::str::from_utf8_unchecked(token) };

            let next = curr_line.peek();
            let mut position = if next.is_some() {
                TokenPosition::Normal
            } else {
                TokenPosition::Eos
            };

            let token = KyTeaToken::from_token(token, self.ord);

            if position == TokenPosition::Normal && matches_eos_marker(token.surface) {
                position = TokenPosition::Eos;
            }

            (position, token)
        })
    }
}

impl<'o, O: TagOrder> Iterator for KyTeaTokenIter<'o, O> {
    type Item = (TokenPosition, KyTeaToken<'o, O>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ret) = self.next_token_on_curr_line() {
            return Some(ret);
        }

        let line = first_non_whitespace_line(&mut self.rest_lines);
        self.curr_line = line
            .map(|s| escaped_delimiter::iter(s.as_bytes(), delim::TOKEN, delim::ESCAPE).peekable());

        self.next_token_on_curr_line()
    }
}

pub struct KyTeaToken<'o, O = Noop> {
    surface: &'o str,
    tags: &'o str,
    ord: &'o O,
}

impl<'o, O> KyTeaToken<'o, O> {
    fn from_token(token: &'o str, ord: &'o O) -> Self {
        let mut it = escaped_delimiter::iter(token.as_bytes(), delim::TAG, delim::ESCAPE);

        let surface = it.next();
        let surface = unsafe { std::str::from_utf8_unchecked(surface.unwrap_or(token.as_bytes())) };

        let tags = unsafe { std::str::from_utf8_unchecked(it.as_slice().unwrap_or_default()) };

        Self { surface, tags, ord }
    }
}

pub struct KyTeaTags<'o> {
    tags: EscapedCursor<'o>,
}

impl<'o, O: TagOrder> Token<'o> for KyTeaToken<'o, O> {
    type Tags = O::Tags<'o, 'o>;

    fn surface(&self) -> Tag<'o> {
        Tag::Escaped(self.surface)
    }

    fn tags(&self) -> Self::Tags {
        self.ord.reorder(KyTeaTags {
            tags: escaped_delimiter::cursor(self.tags.as_bytes(), delim::TAG, delim::ESCAPE),
        })
    }
}

impl<'o> Iterator for KyTeaTags<'o> {
    type Item = Tag<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        if matches!(
            self.tags.position(),
            CursorPosition::Empty | CursorPosition::End
        ) {
            return None;
        }

        let tag = new_tag(self.tags.curr());
        self.tags.move_next();

        Some(tag)
    }
}

pub trait TagOrder {
    type Tags<'o, 't>: Iterator<Item = Tag<'t>>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: KyTeaTags<'t>) -> Self::Tags<'o, 't>;
}

#[derive(Debug, Clone, Copy)]
pub struct Noop;

impl TagOrder for Noop {
    type Tags<'o, 't>
        = KyTeaTags<'t>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: KyTeaTags<'t>) -> Self::Tags<'o, 't> {
        tags
    }
}

pub struct Selected<'a> {
    pub index: &'a [usize],
}

pub struct SelectedTags<'o, 't> {
    ord: SliceIter<'o, usize>,
    tags: EscapedCursor<'t>,
    curr: usize,
}

impl TagOrder for Selected<'_> {
    type Tags<'o, 't>
        = SelectedTags<'o, 't>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: KyTeaTags<'t>) -> Self::Tags<'o, 't> {
        SelectedTags {
            ord: self.index.iter(),
            tags: tags.tags,
            curr: 0,
        }
    }
}

impl<'t> Iterator for SelectedTags<'_, 't> {
    type Item = Tag<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Ordering;

        let idx = *self.ord.next()?;

        if self.tags.position() == CursorPosition::Empty {
            return Some(new_tag(b""));
        }

        match idx.cmp(&self.curr) {
            Ordering::Equal => {}
            Ordering::Less => {
                for _ in idx..self.curr {
                    self.tags.move_prev();
                    self.curr -= 1;
                }
            }
            Ordering::Greater => {
                if self.tags.position() == CursorPosition::Middle {
                    for _ in self.curr..idx {
                        self.tags.move_next();
                        self.curr += 1;
                        if self.tags.position() != CursorPosition::Middle {
                            break;
                        }
                    }
                }
            }
        }
        Some(new_tag(self.tags.curr()))
    }
}

fn new_tag(tag: &[u8]) -> Tag<'_> {
    Tag::Escaped(unsafe { std::str::from_utf8_unchecked(tag) })
}
