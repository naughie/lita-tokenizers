use super::*;

use std::ffi::CString;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

fn model_path() -> PathBuf {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .join("../../tests/kytea")
        .canonicalize()
        .unwrap()
        .join("model.bin")
}

fn setup_model(unpacked_path: &Path) -> IoResult<()> {
    use flate2::read::GzDecoder;

    use std::fs::File;
    use std::io::BufReader;

    let gz_path = unpacked_path.with_added_extension("gz");
    let ch_path = unpacked_path.with_added_extension("checksum");

    let lock_path = unpacked_path.with_added_extension("lock");

    let bytes = {
        use std::io::Read as _;

        let mut bytes = Vec::new();
        let mut gz = GzDecoder::new(BufReader::new(File::open(&gz_path)?));
        gz.read_to_end(&mut bytes)?;
        bytes
    };
    let checksum = blake3::hash(&bytes).to_hex();

    let lock_file = File::options()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .unwrap();
    lock_file.lock().unwrap();

    if ch_path.try_exists()?
        && unpacked_path.try_exists()?
        && let found_checksum = std::fs::read(&ch_path)?
        && found_checksum == checksum.as_bytes()
    {
    } else {
        std::fs::write(unpacked_path, &bytes)?;
        std::fs::write(&ch_path, checksum.as_bytes())?;
    }

    lock_file.unlock().unwrap();

    Ok(())
}

#[test]
fn tokenize() {
    let model_path = {
        let path = model_path();
        setup_model(&path).unwrap();

        &CString::new(path.as_os_str().as_encoded_bytes()).unwrap()
    };

    let mut model = KyTea::new();

    model
        .config()
        .set_debug(DebugLevel::Silent)
        .set_training(false)
        .set_word_bound(c" ")
        .set_input_format(CorpusFormat::Raw);

    model.read_model(model_path).unwrap();

    let mut input = StringStream::new();
    input.push("すもももももももものうち．\n");
    input.push("すもももももももものうち．\n");
    input.push("すもももももももものうち．\n");

    let mut output = StringStream::new();

    let mut ctx = model.context(&mut input, &mut output).unwrap();
    while ctx.predict().unwrap().is_continue() {}

    let bytes = output.as_bytes();
    let expected = [
        "すもも/名詞/すもも",
        "も/助詞/も",
        "も/助詞/も",
        "も/助詞/も",
        "も/助詞/も",
        "も/助詞/も",
        "もの/名詞/もの",
        "うち/名詞/うち",
        "．/補助記号/。\n",
    ]
    .join(" ")
    .repeat(3);

    assert_eq!(
        bytes,
        expected.as_bytes(),
        "prediction failed: {}",
        String::from_utf8_lossy(bytes)
    );
}

#[test]
fn read_model_err() {
    let mut model = KyTea::new();
    assert!(model.read_model(c"non_existing_model").is_err());

    let mut model = KyTea::new();
    assert!(model.read_model(c"./Cargo.toml").is_err());
}
