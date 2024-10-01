//! ES2019: Optional Catch Binding
//!
//! This plugin transform catch clause without parameter to add a parameter called `unused` in catch clause.
//!
//! > This plugin is included in `preset-env`, in ES2019
//!
//! ## Example
//!
//! Input:
//! ```js
//! try {
//!   throw 0;
//! } catch {
//!   doSomethingWhichDoesNotCareAboutTheValueThrown();
//! }
//! ```
//!
//! Output:
//! ```js
//! try {
//!   throw 0;
//! } catch (_unused) {
//!   doSomethingWhichDoesNotCareAboutTheValueThrown();
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-optional-catch-binding](https://babel.dev/docs/babel-plugin-transform-optional-catch-binding).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-optional-catch-binding>
//! * Optional catch binding TC39 proposal: <https://github.com/tc39/proposal-optional-catch-binding>

use oxc_ast::{ast::*, NONE};
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

pub struct OptionalCatchBinding;

impl OptionalCatchBinding {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Traverse<'a> for OptionalCatchBinding {
    /// If CatchClause has no param, add a parameter called `unused`.
    #[allow(clippy::unused_self)]
    fn enter_catch_clause(&mut self, clause: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if clause.param.is_some() {
            return;
        }

        let block_scope_id = clause.body.scope_id.get().unwrap();
        let symbol_id = ctx.generate_uid(
            "unused",
            block_scope_id,
            SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable,
        );
        let name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));
        let binding_identifier = BindingIdentifier::new_with_symbol_id(SPAN, name, symbol_id);
        let binding_pattern_kind =
            ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
        let binding_pattern = ctx.ast.binding_pattern(binding_pattern_kind, NONE, false);
        let param = ctx.ast.catch_parameter(SPAN, binding_pattern);
        clause.param = Some(param);
    }
}
