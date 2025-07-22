use oxc_ast::{
    AstKind,
    ast::{Expression, ImportDeclaration, ImportDeclarationSpecifier},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn consistent_assert_diagnostic(assert_identifier: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Inconsistent assert usage")
        .with_help(format!("Prefer {assert_identifier}.ok(...) over {assert_identifier}(...)"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentAssert;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent usage of the `assert` module.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent usage of the `assert` module can lead to confusion and errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import assert from 'node:assert';
    ///
    /// assert(divide(10, 2) === 5);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import assert from 'node:assert';
    ///
    /// assert.ok(divide(10, 2) === 5);
    /// ```
    ConsistentAssert,
    unicorn,
    pedantic,
    fix,
);

impl Rule for ConsistentAssert {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else { return };

        if !is_assert_module_import(import_decl) {
            return;
        }

        for imported_assert_symbol_id in find_assert_imports(import_decl) {
            check_assert_calls(imported_assert_symbol_id, ctx);
        }
    }
}

fn is_assert_module_import(import_decl: &ImportDeclaration) -> bool {
    is_assert_module(import_decl) || is_strict_assert_module(import_decl)
}

fn is_assert_module(import_decl: &ImportDeclaration) -> bool {
    let module_name = import_decl.source.value.as_str();
    ["assert", "node:assert"].contains(&module_name)
}

fn is_strict_assert_module(import_decl: &ImportDeclaration) -> bool {
    let module_name = import_decl.source.value.as_str();
    ["assert/strict", "node:assert/strict"].contains(&module_name)
}

fn find_assert_imports(import_decl: &ImportDeclaration<'_>) -> Vec<SymbolId> {
    let mut assert_imports: Vec<SymbolId> = Vec::new();

    if let Some(specifiers) = &import_decl.specifiers {
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) => {
                    assert_imports.push(default_specifier.local.symbol_id());
                }
                ImportDeclarationSpecifier::ImportSpecifier(named_specifier) => {
                    let imported = &named_specifier.imported;
                    if imported.name() == "default"
                        || (is_assert_module(import_decl) && imported.name() == "strict")
                    {
                        assert_imports.push(named_specifier.local.symbol_id());
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }

    assert_imports
}

fn check_assert_calls(symbol_id: SymbolId, ctx: &LintContext<'_>) {
    let references = ctx.semantic().symbol_references(symbol_id);

    for reference in references {
        let parent = ctx.nodes().parent_node(reference.node_id());

        match parent.kind() {
            AstKind::CallExpression(call_expr) => {
                if let Expression::Identifier(ident) = &call_expr.callee {
                    ctx.diagnostic_with_fix(
                        consistent_assert_diagnostic(&ident.name, ident.span),
                        |fixer| fixer.insert_text_after(&ident.span, ".ok"),
                    );
                }
            }
            AstKind::ParenthesizedExpression(paren_expr) => {
                let Expression::Identifier(ident) = &paren_expr.expression else {
                    continue;
                };
                ctx.diagnostic_with_fix(
                    consistent_assert_diagnostic(&ident.name, ident.span),
                    |fixer| fixer.insert_text_after(&ident.span, ".ok"),
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "assert(foo)",
        r#"import assert from "assert";"#,
        "import assert from 'node:assert';
			assert;",
        "import customAssert from 'node:assert';
			assert(foo);",
        "function foo (assert) {
				assert(bar);
			}",
        "import assert from 'node:assert';
			function foo (assert) {
				assert(bar);
			}",
        "import {strict} from 'node:assert/strict';
			strict(foo);",
        "import * as assert from 'node:assert';
			assert(foo);",
        "export * as assert from 'node:assert';
			assert(foo);",
        "export {default as assert} from 'node:assert';
			export {assert as strict} from 'node:assert';
			assert(foo);",
        "import assert from 'node:assert/strict';
			console.log(assert)",
    ];

    let fail = vec![
        "import assert from 'assert';
        	assert(foo)",
        "import assert from 'node:assert';
        	assert(foo)",
        "import assert from 'assert/strict';
        	assert(foo)",
        "import assert from 'node:assert/strict';
        	assert(foo)",
        "import customAssert from 'assert';
        	customAssert(foo)",
        "import customAssert from 'node:assert';
        	customAssert(foo)",
        "import assert from 'assert';
        	assert(foo)
        	assert(bar)
        	assert(baz)",
        "import {strict} from 'assert';
        	strict(foo)",
        "import {strict as assert} from 'assert';
        	assert(foo)",
        "import a, {strict as b, default as c} from 'node:assert';
        	import d, {strict as e, default as f} from 'assert';
        	import g, {default as h} from 'node:assert/strict';
        	import i, {default as j} from 'assert/strict';
        	a(foo);
        	b(foo);
        	c(foo);
        	d(foo);
        	e(foo);
        	f(foo);
        	g(foo);
        	h(foo);
        	i(foo);
        	j(foo);",
        "import assert from 'node:assert';
        	assert?.(foo)",
        "import assert from 'assert';
			((
				/* comment */ ((
					/* comment */
					assert
					/* comment */
					)) /* comment */
					(/* comment */ typeof foo === 'string', 'foo must be a string' /** after comment */)
			));",
    ];

    let fix = vec![(
        "import assert from 'assert';
			assert(foo)",
        "import assert from 'assert';
			assert.ok(foo)",
    ), (
        "import assert from 'node:assert';
            assert(foo)",
        "import assert from 'node:assert';
            assert.ok(foo)",
    ), (
        "import assert from 'assert/strict';
            assert(foo)",
        "import assert from 'assert/strict';
            assert.ok(foo)",
    ), (
        "import assert from 'node:assert/strict';
            assert(foo)",
        "import assert from 'node:assert/strict';
            assert.ok(foo)",
    ), (
        "import customAssert from 'assert';
            customAssert(foo)",
        "import customAssert from 'assert';
            customAssert.ok(foo)",
    ), (
        "import customAssert from 'node:assert';
            customAssert(foo)",
        "import customAssert from 'node:assert';
            customAssert.ok(foo)",
    ), (
        "import assert from 'assert';
            assert(foo)
            assert(bar)
            assert(baz)",
        "import assert from 'assert';
            assert.ok(foo)
            assert.ok(bar)
            assert.ok(baz)",
    ), (
        "import {strict} from 'assert';
            strict(foo)",
        "import {strict} from 'assert';
            strict.ok(foo)",
    ), (
        "import {strict as assert} from 'assert';
            assert(foo)",
        "import {strict as assert} from 'assert';
            assert.ok(foo)",
    ), (
        "import a, {strict as b, default as c} from 'node:assert';
            import d, {strict as e, default as f} from 'assert';
            import g, {default as h} from 'node:assert/strict';
            import i, {default as j} from 'assert/strict';
            a(foo);
            b(foo);
            c(foo);
            d(foo);
            e(foo);
            f(foo);
            g(foo);
            h(foo);
            i(foo);
            j(foo);",
        "import a, {strict as b, default as c} from 'node:assert';
            import d, {strict as e, default as f} from 'assert';
            import g, {default as h} from 'node:assert/strict';
            import i, {default as j} from 'assert/strict';
            a.ok(foo);
            b.ok(foo);
            c.ok(foo);
            d.ok(foo);
            e.ok(foo);
            f.ok(foo);
            g.ok(foo);
            h.ok(foo);
            i.ok(foo);
            j.ok(foo);",
    ), (
        "import assert from 'node:assert';
            assert?.(foo)",
        "import assert from 'node:assert';
            assert.ok?.(foo)",
    ), (
        "import assert from 'assert';
            ((
                /* comment */ ((
                    /* comment */
                    assert
                    /* comment */
                    )) /* comment */
                    (/* comment */ typeof foo === 'string', 'foo must be a string' /** after comment */)
            ));",
        "import assert from 'assert';
            ((
                /* comment */ ((
                    /* comment */
                    assert.ok
                    /* comment */
                    )) /* comment */
                    (/* comment */ typeof foo === 'string', 'foo must be a string' /** after comment */)
            ));",
    )];

    Tester::new(ConsistentAssert::NAME, ConsistentAssert::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
