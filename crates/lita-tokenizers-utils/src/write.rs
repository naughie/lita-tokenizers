//! Utilities to write a tokenized output as a single string.
//!
//! It includes the Unicode normalization and handling special characters ([`tokenizers_core::delim`]).
//!
//! For normalization, we adopt NFKC (see the
//! [Unicode Standard Annex #15](https://unicode.org/reports/tr15/#Norm_Forms)).

/// Replaces the delimiters in [`tokenizers_core::delim`] with safe characters.
pub mod replace {
    /// Box Drawings Light Diagonal Upper Right to Lower Left, "╱"
    pub const TAG: char = '\u{2571}';
    /// Open Box, "␣"
    pub const TOKEN: char = '\u{2423}';
}

pub use tag::{NormalizedTag, NormalizedTagChars, normalize_tag};
mod tag {
    use tokenizers_core::Tag;
    use tokenizers_core::delim;

    use super::replace;

    use memchr::Memchr3;

    use unicode_normalization::Recompositions;

    use std::fmt;
    use std::iter::FusedIterator;
    use std::ops::RangeTo;
    use std::str::Chars;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum SpecialByte {
        Escape,
        Tag,
        Token,
        None,
    }

    type Norm<'t> = Recompositions<Chars<'t>>;

    struct SpecialByteBound {
        prefix: RangeTo<usize>,
        test_byte: usize,
    }

    struct NormalizedTagCharsImpl<'t> {
        tag: &'t str,
        from: usize,
        escaped: bool,
        special_it: Memchr3<'t>,
        special_byte: SpecialByte,
        prefix: Option<Norm<'t>>,
    }

    impl<'t> NormalizedTagCharsImpl<'t> {
        fn new(tag: &'t str) -> Self {
            use delim::{ESCAPE, TAG, TOKEN};

            Self {
                tag,
                from: 0,
                escaped: false,
                special_it: memchr::memchr3_iter(ESCAPE, TAG, TOKEN, tag.as_bytes()),
                special_byte: SpecialByte::None,
                prefix: None,
            }
        }

        fn next_impl<F>(&mut self, bound: F) -> Option<char>
        where
            F: for<'a> Fn(&'a mut Memchr3<'t>) -> Option<SpecialByteBound>,
        {
            use delim::{ESCAPE, TAG};
            use delim::{ESCAPE_CHAR, TAG_CHAR, TOKEN_CHAR};
            use unicode_normalization::UnicodeNormalization as _;

            use SpecialByte as Sb;

            loop {
                if self.escaped {
                    match self.special_byte {
                        Sb::Escape => {
                            self.special_byte = Sb::None;
                            self.escaped = false;
                            return Some('\\');
                        }
                        Sb::Tag => {
                            self.special_byte = Sb::None;
                            self.escaped = false;
                            return Some(replace::TAG);
                        }
                        Sb::Token => {
                            self.special_byte = Sb::None;
                            self.escaped = false;
                            return Some(replace::TOKEN);
                        }
                        Sb::None => return None,
                    }
                }

                if self.prefix.is_none() {
                    if let Some(idx) = bound(&mut self.special_it) {
                        let byte = self.tag.as_bytes()[idx.test_byte];
                        self.special_byte = if byte == ESCAPE {
                            Sb::Escape
                        } else if byte == TAG {
                            Sb::Tag
                        } else {
                            Sb::Token
                        };
                        self.prefix = Some(self.tag[self.from..(idx.prefix.end)].nfkc());
                        self.from = idx.test_byte + 1;
                    } else if self.from != self.tag.len() {
                        self.special_byte = Sb::None;
                        self.prefix = Some(self.tag[self.from..].nfkc());
                        self.from = self.tag.len();
                    }
                    self.escaped = false;
                }

                if let Some(prefix) = self.prefix.as_mut() {
                    if let Some(c) = prefix.next() {
                        if c == ESCAPE_CHAR {
                            return Some('\\');
                        } else if c == TAG_CHAR {
                            return Some(replace::TAG);
                        } else if c == TOKEN_CHAR || c.is_whitespace() {
                            return Some(replace::TOKEN);
                        } else {
                            return Some(c);
                        }
                    } else {
                        self.prefix = None;
                    }

                    self.escaped = true;
                } else {
                    return None;
                }
            }
        }
    }

    /// Internal implementation of [`NormalizedTagChars`].
    pub struct NormalizedTagCharsEscaped<'t> {
        inner: NormalizedTagCharsImpl<'t>,
    }

    impl<'t> NormalizedTagCharsEscaped<'t> {
        fn new(tag: &'t str) -> Self {
            Self {
                inner: NormalizedTagCharsImpl::new(tag),
            }
        }
    }

    impl Iterator for NormalizedTagCharsEscaped<'_> {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next_impl(|it| {
                it.nth(1).map(|idx| SpecialByteBound {
                    prefix: ..(idx - 1),
                    test_byte: idx,
                })
            })
        }
    }
    impl FusedIterator for NormalizedTagCharsEscaped<'_> {}

    /// Internal implementation of [`NormalizedTagChars`].
    pub struct NormalizedTagCharsUnescaped<'t> {
        inner: NormalizedTagCharsImpl<'t>,
    }

    impl<'t> NormalizedTagCharsUnescaped<'t> {
        fn new(tag: &'t str) -> Self {
            Self {
                inner: NormalizedTagCharsImpl::new(tag),
            }
        }
    }

    impl Iterator for NormalizedTagCharsUnescaped<'_> {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next_impl(|it| {
                it.next().map(|idx| SpecialByteBound {
                    prefix: ..idx,
                    test_byte: idx,
                })
            })
        }
    }
    impl FusedIterator for NormalizedTagCharsUnescaped<'_> {}

    /// Applies the Unicode normalization and special character replacement.
    ///
    /// Also, [whitespaces](char::is_whitespace()) are replaced with [`replace::TOKEN`].
    ///
    /// ```
    /// # use tokenizers_core as lita_tokenizers_core;
    /// use lita_tokenizers_core::Tag;
    /// use lita_tokenizers_utils::write::normalize_tag;
    ///
    /// // Unescaped tags
    /// assert_eq!(normalize_tag(Tag::Unescaped("abc def")).to_string(), "abc␣def");
    /// assert_eq!(normalize_tag(Tag::Unescaped("ａｂｃ　ｄｅｆ")).to_string(), "abc␣def");
    /// assert_eq!(normalize_tag(Tag::Unescaped("\t/\\")).to_string(), "␣╱\\");
    ///
    /// // Escaped tags
    /// assert_eq!(normalize_tag(Tag::Escaped("abc def")).to_string(), "abc␣def");
    /// assert_eq!(normalize_tag(Tag::Escaped("ａｂｃ　ｄｅｆ")).to_string(), "abc␣def");
    /// assert_eq!(normalize_tag(Tag::Escaped("\\\t\\/\\\\")).to_string(), "␣╱\\");
    /// ```
    ///
    /// You can iterate over [`char`]s as well:
    ///
    /// ```
    /// # use tokenizers_core::Tag;
    /// # use lita_tokenizers_utils::write::normalize_tag;
    /// let tag = Tag::Unescaped("ａｂｃ　ｄｅｆ");
    /// assert_eq!(normalize_tag(tag).chars().collect::<String>(), "abc␣def");
    /// ```
    pub struct NormalizedTag<'t> {
        tag: Tag<'t>,
    }

    /// Iterator of normalized [`char`]s, returned by [`NormalizedTag::chars()`].
    pub enum NormalizedTagChars<'t> {
        Escaped(NormalizedTagCharsEscaped<'t>),
        Unescaped(NormalizedTagCharsUnescaped<'t>),
    }

    impl Iterator for NormalizedTagChars<'_> {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::Escaped(inner) => inner.next(),
                Self::Unescaped(inner) => inner.next(),
            }
        }
    }
    impl FusedIterator for NormalizedTagChars<'_> {}

    impl<'t> NormalizedTag<'t> {
        pub fn new(tag: Tag<'t>) -> Self {
            Self { tag }
        }

        /// Iterates over normalized [`char`]s.
        pub fn chars(&self) -> NormalizedTagChars<'t> {
            match self.tag {
                Tag::Escaped(tag) => {
                    NormalizedTagChars::Escaped(NormalizedTagCharsEscaped::new(tag))
                }
                Tag::Unescaped(tag) => {
                    NormalizedTagChars::Unescaped(NormalizedTagCharsUnescaped::new(tag))
                }
            }
        }
    }

    impl fmt::Display for NormalizedTag<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for c in self.chars() {
                write!(f, "{c}")?;
            }

            Ok(())
        }
    }

    /// Applies the Unicode normalization and special character replacement.
    ///
    /// Alias to [`NormalizedTag::new()`].
    pub fn normalize_tag(tag: Tag<'_>) -> NormalizedTag<'_> {
        NormalizedTag::new(tag)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn normalize_tag_escaped() {
            fn eq(test: &str, expected: &str) {
                let test_to_string = NormalizedTag::new(Tag::Escaped(test)).to_string();
                assert_eq!(test_to_string, expected);

                let test_collect = NormalizedTag::new(Tag::Escaped(test))
                    .chars()
                    .collect::<String>();
                assert_eq!(test_collect, expected);
            }

            eq("", "");
            eq("abc", "abc");
            eq("ａｂｃ", "abc");
            eq("あいう", "あいう");

            eq("\\\\", "\\");
            eq("\\\\\\\\", "\\\\");
            eq("\\\t", "␣");
            eq("\\/", "╱");

            eq("ａｂ\\/ｃ\\\t\\\tｄ ｅ　ｆ\\/ｇｈｉ\n", "ab╱c␣␣d␣e␣f╱ghi␣");
        }

        #[test]
        fn normalize_tag_unescaped() {
            fn eq(test: &str, expected: &str) {
                let test_to_string = NormalizedTag::new(Tag::Unescaped(test)).to_string();
                assert_eq!(test_to_string, expected);

                let test_collect = NormalizedTag::new(Tag::Unescaped(test))
                    .chars()
                    .collect::<String>();
                assert_eq!(test_collect, expected);
            }

            eq("", "");
            eq("abc", "abc");
            eq("ａｂｃ", "abc");
            eq("あいう", "あいう");

            eq("\\", "\\");
            eq("\\\\", "\\\\");
            eq("\t", "␣");
            eq("\\\t", "\\␣");
            eq("/", "╱");

            eq("ａｂ/ｃ\t\tｄ ｅ　ｆ/ｇｈｉ\n", "ab╱c␣␣d␣e␣f╱ghi␣");
        }

        #[test]
        fn fused() {
            fn test_impl_escaped(s: &str) {
                let tag = normalize_tag(Tag::Escaped(s));
                let mut it = tag.chars();
                for _ in &mut it {}
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
            }
            fn test_impl_unescaped(s: &str) {
                let tag = normalize_tag(Tag::Unescaped(s));
                let mut it = tag.chars();
                for _ in &mut it {}
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
                assert_eq!(it.next(), Option::<char>::None);
            }

            test_impl_escaped("");
            test_impl_escaped("abc");
            test_impl_escaped("ａｂｃ");
            test_impl_escaped("あいう");
            test_impl_escaped("\\\\");
            test_impl_escaped("\\\t");
            test_impl_escaped("\\/");
            test_impl_escaped("ａｂ\\/ｃ\\\t\\\tｄ ｅ　ｆ\\/ｇｈｉ\n");

            test_impl_unescaped("");
            test_impl_unescaped("abc");
            test_impl_unescaped("ａｂｃ");
            test_impl_unescaped("あいう");
            test_impl_unescaped("\\");
            test_impl_unescaped("\\\\");
            test_impl_unescaped("\t");
            test_impl_unescaped("\\\t");
            test_impl_unescaped("/");
            test_impl_unescaped("ａｂ/ｃ\t\tｄ ｅ　ｆ/ｇｈｉ\n");
        }
    }
}

pub use token_chars::{NormalizedToken, NormalizedTokenChars, normalize_token};
pub use token_chars::{NormalizedTokenStream, NormalizedTokenStreamChars, normalize_token_stream};
mod token_chars {
    use tokenizers_core::delim;
    use tokenizers_core::{Tag, Token, TokenPosition, TokenStream};

    use super::tag::{NormalizedTagChars, normalize_tag};

    use std::fmt::{self, Display};

    /// Applies [`NormalizedTag`](super::NormalizedTag) to produced [`Tag`]s and places
    /// [`delim::TAG`] between tags.
    ///
    /// ```
    /// use lita_tokenizers_utils::write::normalize_token;
    /// // for test:
    /// use lita_tokenizers_utils::noop_tokenizer::NoopToken;
    ///
    /// let token = NoopToken {
    ///     surface: "ａｂｃ　ｄｅｆ",
    ///     tags: Some("tag1/tag2"),
    /// };
    /// assert_eq!(normalize_token(token).to_string(), "abc␣def/tag1/tag2");
    ///
    /// let token = NoopToken {
    ///     surface: "abc",
    ///     tags: None,
    /// };
    /// assert_eq!(normalize_token(token).to_string(), "abc");
    ///
    /// let token = NoopToken {
    ///     surface: "abc",
    ///     tags: Some("tag1"),
    /// };
    /// assert_eq!(normalize_token(token).to_string(), "abc/tag1");
    /// ```
    ///
    /// You can iterate over [`char`]s as well:
    ///
    /// ```
    /// # use lita_tokenizers_utils::write::normalize_token;
    /// # use lita_tokenizers_utils::noop_tokenizer::NoopToken;
    /// let token = NoopToken {
    ///     surface: "ａｂｃ　ｄｅｆ",
    ///     tags: Some("tag1/tag2"),
    /// };
    /// assert_eq!(normalize_token(token).chars().collect::<String>(), "abc␣def/tag1/tag2");
    /// ```
    pub struct NormalizedToken<T> {
        token: T,
    }

    impl<'o, T> Display for NormalizedToken<T>
    where
        T: Token<'o>,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for c in self.chars() {
                write!(f, "{c}")?;
            }
            Ok(())
        }
    }

    impl<T> NormalizedToken<T> {
        pub fn new(token: T) -> Self {
            Self { token }
        }
    }

    impl<'o, T> NormalizedToken<T>
    where
        T: Token<'o>,
    {
        /// Iterates over normalized [`char`]s.
        pub fn chars(&self) -> NormalizedTokenChars<'o, T::Tags> {
            NormalizedTokenChars {
                surface: normalize_tag(self.token.surface()).chars(),
                tags: self.token.tags(),
                curr_tag: None,
            }
        }
    }

    /// Iterator of normalized [`char`]s, returned by [`NormalizedToken::chars()`].
    pub struct NormalizedTokenChars<'t, T> {
        surface: NormalizedTagChars<'t>,
        tags: T,
        curr_tag: Option<NormalizedTagChars<'t>>,
    }

    impl<'t, T> Iterator for NormalizedTokenChars<'t, T>
    where
        T: Iterator<Item = Tag<'t>>,
    {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(c) = self.surface.next() {
                return Some(c);
            }

            loop {
                if let Some(curr_tag) = self.curr_tag.as_mut() {
                    if let Some(c) = curr_tag.next() {
                        return Some(c);
                    } else {
                        self.curr_tag = None;
                    }
                } else {
                    self.curr_tag = self.tags.next().map(|tag| normalize_tag(tag).chars());
                    if self.curr_tag.is_none() {
                        return None;
                    } else {
                        return Some(delim::TAG_CHAR);
                    }
                }
            }
        }
    }

    /// Applies the Unicode normalization and special character replacement.
    ///
    /// Alias to [`NormalizedToken::new()`].
    pub fn normalize_token<T>(token: T) -> NormalizedToken<T> {
        NormalizedToken::new(token)
    }

    /// Applies [`NormalizedToken`] to produced [`Token`]s and places
    /// [`delim::TOKEN`] between tokens, appending `\n` after [EoS](TokenPosition::Eos) tokens.
    ///
    /// ```
    /// use lita_tokenizers_utils::write::normalize_token_stream;
    /// // for test:
    /// use lita_tokenizers_utils::noop_tokenizer::NoopStream;
    ///
    /// let stream = NoopStream {
    ///     input: "ａｂｃ　ｄｅｆ/tag1/tag2",
    /// };
    /// assert_eq!(normalize_token_stream(stream).to_string(), "abc␣def/tag1/tag2\n");
    ///
    /// let stream = NoopStream {
    ///     input: "token1/tag1",
    /// };
    /// assert_eq!(normalize_token_stream(stream).to_string(), "token1/tag1\n");
    ///
    /// let stream = NoopStream {
    ///     input: "token1/tag1\ttoken2/tag2\ttoken3/tag3",
    /// };
    /// assert_eq!(normalize_token_stream(stream).to_string(), "token1/tag1\ttoken2/tag2\ttoken3/tag3\n");
    /// ```
    ///
    /// You can iterate over [`char`]s as well:
    ///
    /// ```
    /// # use lita_tokenizers_utils::write::normalize_token_stream;
    /// # use lita_tokenizers_utils::noop_tokenizer::NoopStream;
    ///
    /// let stream = NoopStream {
    ///     input: "ａｂｃ　ｄｅｆ/tag1/tag2",
    /// };
    /// assert_eq!(normalize_token_stream(stream).chars().collect::<String>(), "abc␣def/tag1/tag2\n");
    /// ```
    pub struct NormalizedTokenStream<S> {
        stream: S,
    }

    impl<S> Display for NormalizedTokenStream<S>
    where
        S: TokenStream,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for c in self.chars() {
                write!(f, "{c}")?;
            }
            Ok(())
        }
    }

    impl<S> NormalizedTokenStream<S> {
        pub fn new(stream: S) -> Self {
            Self { stream }
        }
    }

    impl<S: TokenStream> NormalizedTokenStream<S> {
        /// Iterates over normalized [`char`]s.
        pub fn chars(&self) -> NormalizedTokenStreamChars<'_, S::IntoIter<'_>, S::Token<'_>> {
            NormalizedTokenStreamChars {
                token_it: self.stream.tokens(),
                curr_token: None,
                delim: DelimState::None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DelimState {
        Token,
        Sentence,
        Stream,
        None,
    }

    /// Iterator of normalized [`char`]s, returned by [`NormalizedTokenStream::chars()`].
    pub struct NormalizedTokenStreamChars<'t, I, T: Token<'t>> {
        token_it: I,
        curr_token: Option<NormalizedTokenChars<'t, T::Tags>>,
        delim: DelimState,
    }

    impl<'t, I, T> Iterator for NormalizedTokenStreamChars<'t, I, T>
    where
        I: Iterator<Item = (TokenPosition, T)>,
        T: Token<'t>,
    {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(curr_token) = self.curr_token.as_mut() {
                    if let Some(c) = curr_token.next() {
                        return Some(c);
                    } else {
                        self.curr_token = None;
                    }
                } else {
                    match self.delim {
                        DelimState::Token => {
                            self.delim = DelimState::None;
                            return Some(delim::TOKEN_CHAR);
                        }
                        DelimState::Sentence => {
                            self.delim = DelimState::None;
                            return Some('\n');
                        }
                        DelimState::Stream => {
                            return None;
                        }
                        DelimState::None => {}
                    }

                    let Some((position, token)) = self.token_it.next() else {
                        self.delim = DelimState::Stream;
                        return None;
                    };
                    self.delim = match position {
                        TokenPosition::Normal => DelimState::Token,
                        TokenPosition::Eos => DelimState::Sentence,
                    };

                    self.curr_token = Some(normalize_token(token).chars());
                }
            }
        }
    }

    /// Applies the Unicode normalization and special character replacement.
    ///
    /// Alias to [`NormalizedTokenStream::new()`].
    pub fn normalize_token_stream<S>(stream: S) -> NormalizedTokenStream<S> {
        NormalizedTokenStream::new(stream)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::noop_tokenizer::NoopTokenizer;
        use tokenizers_core::TokenStreamBuilder;

        #[test]
        fn token_stream() {
            fn eq(test: &str, expected: &str) {
                let test_to_string = normalize_token_stream(
                    TokenStreamBuilder::tokenize(&mut NoopTokenizer, test).unwrap(),
                )
                .to_string();
                assert_eq!(test_to_string, expected);

                let test_collect = normalize_token_stream(
                    TokenStreamBuilder::tokenize(&mut NoopTokenizer, test).unwrap(),
                )
                .chars()
                .collect::<String>();
                assert_eq!(test_collect, expected);
            }

            eq("", "\n");
            eq("abc", "abc\n");
            eq("abc\tdef\tghi", "abc\tdef\tghi\n");
            eq("ａｂｃ\tｄ ｅ　ｆ\tｇｈｉ", "abc\td␣e␣f\tghi\n");
            eq("\tabc\tdef\t\t\t", "\tabc\tdef\t\t\t\n");

            eq("abc/tag", "abc/tag\n");
            eq("abc/tag1/tag2/tag3", "abc/tag1/tag2/tag3\n");
            eq(
                "abc/tag\tdef\tghi/tag1/tag2\tjkl",
                "abc/tag\tdef\tghi/tag1/tag2\tjkl\n",
            );
            eq("abc/tag1///tag2/", "abc/tag1///tag2/\n");

            eq("/tag", "/tag\n");
            eq("/tag1/tag2/tag3", "/tag1/tag2/tag3\n");
        }
    }
}
