//! AST

mod source_type;

pub use source_type::SourceType;

pub type Atom = compact_str::CompactString;

pub type Span = std::ops::Range<usize>;
