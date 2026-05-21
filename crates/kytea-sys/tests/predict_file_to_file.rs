use kytea_sys::{CorpusFormat, Fstream, KyTea};

mod common;

use std::ffi::CString;
use std::path::Path;

fn path_to_cstr(path: &Path) -> CString {
    CString::new(path.as_os_str().as_encoded_bytes()).unwrap()
}

#[test]
fn predict() {
    let model_path = common::get_and_setup_model_path().unwrap();
    let mut model = KyTea::new();

    model.read_model(&model_path).unwrap();

    model
        .config()
        .set_training(false)
        .set_input_format(CorpusFormat::Raw);

    let tempdir = tempfile::tempdir().unwrap();
    let input_path = tempdir.path().join("input");
    let output_path = tempdir.path().join("output");

    std::fs::write(&input_path, "すもももももももものうち\n").unwrap();

    let input_path_c = path_to_cstr(&input_path);
    let output_path_c = path_to_cstr(&output_path);

    let mut input = Fstream::open(&input_path_c).unwrap();
    let mut output = Fstream::create(&output_path_c).unwrap();

    let mut ctx = model.context(&mut input, &mut output).unwrap();
    while ctx.predict().unwrap().is_continue() {}
    output.flush().unwrap();

    let expected = "すもも/名詞/すもも も/助詞/も も/助詞/も も/助詞/も も/助詞/も も/助詞/も もの/名詞/もの うち/名詞/うち\n";

    assert_eq!(std::fs::read_to_string(&output_path).unwrap(), expected);
}
