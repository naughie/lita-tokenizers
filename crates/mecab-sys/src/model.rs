use super::ffi;
use super::{Error, Lattice, Tagger};
use super::{LcAttr, RcAttr};

use std::ffi::CStr;
use std::ptr::NonNull;

/// A thread-safe, shared MeCab model. It wraps C `mecab_model_t`.
pub struct Model {
    pub(crate) inner: NonNull<ffi::mecab_model_t>,
}

impl Model {
    /// Returns the raw pointer to the underlying [`mecab_model_t`](ffi::mecab_model_t).
    ///
    /// The pointer is guaranteed to be [`NonNull`].
    pub fn as_ptr(&self) -> *mut ffi::mecab_model_t {
        self.inner.as_ptr()
    }

    /// Creates a new [`Model`] from a CLI argument string.
    ///
    /// It wraps `mecab_model_new2()`, the wrapper of `MeCab::Model::create(arg)`.
    ///
    /// ```
    /// # fn new_model() {
    /// use mecab_sys::Model;
    ///
    /// let model = Model::from_cli_arg(c"-d /path/to/your/dict -r /path/to/dictrc").unwrap();
    /// # }
    /// ```
    pub fn from_cli_arg(arg: &CStr) -> Result<Self, Error> {
        unsafe {
            let inner = ffi::mecab_model_new2(arg.as_ptr());
            NonNull::new(inner)
                .map(|inner| Self { inner })
                .ok_or_else(Error::global)
        }
    }

    /// Creates a new [`Tagger`] from this model. It is the alias of [`Tagger::new()`].
    pub fn new_tagger(&self) -> Result<Tagger<'_>, Error> {
        Tagger::new(self)
    }

    /// Creates a new [`Lattice`] from this model. It is the alias of [`Lattice::new()`].
    pub fn new_lattice(&self) -> Result<Lattice<'_>, Error> {
        Lattice::new(self)
    }

    /// Returns the transition cost between two attribute IDs.
    ///
    /// It wraps `mecab_model_transition_cost()`, the wrapper of `MeCab::Model::transition_cost()`.
    pub fn transition_cost(&self, rc_attr: RcAttr, lc_attr: LcAttr) -> i32 {
        unsafe {
            ffi::mecab_model_transition_cost(self.as_ptr(), rc_attr.to_raw(), lc_attr.to_raw())
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            ffi::mecab_model_destroy(self.as_ptr());
        }
    }
}

unsafe impl Sync for Model {}
unsafe impl Send for Model {}
