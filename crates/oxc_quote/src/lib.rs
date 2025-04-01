//! # `jsquote!` and `jsquote_expr!` for OXC

#[doc(hidden)]
pub mod private {
    pub use oxc_allocator::*;
    pub use oxc_ast::ast::*;
    pub use oxc_quote_types::private::*;
    pub use oxc_span::*;
}

pub use oxc_quote_proc::*;
