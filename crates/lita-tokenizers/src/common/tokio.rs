pub use super::TokenizerError;
pub use super::{Input, Output};
pub use crate::Error;

use tokenizers_core::TokenStreamBuilder;

use tokenizers_utils::Charset;
use tokenizers_utils::fs::tokio_fs as tokenize_fs;
use tokenizers_utils::io::tokio_io as tokenize_io;

use tokio::fs::File;
use tokio::io::{self, BufWriter};

use futures_util::Stream;

use std::path::Path;

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
