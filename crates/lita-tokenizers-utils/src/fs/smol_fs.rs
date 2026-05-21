//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with [`smol`](async_fs) filesystem.

use tokenizers_core::TokenStreamBuilder;

use super::Error;
use crate::Charset;
use crate::io::async_io as io;

use async_fs::{self as fs, File};
use futures_util::io::BufWriter;

use std::path::Path;

/// Alias of [`io::tokenize()`], wrapping the output file by [`BufWriter`].
pub async fn tokenize_file<T: TokenStreamBuilder>(
    tok: &mut T,
    input: File,
    output: File,
    charset: Charset,
) -> Result<(), Error<T>> {
    io::tokenize(tok, input, BufWriter::new(output), charset).await?;
    Ok(())
}

/// tokenizes files recursively in the directory into the output directory.
///
/// If the `output` does not exist, it creates the directory first.
///
/// Essentially it calls [`tokenize_file()`] with `{input}/path/to/descendant` and
/// `{output}/path/to/descendant`.
pub async fn tokenize_dir<T: TokenStreamBuilder>(
    tok: &mut T,
    input: &Path,
    output: &Path,
    charset: Charset,
) -> Result<(), Error<T>> {
    use futures_util::StreamExt as _;

    fs::create_dir_all(output)
        .await
        .map_err(Error::create_dir)?;

    let read_dir = fs::read_dir(input).await.map_err(Error::read_dir)?;
    let mut stack = vec![(read_dir, output.to_path_buf())];

    'a: while let Some((read_dir, output_dir)) = stack.last_mut() {
        while let Some(entry) = read_dir.next().await {
            let entry = entry.map_err(Error::read_dir)?;
            let input = entry.path();
            let output = output_dir.join(entry.file_name());

            let metadata = fs::metadata(&input).await.map_err(Error::file_metadata)?;

            if metadata.is_dir() {
                fs::create_dir(&output).await.map_err(Error::create_dir)?;
                let read_dir = fs::read_dir(&input).await.map_err(Error::read_dir)?;
                stack.push((read_dir, output));
                continue 'a;
            } else if metadata.is_file() {
                let input = fs::File::open(&input).await.map_err(Error::open_file)?;
                let output = fs::File::create(&output)
                    .await
                    .map_err(Error::create_file)?;

                tokenize_file(tok, input, output, charset).await?;
            }
        }

        stack.pop();
    }

    Ok(())
}

/// Calls [`tokenize_file()`] or [`tokenize_dir()`] depending on the [`Metadata`](std::fs::Metadata)
/// of the input.
pub async fn tokenize_path<T: TokenStreamBuilder>(
    tok: &mut T,
    input: &Path,
    output: &Path,
    charset: Charset,
) -> Result<(), Error<T>> {
    let metadata = fs::metadata(input).await.map_err(Error::file_metadata)?;

    if metadata.is_dir() {
        tokenize_dir(tok, input, output, charset).await?;
    } else {
        let input = fs::File::open(input).await.map_err(Error::open_file)?;
        let output = fs::File::create(output).await.map_err(Error::create_file)?;

        tokenize_file(tok, input, output, charset).await?;
    }

    Ok(())
}
