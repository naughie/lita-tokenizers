pub use super::TokenizerError;
pub use super::{Input, Output};
pub use crate::Error;

use tokenizers_core::TokenStreamBuilder;

use tokenizers_utils::Charset;
use tokenizers_utils::fs::sync_fs as tokenize_fs;
use tokenizers_utils::io::sync_io as tokenize_io;

use std::fs::File;
use std::io::{self, BufWriter};

use std::path::Path;

fn tokenize_impl<T, R, W>(tok: &mut T, rdr: R, wtr: W, charset: Charset) -> Result<(), Error>
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
    R: io::Read,
    W: io::Write,
{
    tokenize_io::tokenize(tok, rdr, wtr, charset).map_err(Into::into)
}

fn open_file(path: &Path) -> Result<File, Error> {
    File::open(path).map_err(Error::ReadIo)
}
fn create_file(path: &Path) -> Result<BufWriter<File>, Error> {
    File::create(path)
        .map_err(Error::WriteIo)
        .map(BufWriter::new)
}

/// Tokenizes the input, writing the tokenized results to the `output`.
///
/// [`Input::Stdin`] and [`Output::Stdout`] are treated just as regular files.
/// If either of `input`/`output` is `Stdin`/`Stdout`, then the other must be a regular file or
/// `Stdin`/`Stdout`.
/// Otherwise it redirects to [`tokenize_path()`](tokenize_fs::tokenize_path()).
pub fn tokenize<T>(
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
        (None, None) => tokenize_impl(tok, io::stdin(), io::stdout(), charset),
        (Some(input), Some(output)) => {
            tokenize_fs::tokenize_path(tok, input, output, charset).map_err(Into::into)
        }
        (Some(input), None) => tokenize_impl(tok, open_file(input)?, io::stdout(), charset),
        (None, Some(output)) => tokenize_impl(tok, io::stdin(), create_file(output)?, charset),
    }
}

/// Repeatedly invokes [`tokenize()`].
pub fn tokenize_stream<'a, T, S, E>(tok: &mut T, stream: S, charset: Charset) -> Result<(), E>
where
    T: TokenStreamBuilder,
    TokenizerError<T>: Into<Error>,
    S: Iterator<Item = Result<(Input<'a>, Output<'a>), E>> + Unpin,
    Error: Into<E>,
{
    for item in stream {
        let (input, output) = item?;

        tokenize(tok, input, output, charset).map_err(Into::into)?;
    }

    Ok(())
}
