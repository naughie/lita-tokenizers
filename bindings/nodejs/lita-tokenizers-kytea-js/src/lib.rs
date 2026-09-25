use napi::bindgen_prelude::*;
use napi_derive::napi;

use kytea_sys::ModelFormat;
use tokenizers_core::delim;

use std::ffi::CString;

fn to_napi_err<E: std::fmt::Display>(e: E) -> Error {
    Error::new(Status::GenericFailure, format!("{e}"))
}

#[napi]
pub struct KyTea(kytea_sys::KyTea);

#[napi]
pub type Stream<'a> = Either<&'a mut StringStream, &'a mut FileStream>;

#[napi]
impl KyTea {
    #[napi(constructor)]
    pub fn new() -> Self {
        use kytea_sys::{CorpusFormat, DebugLevel};

        let mut ffi = kytea_sys::KyTea::new();
        ffi.config()
            .set_debug(DebugLevel::Silent)
            .set_training(false)
            .set_input_format(CorpusFormat::Raw)
            .set_word_bound(delim::TOKEN_CSTR)
            .set_tag_bound(delim::TAG_CSTR)
            .set_elem_bound(delim::TOKEN_CSTR)
            .set_escape(delim::ESCAPE_CSTR);

        Self(ffi)
    }

    #[napi]
    pub fn read_model(&mut self, path: String) -> Result<()> {
        let model_c = CString::new(path).map_err(|_| {
            Error::new(
                Status::InvalidArg,
                "could not convert the model path to CString",
            )
        })?;

        self.0
            .read_model(&model_c, ModelFormat::Unknown)
            .map_err(to_napi_err)?;

        Ok(())
    }

    #[napi]
    pub fn read_model_from_bytes(&mut self, bytes: Buffer, format: String) -> Result<()> {
        use kytea_sys::IspanStream;

        let format = match format.as_str() {
            "t" | "txt" | "text" => ModelFormat::Text,
            "b" | "bin" | "binary" => ModelFormat::Binary,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "model format is either \"t\" (text) or \"b\" (binary)",
                ));
            }
        };

        let bytes: &[u8] = &bytes;
        let mut stream = IspanStream::new(bytes);
        self.0
            .read_model_from_stream(&mut stream, format)
            .map_err(to_napi_err)?;

        Ok(())
    }

    #[napi]
    pub fn tokenize(&mut self, input: Stream<'_>, output: Stream<'_>) -> Result<()> {
        let input_stream: &mut dyn kytea_sys::Stream = match input {
            Either::A(s) => {
                if !s.ffi.as_bytes().ends_with(b"\n") {
                    s.ffi.push("\n");
                }
                &mut s.ffi
            }
            Either::B(s) => {
                if !s.input {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "input file must be opened with the \"r\" mode",
                    ));
                }
                &mut s.ffi
            }
        };

        let output_stream: &mut dyn kytea_sys::Stream = match output {
            Either::A(s) => &mut s.ffi,
            Either::B(s) => {
                if s.input {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "output file must be opened with the \"w\" or \"a\" mode",
                    ));
                }
                &mut s.ffi
            }
        };

        let mut input_stream = input_stream;
        let mut output_stream = output_stream;

        let mut ctx = self
            .0
            .context(&mut input_stream, &mut output_stream)
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("tokenization config is wrong: {e}"),
                )
            })?;

        while ctx.predict().map_err(to_napi_err)?.is_continue() {}

        Ok(())
    }
}

#[napi]
pub struct StringStream {
    ffi: kytea_sys::StringStream,
}

#[napi]
impl StringStream {
    #[napi(constructor)]
    pub fn new(string: Option<String>) -> Self {
        let mut ffi = kytea_sys::StringStream::new();
        if let Some(s) = string {
            ffi.push(&s);
        }
        Self { ffi }
    }

    #[napi]
    pub fn as_str(&self) -> Result<String> {
        std::str::from_utf8(self.ffi.as_bytes())
            .map(str::to_string)
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("tokenization output is invalid UTF-8: {e}"),
                )
            })
    }
}

#[napi]
pub struct FileStream {
    ffi: kytea_sys::Fstream,
    input: bool,
}

#[napi]
impl FileStream {
    #[napi(constructor)]
    pub fn new(path: String, mode: String) -> Result<Self> {
        let path_c = CString::new(path).map_err(|_| {
            Error::new(
                Status::InvalidArg,
                "could not convert the file path to CString",
            )
        })?;

        let (ffi, input) = match mode.as_str() {
            "r" => (
                kytea_sys::Fstream::open(&path_c).map_err(to_napi_err)?,
                true,
            ),
            "w" => (
                kytea_sys::Fstream::create(&path_c).map_err(to_napi_err)?,
                false,
            ),
            "a" => (
                kytea_sys::Fstream::append(&path_c).map_err(to_napi_err)?,
                false,
            ),
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    r#"file mode must be either "r", "w", or "a""#,
                ));
            }
        };

        Ok(Self { ffi, input })
    }

    #[napi]
    pub fn flush(&mut self) -> Result<()> {
        self.ffi.flush().map_err(to_napi_err)?;
        Ok(())
    }
}
