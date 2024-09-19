//! # The JavaScript Oxidation Compiler
//!
//! <https://github.com/oxc-project/oxc>

#[cfg(feature = "full")]
mod compiler;

#[cfg(feature = "full")]
pub use compiler::{Compiler, CompilerInterface};

pub mod allocator {
    #[doc(inline)]
    pub use oxc_allocator::*;
}

pub mod ast {
    #[doc(inline)]
    pub use oxc_ast::*;
}

pub mod diagnostics {
    #[doc(inline)]
    pub use oxc_diagnostics::*;
}

pub mod index {
    #[doc(inline)]
    pub use oxc_index::*;
}

pub mod parser {
    #[doc(inline)]
    pub use oxc_parser::*;
}

pub mod regular_expression {
    #[doc(inline)]
    pub use oxc_regular_expression::*;
}

pub mod span {
    #[doc(inline)]
    pub use oxc_span::*;
}

pub mod syntax {
    #[doc(inline)]
    pub use oxc_syntax::*;
}

#[cfg(feature = "semantic")]
pub mod semantic {
    #[doc(inline)]
    pub use oxc_semantic::*;
}

#[cfg(feature = "transformer")]
pub mod transformer {
    #[doc(inline)]
    pub use oxc_transformer::*;
}

#[cfg(feature = "minifier")]
pub mod minifier {
    #[doc(inline)]
    pub use oxc_minifier::*;
}

#[cfg(feature = "mangler")]
pub mod mangler {
    #[doc(inline)]
    pub use oxc_mangler::*;
}

#[cfg(feature = "codegen")]
pub mod codegen {
    #[doc(inline)]
    pub use oxc_codegen::*;
}

#[cfg(feature = "isolated_declarations")]
pub mod isolated_declarations {
    #[doc(inline)]
    pub use oxc_isolated_declarations::*;
}

#[cfg(feature = "sourcemap")]
pub mod sourcemap {
    #[doc(inline)]
    pub use oxc_sourcemap::*;
}

#[cfg(feature = "cfg")]
pub mod cfg {
    #[doc(inline)]
    pub use oxc_cfg::*;
}
