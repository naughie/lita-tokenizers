use tokenizers_core::TokenStreamBuilder;

use tokenize_fs::Error as FsError;
use tokenizers_utils::Charset;
use tokenizers_utils::fs::tokio_fs as tokenize_fs;
use tokenizers_utils::io::Error as IoError;
use tokenizers_utils::io::tokio_io as tokenize_io;

use tokio::fs::File;
use tokio::io::{self, BufWriter};

use futures_util::Stream;

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

async fn tokenize_impl<T, R, W>(tok: &mut T, rdr: R, wtr: W, charset: Charset) -> Result<(), Error>
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
    R: io::AsyncRead + Unpin,
    W: io::AsyncWrite + Unpin,
{
    tokenize_io::tokenize(tok, rdr, wtr, charset)
        .await
        .map_err(Into::into)
}

async fn open_file(path: &Path) -> Result<File, Error> {
    File::open(path).await.map_err(Error::ReadIo)
}
async fn create_file(path: &Path) -> Result<BufWriter<File>, Error> {
    File::create(path)
        .await
        .map_err(Error::WriteIo)
        .map(BufWriter::new)
}

/// Tokenizes the input, writing the tokenized results to the `output`.
///
/// [`Input::Stdin`] and [`Output::Stdout`] are treated just as regular files.
/// If either of `input`/`output` is `Stdin`/`Stdout`, then the other must be a regular file or
/// `Stdin`/`Stdout`.
/// Otherwise it redirects to [`tokenize_path()`](tokenize_fs::tokenize_path()).
pub async fn tokenize<T>(
    tok: &mut T,
    input: Input<'_>,
    output: Output<'_>,
    charset: Charset,
) -> Result<(), Error>
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
{
    match (input.as_path(), output.as_path()) {
        (None, None) => tokenize_impl(tok, io::stdin(), io::stdout(), charset).await,
        (Some(input), Some(output)) => tokenize_fs::tokenize_path(tok, input, output, charset)
            .await
            .map_err(Into::into),
        (Some(input), None) => {
            tokenize_impl(tok, open_file(input).await?, io::stdout(), charset).await
        }
        (None, Some(output)) => {
            tokenize_impl(tok, io::stdin(), create_file(output).await?, charset).await
        }
    }
}

/// Repeatedly invokes [`tokenize()`].
pub async fn tokenize_stream<'a, T, S, E>(
    tok: &mut T,
    mut stream: S,
    charset: Charset,
) -> Result<(), E>
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
    S: Stream<Item = Result<(Input<'a>, Output<'a>), E>> + Unpin,
    Error: Into<E>,
{
    use futures_util::StreamExt as _;

    while let Some(item) = stream.next().await {
        let (input, output) = item?;

        tokenize(tok, input, output, charset)
            .await
            .map_err(Into::into)?;
    }

    Ok(())
}
