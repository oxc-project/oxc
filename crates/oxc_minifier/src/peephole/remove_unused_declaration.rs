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
        // Unsafe to remove `using`, unable to statically determine usage of [Symbol.dispose].
        if decl.kind.is_using() {
            return false;
        }
        match &decl.id.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                if let Some(symbol_id) = ident.symbol_id.get() {
                    return ctx.scoping().symbol_is_unused(symbol_id);
                }
                false
            }
            BindingPatternKind::ArrayPattern(ident) => ident.is_empty(),
            BindingPatternKind::ObjectPattern(ident) => ident.is_empty(),
            BindingPatternKind::AssignmentPattern(_) => false,
        }
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
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
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
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
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

    /// Remove unused specifiers from import declarations.
    ///
    /// Since we don't know if an import has side effects, we convert imports
    /// with all unused specifiers to side-effect-only imports (`import 'x'`)
    /// rather than removing them entirely.
    ///
    /// ## Example
    ///
    /// Input:
    /// ```js
    /// import a from 'a'
    /// import { b } from 'b'
    ///
    /// if (false) {
    ///   console.log(b)
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// import 'a'
    /// import 'b'
    /// ```
    pub fn remove_unused_import_specifiers(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }

        if ctx.scoping().root_scope_flags().contains_direct_eval() {
            return;
        }

        debug_assert!(!ctx.source_type().is_script(), "imports are not allowed in script mode");

        let Statement::ImportDeclaration(import_decl) = stmt else { return };

        if import_decl.phase.is_some() {
            return;
        }

        let Some(specifiers) = &mut import_decl.specifiers else {
            return;
        };

        let original_len = specifiers.len();

        specifiers.retain(|specifier| {
            let local = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => &s.local,
            };

            let symbol_id = local.symbol_id();
            !ctx.scoping().symbol_is_unused(symbol_id)
        });

        if specifiers.len() != original_len {
            ctx.state.changed = true;
        }

        if specifiers.is_empty() {
            import_decl.specifiers = None;
            ctx.state.changed = true;
        }
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
        test_options("var [] = []", "", &options);
        test_options("var [] = [1]", "", &options);
        test_options("var [] = [foo]", "foo", &options);
        test_options("var {} = {}", "", &options);
        test_options("var {} = { a: 1 }", "", &options);
        test_options("var {} = { foo }", "foo", &options);
        test_options("var {} = { foo: { a } }", "a", &options);
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
        test_same_options("function foo() { bar } eval('foo()')", &options);
    }

    #[test]
    fn remove_unused_class_declaration() {
        let options = CompressOptions::smallest();
        test_options("class C {}", "", &options);
        test_same_options("export class C {}", &options);
        test_options("class C {} C", "", &options);
        test_same_options("class C {} eval('C')", &options);

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
        test_same_options("@dec class C {}", &options);

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

    #[test]
    fn remove_unused_import_specifiers() {
        let options = CompressOptions::smallest();

        test_options("import a from 'a'", "import 'a';", &options);
        test_options("import a from 'a'; foo()", "import 'a'; foo();", &options);

        test_options("import { a } from 'a'", "import 'a';", &options);
        test_options("import { a, b } from 'a'", "import 'a';", &options);

        test_options("import * as a from 'a'", "import 'a';", &options);

        test_options("import a, { b } from 'a'", "import 'a';", &options);
        test_options("import a, * as b from 'a'", "import 'a';", &options);

        test_same_options("import a from 'a'; foo(a);", &options);
        test_same_options("import { a } from 'a'; foo(a);", &options);
        test_same_options("import * as a from 'a'; foo(a);", &options);
        test_same_options("import a, { b } from 'a'; foo(a, b);", &options);

        test_options(
            "import { a, b } from 'a'; foo(a);",
            "import { a } from 'a'; foo(a);",
            &options,
        );
        test_options(
            "import { a, b, c } from 'a'; foo(b);",
            "import { b } from 'a'; foo(b);",
            &options,
        );
        test_options("import a, { b } from 'a'; foo(a);", "import a from 'a'; foo(a);", &options);
        test_options(
            "import a, { b } from 'a'; foo(b);",
            "import { b } from 'a'; foo(b);",
            &options,
        );

        test_options(
            "import a from 'a'; import { b } from 'b'; if (false) { console.log(b) }",
            "import 'a'; import 'b';",
            &options,
        );

        test_same_options("import 'a';", &options);

        test_options("import {} from 'a'", "import 'a';", &options);

        test_options(
            "import a from 'a' with { type: 'json' }",
            "import 'a' with { type: 'json' };",
            &options,
        );
        test_options(
            "import {} from 'a' with { type: 'json' }",
            "import 'a' with { type: 'json' };",
            &options,
        );

        test_options("import { a as b } from 'a'", "import 'a';", &options);
        test_same_options("import { a as b } from 'a'; foo(b);", &options);

        test_same_options("import { a } from 'a'; export { a };", &options);
        // Keep imports when direct eval is present
        test_same_options("import { a } from 'a'; eval('a');", &options);
        test_same_options("import a from 'a'; eval('a');", &options);
        test_same_options("import * as a from 'a'; eval('a');", &options);
        test_same_options("import { a } from 'a'; function f() { eval('a'); }", &options);
    }
}
