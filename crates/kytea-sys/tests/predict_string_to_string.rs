use kytea_sys::{CorpusFormat, KyTea, StringStream};

mod common;

#[test]
fn predict() {
    let model_path = common::get_and_setup_model_path().unwrap();
    let mut model = KyTea::new();

    model.read_model(&model_path).unwrap();

    model
        .config()
        .set_training(false)
        .set_input_format(CorpusFormat::Raw);

    let mut input = {
        let mut ss = StringStream::new();
        ss.push("すもももももももものうち\n");
        ss
    };
    let mut output = StringStream::new();

    let mut ctx = model.context(&mut input, &mut output).unwrap();
    while ctx.predict().unwrap().is_continue() {}

    let expected = "すもも/名詞/すもも も/助詞/も も/助詞/も も/助詞/も も/助詞/も も/助詞/も もの/名詞/もの うち/名詞/うち\n";

    assert_eq!(String::from_utf8_lossy(output.as_bytes()), expected);
}
