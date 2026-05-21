//! Whitespace tokenizer APIs.

use super::{Error, Input, Output, common};
use common::TokenizerError;

use tokenizers_whitespace::Whitespace;

use tokenizers_utils::Charset;

/// Initializes the [whitespace model](Whitespace).
pub fn tokenizer() -> Result<Whitespace, Error> {
    Ok(Whitespace)
}

impl From<TokenizerError<Whitespace>> for Error {
    fn from(value: TokenizerError<Whitespace>) -> Self {
        match value.0 {}
    }
}

/// Executes [`tokenize()`](crate::tokenize()) with whitespace tokenizer
/// initialized by [`tokenizer()`].
pub async fn run(input: Input<'_>, output: Output<'_>, charset: Charset) -> Result<(), Error> {
    let mut tok = Whitespace;

    common::tokenize(&mut tok, input, output, charset).await?;

    Ok(())
}
