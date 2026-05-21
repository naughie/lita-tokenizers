//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with `std` filesystem.

use tokenizers_core::TokenStreamBuilder;

use super::Error;
use crate::Charset;
use crate::io::sync_io as io;

use std::fs::{self, File};
use std::io::BufWriter;

use std::path::Path;

/// Alias of [`io::tokenize()`], wrapping the output file by [`BufWriter`].
pub fn tokenize_file<T: TokenStreamBuilder>(
    tok: &mut T,
    input: File,
    output: File,
    charset: Charset,
) -> Result<(), Error<T>> {
    io::tokenize(tok, input, BufWriter::new(output), charset)?;
    Ok(())
}

/// tokenizes files recursively in the directory into the output directory.
///
/// If the `output` does not exist, it creates the directory first.
///
/// Essentially it calls [`tokenize_file()`] with `{input}/path/to/descendant` and
/// `{output}/path/to/descendant`.
pub fn tokenize_dir<T: TokenStreamBuilder>(
    tok: &mut T,
    input: &Path,
    output: &Path,
    charset: Charset,
) -> Result<(), Error<T>> {
    fs::create_dir_all(output).map_err(Error::create_dir)?;

    let read_dir = fs::read_dir(input).map_err(Error::read_dir)?;
    let mut stack = vec![(read_dir, output.to_path_buf())];

    'a: while let Some((read_dir, output_dir)) = stack.last_mut() {
        for entry in read_dir {
            let entry = entry.map_err(Error::read_dir)?;
            let input = entry.path();
            let output = output_dir.join(entry.file_name());

            let metadata = fs::metadata(&input).map_err(Error::file_metadata)?;

            if metadata.is_dir() {
                fs::create_dir(&output).map_err(Error::create_dir)?;
                let read_dir = fs::read_dir(&input).map_err(Error::read_dir)?;
                stack.push((read_dir, output));
                continue 'a;
            } else if metadata.is_file() {
                let input = fs::File::open(&input).map_err(Error::open_file)?;
                let output = fs::File::create(&output).map_err(Error::create_file)?;

                tokenize_file(tok, input, output, charset)?;
            }
        }

        stack.pop();
    }

    Ok(())
}

/// Calls [`tokenize_file()`] or [`tokenize_dir()`] depending on the [`Metadata`](std::fs::Metadata)
/// of the input.
pub fn tokenize_path<T: TokenStreamBuilder>(
    tok: &mut T,
    input: &Path,
    output: &Path,
    charset: Charset,
) -> Result<(), Error<T>> {
    let metadata = fs::metadata(input).map_err(Error::file_metadata)?;

    if metadata.is_dir() {
        tokenize_dir(tok, input, output, charset)?;
    } else {
        let input = fs::File::open(input).map_err(Error::open_file)?;
        let output = fs::File::create(output).map_err(Error::create_file)?;

        tokenize_file(tok, input, output, charset)?;
    }

    Ok(())
}
