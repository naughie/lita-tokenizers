use std::path::Path;

type Error = Box<dyn std::error::Error>;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const C_LIB_SRC: &str = "mecab-src";

fn compile() -> Result<(), Error> {
    let c_root = Path::new(ROOT).join(C_LIB_SRC);
    let c_src = c_root.join("src");
    println!("cargo::rerun-if-changed={}", c_src.display());

    let rc_env = "MECAB_DEFAULT_RC";
    println!("cargo::rerun-if-env-changed={rc_env}");

    let default_rc = std::env::var(rc_env)
        .map(|s| format!("\"{s}\""))
        .unwrap_or_else(|_| "\"/usr/local/etc/mecabrc\"".to_owned());

    let files = {
        use std::ffi::OsStr;

        let excluded = [
            "mecab.cpp",
            "mecab-cost-train.cpp",
            "mecab-dict-gen.cpp",
            "mecab-dict-index.cpp",
            "mecab-system-eval.cpp",
            "mecab-test-gen.cpp",
        ]
        .map(OsStr::new);

        let mut cpp_files = Vec::new();

        let cpp = OsStr::new("cpp");

        for entry in std::fs::read_dir(&c_src)? {
            let entry = entry?;
            let fname = entry.file_name();

            if Path::new(&fname).extension() == Some(cpp) && !excluded.contains(&&*fname) {
                let path = entry.path();

                cpp_files.push(path);
            }
        }

        cpp_files
    };

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++14")
        .warnings(false)
        .files(files)
        // for `config.h`
        .include(&c_root)
        .include(&c_src)
        .cpp_link_stdlib_static(cfg!(target_env = "musl"))
        .define("HAVE_CONFIG_H", "1")
        .define("DIC_VERSION", "102")
        .define("MECAB_USE_UTF8_ONLY", "")
        .define("MECAB_DEFAULT_RC", &*default_rc);

    if std::env::var("CARGO_CFG_TARGET_ENDIAN").is_ok_and(|endian| endian == "big") {
        build.define("WORDS_BIGENDIAN", "1");
    }

    build.compile("mecab");

    println!("cargo:rustc-link-lib=static=mecab");

    Ok(())
}

fn main() -> Result<(), Error> {
    println!("cargo::rerun-if-changed=build.rs");

    compile()?;

    Ok(())
}
