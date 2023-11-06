use oxc_ast::{
    ast::{BindingPatternKind, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NoObjectAsDefaultParameterDiagnostic {
    #[error("eslint-plugin-unicorn(no-object-as-default-parameter): Do not use an object literal as default for parameter `{1}`.")]
    #[diagnostic(severity(warning))]
    Identifier(#[label] Span, String),
    #[error("eslint-plugin-unicorn(no-object-as-default-parameter): Do not use an object literal as default")]
    #[diagnostic(severity(warning))]
    NonIdentifier(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoObjectAsDefaultParameter;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of an object literal as a default value for a parameter.
    ///
    /// ### Why is this bad?
    ///
    /// Default parameters should not be passed to a function through an object literal. The `foo = {a: false}` parameter works fine if only used with one option. As soon as additional options are added, you risk replacing the whole `foo = {a: false, b: true}` object when passing only one option: `{a: true}`. For this reason, object destructuring should be used instead.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// function foo(foo = {a: false}) {}
    ///
    /// // Good
    /// function foo({a = false} = {}) {}
    /// ```
    NoObjectAsDefaultParameter,
    pedantic
);

impl Rule for NoObjectAsDefaultParameter {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentPattern(assignment_pat) = node.kind() else { return };

        let Expression::ObjectExpression(object_expr) = &assignment_pat.right else {
            return;
        };

        if object_expr.properties.len() == 0 {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else { return };

        if !matches!(parent.kind(), AstKind::FormalParameter(_)) {
            return;
        }

        if let BindingPatternKind::BindingIdentifier(binding_id) = &assignment_pat.left.kind {
            ctx.diagnostic(NoObjectAsDefaultParameterDiagnostic::Identifier(
                object_expr.span,
                binding_id.name.to_string(),
            ));
            return;
        }

        ctx.diagnostic(NoObjectAsDefaultParameterDiagnostic::NonIdentifier(object_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const abc = {};"#,
        r#"const abc = {foo: 123};"#,
        r#"function abc(foo) {}"#,
        r#"function abc(foo = null) {}"#,
        r#"function abc(foo = undefined) {}"#,
        r#"function abc(foo = 123) {}"#,
        r#"function abc(foo = true) {}"#,
        r#"function abc(foo = "bar") {}"#,
        r#"function abc(foo = 123, bar = "foo") {}"#,
        r#"function abc(foo = {}) {}"#,
        r#"function abc({foo = 123} = {}) {}"#,
        r#"(function abc() {})(foo = {a: 123})"#,
        r#"const abc = foo => {};"#,
        r#"const abc = (foo = null) => {};"#,
        r#"const abc = (foo = undefined) => {};"#,
        r#"const abc = (foo = 123) => {};"#,
        r#"const abc = (foo = true) => {};"#,
        r#"const abc = (foo = "bar") => {};"#,
        r#"const abc = (foo = 123, bar = "foo") => {};"#,
        r#"const abc = (foo = {}) => {};"#,
        r#"const abc = ({a = true, b = "foo"}) => {};"#,
        r#"const abc = function(foo = 123) {}"#,
        r#"const {abc = {foo: 123}} = bar;"#,
        r#"const {abc = {null: "baz"}} = bar;"#,
        r#"const {abc = {foo: undefined}} = undefined;"#,
        r#"const abc = ([{foo = false, bar = 123}]) => {};"#,
        r#"const abc = ({foo = {a: 123}}) => {};"#,
        r#"const abc = ([foo = {a: 123}]) => {};"#,
        r#"const abc = ({foo: bar = {a: 123}}) => {};"#,
        r#"const abc = () => (foo = {a: 123});"#,
    ];

    let fail = vec![
        r#"function abc(foo = {a: 123}) {}"#,
        r#"async function * abc(foo = {a: 123}) {}"#,
        r#"function abc(foo = {a: false}) {}"#,
        r#"function abc(foo = {a: "bar"}) {}"#,
        r#"function abc(foo = {a: "bar", b: {c: true}}) {}"#,
        r#"const abc = (foo = {a: false}) => {};"#,
        r#"const abc = (foo = {a: 123, b: false}) => {};"#,
        r#"const abc = (foo = {a: false, b: 1, c: "test", d: null}) => {};"#,
        r#"const abc = function(foo = {a: 123}) {}"#,
        r#"function abc(foo = {a: 123}) {}"#,
        r#"const abc = (foo = {a: false}) => {};"#,
        r#"function abc({a} = {a: 123}) {}"#,
        r#"function abc([a] = {a: 123}) {}"#,
    ];

    Tester::new_without_config(NoObjectAsDefaultParameter::NAME, pass, fail).test_and_snapshot();
}
