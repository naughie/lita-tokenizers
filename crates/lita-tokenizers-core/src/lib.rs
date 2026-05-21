//! Core traits and configurations for tokenizers, which are used internally by
//! [LiTA](https://www.lsta.media.kyoto-u.ac.jp/lita/).
//!
//! Note that this is very the fundamental and low-level APIs:
//! in most cases you shold prefer the `lita-tokenizers` crate instead.
//!
//! # Terminology
//!
//! A **surface** is a byte-sequence of a word *that appears in the input text*,
//! neither a morpheme nor a lemma (dictionary form).
//! For example, in the input text `We are programmers.`, you find three surfaces `We`, `are`, and
//! `programmers`, while the lemmata are `we`, `be`, and `programmer`.
//!
//! A **tag** is the linguistic information associated to a word.
//! It include, but not limited to, part-of-speech, pronunciation, prosody, meaning,
//! morphological analysis, and canonical form,
//! It depends on [`Tokenizer`]s.
//!
//! A **token** corresponds with a word, to which are associated a surface and zero or more tags.
//! Although "word", the unit of tokens, is a deliberate measure and a [`Tokenizer`] can choose its
//! own definition of "word," we mean to tokenize an input text into *meaningful* units.
//!
//! Additionally, whether symbols (punctuation, quotations, parentheses, etc.) are ignored,
//! included to surrounding tokens, or recognized as isolated tokens,
//! depends on [`Tokenizer`]s.
//!
//!
//! ## Examples
//!
//! An example tokenizer turns the sentence `We are programmers.` into the following tokens:
//!
//! ```
//! # struct Token {
//! #     surface: &'static str,
//! #     tags: [&'static str; 3],
//! # }
//! # fn tokens() -> [Token; 3] {
//! // input: "We are programmers."
//! [
//!     Token {
//!         surface: "We",
//!         tags: ["pronoun", "1st-person/pl", "we"],
//!     },
//!     Token {
//!         surface: "are",
//!         tags: ["verb", "act/pres", "be"],
//!     },
//!     Token {
//!         surface: "programmers",
//!         tags: ["noun", "pl/subj", "programmer"],
//!     },
//! ]
//! # }
//! ```
//!
//! Example of a Greek [`Tokenizer`]:
//!
//! ```
//! # struct Token {
//! #     surface: &'static str,
//! #     tags: [&'static str; 3],
//! # }
//! # fn tokens() -> [Token; 3] {
//! // input: "χαλεπὰ τὰ καλά."
//! [
//!     Token {
//!         surface: "χαλεπὰ",
//!         tags: ["adjective", "n/pl/nom", "χαλεπός"],
//!     },
//!     Token {
//!         surface: "τὰ",
//!         tags: ["definite-article", "n/pl/nom", "ὁ"],
//!     },
//!     Token {
//!         surface: "καλά",
//!         tags: ["adjective", "n/pl/nom", "καλός"],
//!     },
//! ]
//! # }
//! ```
//!
//! Example of a Japanese [`Tokenizer`]:
//!
//! ```
//! # struct Token {
//! #     surface: &'static str,
//! #     tags: [&'static str; 2],
//! # }
//! # fn tokens() -> [Token; 3] {
//! // input: "春はあけぼの。"
//! [
//!     Token {
//!         surface: "春",
//!         tags: ["名詞", "はる"],
//!     },
//!     Token {
//!         surface: "は",
//!         tags: ["助詞", "は"],
//!     },
//!     Token {
//!         surface: "あけぼの",
//!         tags: ["名詞", "あけぼの"],
//!     },
//! ]
//! # }
//! ```

/// Delimiters which should be put in tokenized text.
///
/// Each delimiter is guaranteed to be a single-byte ASCII (`0x00..=0x7f`).
/// Hence it is equipped with four variants:
///
/// - as a [`u8`] byte,
/// - as a UTF-8 [`&str`](`str`) (`*_STR`),
/// - as a null-terminated [`&CStr`](`std::ffi::CStr`) (`*_CSTR`),
/// - and as a Unicode codepoint [`char`] (`*_CHAR`).
pub mod delim {
    use std::ffi::CStr;

    /// Delimiter of tokens.
    ///
    /// Looks like `{token1}{delim::TOKEN}{token2}`
    ///
    /// Or formally,
    ///
    /// ```text
    /// Sentence = Token (delim::TOKEN Token)* \n
    /// ```
    pub const TOKEN: u8 = b'\t';
    /// Delimiter of tags.
    ///
    /// `{token}` = `{surface}{delim::TAG}{tag1}{delim::TAG}{tag2}`
    ///
    /// Or formally,
    ///
    /// ```text
    /// Token = Surface (delim::TAG TAG)*
    /// ```
    pub const TAG: u8 = b'/';
    /// Escape characters that escapes [`TOKEN`] and [`TAG`] delimiters.
    pub const ESCAPE: u8 = b'\\';

    /// String representation of [`TOKEN`].
    pub const TOKEN_STR: &str = "\t";
    /// String representation of [`TAG`].
    pub const TAG_STR: &str = "/";
    /// String representation of [`ESCAPE`].
    pub const ESCAPE_STR: &str = "\\";

    /// Null-terminated string representation of [`TOKEN`].
    pub const TOKEN_CSTR: &CStr = c"\t";
    /// Null-terminated string representation of [`TAG`].
    pub const TAG_CSTR: &CStr = c"/";
    /// Null-terminated string representation of [`ESCAPE`].
    pub const ESCAPE_CSTR: &CStr = c"\\";

    /// Unicode codepoint of [`TOKEN`].
    pub const TOKEN_CHAR: char = '\t';
    /// Unicode codepoint of [`TAG`].
    pub const TAG_CHAR: char = '/';
    /// Unicode codepoint of [`ESCAPE`].
    pub const ESCAPE_CHAR: char = '\\';

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn consistent_cast() {
            fn helper(as_byte: u8, as_str: &str, as_cstr: &CStr, as_char: char) {
                let s = [as_byte];
                let s = std::str::from_utf8(&s);
                assert!(s.is_ok(), "0x{as_byte:02x} is not a valid UTF-8 character");
                assert_eq!(s.unwrap(), as_str);
                assert_eq!(as_byte as char, as_char);

                assert_eq!(as_cstr.to_bytes(), &[as_byte]);
                assert_eq!(as_cstr.to_bytes_with_nul(), &[as_byte, 0]);
                assert!(
                    as_cstr.to_str().is_ok_and(|cs| cs == as_str),
                    "`as_cstr` does not equal to `as_str` for `{as_str:?}`"
                );
            }
            helper(TOKEN, TOKEN_STR, TOKEN_CSTR, TOKEN_CHAR);
            helper(TAG, TAG_STR, TAG_CSTR, TAG_CHAR);
            helper(ESCAPE, ESCAPE_STR, ESCAPE_CSTR, ESCAPE_CHAR);
        }

        #[test]
        fn can_be_escaped() {
            fn helper(byte: u8) {
                let s = [ESCAPE, byte];
                assert!(
                    std::str::from_utf8(&s).is_ok(),
                    "{s:02x?} is not a valid UTF-8 string"
                );
            }
            helper(TOKEN);
            helper(TAG);
            helper(ESCAPE);
        }
    }
}

pub trait Tokenizer {
    type StreamBuilder<'t>: TokenStreamBuilder<Error = Self::Error>
    where
        Self: 't;
    type Error;

    fn builder<'t>(&'t mut self) -> Result<Self::StreamBuilder<'t>, Self::Error>;
}

impl<T: Tokenizer> Tokenizer for &mut T {
    type StreamBuilder<'t>
        = T::StreamBuilder<'t>
    where
        Self: 't;
    type Error = T::Error;

    fn builder<'t>(&'t mut self) -> Result<Self::StreamBuilder<'t>, Self::Error> {
        <T as Tokenizer>::builder(self)
    }
}

pub trait TokenStreamBuilder {
    type Output<'t, 's>: TokenStream
    where
        Self: 't;
    type Error;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error>;
}

impl<T: TokenStreamBuilder> TokenStreamBuilder for &mut T {
    type Output<'t, 's>
        = T::Output<'t, 's>
    where
        Self: 't;
    type Error = T::Error;

    fn tokenize<'t, 's>(&'t mut self, input: &'s str) -> Result<Self::Output<'t, 's>, Self::Error> {
        <T as TokenStreamBuilder>::tokenize(self, input)
    }
}

pub trait TokenStream {
    type IntoIter<'o>: Iterator<Item = (TokenPosition, Self::Token<'o>)>
    where
        Self: 'o;
    type Token<'o>: Token<'o>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o>;
}

impl<S: TokenStream> TokenStream for &S {
    type IntoIter<'o>
        = S::IntoIter<'o>
    where
        Self: 'o;
    type Token<'o>
        = S::Token<'o>
    where
        Self: 'o;

    fn tokens<'o>(&'o self) -> Self::IntoIter<'o> {
        <S as TokenStream>::tokens(self)
    }
}

/// Represents whether a [token](`TokenStream::Token`) is an end-of-sentence token or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenPosition {
    Normal,
    Eos,
}

pub trait Token<'o> {
    type Tags: Iterator<Item = Tag<'o>>;

    fn surface(&self) -> Tag<'o>;

    fn tags(&self) -> Self::Tags;
}

impl<'o, T: Token<'o>> Token<'o> for &T {
    type Tags = T::Tags;

    fn surface(&self) -> Tag<'o> {
        <T as Token<'o>>::surface(self)
    }

    fn tags(&self) -> Self::Tags {
        <T as Token<'o>>::tags(self)
    }
}

/// Represents both a surface and a tag.
///
/// A `Tag` is escaped if
///
/// - [`delim::TOKEN`] always follows the odd number of [`delim::ESCAPE`]s,
/// - [`delim::TAG`] always follows the odd number of [`delim::ESCAPE`]s,
/// - [`delim::ESCAPE`] must be followed by [`delim::TOKEN`], [`delim::TAG`], or paired with another [`delim::ESCAPE`].
///
/// In regex, an escaped `Tag` is expressed as
///
/// ```text
/// ( (ESCAPE TOKEN) | (ESCAPE TAG) | (ESCAPE ESCAPE) | [^ TOKEN TAG ESCAPE] )*
/// ```
///
/// Otherwise it should be treated as "unescaped."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag<'o> {
    Escaped(&'o str),
    Unescaped(&'o str),
}

impl<'o> Tag<'o> {
    pub fn as_str(self) -> &'o str {
        match self {
            Self::Escaped(s) => s,
            Self::Unescaped(s) => s,
        }
    }
}
