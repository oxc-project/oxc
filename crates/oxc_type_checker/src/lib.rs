//! # Oxc Type Checker
//!
//! An **experimental**, work-in-progress type checker for JavaScript and TypeScript.
//!
//! This crate is intentionally a thin scaffold. It does *not* type check anything yet.
//! Instead it provides the plumbing you need to start experimenting with a type checker
//! built on top of oxc:
//!
//! - a [`TypeChecker`] entry point with room for [`TypeCheckerOptions`],
//! - an AST walk over the program via [`oxc_ast_visit::Visit`],
//! - access to symbol and scope information through [`oxc_semantic::Semantic`], and
//! - a [`Diagnostics`] collector plus a [`type_error`] helper for reporting problems.
//!
//! ## Usage
//!
//! ```rust
//! use oxc_allocator::Allocator;
//! use oxc_type_checker::TypeChecker;
//! use oxc_parser::Parser;
//! use oxc_semantic::SemanticBuilder;
//! use oxc_span::SourceType;
//!
//! let allocator = Allocator::default();
//! let source_text = "const x: number = 1;";
//! let source_type = SourceType::ts();
//!
//! let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
//! let semantic_ret = SemanticBuilder::new().build(&parser_ret.program);
//!
//! let checker_ret = TypeChecker::new().check(&parser_ret.program, &semantic_ret.semantic);
//! assert!(checker_ret.diagnostics.is_empty());
//! ```
//!
//! ## Adding a check
//!
//! Type checks live in the private `TypeCheckerVisitor`. Override the relevant `visit_*`
//! method, inspect the node (using `self.semantic` to resolve identifiers and look up
//! symbols), push a [`type_error`] onto `self.diagnostics` when something is wrong, and
//! remember to keep walking the subtree with the matching `walk_*` function:
//!
//! ```ignore
//! use oxc_ast::ast::TSTypeAliasDeclaration;
//! use oxc_ast_visit::{Visit, walk::walk_ts_type_alias_declaration};
//!
//! impl<'a> Visit<'a> for TypeCheckerVisitor<'a, '_> {
//!     fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
//!         // ... inspect `decl`, and on error:
//!         // self.diagnostics.push(type_error("some type error", decl.span));
//!         walk_ts_type_alias_declaration(self, decl);
//!     }
//! }
//! ```
//!
//! See `examples/checker.rs` for a runnable end-to-end example.

use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_diagnostics::Diagnostics;
use oxc_semantic::Semantic;

mod diagnostics;
mod fold;

pub mod extension;
pub mod project;
pub mod tsconfig;
pub mod tspath;
pub mod vfsmatch;

#[cfg(test)]
mod tests;

pub use crate::diagnostics::type_error;

/// Options controlling how the [`TypeChecker`] behaves.
///
/// Empty for now. Add knobs here — strictness flags, the target `lib`, and so on — as
/// the checker grows.
#[derive(Debug, Default, Clone)]
pub struct TypeCheckerOptions;

/// The result of running [`TypeChecker::check`].
#[non_exhaustive]
pub struct TypeCheckerReturn {
    /// Type errors and warnings collected during checking.
    pub diagnostics: Diagnostics,
}

/// An experimental type checker.
///
/// A [`TypeChecker`] is cheap to create and can be reused to check multiple programs. Build
/// one with [`TypeChecker::new`] (optionally [`TypeChecker::with_options`]) and run it with
/// [`TypeChecker::check`].
#[derive(Debug, Default, Clone)]
pub struct TypeChecker {
    options: TypeCheckerOptions,
}

impl TypeChecker {
    /// Create a checker with default [options](TypeCheckerOptions).
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the checker's [options](TypeCheckerOptions).
    #[must_use]
    pub fn with_options(mut self, options: TypeCheckerOptions) -> Self {
        self.options = options;
        self
    }

    /// Type check `program`.
    ///
    /// `semantic` supplies the symbol table and scope tree for `program` (build it with
    /// [`oxc_semantic::SemanticBuilder`]). The returned [`TypeCheckerReturn`] carries any
    /// diagnostics that were produced — an empty list means no problems were found.
    pub fn check<'a>(&self, program: &Program<'a>, semantic: &Semantic<'a>) -> TypeCheckerReturn {
        let mut visitor = TypeCheckerVisitor {
            semantic,
            options: &self.options,
            diagnostics: Diagnostics::new(),
        };
        visitor.visit_program(program);
        TypeCheckerReturn { diagnostics: visitor.diagnostics }
    }
}

/// Walks the AST and accumulates diagnostics.
///
/// This is where type checking happens. Right now it inherits the default (no-op) walk
/// from [`Visit`], so it reports nothing; override `visit_*` methods to add checks. See
/// the crate-level docs for the pattern.
struct TypeCheckerVisitor<'a, 'c> {
    /// Symbol and scope information for the program being checked. Use it to resolve
    /// identifiers to symbols, inspect symbol flags, walk references, and so on.
    #[expect(dead_code, reason = "scaffold: available to the checks you add")]
    semantic: &'c Semantic<'a>,
    /// Configuration for the checks you add.
    #[expect(dead_code, reason = "scaffold: available to the checks you add")]
    options: &'c TypeCheckerOptions,
    /// Type errors and warnings, surfaced via [`TypeCheckerReturn::diagnostics`].
    diagnostics: Diagnostics,
}

impl<'a> Visit<'a> for TypeCheckerVisitor<'a, '_> {
    // Override `visit_*` methods here to implement type checks.
    //
    // Reach for `self.semantic` when a check needs symbol or scope information, push onto
    // `self.diagnostics` (e.g. with `type_error(..)`) to report a problem, and call the
    // matching `walk_*` free function so traversal continues into the node's children.
}
