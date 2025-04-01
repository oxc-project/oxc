//! Support types for `oxc_quote`.
//!
//! **This trait isn't meant to be used directly.**
//! It's only internal to the OXC project, providing
//! support types for generated AST code that is used
//! by `oxc_quote` macros.

pub trait ToRust {
    fn to_rust(&self) -> Node;
}

#[derive(Debug)]
pub struct Struct {
    pub name: &'static str,
    pub fields: Vec<(&'static str, Node)>,
}

#[derive(Debug)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct Enum {
    pub name: &'static str,
    pub variant: &'static str,
    pub field: Option<Node>,
}

#[derive(Debug)]
pub enum Node {
    Span(Span),
    Vec(Vec<Node>),
    Struct(Box<Struct>),
    Enum(Box<Enum>),
    Bool(bool),
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    Usize(usize),
    TryIntoUnwrap(Box<Node>),
    String(String),
    Atom(String),
    Option(Option<Box<Node>>),
    Box(Box<Node>),
    // Always None, since there's no meaningful way
    // to associate semantic information during a quote
    // (and Cell<Option> is only used for semantic fields).
    CellOption,
    Cell(Box<Node>),
}

macro_rules! impl_prims {
    ($($ty:ty => $var:tt),* $(,)?) => {
        $(impl ToRust for $ty {
            #[inline]
            fn to_rust(&self) -> Node {
                Node::$var(*self)
            }
        })*
    }
}

impl_prims! {
    bool => Bool,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
    usize => Usize,
    isize => Isize,
    f32 => F32,
    f64 => F64,
}

impl ToRust for String {
    fn to_rust(&self) -> Node {
        Node::String(self.clone())
    }
}

impl ToRust for &str {
    fn to_rust(&self) -> Node {
        Node::String((*self).to_string())
    }
}
