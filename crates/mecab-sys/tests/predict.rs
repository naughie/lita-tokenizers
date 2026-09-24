use mecab_sys::Model;

use std::ffi::CString;
use std::path::{Path, PathBuf};

fn model_path() -> PathBuf {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root.join("tests/models/dict").canonicalize().unwrap()
}

fn build_mecab_arg(dict: &Path) -> CString {
    let rc = dict.join("dicrc");
    let arg = format!("-d {} -r {}", dict.display(), rc.display());
    CString::new(arg).unwrap()
}

#[test]
fn predict() {
    use std::fmt::Write as _;

    let model = Model::from_cli_arg(&build_mecab_arg(&model_path())).unwrap();

    let tagger = model.new_tagger().unwrap();
    let mut lattice = model.new_lattice().unwrap();

    let mut lattice = lattice.set_sentence("すもももももももものうち");
    tagger.parse(&mut lattice).unwrap();

    let mut ret = String::new();
    for node in lattice.bos_node() {
        let surface = node.surface();
        let feat = node.feature();

        writeln!(ret, "{surface}: {feat}").ok();
    }

    let mut expected = [
        ": BOS/EOS,BOS/EOS",
        "すもも: 名詞,すもも",
        "も: 助詞,も",
        "もも: 名詞,もも",
        "も: 助詞,も",
        "もも: 名詞,もも",
        "の: 助詞,の",
        "うち: 名詞,うち",
        ": BOS/EOS,BOS/EOS",
    ]
    .join("\n");
    expected.push('\n');
    assert_eq!(ret, expected);
}

#[test]
fn cursor() {
    let model = Model::from_cli_arg(&build_mecab_arg(&model_path())).unwrap();

    let tagger = model.new_tagger().unwrap();
    let mut lattice = model.new_lattice().unwrap();

    let mut lattice = lattice.set_sentence("すもももももももものうち");
    tagger.parse(&mut lattice).unwrap();

    let mut cursor = lattice.bos_node();
    assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));

    cursor.move_next();
    let first_node = cursor.curr().unwrap();
    assert_eq!(first_node.surface(), "すもも");

    cursor.move_prev();
    assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));

    cursor.move_next();
    assert_eq!(first_node.surface(), cursor.curr().unwrap().surface());

    cursor.move_next();
    cursor.move_next();
    cursor.move_next();
    cursor.move_next();
    cursor.move_next();
    cursor.move_next();
    assert_eq!(cursor.curr().unwrap().surface(), "うち");

    cursor.move_next();
    assert!(cursor.curr().is_some_and(|node| node.kind().is_eos()));

    cursor.move_next();
    assert!(cursor.curr().is_none());

    cursor.move_prev();
    assert!(cursor.curr().is_none());

    let mut cursor = lattice.bos_node();
    while cursor.curr().is_some_and(|node| !node.kind().is_eos()) {
        cursor.move_next();
    }
    assert!(cursor.curr().is_some_and(|node| node.kind().is_eos()));

    cursor.move_next();
    assert!(cursor.curr().is_none());
    cursor.move_prev();
    assert!(cursor.curr().is_none());

    let mut cursor = lattice.bos_node();
    assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
    cursor.move_prev();
    assert!(cursor.curr().is_none());
    cursor.move_next();
    assert!(cursor.curr().is_none());

    let mut cursor = lattice.bos_node();
    for _ in &mut cursor {}
    assert!(cursor.curr().is_none());
    cursor.move_prev();
    assert!(cursor.curr().is_none());
}
