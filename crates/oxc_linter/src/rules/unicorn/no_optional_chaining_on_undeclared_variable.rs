use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_optional_chaining_on_undeclared_variable_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Optional chaining on undeclared variable `{name}` throws a ReferenceError."
    ))
    .with_help(format!("Declare `{name}` before using optional chaining."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOptionalChainingOnUndeclaredVariable;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows optional chaining on undeclared variables.
    ///
    /// ### Why is this bad?
    ///
    /// Optional chaining only guards against `null` and `undefined` values. It does not guard
    /// against an undeclared root variable, which throws a `ReferenceError` before the chain is
    /// evaluated.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// undeclared?.property;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let declared;
    /// declared?.property;
    /// ```
    NoOptionalChainingOnUndeclaredVariable,
    unicorn,
    correctness,
    none,
    version = "next",
    short_description = "Disallow optional chaining on undeclared variables.",
);

impl Rule for NoOptionalChainingOnUndeclaredVariable {
    fn run_once(&self, ctx: &LintContext) {
        for reference_ids in ctx.scoping().root_unresolved_references_ids() {
            for reference_id in reference_ids {
                let reference = ctx.scoping().get_reference(reference_id);
                if reference.is_type() {
                    continue;
                }

                let name = ctx.semantic().reference_name(reference);
                if ctx.is_global_defined(name) {
                    continue;
                }

                let node = ctx.nodes().get_node(reference.node_id());
                if is_optional_chain_root(node.id(), ctx) {
                    ctx.diagnostic(no_optional_chaining_on_undeclared_variable_diagnostic(
                        name,
                        node.kind().span(),
                    ));
                }
            }
        }
    }
}

fn is_optional_chain_root(mut node_id: NodeId, ctx: &LintContext) -> bool {
    let mut has_optional_operation = false;

    loop {
        let node = ctx.nodes().get_node(node_id);
        let parent = ctx.nodes().parent_node(node_id);

        match parent.kind() {
            kind if kind.is_member_expression_kind() => {
                let member = kind.as_member_expression_kind().unwrap();
                if !member.object().span().contains_inclusive(node.kind().span()) {
                    return false;
                }
                has_optional_operation |= member.optional();
            }
            AstKind::CallExpression(call) => {
                if !call.callee.span().contains_inclusive(node.kind().span()) {
                    return false;
                }
                // A non-optional call before the optional operation produces a value. The
                // identifier used to call it is not the base guarded by optional chaining.
                if !call.optional && !has_optional_operation {
                    return false;
                }
                has_optional_operation |= call.optional;
            }
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSInstantiationExpression(_)
            | AstKind::TSNonNullExpression(_) => {}
            AstKind::ChainExpression(_) => return has_optional_operation,
            _ => return false,
        }

        node_id = parent.id();
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::{TestCase, Tester};

    let pass: Vec<TestCase> = vec![
        "let foo; foo?.bar;".into(),
        "function fn(foo) { foo?.(); }".into(),
        ("foo?.bar;", None, Some(json!({ "globals": { "foo": "readonly" } }))).into(),
        "globalThis.foo?.bar;".into(),
        "globalThis.foo?.();".into(),
        "getFoo()?.bar;".into(),
        "(foo || bar)?.baz;".into(),
        "this?.foo;".into(),
        "foo().bar?.baz;".into(),
        "let foo = {}; let bar; foo[bar]?.baz;".into(),
        "let foo; let bar; foo?.[bar];".into(),
        "let foo; let bar; foo?.(bar);".into(),
        "class Foo extends Bar {
                method() {
                    super.foo?.();
                }
            }"
        .into(),
        "let foo;
            function fn() {
                foo?.bar;
            }"
        .into(),
        "let foo; (foo?.bar as Foo)?.baz;".into(),
        "let foo; (foo<string>)?.bar;".into(),
        r#"import {foo} from "foo"; foo?.bar;"#.into(),
        "type foo = {}; const foo = {}; foo?.bar;".into(),
        ("type foo = {}; foo?.bar;", None, Some(json!({ "globals": { "foo": "readonly" } })))
            .into(),
    ];

    let fail: Vec<TestCase> = vec![
        "foo?.bar;",
        "foo?.();",
        "foo?.bar();",
        "foo?.bar?.baz;",
        "foo.bar?.();",
        "foo.bar?.baz;",
        "foo?.().bar?.baz;",
        "foo?.bar().baz?.qux;",
        "(foo?.bar)?.baz;",
        "(foo?.bar)?.();",
        "foo[bar]?.baz;",
        "foo?.[bar];",
        "function fn() {
                foo?.bar;
            }",
        "(foo as Foo)?.bar;",         // {"parser": parsers.typescript},
        "foo!.bar?.();",              // {"parser": parsers.typescript},
        "(foo?.bar as Foo)?.baz;",    // {"parser": parsers.typescript},
        "(foo<string>)?.bar;",        // {"parser": parsers.typescript},
        "(foo<string>)?.();",         // {"parser": parsers.typescript},
        "(foo<string>).bar?.baz;",    // {"parser": parsers.typescript},
        "type foo = {}; foo?.bar;",   // {"parser": parsers.typescript},
        "interface foo {} foo?.bar;", // {"parser": parsers.typescript},
        r#"import type {foo} from "foo"; foo?.bar;"#, // {"parser": parsers.typescript},
        r#"import {type foo} from "foo"; foo.bar?.baz;"#, // {"parser": parsers.typescript}
    ]
    .into_iter()
    .map(Into::into)
    .collect();

    Tester::new(
        NoOptionalChainingOnUndeclaredVariable::NAME,
        NoOptionalChainingOnUndeclaredVariable::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("ts")
    .test_and_snapshot();
}
