use super::ffi;
use super::{Error, Model, Node, NodeCursor};

use bitflags::bitflags;

use std::ffi::CStr;
use std::ffi::c_char;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A MeCab tagger for analyzing text. It wraps C `mecab_t`.
///
/// The lifetime parameter is bound to [`Model`](`Model::new_tagger()`).
pub struct Tagger<'a> {
    inner: NonNull<ffi::mecab_t>,
    _marker: PhantomData<&'a Model>,
}

impl<'a> Tagger<'a> {
    /// Returns the raw pointer to the underlying [`mecab_t`](ffi::mecab_t).
    ///
    /// The pointer is guaranteed to be [`NonNull`].
    pub fn as_ptr(&self) -> *mut ffi::mecab_t {
        self.inner.as_ptr()
    }

    /// Creates a new [`Tagger`] from a [`Model`].
    ///
    /// It wraps `mecab_model_new_tagger()`, the wrapper of `MeCab::Model::createTagger()`.
    pub fn new(model: &'a Model) -> Result<Self, Error> {
        unsafe {
            let inner = ffi::mecab_model_new_tagger(model.inner.as_ptr());

            NonNull::new(inner)
                .map(|inner| Self {
                    inner,
                    _marker: PhantomData,
                })
                .ok_or_else(Error::global)
        }
    }

    /// Parses the sentence set in the [`Lattice`].
    ///
    /// It wraps `mecab_parse_lattice()`, the wrapper of `MeCab::Tagger::parse(MeCab::Lattice *lattice)`.
    pub fn parse(&self, lattice: &mut LatticeGuard<'_, '_, '_>) -> Result<(), Error> {
        unsafe {
            let result = ffi::mecab_parse_lattice(self.as_ptr(), lattice.lattice.as_ptr());
            if result == 0 {
                Err(Error::with_lattice(lattice.as_ref()))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for Tagger<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::mecab_destroy(self.as_ptr());
        }
    }
}

unsafe impl Send for Tagger<'_> {}

/// A MeCab lattice representing a search space for morphological analysis.
/// It wraps C `mecab_lattice_t`.
///
/// The lifetime parameter is bound to [`Model`](`Model::new_lattice()`).
pub struct Lattice<'a> {
    inner: NonNull<ffi::mecab_lattice_t>,
    _marker: PhantomData<&'a Model>,
}

impl<'a> Lattice<'a> {
    /// Returns the raw pointer to the underlying [`mecab_lattice_t`](ffi::mecab_lattice_t).
    ///
    /// The pointer is guaranteed to be [`NonNull`].
    pub fn as_ptr(&self) -> *mut ffi::mecab_lattice_t {
        self.inner.as_ptr()
    }

    /// Creates a new [`Lattice`] from a [`Model`].
    ///
    /// It wraps `mecab_model_new_lattice()`, the wrapper of `MeCab::Model::createLattice()`.
    pub fn new(model: &'a Model) -> Result<Self, Error> {
        unsafe {
            let inner = ffi::mecab_model_new_lattice(model.inner.as_ptr());
            NonNull::new(inner)
                .map(|inner| Self {
                    inner,
                    _marker: PhantomData,
                })
                .ok_or_else(Error::global)
        }
    }

    /// Sets the sentence to be analyzed and returns a [`LatticeGuard`].
    ///
    /// It wraps `mecab_lattice_set_sentence2()`, the wrapper of `MeCab::Lattice::set_sentence(sentence, len)`.
    ///
    /// Since the analysis result is borrowed from the input `s`, the input should be captured in a
    /// [`LatticeGuard`] during analysis.
    pub fn set_sentence<'l, 's>(&'l mut self, s: &'s str) -> LatticeGuard<'a, 'l, 's> {
        unsafe {
            ffi::mecab_lattice_set_sentence2(self.as_ptr(), s.as_ptr() as *const c_char, s.len());
            LatticeGuard {
                lattice: self,
                _marker: PhantomData,
            }
        }
    }

    /// Clears the lattice.
    ///
    /// It wraps `mecab_lattice_clear()`, the wrapper of `MeCab::Lattice::clear()`.
    pub fn clear(&mut self) {
        unsafe { ffi::mecab_lattice_clear(self.as_ptr()) }
    }

    /// Returns `true` if the lattice is available.
    ///
    /// It wraps `mecab_lattice_is_available()`, the wrapper of `MeCab::Lattice::is_available()`.
    pub fn is_available(&self) -> bool {
        unsafe {
            let res = ffi::mecab_lattice_is_available(self.as_ptr());
            res != 0
        }
    }

    /// Returns the CRF normalization factor.
    ///
    /// It wraps `mecab_lattice_get_z()`, the wrapper of `MeCab::Lattice::Z()`.
    pub fn crf_norm_factor(&self) -> f64 {
        unsafe { ffi::mecab_lattice_get_z(self.as_ptr()) }
    }

    /// Sets the CRF normalization factor.
    ///
    /// It wraps `mecab_lattice_set_z()`, the wrapper of `MeCab::Lattice::set_Z()`.
    pub fn set_crf_norm_factor(&mut self, z: f64) {
        unsafe { ffi::mecab_lattice_set_z(self.as_ptr(), z) }
    }

    /// Returns the temperature parameter.
    ///
    /// It wraps `mecab_lattice_get_theta()`, the wrapper of `MeCab::Lattice::theta()`.
    pub fn temparature(&self) -> f64 {
        unsafe { ffi::mecab_lattice_get_theta(self.as_ptr()) }
    }

    /// Sets the temperature parameter.
    ///
    /// It wraps `mecab_lattice_set_theta()`, the wrapper of `MeCab::Lattice::set_theta()`.
    pub fn set_temparature(&mut self, theta: f64) {
        unsafe { ffi::mecab_lattice_set_theta(self.as_ptr(), theta) }
    }

    /// Returns the request type flags.
    ///
    /// It wraps `mecab_lattice_get_request_type()`, the wrapper of `MeCab::Lattice::request_type()`.
    pub fn request_type(&self) -> RequestType {
        unsafe { RequestType::from_raw(ffi::mecab_lattice_get_request_type(self.as_ptr())) }
    }

    /// Sets the request type flags.
    ///
    /// It wraps `mecab_lattice_set_request_type()`, the wrapper of `MeCab::Lattice::set_request_type()`.
    pub fn set_request_type(&mut self, request_type: RequestType) {
        unsafe { ffi::mecab_lattice_set_request_type(self.as_ptr(), request_type.as_raw()) }
    }

    /// Returns the boundary constraint at the given position.
    ///
    /// It wraps `mecab_lattice_get_boundary_constraint()`, the wrapper of `MeCab::Lattice::boundary_constraint(pos)`.
    pub fn boundary_constraint(&self, pos: usize) -> BoundaryConstraintType {
        unsafe {
            BoundaryConstraintType::from_raw(ffi::mecab_lattice_get_boundary_constraint(
                self.as_ptr(),
                pos,
            ))
        }
    }

    /// Sets the boundary constraint at the given position.
    ///
    /// It wraps `mecab_lattice_set_boundary_constraint()`, the wrapper of `MeCab::Lattice::boundary_constraint(pos, type)`.
    pub fn set_boundary_constraint(&mut self, pos: usize, constraint: BoundaryConstraintType) {
        unsafe {
            ffi::mecab_lattice_set_boundary_constraint(self.as_ptr(), pos, constraint.as_raw());
        }
    }
}

impl Drop for Lattice<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::mecab_lattice_destroy(self.as_ptr());
        }
    }
}

unsafe impl Send for Lattice<'_> {}

/// A guard for a lattice that holds a reference to the sentence string.
///
/// It is returned by [`Lattice::set_sentence()`].
pub struct LatticeGuard<'a, 'l, 's> {
    lattice: &'l mut Lattice<'a>,
    _marker: PhantomData<&'s c_char>,
}

impl<'a> AsRef<Lattice<'a>> for LatticeGuard<'a, '_, '_> {
    fn as_ref(&self) -> &Lattice<'a> {
        self.lattice
    }
}
impl<'a> AsMut<Lattice<'a>> for LatticeGuard<'a, '_, '_> {
    fn as_mut(&mut self) -> &mut Lattice<'a> {
        self.lattice
    }
}

impl<'a, 'l> LatticeGuard<'a, 'l, '_> {
    /// Returns the underlying [`Lattice`].
    pub fn into_inner(self) -> &'l mut Lattice<'a> {
        self.lattice
    }
}

impl LatticeGuard<'_, '_, '_> {
    /// Returns the analysis result as a string.
    ///
    /// It wraps `mecab_lattice_tostr()`, the wrapper of `MeCab::Lattice::toString()`.
    pub fn to_str(&mut self) -> &str {
        unsafe {
            let s = ffi::mecab_lattice_tostr(self.lattice.as_ptr());
            let s = CStr::from_ptr(s);
            std::str::from_utf8_unchecked(s.to_bytes())
        }
    }

    /// Returns a cursor to the BoS node.
    ///
    /// It wraps `mecab_lattice_get_bos_node()`, the wrapper of `MeCab::Lattice::bos_node()`.
    pub fn bos_node(&self) -> NodeCursor<'_> {
        unsafe {
            let node = ffi::mecab_lattice_get_bos_node(self.lattice.as_ptr());
            let curr = Node::from_ptr(node);
            NodeCursor { curr }
        }
    }

    /// Returns a cursor to the EoS node.
    ///
    /// It wraps `mecab_lattice_get_eos_node()`, the wrapper of `MeCab::Lattice::eos_node()`.
    pub fn eos_node(&self) -> NodeCursor<'_> {
        unsafe {
            let node = ffi::mecab_lattice_get_eos_node(self.lattice.as_ptr());
            let curr = Node::from_ptr(node);
            NodeCursor { curr }
        }
    }
}

impl<'s> LatticeGuard<'_, '_, 's> {
    /// Returns the sentence string.
    ///
    /// It wraps `mecab_lattice_get_sentence()`, the wrapper of `MeCab::Lattice::sentence()`.
    ///
    /// The returned value should be equal to the input sentence of [`Lattice::set_sentence()`].
    pub fn sentence(&self) -> &str {
        unsafe {
            let ptr = ffi::mecab_lattice_get_sentence(self.lattice.as_ptr());
            if ptr.is_null() {
                ""
            } else {
                let slice = std::slice::from_raw_parts(ptr as *const u8, self.sentence_len());
                std::str::from_utf8_unchecked(slice)
            }
        }
    }

    /// Returns the length of the sentence.
    ///
    /// It wraps `mecab_lattice_get_size()`, the wrapper of `MeCab::Lattice::size()`.
    pub fn sentence_len(&self) -> usize {
        unsafe { ffi::mecab_lattice_get_size(self.lattice.as_ptr()) }
    }
}

bitflags! {
    /// Request type flags for MeCab analysis.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RequestType: u8 {
        /// One best result is obtained (default mode).
        const ONE_BEST = 1;
        /// Set this flag if you want to obtain N best results.
        const NBEST = 1 << 1;
        /// Set this flag if you want to enable a partial parsing mode.
        /// When this flag is set, the input sentence needs to be written
        /// in partial parsing format.
        const PARTIAL = 1 << 2;
        /// Set this flag if you want to obtain marginal probabilities.
        /// Marginal probability is set in `Node::prob()`.
        /// The parsing speed will get 3-5 times slower than the default mode.
        const MARGINAL_PROB = 1 << 3;
        /// Set this flag if you want to obtain alternative results.
        /// Not implemented.
        const ALTERNATIVE = 1 << 4;
        /// When this flag is set, the result linked-list (`Node::next/prev`)
        /// traverses all nodes in the lattice.
        const ALL_MORPHS = 1 << 5;
        /// When this flag is set, tagger internally copies the body of passed
        /// sentence into internal buffer.
        const ALLOCATE_SENTENCE = 1 << 6;
    }
}

impl Default for RequestType {
    fn default() -> Self {
        Self::ONE_BEST
    }
}

impl RequestType {
    /// Returns the raw bits as [`i32`].
    pub fn as_raw(self) -> i32 {
        self.bits() as _
    }

    /// Creates a [`RequestType`] from raw bits.
    pub fn from_raw(request_type: i32) -> Self {
        Self::from_bits_truncate(request_type as u8)
    }
}

/// Boundary constraint type for MeCab analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoundaryConstraintType {
    /// The token boundary is not specified.
    Any,
    /// The position is a strong token boundary.
    Token,
    /// The position is not a token boundary.
    InsideToken,
}

impl BoundaryConstraintType {
    /// Returns the raw ID.
    pub fn as_raw(self) -> i32 {
        match self {
            Self::Any => 0,
            Self::Token => 1,
            Self::InsideToken => 2,
        }
    }

    /// Creates a [`BoundaryConstraintType`] from a raw ID.
    pub fn from_raw(constraint: i32) -> Self {
        match constraint {
            1 => Self::Token,
            2 => Self::InsideToken,
            _ => Self::Any,
        }
    }
}
