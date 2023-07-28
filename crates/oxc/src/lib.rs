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

pub mod ast_lower {
    #[doc(inline)]
    pub use oxc_ast_lower::*;
}

pub mod diagnostics {
    #[doc(inline)]
    pub use oxc_diagnostics::*;
}

pub mod formatter {
    #[doc(inline)]
    pub use oxc_formatter::*;
}

pub mod hir {
    #[doc(inline)]
    pub use oxc_hir::*;
}

pub mod index {
    #[doc(inline)]
    pub use oxc_index::*;
}

pub mod minifier {
    #[doc(inline)]
    pub use oxc_minifier::*;
}

pub mod parser {
    #[doc(inline)]
    pub use oxc_parser::*;
}

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

#[allow(unused_imports)]
#[test]
fn import_smoke_test() {
    use crate::{
        allocator::Allocator, ast::ast::Program as AstProgram, ast_lower::AstLower,
        diagnostics::Error, formatter::Formatter, hir::hir::Program as HirProgram, index::IndexVec,
        minifier::Minifier, parser::Parser, semantic::Semantic, span::Span, syntax::NumberBase,
    };
}
