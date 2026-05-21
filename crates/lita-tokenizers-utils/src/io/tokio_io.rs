//! Wrapper of [`Tokenizer`](tokenizers_core::Tokenizer) with [`tokio`](tokio::io) interface.

use tokenizers_core::TokenStreamBuilder;

use super::Error;
use crate::write::normalize_token_stream;
use crate::{Charset, decode};

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

use std::io::Error as IoError;

/// [`tokio`] version of [`sync_io::write_io()`](super::sync_io::write_io()).
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// use lita_tokenizers_utils::io::tokio_io::write;
/// // for test:
/// use lita_tokenizers_utils::noop_tokenizer::NoopTokenizer;
///
/// let mut tok = NoopTokenizer;
///
/// let input = b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.";
/// let mut output = Vec::new();
///
/// write(&mut tok, input, &mut output, Default::default()).await.unwrap();
///
/// assert_eq!(output, b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.\n");
/// # }
/// ```
pub async fn write<T, W>(
    tok: &mut T,
    bytes: &[u8],
    mut wtr: W,
    charset: Charset,
) -> Result<(), Error<T, IoError>>
where
    T: TokenStreamBuilder,
    W: AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt as _;

    let input = decode(bytes, charset).map_err(Error::unknown_encoding)?;

    let output = tok.tokenize(&input).map_err(Error::Tokenizer)?;
    let output = normalize_token_stream(output);

    let mut buf = [0u8; char::MAX_LEN_UTF8];

    for c in output.chars() {
        let buf = c.encode_utf8(&mut buf);
        wtr.write_all(buf.as_bytes())
            .await
            .map_err(Error::write_io)?;
    }

    wtr.flush().await.map_err(Error::write_io)?;

    Ok(())
}

/// Alias of [`write()`] with the input obtained by [`read_to_end()`](tokio::io::AsyncReadExt::read_to_end()).
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// use lita_tokenizers_utils::io::tokio_io::tokenize;
/// // for test:
/// use lita_tokenizers_utils::noop_tokenizer::NoopTokenizer;
///
/// let mut tok = NoopTokenizer;
///
/// let input = b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.";
/// let mut output = Vec::new();
///
/// tokenize(&mut tok, &input[..], &mut output, Default::default()).await.unwrap();
///
/// assert_eq!(output, b"The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog.\n");
/// # }
/// ```
pub async fn tokenize<T, R, W>(
    tok: &mut T,
    mut rdr: R,
    wtr: W,
    charset: Charset,
) -> Result<(), Error<T, IoError>>
where
    T: TokenStreamBuilder,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    use tokio::io::AsyncReadExt as _;

    let mut buf = Vec::new();
    rdr.read_to_end(&mut buf).await.map_err(Error::read_io)?;
    write(tok, &buf, wtr, charset).await
}
