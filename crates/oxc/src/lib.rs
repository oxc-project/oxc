//! # The JavaScript Oxidation Compiler
//!
//! <https://github.com/web-infra-dev/oxc>

pub mod allocator {
    #[doc(inline)]
    pub use oxc_allocator::*;
}

pub mod ast {
    #[doc(inline)]
    pub use oxc_ast::*;
}

#[cfg(feature = "minifier")]
pub mod ast_lower {
    #[doc(inline)]
    pub use oxc_ast_lower::*;
}

pub mod diagnostics {
    #[doc(inline)]
    pub use oxc_diagnostics::*;
}

#[cfg(feature = "formatter")]
pub mod formatter {
    #[doc(inline)]
    pub use oxc_formatter::*;
}

#[cfg(feature = "minifier")]
pub mod hir {
    #[doc(inline)]
    pub use oxc_hir::*;
}

pub mod index {
    #[doc(inline)]
    pub use oxc_index::*;
}

#[cfg(feature = "minifier")]
pub mod minifier {
    #[doc(inline)]
    pub use oxc_minifier::*;
}

pub mod parser {
    #[doc(inline)]
    pub use oxc_parser::*;
}

#[cfg(feature = "semantic")]
pub mod semantic {
    #[doc(inline)]
    pub use oxc_semantic::*;
}

pub mod span {
    #[doc(inline)]
    pub use oxc_span::*;
}

pub mod syntax {
    #[doc(inline)]
    pub use oxc_syntax::*;
}
