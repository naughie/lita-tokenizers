//! The implementation of [`lita-tokenizers-core`](`tokenizers_core`) for [`mecab-sys`](`mecab_sys`).
//!
//! Note that this is very the fundamental and low-level APIs: in most cases you shold prefer the `lita-tokenizers` crate instead.

use tokenizers_core::{Tag, Token, TokenPosition, TokenStream, TokenStreamBuilder, Tokenizer};

pub use mecab_sys as sys;
use mecab_sys::Error;
use mecab_sys::{Lattice, LatticeGuard, Model, Tagger};
use mecab_sys::{Node, NodeCursor};

use std::cmp::Ordering;
use std::ffi::CStr;
use std::ops::Range;
use std::slice::Iter as SliceIter;

pub struct MeCab<O = Noop> {
    model: Model,
    ord: O,
}

impl<O> MeCab<O> {
    pub fn new(model: Model, ord: O) -> Self {
        Self { model, ord }
    }

    pub fn from_cli_args(args: &CStr, ord: O) -> Result<Self, Error> {
        let model = Model::from_cli_arg(args)?;
        Ok(Self { model, ord })
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }
}

impl<O: TagOrder> Tokenizer for MeCab<O> {
    type StreamBuilder<'a>
        = MeCabTagger<'a, O>
    where
        Self: 'a;
    type Error = Error;

    fn builder<'a>(&'a mut self) -> Result<Self::StreamBuilder<'a>, Self::Error> {
        let tagger = self.model.new_tagger()?;
        let lattice = self.model.new_lattice()?;
        Ok(MeCabTagger {
            tagger,
            lattice,
            ord: &self.ord,
        })
    }
}

pub struct MeCabTagger<'a, O = Noop> {
    tagger: Tagger<'a>,
    lattice: Lattice<'a>,
    ord: &'a O,
}

impl<'a, O: TagOrder> TokenStreamBuilder for MeCabTagger<'a, O> {
    type Output<'t, 's>
        = MeCabStream<'a, 't, 's, O>
    where
        Self: 't;
    type Error = Error;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error> {
        let mut lattice = self.lattice.set_sentence(input);
        self.tagger.parse(&mut lattice)?;
        Ok(MeCabStream {
            lattice,
            ord: self.ord,
        })
    }
}

pub struct MeCabStream<'a, 'l, 's, O = Noop> {
    lattice: LatticeGuard<'a, 'l, 's>,
    ord: &'a O,
}

impl<'a, 'l, 's, O: TagOrder> TokenStream for MeCabStream<'a, 'l, 's, O> {
    type IntoIter<'o>
        = MeCabTokenIter<'o, O>
    where
        Self: 'o;
    type Token<'o>
        = MeCabToken<'o, O>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o> {
        MeCabTokenIter {
            cursor: self.lattice.bos_node(),
            ord: self.ord,
        }
    }
}

pub struct MeCabTokenIter<'o, O = Noop> {
    cursor: NodeCursor<'o>,
    ord: &'o O,
}

impl<'o, O: TagOrder> Iterator for MeCabTokenIter<'o, O> {
    type Item = (TokenPosition, MeCabToken<'o, O>);

    fn next(&mut self) -> Option<Self::Item> {
        use tokenizers_utils::str::matches_eos_marker;

        self.cursor.move_next();
        let node = self.cursor.curr()?;
        if node.kind().is_eos() {
            return None;
        }

        let position = if matches_eos_marker(node.surface())
            || node.next().is_none_or(|next| next.kind().is_eos())
        {
            TokenPosition::Eos
        } else {
            TokenPosition::Normal
        };

        Some((
            position,
            MeCabToken {
                node,
                ord: self.ord,
            },
        ))
    }
}

pub struct MeCabToken<'o, O = Noop> {
    node: Node<'o>,
    ord: &'o O,
}

pub struct MeCabTags<'o> {
    tags: Option<&'o str>,
}

impl<'o, O: TagOrder> Token<'o> for MeCabToken<'o, O> {
    type Tags = O::Tags<'o, 'o>;

    fn surface(&self) -> Tag<'o> {
        Tag::Unescaped(self.node.surface())
    }

    fn tags(&self) -> Self::Tags {
        self.ord.reorder(MeCabTags {
            tags: Some(self.node.feature()),
        })
    }
}

impl<'o> Iterator for MeCabTags<'o> {
    type Item = Tag<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        let tags = self.tags?;

        if let Some(idx) = tags.find(',') {
            self.tags = Some(&tags[(idx + 1)..]);
            Some(Tag::Unescaped(&tags[..idx]))
        } else {
            self.tags = None;
            Some(Tag::Unescaped(tags))
        }
    }
}

pub trait TagOrder {
    type Tags<'o, 't>: Iterator<Item = Tag<'t>>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: MeCabTags<'t>) -> Self::Tags<'o, 't>;
}

#[derive(Debug, Clone, Copy)]
pub struct Noop;

impl TagOrder for Noop {
    type Tags<'o, 't>
        = MeCabTags<'t>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: MeCabTags<'t>) -> Self::Tags<'o, 't> {
        tags
    }
}

pub struct Selected<'a> {
    pub index: &'a [usize],
}

pub struct SelectedTags<'o, 't> {
    ord: SliceIter<'o, usize>,
    tags: &'t str,
    curr: usize,
    curr_span: Range<usize>,
}

impl TagOrder for Selected<'_> {
    type Tags<'o, 't>
        = SelectedTags<'o, 't>
    where
        Self: 'o;

    fn reorder<'o, 't>(&'o self, tags: MeCabTags<'t>) -> Self::Tags<'o, 't> {
        let tags = tags.tags.unwrap_or_default();
        let end = tags.find(',').unwrap_or(tags.len());

        SelectedTags {
            ord: self.index.iter(),
            tags,
            curr: 0,
            curr_span: 0..end,
        }
    }
}

impl<'t> Iterator for SelectedTags<'_, 't> {
    type Item = Tag<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = *self.ord.next()?;

        match idx.cmp(&self.curr) {
            Ordering::Equal => {}
            Ordering::Less => {
                for _ in idx..self.curr {
                    let end = self.curr_span.start - 1;
                    let start = if let Some(idx) = self.tags[..end].rfind(',') {
                        idx + 1
                    } else {
                        0
                    };
                    self.curr_span = start..end;
                    self.curr -= 1;
                }
            }
            Ordering::Greater => {
                for _ in self.curr..idx {
                    if self.curr_span.end == self.tags.len() {
                        return Some(Tag::Unescaped(""));
                    }

                    let start = self.curr_span.end + 1;
                    let end = if let Some(idx) = self.tags[start..].find(',') {
                        start + idx
                    } else {
                        self.tags.len()
                    };
                    self.curr_span = start..end;
                    self.curr += 1;
                }
            }
        }

        Some(Tag::Unescaped(&self.tags[self.curr_span.clone()]))
    }
}
