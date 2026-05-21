#![cfg_attr(docsrs, feature(doc_cfg))]

//! Exposes the high-level APIs of [`lita-tokenizers-core`](`tokenizers_core`).
//!
//! Read the module-level documentation for more details.
//!
//! ## Feature flags
//!
//! This crate has several feature flags:
//!
//! - `str` (*enabled by default*): utilities for pure string manipulation.
//! - `decode` (*disabled by default*): When disabled, we accept only UTF-8 inputs via [`String::from_utf8_lossy()`). When enabled, you can pass an input text in other encodings.
//! - `write` (*enabled by default*): utilities to write a tokenized output as a single string.
//! - `sync-io` (*disabled by default*): wraps an input/output by `std::io::{Read, Write}`. It implies `write`.
//! - `async-io` (*disabled by default*): wraps an input/output by `futures::io::{AsyncRead, AsyncWrite}`. It implies `write`.
//! - `tokio-io` (*disabled by default*): wraps an input/output by `tokio::io::{AsyncRead, AsyncWrite}`. It implies `write`.
//! - `sync-fs` (*disabled by default*): wraps an input/output by `std::fs`. It implies `sync-io`.
//! - `smol-fs` (*disabled by default*): wraps an input/output by `smol::fs`. It implies `async-io`.
//! - `tokio-fs` (*disabled by default*): wraps an input/output by `tokio::fs`. It implies `tokio-io`.

#[cfg(feature = "str")]
pub mod str;

#[cfg(feature = "decode")]
pub mod decode;
#[cfg(not(feature = "decode"))]
pub mod decode {
    use std::borrow::Cow;

    pub type UnknownEncoding = std::convert::Infallible;

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Charset {
        #[default]
        Utf8,
    }
    pub fn decode(bytes: &[u8], _charset: Charset) -> Result<Cow<'_, str>, UnknownEncoding> {
        Ok(String::from_utf8_lossy(bytes))
    }
}
pub use decode::{Charset, UnknownEncoding, decode};

#[cfg(any(feature = "sync-io", feature = "async-io", feature = "tokio-io"))]
pub mod io;

#[cfg(any(feature = "sync-fs", feature = "tokio-fs", feature = "smol-fs"))]
pub mod fs;

#[cfg(feature = "write")]
pub mod write;
#[cfg(feature = "write")]
pub use write::{NormalizedTag, NormalizedTagChars, normalize_tag};
#[cfg(feature = "write")]
pub use write::{
    NormalizedToken, NormalizedTokenChars, NormalizedTokenStream, NormalizedTokenStreamChars,
    normalize_token, normalize_token_stream,
};

pub mod noop_tokenizer;
