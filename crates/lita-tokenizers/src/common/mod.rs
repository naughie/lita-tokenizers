#[cfg(feature = "tokio")]
pub mod tokio;

pub mod sync;

use tokenizers_core::TokenStreamBuilder;

use tokenizers_utils::fs::Error as FsError;
use tokenizers_utils::io::Error as IoError;

use std::path::Path;
use std::path::PathBuf;

use super::Error;

/// Auxiliary type that expresses an Tokenizer error.
///
/// This should be never constructed outside our crate.
/// The only usage of it is the trait bound of functions in this module (like [`tokenize()`]):
///
/// ```no_run
/// TokenizerError<T>: Into<Error>,
/// ```
pub struct TokenizerError<T: TokenStreamBuilder>(pub T::Error);

/// Represents an input text file or directory.
#[derive(Debug, Clone)]
pub enum Input<'a> {
    Path(&'a Path),
    PathOwned(PathBuf),
    Stdin,
}

impl Input<'_> {
    fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(p) => Some(p),
            Self::PathOwned(p) => Some(p),
            Self::Stdin => None,
        }
    }
}

/// Represents an output text file or directory.
#[derive(Debug, Clone)]
pub enum Output<'a> {
    Path(&'a Path),
    PathOwned(PathBuf),
    Stdout,
}

impl Output<'_> {
    fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(p) => Some(p),
            Self::PathOwned(p) => Some(p),
            Self::Stdout => None,
        }
    }
}

impl<T> From<IoError<T>> for Error
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
{
    fn from(value: IoError<T>) -> Self {
        FsError::<T>::from(value).into()
    }
}

impl<T> From<FsError<T>> for Error
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
{
    fn from(value: FsError<T>) -> Self {
        match value {
            FsError::UnknownEncoding(e) => Error::UnknownEncoding(e),
            FsError::Fs(e) => Error::Fs(e),
            FsError::ReadIo(e) => Error::ReadIo(e),
            FsError::WriteIo(e) => Error::WriteIo(e),
            FsError::Tokenizer(e) => TokenizerError(e).into(),
        }
    }
}
