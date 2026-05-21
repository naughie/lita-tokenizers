//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with various filesystem IOs.
//!
//! - [`sync_fs`] corresponds to [`std::fs`].
//! - [`smol_fs`] corresponds to [`smol::fs`](smol_fs).
//! - [`tokio_fs`] corresponds to [`tokio::fs`].
//!
//! In either cases are there three functions: `tokenize_file()`, `tokenize_dir()`, and
//! `tokenize_path()`.
//!
//! `tokenize_file()` reads the input from a file and outputs to a file.
//!
//! `tokenize_dir()` reads all of the files recursively and outputs to a directory with nested
//! structure as-is.
//!
//! `tokenize_path()` dynamically determines the filetype and calls `tokenize_file()` or
//! `tokenize_dir()` depending on the filetype.

#[cfg(feature = "sync-fs")]
pub mod sync_fs;

#[cfg(feature = "tokio-fs")]
pub mod tokio_fs;

#[cfg(feature = "smol-fs")]
pub mod smol_fs;

use crate::io;

use tokenizers_core::TokenStreamBuilder;

use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io::Error as IoError;

/// Indicates the filesystem errors.
#[derive(Debug)]
pub enum FsError {
    /// Could not [open a file](std::fs::File::open()).
    OpenFile(IoError),
    /// Could not [read a directory](std::fs::read_dir()).
    ReadDir(IoError),
    /// Could not get the [metadata of a file](std::fs::metadata())
    FileMetadata(IoError),

    /// Could not [create a file](std::fs::File::create()).
    CreateFile(IoError),
    /// Could not [create a directory](std::fs::create_dir_all()).
    CreateDir(IoError),
}

#[derive(Debug)]
pub enum Error<Tok: TokenStreamBuilder> {
    /// The input text (the content of a file) is unknown encoding.
    UnknownEncoding(crate::UnknownEncoding),
    /// IO error for reading.
    ReadIo(IoError),
    /// IO error for writing.
    WriteIo(IoError),
    /// Filesystem errors.
    Fs(FsError),
    /// Tokenization fails.
    Tokenizer(Tok::Error),
}

impl Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenFile(e) => write!(f, "could not open file: {e}"),
            Self::ReadDir(e) => write!(f, "could not read directory: {e}"),
            Self::FileMetadata(e) => write!(f, "could not read file metadata: {e}"),
            Self::CreateFile(e) => write!(f, "could not create file: {e}"),
            Self::CreateDir(e) => write!(f, "could not create directory: {e}"),
        }
    }
}

impl<Tok> Display for Error<Tok>
where
    Tok: TokenStreamBuilder,
    Tok::Error: StdError,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEncoding(e) => e.fmt(f),
            Self::ReadIo(e) => write!(f, "could not read input: {e}"),
            Self::WriteIo(e) => write!(f, "could not write the tokenized output: {e}"),
            Self::Fs(e) => e.fmt(f),
            Self::Tokenizer(e) => write!(f, "tokenization failed: {e}"),
        }
    }
}

impl StdError for FsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::OpenFile(e) => Some(e),
            Self::ReadDir(e) => Some(e),
            Self::FileMetadata(e) => Some(e),
            Self::CreateFile(e) => Some(e),
            Self::CreateDir(e) => Some(e),
        }
    }
}

impl<Tok> StdError for Error<Tok>
where
    Tok: TokenStreamBuilder + fmt::Debug,
    Tok::Error: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::UnknownEncoding(e) => Some(e),
            Self::ReadIo(e) => Some(e),
            Self::WriteIo(e) => Some(e),
            Self::Fs(e) => Some(e),
            Self::Tokenizer(e) => Some(e),
        }
    }
}

impl<Tok: TokenStreamBuilder> From<io::Error<Tok>> for Error<Tok> {
    fn from(value: io::Error<Tok>) -> Self {
        use io::{Error, ReadError, WriteError};
        match value {
            Error::Read(ReadError::UnknownEncoding(e)) => Self::UnknownEncoding(e),
            Error::Read(ReadError::ReadIo(e)) => Self::ReadIo(e),
            Error::Write(WriteError::WriteIo(e)) => Self::WriteIo(e),
            Error::Tokenizer(e) => Self::Tokenizer(e),
        }
    }
}

impl<Tok: TokenStreamBuilder> Error<Tok> {
    fn open_file(e: IoError) -> Self {
        Self::Fs(FsError::OpenFile(e))
    }
    fn read_dir(e: IoError) -> Self {
        Self::Fs(FsError::ReadDir(e))
    }
    fn file_metadata(e: IoError) -> Self {
        Self::Fs(FsError::FileMetadata(e))
    }
    fn create_file(e: IoError) -> Self {
        Self::Fs(FsError::CreateFile(e))
    }
    fn create_dir(e: IoError) -> Self {
        Self::Fs(FsError::CreateDir(e))
    }
}
