//! `mecab-sys` is the FFI wrapper for [MeCab](https://taku910.github.io/mecab/).
//! While [`ffi`] provides the FFI bindings, the top-level items are the safe wrapper for MeCab.
//!
//! # Examples
//!
//! ```
//! # fn analyze() {
//! use mecab_sys::Model;
//!
//! let model = Model::from_cli_arg(c"-d /path/to/your/dict -r /path/to/dictrc").unwrap();
//!
//! let tagger = model.new_tagger().unwrap();
//! let mut lattice = model.new_lattice().unwrap();
//!
//! let mut lattice = lattice.set_sentence("すもももももももものうち");
//! tagger.parse(&mut lattice).unwrap();
//!
//! for node in lattice.bos_node() {
//!     let surface = node.surface();
//!     let feat = node.feature();
//!
//!     println!("{surface}: {feat}");
//! }
//! # }
//! ```
//!
//! ## Cursor API
//!
//! [`lattice.bos_node()`](`LatticeGuard::bos_node()`) returns a [`NodeCursor`], which can be
//! used either as an iterator and as a cursor of [`Node`]s.
//!
//! ```
//! # fn analyze() {
//! use mecab_sys::Model;
//!
//! let model = Model::from_cli_arg(c"-d /path/to/your/dict -r /path/to/dictrc").unwrap();
//!
//! let tagger = model.new_tagger().unwrap();
//! let mut lattice = model.new_lattice().unwrap();
//!
//! let mut lattice = lattice.set_sentence("すもももももももものうち");
//! tagger.parse(&mut lattice).unwrap();
//!
//! let mut cursor = lattice.bos_node();
//! assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
//!
//! cursor.move_next();
//! cursor.move_next();
//! if let Some(node) = cursor.curr() {
//!     let surface = node.surface();
//!     let feat = node.feature();
//!
//!     println!("{surface}: {feat}");
//! }
//!
//! cursor.move_prev();
//! if let Some(node) = cursor.curr() {
//!     let surface = node.surface();
//!     let feat = node.feature();
//!
//!     println!("{surface}: {feat}");
//! }
//! # }
//! ```
//!
//! After you call `move_next()` on the EoS node, call `move_prev()` on the BoS node, or consume as
//! an iterator, then the cursor shifted to the "dead" state, never being back to the alive state.
//!
//! ```
//! # use mecab_sys::LatticeGuard;
//! # fn analyze(lattice: &LatticeGuard<'_, '_, '_>) {
//! let mut cursor = lattice.bos_node();
//! while cursor.curr().is_some_and(|node| !node.kind().is_eos()) {
//!     cursor.move_next();
//! }
//! assert!(cursor.curr().is_some_and(|node| node.kind().is_eos()));
//!
//! // Call `move_next()` on the EoS node
//! cursor.move_next();
//! assert!(cursor.curr().is_none());
//!
//! // Never back to the original node
//! cursor.move_prev();
//! assert!(cursor.curr().is_none());
//!
//! let mut cursor = lattice.bos_node();
//! assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
//!
//! // Call `move_prev()` on the BoS node
//! cursor.move_prev();
//! assert!(cursor.curr().is_none());
//!
//! // Never back to the original node
//! cursor.move_next();
//! assert!(cursor.curr().is_none());
//!
//! let mut cursor = lattice.bos_node();
//! // Consume the iterator, reaching at the EoS node
//! for _ in &mut cursor {}
//! assert!(cursor.curr().is_none());
//!
//! // Never back to alive nodes
//! cursor.move_prev();
//! assert!(cursor.curr().is_none());
//! # }
//! ```

/// The raw FFI layer for MeCab, generated from `mecab.h`.
pub mod ffi;

#[cfg(test)]
mod tests;

mod model;
pub use model::Model;

mod tagger;
pub use tagger::{BoundaryConstraintType, RequestType};
pub use tagger::{Lattice, LatticeGuard, Tagger};

mod node;
pub use node::{LcAttr, RcAttr};
pub use node::{Node, NodeCursor, NodeKind};

pub use errors::Error;
mod errors {
    use super::Lattice;
    use super::ffi;

    use std::ffi::CStr;
    use std::fmt;

    fn get_mecab_error(mecab: *mut ffi::mecab_t) -> String {
        unsafe {
            let err_ptr = ffi::mecab_strerror(mecab);
            if err_ptr.is_null() {
                return "unknown MeCab error".to_string();
            }
            CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
        }
    }

    fn get_lattice_error(lattice: *mut ffi::mecab_lattice_t) -> String {
        unsafe {
            let err_ptr = ffi::mecab_lattice_strerror(lattice);
            if err_ptr.is_null() {
                return "unknown MeCab lattice error".to_string();
            }
            CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
        }
    }

    fn get_global_error() -> String {
        get_mecab_error(std::ptr::null_mut())
    }

    /// Represents an error returned by MeCab.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Error(String);

    impl Error {
        pub(super) fn global() -> Self {
            Self(get_global_error())
        }

        pub(super) fn with_lattice(lattice: &Lattice) -> Self {
            Self(get_lattice_error(lattice.as_ptr()))
        }

        /// Returns the underlying error message.
        pub fn into_inner(self) -> String {
            self.0
        }

        /// Returns the reference to the underlying error message.
        pub fn to_inner(&self) -> &str {
            &self.0
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for Error {}
}
