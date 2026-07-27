use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_destructuring::{
        DOCUMENTATION, PreferDestructuring as PreferDestructuringInner, PreferDestructuringConfig,
    },
};

#[derive(Debug, Default, Clone)]
pub struct PreferDestructuring(PreferDestructuringInner);

declare_oxc_lint!(
    PreferDestructuring,
    typescript,
    style,
    conditional_fix,
    config = PreferDestructuringConfig,
    docs = DOCUMENTATION,
    version = "next",
    short_description = "Require destructuring from arrays and/or objects.",
);

impl Rule for PreferDestructuring {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        PreferDestructuringInner::from_configuration(value).map(Self)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign_expr) if assign_expr.operator.is_assign() => {
                self.0.run_on_assignment_expression(assign_expr, ctx);
            }
            AstKind::VariableDeclarator(declarator) => {
                self.0.run_on_variable_declarator(declarator, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var [foo] = array;", None),
        ("var { foo } = object;", None),
        // Declarations with a type annotation are ignored unless explicitly enforced.
        ("const foo: string = object.foo;", None),
        ("const foo: string = object['foo'];", None),
        ("const foo: string = array[0];", None),
        ("var foo = array[someIndex];", None),
        ("var foo = object.bar;", Some(serde_json::json!([{ "object": false }]))),
        ("var foo = array[0];", Some(serde_json::json!([{ "array": false }]))),
        ("({ foo } = object);", None),
        ("[foo] = array;", None),
        ("class Foo extends Bar { static foo() {var foo = super.foo} }", None),
        ("var foo = object?.foo;", None),
        ("var foo = array?.[0];", None),
        (
            "const foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": false }]),
            ),
        ),
    ];

    let fail = vec![
        ("var foo = array[0];", None),
        ("foo = array[0];", None),
        ("var foo = object.foo;", None),
        ("var foo = object['foo'];", None),
        (
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object['foo'];",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = array[0];",
            Some(
                serde_json::json!([{ "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foobar = object.bar;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        ("var foo = object.foo;", "var {foo} = object;", None),
        ("var foo = object['foo'];", "var {foo} = object;", None),
        // Type annotations disable the autofix even when the rule is enforced for them.
        (
            "var foo: string = object.foo;",
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
