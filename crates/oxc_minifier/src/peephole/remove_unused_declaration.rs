use oxc_ast::ast::*;

use crate::{CompressOptionsUnused, ctx::Ctx};

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    fn can_remove_unused_declarators(ctx: &Ctx<'a, '_>) -> bool {
        ctx.state.options.unused != CompressOptionsUnused::Keep
            && !Self::keep_top_level_var_in_script_mode(ctx)
            && !ctx.scoping().root_scope_flags().contains_direct_eval()
    }

    pub fn should_remove_unused_declarator(
        decl: &VariableDeclarator<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> bool {
        if !Self::can_remove_unused_declarators(ctx) {
            return false;
        }
        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
            // Unsafe to remove `using`, unable to statically determine usage of [Symbol.dispose].
            if decl.kind.is_using() {
                return false;
            }
            if let Some(symbol_id) = ident.symbol_id.get() {
                return ctx.scoping().symbol_is_unused(symbol_id);
            }
        }
        false
    }

    pub fn remove_unused_variable_declaration(
        mut stmt: Statement<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        let Statement::VariableDeclaration(var_decl) = &mut stmt else { return Some(stmt) };
        if !Self::can_remove_unused_declarators(ctx) {
            return Some(stmt);
        }
        var_decl.declarations.retain(|decl| !Self::should_remove_unused_declarator(decl, ctx));
        if var_decl.declarations.is_empty() {
            return None;
        }
        Some(stmt)
    }

    pub fn remove_unused_function_declaration(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::FunctionDeclaration(f) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &f.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return;
        }
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return;
        }
        *stmt = ctx.ast.statement_empty(f.span);
        ctx.state.changed = true;
    }

    pub fn remove_unused_class_declaration(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ClassDeclaration(c) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &c.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return;
        }
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return;
        }
        if let Some(changed) = Self::remove_unused_class(c, ctx).map(|exprs| {
            if exprs.is_empty() {
                ctx.ast.statement_empty(c.span)
            } else {
                let expr = ctx.ast.expression_sequence(c.span, exprs);
                ctx.ast.statement_expression(c.span, expr)
            }
        }) {
            *stmt = changed;
            ctx.state.changed = true;
        }
    }

    /// Do remove top level vars in script mode.
    pub fn keep_top_level_var_in_script_mode(ctx: &Ctx<'a, '_>) -> bool {
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
        test_same_options("using x = foo", &options);
        test_same_options("await using x = foo", &options);

        test_options("for (var x; ; );", "for (; ;);", &options);
        test_options("for (var x = 1; ; );", "for (; ;);", &options);
        test_same_options("for (var x = foo; ; );", &options); // can be improved
    }

    #[test]
    fn remove_unused_function_declaration() {
        let options = CompressOptions::smallest();
        test_options("function foo() {}", "", &options);
        test_same_options("function foo() { bar } foo()", &options);
        test_same_options("export function foo() {} foo()", &options);
    }

    #[test]
    fn remove_unused_class_declaration() {
        let options = CompressOptions::smallest();
        // extends
        test_options("class C {}", "", &options);
        test_options("class C extends Foo {}", "Foo", &options);

        // static block
        test_options("class C { static {} }", "", &options);
        test_same_options("class C { static { foo } }", &options);

        // method
        test_options("class C { foo() {} }", "", &options);
        test_options("class C { [foo]() {} }", "foo", &options);
        test_options("class C { static foo() {} }", "", &options);
        test_options("class C { static [foo]() {} }", "foo", &options);
        test_options("class C { [1]() {} }", "", &options);
        test_options("class C { static [1]() {} }", "", &options);

        // property
        test_options("class C { foo }", "", &options);
        test_options("class C { foo = bar }", "", &options);
        test_options("class C { foo = 1 }", "", &options);
        // TODO: would be nice if this is removed but the one with `this` is kept.
        test_same_options("class C { static foo = bar }", &options);
        test_same_options("class C { static foo = this.bar = {} }", &options);
        test_options("class C { static foo = 1 }", "", &options);
        test_options("class C { [foo] = bar }", "foo", &options);
        test_options("class C { [foo] = 1 }", "foo", &options);
        test_same_options("class C { static [foo] = bar }", &options);
        test_options("class C { static [foo] = 1 }", "foo", &options);

        // accessor
        test_options("class C { accessor foo = 1 }", "", &options);
        test_options("class C { accessor [foo] = 1 }", "foo", &options);

        // order
        test_options("class _ extends A { [B] = C; [D]() {} }", "A, B, D", &options);

        // decorators
        test_same_options("class C { @dec foo() {} }", &options);

        // TypeError
        test_same_options("class C extends (() => {}) {}", &options);
    }

    #[test]
    fn keep_in_script_mode() {
        let options = CompressOptions::smallest();
        let source_type = SourceType::cjs();
        test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
        test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);

        test_options_source_type("class C {}", "class C {}", source_type, &options);
    }
}
