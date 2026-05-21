use super::*;

use std::ffi::CString;
use std::path::{Path, PathBuf};

fn dict_path() -> PathBuf {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .join("../../tests/mecab/dict")
        .canonicalize()
        .unwrap()
}

fn build_mecab_arg(dict: &Path) -> CString {
    let rc = dict.join("dicrc");
    let arg = format!("-d {} -r {}", dict.display(), rc.display());
    CString::new(arg).unwrap()
}

#[test]
fn tokenize() {
    let dict = dict_path();
    let arg = build_mecab_arg(&dict);
    let model = Model::from_cli_arg(&arg);
    assert!(
        model.is_ok(),
        "could not initialize the model: {:?}",
        model.map(|_| ()).unwrap_err()
    );

    let model = model.unwrap();
    let tagger = model.new_tagger().unwrap();
    let mut lattice = model.new_lattice().unwrap();

    {
        let mut lattice = lattice.set_sentence("すもももももももものうち");
        assert_eq!(lattice.sentence(), "すもももももももものうち");

        tagger.parse(&mut lattice).unwrap();

        let mut cursor = lattice.bos_node();

        let expected = [
            ("すもも", "名詞", "すもも"),
            ("も", "助詞", "も"),
            ("もも", "名詞", "もも"),
            ("も", "助詞", "も"),
            ("もも", "名詞", "もも"),
            ("の", "助詞", "の"),
            ("うち", "名詞", "うち"),
        ]
        .into_iter()
        .collect::<Vec<_>>();

        assert_eq!(cursor.curr().map(Node::kind), Some(NodeKind::Bos));
        cursor.move_next();

        let mut found = Vec::new();
        for _ in 0..7 {
            let node = cursor.curr().unwrap();

            let feat = node.feature();
            let mut feat = feat.split(',');

            let pos = feat.next().unwrap();
            let reading = feat.nth(5).unwrap();

            found.push((node.surface(), pos, reading));
            cursor.move_next();
        }

        assert_eq!(cursor.curr().map(Node::kind), Some(NodeKind::Eos));
        cursor.move_next();
        assert_eq!(cursor.curr().map(Node::kind), None);

        assert_eq!(expected, found);
    }
}
