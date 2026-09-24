use kytea_sys::{CorpusFormat, KyTea, StringStream};

mod common;

#[test]
fn predict() {
    predict_impl(common::model_bin_file());
    predict_impl(common::model_txt_file());
    predict_impl(common::model_bin_span());
    predict_impl(common::model_txt_span());
}

fn predict_impl(mut model: KyTea) {
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
