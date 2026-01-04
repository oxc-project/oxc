use std::cell::Cell;

use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, IdentifierReference, Program, TSEnumMemberName},
};
use oxc_ast_visit::{Visit, walk::walk_ts_enum_member_name};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

/// Re-export of [`Stats`] from oxc_parser.
pub use oxc_parser::Stats;

/// Gather [`Stats`] by visiting AST and counting nodes, scopes, symbols, and references.
///
/// Nodes, scopes and references counts will be exactly accurate.
/// Symbols count may be an over-estimate if there are multiple declarations for a single symbol.
/// e.g. `var x; var x;` will produce a count of 2 symbols, but this is actually only 1 symbol.
///
/// [`Semantic::stats`]: super::Semantic::stats
pub fn count_stats(program: &Program) -> Stats {
    let mut counter = Counter::default();
    counter.visit_program(program);
    counter.stats
}

#[derive(Default)]
struct Counter {
    stats: Stats,
}

/// Visitor to count nodes, scopes, symbols and references in AST
impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.stats.nodes += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.stats.scopes += 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.stats.nodes += 1;
        self.stats.symbols += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.stats.nodes += 1;
        self.stats.references += 1;
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        self.stats.symbols += 1;
        walk_ts_enum_member_name(self, it);
    }
}
