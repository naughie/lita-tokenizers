//! The implementation of [`lita-tokenizers-core`](`tokenizers_core`) for the whitespace tokenizer.
//!
//! Note that this is very the fundamental and low-level APIs: in most cases you shold prefer the `lita-tokenizers` crate instead.

use tokenizers_core::{Tag, Token, TokenPosition, TokenStream, TokenStreamBuilder, Tokenizer};

use std::convert::Infallible;
use std::iter::Peekable;
use std::str::Lines;
use std::str::Split;
use std::str::SplitWhitespace;

pub struct Whitespace;

impl Tokenizer for Whitespace {
    type StreamBuilder<'t>
        = Self
    where
        Self: 't;
    type Error = Infallible;

    fn builder<'t>(&'t mut self) -> Result<Self::StreamBuilder<'t>, Self::Error> {
        Ok(Self)
    }
}

impl TokenStreamBuilder for Whitespace {
    type Output<'t, 's>
        = WhitespaceStream<'s>
    where
        Self: 't;
    type Error = Infallible;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error> {
        Ok(WhitespaceStream {
            input: input.trim(),
        })
    }
}

pub struct WhitespaceStream<'s> {
    input: &'s str,
}

impl<'s> TokenStream for WhitespaceStream<'s> {
    type IntoIter<'o>
        = WhitespaceIter<'o>
    where
        Self: 'o;
    type Token<'o>
        = WhitespaceToken<'o>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o> {
        let haystack = self.input;
        let mut lines = haystack.lines();

        let curr_line =
            first_non_whitespace_line(&mut lines).map(|s| s.split_whitespace().peekable());

        WhitespaceIter {
            curr_line,
            rest_lines: lines,
        }
    }
}

fn first_non_whitespace_line<'s>(lines: &mut Lines<'s>) -> Option<&'s str> {
    lines.find(|line| !line.trim_start().is_empty())
}

pub struct WhitespaceIter<'s> {
    curr_line: Option<Peekable<SplitWhitespace<'s>>>,
    rest_lines: Lines<'s>,
}

impl<'s> WhitespaceIter<'s> {
    fn next_token_on_curr_line(&mut self) -> Option<(TokenPosition, WhitespaceToken<'s>)> {
        use tokenizers_utils::str::ends_with_eos_marker;

        let curr_line = self.curr_line.as_mut()?;

        curr_line.next().map(|token| {
            let position = if ends_with_eos_marker(token) || curr_line.peek().is_none() {
                TokenPosition::Eos
            } else {
                TokenPosition::Normal
            };

            (position, WhitespaceToken::from_token(token))
        })
    }
}

impl<'o> Iterator for WhitespaceIter<'o> {
    type Item = (TokenPosition, WhitespaceToken<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ret) = self.next_token_on_curr_line() {
            return Some(ret);
        }

        let line = first_non_whitespace_line(&mut self.rest_lines);

        self.curr_line = line.map(|s| s.split_whitespace().peekable());

        self.next_token_on_curr_line()
    }
}

pub struct WhitespaceToken<'s> {
    surface: &'s str,
    tags: Option<&'s str>,
}

pub struct WhitespaceTags<'o> {
    tags: Split<'o, char>,
}

impl<'s> WhitespaceToken<'s> {
    fn from_token(token: &'s str) -> Self {
        if let Some((surface, tags)) = token.split_once('/') {
            Self {
                surface,
                tags: Some(tags),
            }
        } else {
            Self {
                surface: token,
                tags: None,
            }
        }
    }
}

impl<'o> Token<'o> for WhitespaceToken<'o> {
    type Tags = WhitespaceTags<'o>;

    fn surface(&self) -> Tag<'o> {
        Tag::Unescaped(self.surface)
    }

    fn tags(&self) -> Self::Tags {
        let mut tags = self.tags.unwrap_or_default().split('/');
        if self.tags.is_none() {
            for _ in tags.by_ref() {}
        }
        WhitespaceTags { tags }
    }
}

impl<'o> Iterator for WhitespaceTags<'o> {
    type Item = Tag<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tags.next().map(Tag::Unescaped)
    }
}
