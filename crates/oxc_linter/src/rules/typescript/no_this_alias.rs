use oxc_ast::{
    ast::{AssignmentTarget, BindingPatternKind, Expression, SimpleAssignmentTarget},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-this-alias): Unexpected aliasing of 'this' to local variable.")]
#[diagnostic(
    severity(warning),
    help(
        "Assigning a variable to this instead of properly using arrow lambdas may be a symptom of pre-ES6 practices or not managing scope well."
    )
)]
struct NoThisAliasDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "typescript-eslint(no-this-alias): Unexpected aliasing of members of 'this' to local variables."
)]
#[diagnostic(
    severity(warning),
    help("Disabling destructuring of this is not a default, consider allowing destructuring")
)]
struct NoThisDestructureDiagnostic(#[label] pub Span);

#[derive(Debug, Clone)]
pub struct NoThisAlias {
    allow_destructuring: bool,
    allow_names: Vec<Atom>,
}

impl Default for NoThisAlias {
    fn default() -> Self {
        Self { allow_destructuring: true, allow_names: vec![] }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary constraints on generic types.
    ///
    /// ### Why is this bad?
    ///
    /// Generic type parameters (<T>) in TypeScript may be "constrained" with an extends keyword.
    /// When no extends is provided, type parameters default a constraint to unknown. It is therefore redundant to extend from any or unknown.
    ///
    /// the rule doesn't allow const {allowedName} = this
    /// this is to keep 1:1 with eslint implementation
    /// sampe with obj.<allowedName> = this
    /// ```
    NoThisAlias,
    correctness
);

impl Rule for NoThisAlias {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        let allowed_names = value
            .get(0)
            .and_then(|v| v.get("allow_names"))
            .and_then(serde_json::Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .map(serde_json::Value::as_str)
            .filter(std::option::Option::is_some)
            .map(|x| Atom::from(x.unwrap().to_string()))
            .collect::<Vec<Atom>>();

        Self {
            allow_destructuring: obj
                .and_then(|v| v.get("allow_destructuring"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
            allow_names: allowed_names,
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }
        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                let Some(init) = &decl.init else { return };

                if !rhs_is_this_reference(init) {
                    return;
                }

                if self.allow_destructuring
                    && !matches!(decl.id.kind, BindingPatternKind::BindingIdentifier(_))
                {
                    return;
                }

                if let BindingPatternKind::BindingIdentifier(identifier) = &decl.id.kind {
                    if !self.allow_names.contains(&identifier.name) {
                        ctx.diagnostic(NoThisAliasDiagnostic(identifier.span));
                    }

                    return;
                }
                ctx.diagnostic(NoThisDestructureDiagnostic(decl.id.kind.span()));
            }
            AstKind::AssignmentExpression(assignment) => {
                if !rhs_is_this_reference(&assignment.right) {
                    return;
                }
                match &assignment.left {
                    AssignmentTarget::AssignmentTargetPattern(pat) => {
                        if self.allow_destructuring {
                            return;
                        }
                        ctx.diagnostic(NoThisDestructureDiagnostic(pat.span()));
                    }
                    AssignmentTarget::SimpleAssignmentTarget(pat) => match pat {
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                            if !self.allow_names.contains(&id.name) {
                                ctx.diagnostic(NoThisAliasDiagnostic(id.span));
                            }
                        }
                        _ => {
                            if let Some(expr) = pat.get_expression() {
                                if let Some(id) = expr.get_identifier_reference() {
                                    if !self.allow_names.contains(&id.name) {
                                        ctx.diagnostic(NoThisAliasDiagnostic(id.span));
                                    }
                                }
                            }
                        }
                    },
                }
            }
            _ => {}
        }
    }
}

fn rhs_is_this_reference(rhs_expression: &Expression) -> bool {
    matches!(rhs_expression, Expression::ThisExpression(_))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // allow destructuring
        (
            "const { props, state } = this;",
            Some(serde_json::json!([{ "allow_destructuring": true }])),
        ),
        ("const { length } = this;", Some(serde_json::json!([{ "allow_destructuring": true }]))),
        (
            "const { length, toString } = this;",
            Some(serde_json::json!([{ "allow_destructuring": true }])),
        ),
        ("const [foo] = this;", Some(serde_json::json!([{ "allow_destructuring": true }]))),
        ("const [foo, bar] = this;", Some(serde_json::json!([{ "allow_destructuring": true }]))),
        // allow list
        ("const self = this;", Some(serde_json::json!([{ "allow_names": vec!["self"] }]))),
    ];

    let fail = vec![
        ("const self = this;", None),
        (
            "const { props, state } = this;",
            Some(serde_json::json!([{ "allow_destructuring": false }])),
        ),
        (
            "const [ props, state ] = this;",
            Some(serde_json::json!([{ "allow_destructuring": false }])),
        ),
        ("let foo; \nconst other =3;\n\n\n\nfoo = this", None),
        ("let foo; (foo as any) = this", None),
        (
            "function testFunction() {
            let inFunction = this;
          }",
            None,
        ),
        (
            "const testLambda = () => {
            const inLambda = this;
          };",
            None,
        ),
        // this slightly modified because conflicting ID's wont compile
        (
            "class TestClass {
            constructor() {
              const inConstructor = this;
              const asThis: this = this;

              const asString = 'this';
              const asArray = [this];
              const asArrayString = ['this'];
            }

            public act(scope: this = this) {
              const inMemberFunction = this;
              const { act1 } = this;
              const { act2, constructor } = this;
              const [foo1] = this;
              const [foo, bar] = this;
            }
          }",
            Some(serde_json::json!([{ "allow_destructuring": false }])),
        ),
    ];

    Tester::new(NoThisAlias::NAME, pass, fail).test_and_snapshot();
}
