use oxc_ast::{
    ast::{AssignmentTarget, BindingPatternKind, Expression, SimpleAssignmentTarget},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-this-assignment): Do not assign `this` to `{1}`")]
#[diagnostic(
    severity(warning),
    help("Reference `this` directly instead of assigning it to a variable.")
)]
struct NoThisAssignmentDiagnostic(#[label] pub Span, Atom);

#[derive(Debug, Default, Clone)]
pub struct NoThisAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow assigning `this` to a variable.
    ///
    /// ### Why is this bad?
    ///
    /// Assigning `this` to a variable is unnecessary and confusing.
    ///
    /// ### Example
    /// ```javascript
    /// // fail
    /// const foo = this;
    /// class Bar {
    /// 	method() {
    /// 		foo.baz();
    /// 	}
    /// }
    ///
    /// new Bar().method();
    ///
    /// // pass
    /// class Bar {
    /// 	constructor(fooInstance) {
    /// 		this.fooInstance = fooInstance;
    /// 	}
    /// 	method() {
    /// 		this.fooInstance.baz();
    /// 	}
    /// }
    ///
    /// new Bar(this).method();
    /// ```
    NoThisAssignment,
    pedantic
);

impl Rule for NoThisAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_decl) => {
                let Some(init) = &variable_decl.init else {
                    return;
                };

                if !matches!(init.without_parenthesized(), Expression::ThisExpression(_)) {
                    return;
                }

                let BindingPatternKind::BindingIdentifier(binding_ident) = &variable_decl.id.kind
                else {
                    return;
                };

                ctx.diagnostic(NoThisAssignmentDiagnostic(
                    variable_decl.span,
                    binding_ident.name.clone(),
                ));
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                if !matches!(
                    assignment_expr.right.without_parenthesized(),
                    Expression::ThisExpression(_)
                ) {
                    return;
                }

                let AssignmentTarget::SimpleAssignmentTarget(simple_assignment_target) =
                    &assignment_expr.left
                else {
                    return;
                };

                let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) =
                    simple_assignment_target
                else {
                    return;
                };

                ctx.diagnostic(NoThisAssignmentDiagnostic(
                    assignment_expr.span,
                    ident.name.clone(),
                ));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const {property} = this;",
        r"const property = this.property;",
        r"const [element] = this;",
        r"const element = this[0];",
        r"([element] = this);",
        r"element = this[0];",
        r"property = this.property;",
        r"const [element] = [this];",
        r"([element] = [this]);",
        r"const {property} = {property: this};",
        r"({property} = {property: this});",
        r"const self = true && this;",
        r"const self = false || this;",
        r"const self = false ?? this;",
        r"foo.bar = this;",
        r"function foo(a = this) {}",
        r"function foo({a = this}) {}",
        r"function foo([a = this]) {}",
    ];

    let fail = vec![
        r"const foo = this;",
        r"let foo;foo = this;",
        r"var foo = bar, baz = this;",
        r"var foo = (bar), baz = (this);",
    ];

    Tester::new_without_config(NoThisAssignment::NAME, pass, fail).test_and_snapshot();
}
