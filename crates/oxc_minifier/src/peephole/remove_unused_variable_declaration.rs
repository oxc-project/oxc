use oxc_ast::ast::*;

use crate::{CompressOptionsUnused, ctx::Ctx};

use super::{PeepholeOptimizations, State};

impl<'a> PeepholeOptimizations {
    pub fn should_remove_unused_declarator(
        decl: &VariableDeclarator<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return false;
        }
        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
            if Self::keep_top_level_var_in_script_mode(ctx) {
                return false;
            }
            // It is unsafe to remove if direct eval is involved.
            if ctx.scoping().root_scope_flags().contains_direct_eval() {
                return false;
            }
            if let Some(symbol_id) = ident.symbol_id.get() {
                return ctx.scoping().symbol_is_unused(symbol_id);
            }
        }
        false
    }

    pub fn remove_unused_function_declaration(
        f: &Function<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return None;
        }
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return None;
        }
        let id = f.id.as_ref()?;
        let symbol_id = id.symbol_id.get()?;
        if ctx.scoping().symbol_is_unused(symbol_id) {
            return Some(ctx.ast.statement_empty(f.span));
        }
        None
    }

    pub fn remove_unused_assignment_expression(
        &self,
        _e: &mut Expression<'a>,
        _state: &mut State,
        _ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        // let Expression::AssignmentExpression(assign_expr) = e else { return false };
        // if matches!(
        // ctx.state.options.unused,
        // CompressOptionsUnused::Keep | CompressOptionsUnused::KeepAssign
        // ) {
        // return false;
        // }
        // let Some(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)) =
        // assign_expr.left.as_simple_assignment_target()
        // else {
        // return false;
        // };
        // if Self::keep_top_level_var_in_script_mode(ctx) {
        // return false;
        // }
        // let Some(reference_id) = ident.reference_id.get() else { return false };
        // let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
        // return false;
        // };
        // // Keep error for assigning to `const foo = 1; foo = 2`.
        // if ctx.scoping().symbol_flags(symbol_id).is_const_variable() {
        // return false;
        // }
        // if !ctx.scoping().get_resolved_references(symbol_id).all(|r| !r.flags().is_read()) {
        // return false;
        // }
        // *e = assign_expr.right.take_in(ctx.ast);
        // state.changed = true;
        false
    }

    /// Do remove top level vars in script mode.
    fn keep_top_level_var_in_script_mode(ctx: &Ctx<'a, '_>) -> bool {
        ctx.scoping.current_scope_id() == ctx.scoping().root_scope_id()
            && ctx.source_type().is_script()
    }
}

#[cfg(test)]
mod test {
    use oxc_span::SourceType;

    use crate::{
        CompressOptions,
        tester::{
            test_options, test_options_source_type, test_same_options,
            test_same_options_source_type,
        },
    };

    #[test]
    fn remove_unused_variable_declaration() {
        let options = CompressOptions::smallest();
        test_options("var x", "", &options);
        test_options("var x = 1", "", &options);
        test_options("var x = foo", "foo", &options);
        test_same_options("var x; foo(x)", &options);
        test_same_options("export var x", &options);
    }

    #[test]
    fn remove_unused_function_declaration() {
        let options = CompressOptions::smallest();
        test_options("function foo() {}", "", &options);
        test_same_options("function foo() {} foo()", &options);
        test_same_options("export function foo() {} foo()", &options);
    }

    #[test]
    #[ignore]
    fn remove_unused_assignment_expression() {
        let options = CompressOptions::smallest();
        test_options("var x = 1; x = 2;", "", &options);
        test_options("var x = 1; x = 2;", "", &options);
        test_options("var x = 1; x = foo();", "foo()", &options);
        test_same_options("export let foo; foo = 0;", &options);
        test_same_options("var x = 1; x = 2, foo(x)", &options);
        test_same_options("function foo() { return t = x(); } foo();", &options);
        test_options(
            "function foo() { var t; return t = x(); } foo();",
            "function foo() { return x(); } foo();",
            &options,
        );
        test_options(
            "function foo(t) { return t = x(); } foo();",
            "function foo(t) { return x(); } foo();",
            &options,
        );
    }

    #[test]
    #[ignore]
    fn keep_in_script_mode() {
        let options = CompressOptions::smallest();
        let source_type = SourceType::cjs();
        test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
        test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);

        test_options_source_type(
            "function foo() { var x = 1; x = 2; bar() } foo()",
            "function foo() { bar() } foo()",
            source_type,
            &options,
        );
    }
}
