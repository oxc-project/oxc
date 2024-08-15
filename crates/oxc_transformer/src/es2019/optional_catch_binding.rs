use std::cell::Cell;

use oxc_ast::ast::*;
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::context::Ctx;

/// ES2019: Optional Catch Binding
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-optional-catch-binding>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-optional-catch-binding>
pub struct OptionalCatchBinding<'a> {
    _ctx: Ctx<'a>,
}

impl<'a> OptionalCatchBinding<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { _ctx: ctx }
    }

    /// If CatchClause has no param, add a parameter called `unused`.
    ///
    /// ```ts
    /// try {}
    /// catch {}
    /// ```
    /// too
    /// ```ts
    /// try {}
    /// catch (_unused) {}
    /// ```
    #[allow(clippy::unused_self)]
    pub fn transform_catch_clause(&self, clause: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if clause.param.is_some() {
            return;
        }
        let symbol_id =
            ctx.generate_uid("unused", ctx.scoping.current_scope_id(), SymbolFlags::CatchVariable);
        let name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));
        let binding_identifier =
            BindingIdentifier { span: SPAN, symbol_id: Cell::new(Some(symbol_id)), name };
        let binding_pattern_kind =
            ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
        let binding_pattern =
            ctx.ast.binding_pattern(binding_pattern_kind, None::<TSTypeAnnotation<'a>>, false);
        let param = ctx.ast.catch_parameter(SPAN, binding_pattern);
        clause.param = Some(param);
    }
}
