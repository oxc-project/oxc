use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_global_object_property_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not assign properties on the global object.")
        .with_help("Store application state in a module or another explicitly owned object.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoGlobalObjectPropertyAssignment;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assigning properties on the global object.
    ///
    /// ### Why is this bad?
    ///
    /// Mutating the global object creates implicit shared state and can overwrite
    /// properties provided by the runtime or another library.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// globalThis.appState = state;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const app = { state };
    /// ```
    NoGlobalObjectPropertyAssignment,
    unicorn,
    suspicious,
    none,
    version = "next",
    short_description = "Disallow assigning properties on the global object.",
);

impl Rule for NoGlobalObjectPropertyAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member) = node.kind().as_member_expression_kind() else { return };
        if member.static_property_name().is_none() {
            return;
        }

        let Expression::Identifier(object) = member.object().get_inner_expression() else {
            return;
        };
        if !matches!(object.name.as_str(), "global" | "globalThis" | "self" | "window")
            || !object.is_global_reference(ctx.scoping())
            || !is_assignment_target(node, member, ctx)
        {
            return;
        }

        ctx.diagnostic(no_global_object_property_assignment_diagnostic(node.span()));
    }
}

fn is_assignment_target<'a>(
    node: &AstNode<'a>,
    member: oxc_ast::MemberExpressionKind<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let mut target = node;
    let mut parent = ctx.nodes().parent_node(target.id());
    while matches!(
        parent.kind(),
        AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSNonNullExpression(_)
    ) {
        target = parent;
        parent = ctx.nodes().parent_node(target.id());
    }

    if target.id() == node.id() {
        return member.is_assigned_to_in_parent(&parent.kind());
    }

    match parent.kind() {
        AstKind::AssignmentExpression(assignment) => assignment.left.span() == target.span(),
        AstKind::UpdateExpression(update) => update.argument.span() == target.span(),
        AstKind::ForInStatement(for_in) => for_in.left.span() == target.span(),
        AstKind::ForOfStatement(for_of) => for_of.left.span() == target.span(),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "globalThis.foo",
        "window.foo()",
        "globalThis = value",
        "globalThis[property] = value",
        "delete globalThis.foo",
        "Object.assign(globalThis, {foo: value})",
        r#"Reflect.set(globalThis, "foo", value)"#,
        "function test(window) {
                window.foo = 1;
            }",
        "const global = {};
            global.foo = 1;",
        "const globalThis = {};
            globalThis.foo = 1;",
        "const self = {};
            self.foo++;",
        "const root = globalThis;
            root.foo = 1;",
    ];

    let fail = vec![
        "globalThis.foo = 1",
        "window.foo += 1",
        "self.foo ||= value",
        "global.foo++",
        r#"globalThis["foo"] = 1"#,
        "({
                foo: globalThis.foo,
            } = object);",
        "[globalThis.foo] = array",
        "({...globalThis.foo} = object)",
        "[...globalThis.foo] = array",
        "for (globalThis.foo of iterable) {}",
        "for (globalThis.foo in object) {}",
        "globalThis!.foo = 1",                // {"parser": parsers.typescript},
        "(globalThis as any).foo = 1",        // {"parser": parsers.typescript},
        "(<any>globalThis).foo = 1",          // {"parser": parsers.typescript},
        "(globalThis satisfies any).foo = 1", // {"parser": parsers.typescript},
        "globalThis.foo! = 1",                // {"parser": parsers.typescript}
    ];

    Tester::new(
        NoGlobalObjectPropertyAssignment::NAME,
        NoGlobalObjectPropertyAssignment::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("ts")
    .test_and_snapshot();
}
