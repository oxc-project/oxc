use oxc_ast::{
    ast::{AssignmentTarget, BindingPatternKind, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_this_assignment_diagnostic(span: Span, ident_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not assign `this` to `{ident_name}`"))
        .with_help("Reference `this` directly instead of assigning it to a variable.")
        .with_label(span)
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = this;
    /// class Bar {
    /// 	method() {
    /// 		foo.baz();
    /// 	}
    /// }
    ///
    /// new Bar().method();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
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
    unicorn,
    pedantic
);

impl Rule for NoThisAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_decl) => {
                let Some(init) = &variable_decl.init else {
                    return;
                };

                if !matches!(init.without_parentheses(), Expression::ThisExpression(_)) {
                    return;
                }

                let BindingPatternKind::BindingIdentifier(binding_ident) = &variable_decl.id.kind
                else {
                    return;
                };

                ctx.diagnostic(no_this_assignment_diagnostic(
                    variable_decl.span,
                    binding_ident.name.as_str(),
                ));
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                if !matches!(
                    assignment_expr.right.without_parentheses(),
                    Expression::ThisExpression(_)
                ) {
                    return;
                }

                let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment_expr.left
                else {
                    return;
                };

                ctx.diagnostic(no_this_assignment_diagnostic(
                    assignment_expr.span,
                    ident.name.as_str(),
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

    Tester::new(NoThisAssignment::NAME, NoThisAssignment::PLUGIN, pass, fail).test_and_snapshot();
}
