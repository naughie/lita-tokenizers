//! KyTea tokenizer APIs.

use super::{Error, Input, Output, common};
use common::TokenizerError;

use tokenizers_core::Tokenizer;

use tokenizers_utils::Charset;

use tokenizers_kytea::KyTeaStreamBuilder;
use tokenizers_kytea::TagOrder;
use tokenizers_kytea::{KyTea, Noop, Selected};

use std::ffi::CString;
use std::path::Path;

/// Represents which tags and in what order should be printed.
///
/// Suppose that KyTea output the following tags:
///
/// ```text
/// surface/tag1/tag2/tag3
/// ```
///
/// `TagIndex::Asis` means the output are left as-is:
///
/// ```text
/// # TagIndex::Asis
/// # output: unchanged
/// surface/tag1/tag2/tag3
/// ```
///
/// On the other hand, `TagIndex::Specified` means only the selected tags will be printed.
/// For example, `TagIndex::Specified(&[])` results in
///
/// ```text
/// # TagIndex::Specified(&[])
/// # output: no tags
/// surface
/// ```
///
/// `TagIndex::Specified(&[2, 1, 0, 1, 0, 0, 2])` outputs
///
/// ```text
/// # TagIndex::Specified(&[2, 1, 0, 1, 0, 0, 2])
/// # output: with specified tags
/// surface/tag3/tag2/tag1/tag2/tag1/tag1/tag3
/// ```
///
/// The out-of-bound index corresponds to an empty string:
///
/// ```text
/// # TagIndex::Specified(&[3, 0, 3])
/// # output: with empty tags
/// surface//tag1/
/// ```
pub enum TagIndex<'a> {
    Asis,
    Specified(&'a [usize]),
}

impl<O: TagOrder> From<TokenizerError<KyTeaStreamBuilder<'_, O>>> for Error {
    fn from(value: TokenizerError<KyTeaStreamBuilder<'_, O>>) -> Self {
        Error::KyTea(value.0)
    }
}

async fn run_dispatched(
    input: Input<'_>,
    output: Output<'_>,
    charset: Charset,
    model: &Path,
    ord: impl TagOrder,
) -> Result<(), Error> {
    let mut tok = tokenizer(model, ord)?;
    let mut tok = tok.builder().map_err(Error::KyTea)?;

    common::tokenize(&mut tok, input, output, charset).await?;

    Ok(())
}

/// Initializes the [KyTea model](tokenizers_kytea::sys::KyTea) and reads the model.
pub fn tokenizer<O: TagOrder>(model: &Path, ord: O) -> Result<KyTea<O>, Error> {
    let model =
        CString::new(model.as_os_str().as_encoded_bytes()).map_err(Error::KyTeaModelPath)?;
    let mut tok = KyTea::new(ord);
    tok.model()
        .read_model(&model)
        .map_err(Error::KyTeaModelRead)?;
    Ok(tok)
}

/// Executes [`tokenize()`](crate::tokenize()) with KyTea initialized by [`tokenizer()`].
pub async fn run(
    input: Input<'_>,
    output: Output<'_>,
    charset: Charset,
    model: &Path,
    tag: TagIndex<'_>,
) -> Result<(), Error> {
    match tag {
        TagIndex::Asis => run_dispatched(input, output, charset, model, Noop).await,
        TagIndex::Specified(index) => {
            run_dispatched(input, output, charset, model, Selected { index }).await
        }
    }
}
