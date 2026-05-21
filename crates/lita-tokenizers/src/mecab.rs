//! MeCab tokenizer APIs.

use super::{Error, Input, Output, common};
use common::TokenizerError;

use tokenizers_core::Tokenizer;

use tokenizers_utils::Charset;

use tokenizers_mecab::MeCab;
use tokenizers_mecab::MeCabTagger;
use tokenizers_mecab::TagOrder;
use tokenizers_mecab::{Noop, Selected};

use std::ffi::CString;
use std::path::Path;

/// Represents where the model files are.
///
/// We convert it to the MeCab CLI argument `-d {dict} -r {rc}\0`.
///
/// Note that these paths should not include whitespaces due to the limitation of MeCab.
pub struct ModelPath<'a> {
    pub dict: &'a Path,
    pub rc: &'a Path,
}

/// Represents which tags and in what order should be printed.
///
/// Suppose that MeCab output the following tags:
///
/// ```text
/// surface,tag1,tag2,tag3
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

impl<O: TagOrder> From<TokenizerError<MeCabTagger<'_, O>>> for Error {
    fn from(value: TokenizerError<MeCabTagger<'_, O>>) -> Self {
        Error::MeCab(value.0)
    }
}

async fn run_dispatched(
    input: Input<'_>,
    output: Output<'_>,
    charset: Charset,
    model: ModelPath<'_>,
    ord: impl TagOrder,
) -> Result<(), Error> {
    let mut tok = tokenizer(model, ord)?;

    let mut tok = tok.builder().map_err(Error::MeCab)?;

    common::tokenize(&mut tok, input, output, charset).await?;

    Ok(())
}

/// Initializes the [MeCab model](tokenizers_mecab::sys::Model) and reads the model.
pub fn tokenizer<O: TagOrder>(model: ModelPath<'_>, ord: O) -> Result<MeCab<O>, Error> {
    let arg = {
        let dict = model.dict.as_os_str().as_encoded_bytes();
        let rc = model.rc.as_os_str().as_encoded_bytes();

        let mut arg = Vec::with_capacity(dict.len() + rc.len() + 8);

        arg.extend_from_slice(b"-d ");
        arg.extend_from_slice(dict);
        arg.extend_from_slice(b" -r ");
        arg.extend_from_slice(rc);

        CString::new(arg).map_err(Error::MeCabModelPath)?
    };
    let tok = MeCab::from_cli_args(&arg, ord).map_err(Error::MeCabModelRead)?;
    Ok(tok)
}

/// Executes [`tokenize()`](crate::tokenize()) with MeCab initialized by [`tokenizer()`].
pub async fn run(
    input: Input<'_>,
    output: Output<'_>,
    charset: Charset,
    model: ModelPath<'_>,
    tag: TagIndex<'_>,
) -> Result<(), Error> {
    match tag {
        TagIndex::Asis => run_dispatched(input, output, charset, model, Noop).await,
        TagIndex::Specified(index) => {
            run_dispatched(input, output, charset, model, Selected { index }).await
        }
    }
}
