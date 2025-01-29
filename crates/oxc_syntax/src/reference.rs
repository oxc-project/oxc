#![allow(missing_docs)] // fixme
use bitflags::bitflags;
use nonmax::NonMaxU32;
use oxc_allocator::CloneIn;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use crate::{node::NodeId, symbol::SymbolId};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReferenceId(NonMaxU32);

impl Idx for ReferenceId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is a legal value for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ReferenceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.get())
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type ReferenceId = number;
export type ReferenceFlags = {
    None: 0,
    Read: 0b1,
    Write: 0b10,
    Type: 0b100,
    Value: 0b11
}
"#;

bitflags! {
    /// Describes how a symbol is being referenced in the AST.
    ///
    /// There are three general categories of references:
    /// 1. Values being referenced as values
    /// 2. Types being referenced as types
    /// 3. Values being used in type contexts
    ///
    /// ## Values
    /// Whether a reference is considered [`Read`] or [`Write`] is determined according to ECMA spec.
    ///
    /// See comments on [`Read`] and [`Write`] below.
    ///
    /// Counter-intuitively, `y` in `x = y = z` is [`Write`] only. `x = y = z` is equivalent to:
    ///
    /// ```js
    /// var _temp = z;
    /// y = _temp;
    /// x = _temp;
    /// ```
    ///
    /// See <https://github.com/oxc-project/oxc/issues/5165#issuecomment-2488333549> for a runtime test
    /// to determine Read/Write operations in a code snippet.
    ///
    /// ## Value as Type
    /// The [`ValueAsType`] flag is a temporary marker for references that need to
    /// resolve to value symbols initially, but will ultimately be treated as type references.
    /// This flag is crucial in scenarios like TypeScript's `typeof` operator.
    ///
    /// For example, in `type T = typeof a`:
    /// 1. The reference to 'a' is initially flagged with [`ValueAsType`].
    /// 2. This ensures that during symbol resolution, 'a' should be a value symbol.
    /// 3. However, the final resolved reference's flags will be treated as a type.
    ///
    /// ## Types
    /// Type references are indicated by [`Type`]. These are used primarily in
    /// type definitions and signatures. Types can never be re-assigned, so
    /// there is no read/write distinction for type references.
    ///
    /// [`Read`]: ReferenceFlags::Read
    /// [`Write`]: ReferenceFlags::Write
    /// [`Type`]: ReferenceFlags::Type
    /// [`ValueAsType`]: ReferenceFlags::ValueAsType
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    #[cfg_attr(feature = "serialize", derive(Serialize))]
    pub struct ReferenceFlags: u8 {
        const None = 0;
        /// Symbol is being read from as a Value.
        ///
        /// Whether a reference is `Read` is as defined in the spec:
        ///
        /// Under `Runtime Semantics: Evaluation`, when [`GetValue`](https://tc39.es/ecma262/#sec-getvalue)
        /// is called on a expression, and the expression is an `IdentifierReference`.
        ///
        /// For example:
        /// ```text
        /// 1. Let lRef be ? Evaluation of Expression.
        /// 2. Perform ? GetValue(lRef).
        /// ```
        const Read = 1 << 0;
        /// Symbol is being written to as a Value.
        ///
        /// Whether a reference is `Write` is as defined in the spec:
        ///
        /// Under `Runtime Semantics: Evaluation`, when [`PutValue`](https://tc39.es/ecma262/#sec-putvalue)
        /// is called on a expression, and the expression is an `IdentifierReference`.
        ///
        /// For example:
        /// ```text
        /// 1. Let lhs be ? Evaluation of LeftHandSideExpression.
        /// 2. Perform ? PutValue(lhs, newValue).
        /// ```
        const Write = 1 << 1;
        /// Used in type definitions.
        const Type = 1 << 2;
        /// A value symbol is used in a type context, such as in `typeof` expressions.
        const ValueAsType = 1 << 3;
        /// The symbol being referenced is a value.
        ///
        /// Note that this does not necessarily indicate the reference is used
        /// in a value context, since type queries are also flagged as [`Read`].
        ///
        /// [`Read`]: ReferenceFlags::Read
        const Value = Self::Read.bits() | Self::Write.bits();
    }
}

impl ReferenceFlags {
    #[inline]
    pub const fn read() -> Self {
        Self::Read
    }

    #[inline]
    pub const fn write() -> Self {
        Self::Write
    }

    #[inline]
    pub const fn read_write() -> Self {
        Self::Value
    }

    /// The identifier is read from. It may also be written to.
    #[inline]
    pub const fn is_read(&self) -> bool {
        self.intersects(Self::Read)
    }

    /// The identifier is only read from.
    #[inline]
    pub const fn is_read_only(&self) -> bool {
        !self.contains(Self::Write)
    }

    /// The identifier is written to. It may also be read from.
    #[inline]
    pub const fn is_write(&self) -> bool {
        self.intersects(Self::Write)
    }

    /// The identifier is only written to. It is not read from in this reference.
    #[inline]
    pub const fn is_write_only(&self) -> bool {
        !self.contains(Self::Read)
    }

    /// The identifier is both read from and written to, e.g `a += 1`.
    #[inline]
    pub fn is_read_write(&self) -> bool {
        self.contains(Self::Read | Self::Write)
    }

    /// Checks if the reference is a value being used in a type context.
    #[inline]
    pub fn is_value_as_type(&self) -> bool {
        self.contains(Self::ValueAsType)
    }

    /// The identifier is used in a type definition.
    #[inline]
    pub const fn is_type(&self) -> bool {
        self.contains(Self::Type)
    }

    #[inline]
    pub const fn is_type_only(self) -> bool {
        matches!(self, Self::Type)
    }

    #[inline]
    pub const fn is_value(&self) -> bool {
        self.intersects(Self::Value)
    }
}

impl<'alloc> CloneIn<'alloc> for ReferenceFlags {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc oxc_allocator::Allocator) -> Self::Cloned {
        *self
    }
}

/// Describes where and how a Symbol is used in the AST.
///
/// References indicate how they are being used using [`ReferenceFlags`]. Refer
/// to the documentation for [`ReferenceFlags`] for more information.
///
/// ## Resolution
/// References to symbols that could be resolved have their `symbol_id` field
/// populated. [`None`] indicates that either a global variable or a
/// non-existent symbol is being referenced.
///
/// The node identified by `node_id` will be an `IdentifierReference`.
/// Note that declarations do not count as references, even if the declaration
/// is being used in an expression.
///
/// ```ts
/// const arr = [1, 2, 3].map(function mapper(x) { return x + 1; });
/// //      Not considered a reference ^^^^^^
/// ```
#[cfg_attr(feature = "serialize", derive(Serialize), serde(rename_all = "camelCase"))]
#[derive(Debug, Clone)]
pub struct Reference {
    /// The AST node making the reference.
    node_id: NodeId,
    /// The symbol being referenced.
    ///
    /// This will be [`None`] if no symbol could be found within
    /// the reference's scope tree. Usually this indicates a global variable or
    /// a reference to a non-existent symbol.
    symbol_id: Option<SymbolId>,
    /// Describes how this referenced is used by other AST nodes. References can
    /// be reads, writes, or both.
    flags: ReferenceFlags,
}

impl Reference {
    /// Create a new unresolved reference.
    #[inline]
    pub fn new(node_id: NodeId, flags: ReferenceFlags) -> Self {
        Self { node_id, symbol_id: None, flags }
    }

    /// Create a new resolved reference on a symbol.
    #[inline]
    pub fn new_with_symbol_id(node_id: NodeId, symbol_id: SymbolId, flags: ReferenceFlags) -> Self {
        Self { node_id, symbol_id: Some(symbol_id), flags }
    }

    /// Get the id of the node that is referencing the symbol.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the id of the symbol being referenced.
    ///
    /// Will return [`None`] if the symbol could not be resolved.
    #[inline]
    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.symbol_id
    }

    #[inline]
    pub fn set_symbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }

    #[inline]
    pub fn flags(&self) -> ReferenceFlags {
        self.flags
    }

    #[inline]
    pub fn flags_mut(&mut self) -> &mut ReferenceFlags {
        &mut self.flags
    }

    /// Returns `true` if the identifier value was read.
    ///
    /// This is not mutually exclusive with [`Reference::is_write`].
    #[inline]
    pub fn is_read(&self) -> bool {
        self.flags.is_read()
    }

    /// Returns `true` if the identifier was written to.
    ///
    /// This is not mutually exclusive with [`Reference::is_read`].
    #[inline]
    pub fn is_write(&self) -> bool {
        self.flags.is_write()
    }

    /// Returns `true` if this reference is used in a value context.
    pub fn is_value(&self) -> bool {
        self.flags.is_value()
    }

    /// Returns `true` if this reference is used in a type context.
    #[inline]
    pub fn is_type(&self) -> bool {
        self.flags.is_type()
    }
}
