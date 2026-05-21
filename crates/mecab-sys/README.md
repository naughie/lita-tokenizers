# mecab-sys

**Rust FFI bindings for [MeCab](https://taku910.github.io/mecab/).**

`mecab-sys` provides raw, unsafe FFI bindings to the MeCab (C++) library.
This crate is designed to be completely self-contained: it bundles the MeCab source code, compiles it directly using the `cc` crate, and links it statically. **You do not need to install MeCab on your system to use this crate.**


**Only 1-best prediction mode is supported now.**
If you want to train a model, or if you want to predict N-best solutions, still you have to use MeCab.

MeCab is configured with UTF-8 only.

---


## Prerequisites

This crate **does not** bundle the MeCab model files. To actually use MeCab for text analysis, you must download (or train by yourself) a pre-trained model.

You will also need a standard C++ compiler (like `g++` or `clang++`) installed on your system so the `cc` crate can build the bundled C++ source.
To configure the compiler behavior, check the documentation of [cc](https://crates.io/crates/cc).



## Installation

Add `mecab-sys` to your `Cargo.toml`:

```toml
[dependencies]
mecab-sys = "0.1.0"
```


## Usage

Basic usage for Japanese morphological analysis:

```rust
use mecab_sys::Model;

fn main() {
    let model = Model::from_cli_arg(c"-d /path/to/your/dict -r /path/to/dictrc").unwrap();

    let tagger = model.new_tagger().unwrap();
    let mut lattice = model.new_lattice().unwrap();

    let mut lattice = lattice.set_sentence("すもももももももものうち");
    tagger.parse(&mut lattice).unwrap();

    for node in lattice.bos_node() {
        let surface = node.surface();
        let feat = node.feature();

        println!("{surface}: {feat}");
    }
}
```
