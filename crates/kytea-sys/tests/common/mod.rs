use std::ffi::CString;
use std::path::{Path, PathBuf};

use kytea_sys::{IspanStream, KyTea, ModelFormat};

fn model_path_bin() -> PathBuf {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .join("tests/models")
        .canonicalize()
        .unwrap()
        .join("model.bin")
}

fn model_path_text() -> PathBuf {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .join("tests/models")
        .canonicalize()
        .unwrap()
        .join("model.txt")
}

pub fn model_bin_file() -> KyTea {
    let path = model_path_bin();
    let path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

    let mut model = KyTea::new();
    model.read_model(&path, ModelFormat::Unknown).unwrap();
    model
}

pub fn model_txt_file() -> KyTea {
    let path = model_path_text();
    let path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

    let mut model = KyTea::new();
    model.read_model(&path, ModelFormat::Unknown).unwrap();
    model
}

pub fn model_bin_span() -> KyTea {
    let path = model_path_bin();
    let content = std::fs::read(path).unwrap();
    let mut content = IspanStream::new(&content);

    let mut model = KyTea::new();
    model
        .read_model_from_stream(&mut content, ModelFormat::Binary)
        .unwrap();

    model
}

pub fn model_txt_span() -> KyTea {
    let path = model_path_text();
    let content = std::fs::read(path).unwrap();
    let mut content = IspanStream::new(&content);

    let mut model = KyTea::new();
    model
        .read_model_from_stream(&mut content, ModelFormat::Text)
        .unwrap();

    model
}
