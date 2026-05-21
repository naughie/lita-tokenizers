//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with various IOs.
//!
//! - [`sync_io::tokenize()`] takes `std::io::{Read, Write}` as input/output.
//! - [`async_io::tokenize()`] takes `futures::io::{AsyncRead, AsyncWrite}` as input/output.
//! - [`tokio_io::tokenize()`] takes `tokio::io::{AsyncRead, AsyncWrite}` as input/output.
//!
//!
//! Additionally [`sync_io`] provides the APIs that utilize `std`'s [`fmt::Write`](std::fmt::Write)
//! and [`io::Write`](std::io::Write).

#[cfg(feature = "sync-io")]
pub mod sync_io;

#[cfg(feature = "async-io")]
pub mod async_io;
#[cfg(feature = "tokio-io")]
pub mod tokio_io;

use tokenizers_core::TokenStreamBuilder;

use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io::Error as IoError;

/// Indicates that an error happened while reading input.
#[derive(Debug, Clone)]
pub enum ReadError<Io = IoError> {
    /// The input byte sequence is of unknown encoding.
    UnknownEncoding(crate::UnknownEncoding),
    /// IO error for reading.
    ReadIo(Io),
}

impl<Io: StdError> Display for ReadError<Io> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEncoding(e) => e.fmt(f),
            Self::ReadIo(e) => write!(f, "could not read the input: {e}"),
        }
    }
}
impl<Io: StdError + 'static> StdError for ReadError<Io> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::UnknownEncoding(e) => Some(e),
            Self::ReadIo(e) => Some(e),
        }
    }
}

/// Indicates that an error happened while writing output.
#[derive(Debug, Clone)]
pub enum WriteError<Io = IoError> {
    /// IO error for writing.
    WriteIo(Io),
}

impl<Io: StdError> Display for WriteError<Io> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteIo(e) => write!(f, "could not write the tokenized output: {e}"),
        }
    }
}
impl<Io: StdError + 'static> StdError for WriteError<Io> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::WriteIo(e) => Some(e),
        }
    }
}

/// Indicates an error during `tokenize()` functions under the submodules.
#[derive(Debug, Clone)]
pub enum Error<Tok: TokenStreamBuilder, Io = IoError> {
    Read(ReadError<Io>),
    Tokenizer(Tok::Error),
    Write(WriteError<Io>),
}

impl<Tok, Io> Display for Error<Tok, Io>
where
    Tok: TokenStreamBuilder,
    Tok::Error: StdError,
    Io: StdError,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(e) => e.fmt(f),
            Self::Tokenizer(e) => write!(f, "tokenization failed: {e}"),
            Self::Write(e) => e.fmt(f),
        }
    }
}

impl<Tok, Io> StdError for Error<Tok, Io>
where
    Tok: TokenStreamBuilder + fmt::Debug,
    Tok::Error: StdError + 'static,
    Io: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            Self::Tokenizer(e) => Some(e),
            Self::Write(e) => Some(e),
        }
    }
}

impl<Tok: TokenStreamBuilder, Io> Error<Tok, Io> {
    fn unknown_encoding(e: crate::UnknownEncoding) -> Self {
        Self::Read(ReadError::UnknownEncoding(e))
    }

    fn write_io(e: Io) -> Self {
        Self::Write(WriteError::WriteIo(e))
    }

    fn read_io(e: Io) -> Self {
        Self::Read(ReadError::ReadIo(e))
    }
}
