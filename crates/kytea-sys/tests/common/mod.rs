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

pub fn get_and_setup_model_path() -> IoResult<CString> {
    let path = model_path();
    setup_model(&path).unwrap();

    Ok(CString::new(path.as_os_str().as_encoded_bytes()).unwrap())
}
