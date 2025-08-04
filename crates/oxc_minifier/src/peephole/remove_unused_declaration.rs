use oxc_ast::ast::*;

use crate::{CompressOptionsUnused, ctx::Ctx};

use super::PeepholeOptimizations;

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
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return None;
        }
        Some(ctx.ast.statement_empty(f.span))
    }

    pub fn remove_unused_class_declaration(
        c: &mut Class<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return None;
        }
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return None;
        }
        let id = c.id.as_ref()?;
        let symbol_id = id.symbol_id.get()?;
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return None;
        }
        Self::remove_unused_class(c, ctx).map(|exprs| {
            if exprs.is_empty() {
                ctx.ast.statement_empty(c.span)
            } else {
                let expr = ctx.ast.expression_sequence(c.span, exprs);
                ctx.ast.statement_expression(c.span, expr)
            }
        })
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
        test_options("class C { static foo = bar }", "bar", &options);
        test_options("class C { static foo = 1 }", "", &options);
        test_options("class C { [foo] = bar }", "foo", &options);
        test_options("class C { [foo] = 1 }", "foo", &options);
        test_options("class C { static [foo] = bar }", "foo, bar", &options);
        test_options("class C { static [foo] = 1 }", "foo", &options);

        // accessor
        test_options("class C { accessor foo = 1 }", "", &options);
        test_options("class C { accessor [foo] = 1 }", "foo", &options);

        // order
        test_options(
            "class _ extends A { static [B] = C; static [D]() {} }",
            "A, B, C, D",
            &options,
        );

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
