use super::ffi;

use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A single node (morpheme) in the MeCab lattice. It wraps C `mecab_node_t`.
///
/// The lifetime parameter is bound to
/// [`Lattice` and the input sentence](`crate::LatticeGuard::bos_node()`).
#[derive(Clone, Copy)]
pub struct Node<'a> {
    inner: NonNull<ffi::mecab_node_t>,
    _marker: PhantomData<&'a ffi::mecab_node_t>,
}

impl<'a> Node<'a> {
    /// Returns the raw pointer to the underlying [`mecab_node_t`](ffi::mecab_node_t).
    pub fn as_ptr(&self) -> *mut ffi::mecab_node_t {
        self.inner.as_ptr()
    }

    pub(crate) fn from_ptr(inner: *mut ffi::mecab_node_t) -> Option<Self> {
        NonNull::new(inner).map(|inner| Node {
            inner,
            _marker: PhantomData,
        })
    }

    /// Returns the next node in the same lattice.
    ///
    /// It wraps `mecab_node_t::next`.
    ///
    /// `None` is likely to imply that `self` is [EoS](`NodeKind`).
    pub fn next(self) -> Option<Self> {
        unsafe {
            let node = (*self.as_ptr()).next;
            Node::from_ptr(node)
        }
    }

    /// Returns the previous node in the same lattice.
    ///
    /// It wraps `mecab_node_t::prev`.
    ///
    /// `None` is likely to imply that `self` is [BoS](`NodeKind`).
    pub fn prev(self) -> Option<Self> {
        unsafe {
            let node = (*self.as_ptr()).prev;
            Node::from_ptr(node)
        }
    }

    /// Returns the unique node ID.
    ///
    /// It wraps `mecab_node_t::id`.
    pub fn id(self) -> u32 {
        unsafe { (*self.as_ptr()).id }
    }

    /// Returns the feature string of this node.
    ///
    /// It wraps `mecab_node_t::feature`.
    pub fn feature(self) -> &'a str {
        unsafe {
            let ptr = (*self.as_ptr()).feature;
            let s = CStr::from_ptr(ptr);
            std::str::from_utf8_unchecked(s.to_bytes())
        }
    }

    /// Returns the surface string of this node.
    ///
    /// It wraps `mecab_node_t::surface` and `mecab_node_t::length`.
    pub fn surface(self) -> &'a str {
        unsafe {
            let ptr = (*self.as_ptr()).surface;
            let len = (*self.as_ptr()).length;

            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                ptr as *const u8,
                len as usize,
            ))
        }
    }

    /// Returns the status of this node.
    ///
    /// It wraps `mecab_node_t::stat`.
    pub fn kind(self) -> NodeKind {
        use ffi::{MECAB_BOS_NODE, MECAB_EON_NODE, MECAB_EOS_NODE, MECAB_NOR_NODE, MECAB_UNK_NODE};

        unsafe {
            let kind = (*self.as_ptr()).stat as u32;
            if kind == MECAB_BOS_NODE {
                NodeKind::Bos
            } else if kind == MECAB_EOS_NODE {
                NodeKind::Eos
            } else if kind == MECAB_UNK_NODE {
                NodeKind::Unk
            } else if kind == MECAB_EON_NODE {
                NodeKind::EoNbest
            } else {
                debug_assert_eq!(kind, MECAB_NOR_NODE);
                NodeKind::Normal
            }
        }
    }

    /// Returns the left attribute ID.
    ///
    /// It wraps `mecab_node_t::lcAttr`.
    pub fn lc_attr(self) -> LcAttr {
        unsafe { LcAttr((*self.as_ptr()).lcAttr) }
    }

    /// Returns the right attribute ID.
    ///
    /// It wraps `mecab_node_t::rcAttr`.
    pub fn rc_attr(self) -> RcAttr {
        unsafe { RcAttr((*self.as_ptr()).rcAttr) }
    }

    /// Returns the unique part of speech ID.
    ///
    /// It wraps `mecab_node_t::posid`.
    pub fn pos_id(self) -> u16 {
        unsafe { (*self.as_ptr()).posid }
    }

    /// Returns the character type ID.
    ///
    /// It wraps `mecab_node_t::char_type`.
    pub fn char_type(self) -> u8 {
        unsafe { (*self.as_ptr()).char_type }
    }

    /// Returns `true` if this node is part of the best path.
    ///
    /// It wraps `mecab_node_t::isbest`.
    pub fn is_best(self) -> bool {
        unsafe { (*self.as_ptr()).isbest == 1 }
    }

    /// Returns the forward accumulative log summation.
    ///
    /// It wraps `mecab_node_t::alpha`.
    pub fn alpha(self) -> f32 {
        unsafe { (*self.as_ptr()).alpha }
    }

    /// Returns the backward accumulative log summation.
    ///
    /// It wraps `mecab_node_t::beta`.
    pub fn beta(self) -> f32 {
        unsafe { (*self.as_ptr()).beta }
    }

    /// Returns the marginal probability.
    ///
    /// It wraps `mecab_node_t::prob`.
    pub fn prob(self) -> f32 {
        unsafe { (*self.as_ptr()).prob }
    }

    /// Returns the word cost.
    ///
    /// It wraps `mecab_node_t::wcost`.
    pub fn wcost(self) -> i16 {
        unsafe { (*self.as_ptr()).wcost }
    }

    /// Returns the best accumulative cost from BoS to this node.
    ///
    /// It wraps `mecab_node_t::cost`.
    pub fn cost(self) -> i64 {
        unsafe { (*self.as_ptr()).cost }
    }
}

/// Status of a MeCab node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    /// Beginning of sentence.
    Bos,
    /// End of sentence.
    Eos,
    /// Unknown word.
    Unk,
    /// End of N-best.
    EoNbest,
    /// Normal word.
    Normal,
}

impl NodeKind {
    /// Returns true iff `self` is `Bos`.
    pub fn is_bos(self) -> bool {
        self == Self::Bos
    }

    /// Returns true iff `self` is `Eos`.
    pub fn is_eos(self) -> bool {
        self == Self::Eos
    }
}

/// A cursor for iterating over nodes in a lattice.
///
/// The lifetime parameter is bound to
/// [`Lattice` and the input sentence](`crate::LatticeGuard::bos_node()`).
///
/// The cursor can be used either as an iterator and as a cursor of [`Node`]s.
///
/// As an iterator:
///
/// ```
/// # use mecab_sys::LatticeGuard;
/// # fn analyze(lattice: &LatticeGuard<'_, '_, '_>) {
/// let cursor = lattice.bos_node();
/// for node in cursor {
///     let surface = node.surface();
///     let feat = node.feature();
///
///     println!("{surface}: {feat}");
/// }
/// # }
/// ```
///
/// As a cursor:
///
/// ```
/// # use mecab_sys::LatticeGuard;
/// # fn analyze(lattice: &LatticeGuard<'_, '_, '_>) {
/// let mut cursor = lattice.bos_node();
/// while let Some(node) = cursor.curr() {
///     let surface = node.surface();
///     let feat = node.feature();
///
///     println!("{surface}: {feat}");
///
///     cursor.move_next();
/// }
/// # }
/// ```
///
/// The cursor can move back and forth:
///
/// ```
/// # use mecab_sys::LatticeGuard;
/// # fn analyze(lattice: &LatticeGuard<'_, '_, '_>) {
/// let mut cursor = lattice.bos_node();
/// assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
///
/// cursor.move_next();
/// cursor.move_next();
/// if let Some(node) = cursor.curr() {
///     let surface = node.surface();
///     let feat = node.feature();
///
///     println!("{surface}: {feat}");
/// }
///
/// cursor.move_prev();
/// if let Some(node) = cursor.curr() {
///     let surface = node.surface();
///     let feat = node.feature();
///
///     println!("{surface}: {feat}");
/// }
///
/// cursor.move_prev();
/// assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
/// # }
/// ```
///
///
/// After you call `move_next()` on the EoS node, call `move_prev()` on the BoS node, or consume as
/// an iterator, then the cursor shifted to the "dead" state, never being back to the alive state.
///
/// ```
/// # use mecab_sys::LatticeGuard;
/// # fn analyze(lattice: &LatticeGuard<'_, '_, '_>) {
/// let mut cursor = lattice.bos_node();
/// while cursor.curr().is_some_and(|node| !node.kind().is_eos()) {
///     cursor.move_next();
/// }
/// assert!(cursor.curr().is_some_and(|node| node.kind().is_eos()));
///
/// // Call `move_next()` on the EoS node
/// cursor.move_next();
/// assert!(cursor.curr().is_none());
///
/// // Never back to the original node
/// cursor.move_prev();
/// assert!(cursor.curr().is_none());
///
/// let mut cursor = lattice.bos_node();
/// assert!(cursor.curr().is_some_and(|node| node.kind().is_bos()));
///
/// // Call `move_prev()` on the BoS node
/// cursor.move_prev();
/// assert!(cursor.curr().is_none());
///
/// // Never back to the original node
/// cursor.move_next();
/// assert!(cursor.curr().is_none());
///
/// let mut cursor = lattice.bos_node();
/// // Consume the iterator, reaching at the EoS node
/// for _ in &mut cursor {}
/// assert!(cursor.curr().is_none());
///
/// // Never back to alive nodes
/// cursor.move_prev();
/// assert!(cursor.curr().is_none());
/// # }
/// ```
pub struct NodeCursor<'a> {
    pub(crate) curr: Option<Node<'a>>,
}

impl<'a> NodeCursor<'a> {
    /// Moves the cursor to the next node.
    ///
    /// It is equivalent to calling [`Node::next()`] on the [current node](`Self::curr()`).
    pub fn move_next(&mut self) {
        if let Some(curr) = self.curr {
            self.curr = curr.next();
        }
    }

    /// Moves the cursor to the previous node.
    ///
    /// It is equivalent to calling [`Node::prev()`] on the [current node](`Self::curr()`).
    pub fn move_prev(&mut self) {
        if let Some(curr) = self.curr {
            self.curr = curr.prev();
        }
    }

    /// Returns the current node pointed to by the cursor.
    pub fn curr(&self) -> Option<Node<'a>> {
        self.curr
    }
}

impl<'a> Iterator for NodeCursor<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.curr?;
        self.move_next();
        Some(res)
    }
}

/// Left attribute ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LcAttr(u16);

/// Right attribute ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcAttr(u16);

impl LcAttr {
    /// Creates a new [`LcAttr`] from a raw ID.
    pub fn from_raw(attr: u16) -> Self {
        Self(attr)
    }
    /// Returns the raw ID.
    pub fn to_raw(self) -> u16 {
        self.0
    }
}
impl RcAttr {
    /// Creates a new [`RcAttr`] from a raw ID.
    pub fn from_raw(attr: u16) -> Self {
        Self(attr)
    }
    /// Returns the raw ID.
    pub fn to_raw(self) -> u16 {
        self.0
    }
}
