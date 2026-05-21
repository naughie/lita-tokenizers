//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with `std` interface.

use tokenizers_core::TokenStreamBuilder;

use super::Error;
use crate::write::NormalizedTokenStream;
use crate::{Charset, decode};

use std::fmt::Error as FmtError;
use std::fmt::Write as FmtWrite;
use std::io::Error as IoError;
use std::io::Read;
use std::io::Write as IoWrite;

/// Reads the input `bytes`, decodes it as `charset`, then writes the tokenized output to `wtr`.
///
/// ```
/// use lita_tokenizers_utils::io::sync_io::write_fmt;
/// // for test:
/// use lita_tokenizers_utils::noop_tokenizer::NoopTokenizer;
///
/// let mut tok = NoopTokenizer;
///
/// let input = b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.";
/// let mut output = String::new();
///
/// write_fmt(&mut tok, input, &mut output, Default::default()).unwrap();
///
/// assert_eq!(output, "The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.\n");
/// ```
pub fn write_fmt<T, W>(
    tok: &mut T,
    bytes: &[u8],
    mut wtr: W,
    charset: Charset,
) -> Result<(), Error<T, FmtError>>
where
    T: TokenStreamBuilder,
    W: FmtWrite,
{
    let input = decode(bytes, charset).map_err(Error::unknown_encoding)?;

    let output = tok.tokenize(&input).map_err(Error::Tokenizer)?;
    let output = NormalizedTokenStream::new(output);

    write!(wtr, "{output}").map_err(Error::write_io)?;

    Ok(())
}

/// Reads the input `bytes`, decodes it as `charset`, then writes the tokenized output to `wtr`.
///
/// ```
/// use lita_tokenizers_utils::io::sync_io::write_io;
/// // for test:
/// use lita_tokenizers_utils::noop_tokenizer::NoopTokenizer;
///
/// let mut tok = NoopTokenizer;
///
/// let input = b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.";
/// let mut output = Vec::new();
///
/// write_io(&mut tok, input, &mut output, Default::default()).unwrap();
///
/// assert_eq!(output, b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.\n");
/// ```
pub fn write_io<T, W>(
    tok: &mut T,
    bytes: &[u8],
    mut wtr: W,
    charset: Charset,
) -> Result<(), Error<T, IoError>>
where
    T: TokenStreamBuilder,
    W: IoWrite,
{
    let input = decode(bytes, charset).map_err(Error::unknown_encoding)?;

    let output = tok.tokenize(&input).map_err(Error::Tokenizer)?;
    let output = NormalizedTokenStream::new(output);

    write!(wtr, "{output}").map_err(Error::write_io)?;

    wtr.flush().map_err(Error::write_io)?;

    Ok(())
}

/// Alias of [`write_io()`] with the input obtained by [`read_to_end()`](Read::read_to_end()).
///
/// ```
/// use lita_tokenizers_utils::io::sync_io::tokenize;
/// // for test:
/// use lita_tokenizers_utils::noop_tokenizer::NoopTokenizer;
///
/// let mut tok = NoopTokenizer;
///
/// let input = b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.";
/// let mut output = Vec::new();
///
/// tokenize(&mut tok, &input[..], &mut output, Default::default()).unwrap();
///
/// assert_eq!(output, b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.\n");
/// ```
pub fn tokenize<T, R, W>(
    tok: &mut T,
    mut rdr: R,
    wtr: W,
    charset: Charset,
) -> Result<(), Error<T, IoError>>
where
    T: TokenStreamBuilder,
    R: Read,
    W: IoWrite,
{
    let mut buf = Vec::new();
    rdr.read_to_end(&mut buf).map_err(Error::read_io)?;
    write_io(tok, &buf, wtr, charset)
}
