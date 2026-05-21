//! [`Tokenizer`] implementation that does nothing: takes an input string and merely parses it by
//! [`delim`](tokenizers_core::delim)iters.
//!
//! Note that it does not recognize the newline characters (`\n`).
//!
//! This tokenizer is just for testing.

use tokenizers_core::delim;
use tokenizers_core::{Tag, Token, TokenPosition, TokenStream, TokenStreamBuilder, Tokenizer};

use std::convert::Infallible;

#[derive(Debug, Clone, Copy)]
pub struct NoopTokenizer;

impl Tokenizer for NoopTokenizer {
    type StreamBuilder<'t>
        = Self
    where
        Self: 't;
    type Error = Infallible;

    fn builder<'t>(&'t mut self) -> Result<Self::StreamBuilder<'t>, Self::Error> {
        Ok(Self)
    }
}

impl TokenStreamBuilder for NoopTokenizer {
    type Output<'t, 's>
        = NoopStream<'s>
    where
        Self: 't;
    type Error = Infallible;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error> {
        Ok(NoopStream { input })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoopStream<'s> {
    pub input: &'s str,
}

impl<'s> TokenStream for NoopStream<'s> {
    type IntoIter<'o>
        = NoopTokenIter<'o>
    where
        Self: 'o;
    type Token<'o>
        = NoopToken<'o>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o> {
        NoopTokenIter {
            haystack: Some(self.input),
            start: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoopTokenIter<'s> {
    haystack: Option<&'s str>,
    start: usize,
}

impl<'o> Iterator for NoopTokenIter<'o> {
    type Item = (TokenPosition, NoopToken<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        use crate::str::ends_with_eos_marker;

        let haystack = *self.haystack.as_ref()?;
        let start = self.start;

        let token = if let Some(idx) = memchr::memchr(delim::TOKEN, &haystack.as_bytes()[start..]) {
            let idx = start + idx;
            self.start = idx + 1;
            &haystack[start..idx]
        } else {
            self.haystack = None;
            &haystack[start..]
        };

        let position = if self.haystack.is_none() || ends_with_eos_marker(token) {
            TokenPosition::Eos
        } else {
            TokenPosition::Normal
        };

        let token = memchr::memchr(delim::TAG, token.as_bytes())
            .map(|idx| NoopToken {
                surface: &token[..idx],
                tags: Some(&token[(idx + 1)..]),
            })
            .unwrap_or_else(|| NoopToken {
                surface: token,
                tags: None,
            });

        Some((position, token))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoopToken<'s> {
    pub surface: &'s str,
    pub tags: Option<&'s str>,
}

#[derive(Debug, Clone)]
pub struct NoopTags<'s> {
    haystack: Option<&'s str>,
    start: usize,
}

impl<'o> Token<'o> for NoopToken<'o> {
    type Tags = NoopTags<'o>;

    fn surface(&self) -> Tag<'o> {
        Tag::Unescaped(self.surface)
    }

    fn tags(&self) -> Self::Tags {
        NoopTags {
            haystack: self.tags,
            // delim_it: memchr_iter(delim::TAG, haystack.as_bytes()),
            start: 0,
        }
    }
}

impl<'o> Iterator for NoopTags<'o> {
    type Item = Tag<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        let haystack = *self.haystack.as_ref()?;
        let start = self.start;

        let tag = if let Some(idx) = memchr::memchr(delim::TAG, &haystack.as_bytes()[start..]) {
            let idx = start + idx;
            self.start = idx + 1;
            &haystack[start..idx]
        } else {
            self.haystack = None;
            &haystack[start..]
        };

        Some(Tag::Unescaped(tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_tokenizer() {
        fn test_impl<I, J>(input: &str, expected: I)
        where
            I: IntoIterator<Item = (&'static str, J)>,
            J: IntoIterator<Item = &'static str>,
        {
            let stream = TokenStreamBuilder::tokenize(&mut NoopTokenizer, input).unwrap();
            let tokens = stream.tokens();

            let found = tokens
                .map(|(pos, token)| {
                    let Tag::Unescaped(surface) = token.surface() else {
                        unreachable!();
                    };
                    let surface = match pos {
                        TokenPosition::Normal => format!("{surface}|n"),
                        TokenPosition::Eos => format!("{surface}|e"),
                    };
                    let tags = token
                        .tags()
                        .map(|tag| {
                            let Tag::Unescaped(tag) = tag else {
                                unreachable!();
                            };
                            tag.to_owned()
                        })
                        .collect::<Vec<_>>();
                    (surface, tags)
                })
                .collect::<Vec<_>>();

            let expected = expected
                .into_iter()
                .map(|(surface, tags)| {
                    (
                        surface.to_owned(),
                        tags.into_iter().map(String::from).collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();

            assert_eq!(found, expected);
        }

        const NO_TAG: [&str; 0] = [];

        test_impl("", [("|e", NO_TAG)]);
        test_impl("abc", [("abc|e", NO_TAG)]);
        test_impl("abc.", [("abc.|e", NO_TAG)]);

        test_impl(
            "abc\tdef.\tghi.\tjkl\tmno\tpqr\tstu.\twxy\tz",
            [
                ("abc|n", NO_TAG),
                ("def.|e", NO_TAG),
                ("ghi.|e", NO_TAG),
                ("jkl|n", NO_TAG),
                ("mno|n", NO_TAG),
                ("pqr|n", NO_TAG),
                ("stu.|e", NO_TAG),
                ("wxy|n", NO_TAG),
                ("z|e", NO_TAG),
            ],
        );
        test_impl(
            "abc\tdef\t\t\tghi\t\t",
            [
                ("abc|n", NO_TAG),
                ("def|n", NO_TAG),
                ("|n", NO_TAG),
                ("|n", NO_TAG),
                ("ghi|n", NO_TAG),
                ("|n", NO_TAG),
                ("|e", NO_TAG),
            ],
        );

        test_impl("abc/tag", [("abc|e", ["tag"])]);
        test_impl("abc/tag1/tag2/tag3", [("abc|e", ["tag1", "tag2", "tag3"])]);
        test_impl("abc/tag/", [("abc|e", ["tag", ""])]);
        test_impl("abc/tag////", [("abc|e", ["tag", "", "", "", ""])]);
        test_impl("abc/tag///another", [("abc|e", ["tag", "", "", "another"])]);
        test_impl("/", [("|e", [""])]);
        test_impl("/tag", [("|e", ["tag"])]);
        test_impl("/tag", [("|e", ["tag"])]);

        test_impl(
            "abc\t/tag",
            [
                ("abc|n", NO_TAG.iter().copied()),
                ("|e", ["tag"].iter().copied()),
            ],
        );
        test_impl("abc/tag1\t/tag2", [("abc|n", ["tag1"]), ("|e", ["tag2"])]);
        test_impl(
            "abc/tag1//\t/tag2",
            [
                ("abc|n", ["tag1", "", ""].iter().copied()),
                ("|e", ["tag2"].iter().copied()),
            ],
        );

        test_impl(
            "abc/tag\tdef/tag2-1/tag2-2\tghi\tjkl/tag",
            [
                ("abc|n", ["tag"].iter().copied()),
                ("def|n", ["tag2-1", "tag2-2"].iter().copied()),
                ("ghi|n", [].iter().copied()),
                ("jkl|e", ["tag"].iter().copied()),
            ],
        );
    }
}
