use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentOperator, AssignmentTarget, Expression, FormalParameter,
        MethodDefinitionKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{Atom, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unnecessary_parameter_property_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Assignment of parameter property is unnecessary")
        .with_help("Remove the unnecessary assignment")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryParameterPropertyAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents unnecessary assignment of parameter properties.
    ///
    /// ### Why is this bad?
    ///
    /// Constructor parameters marked with one of the visibility modifiers
    /// public, private, protected, or readonly are automatically initialized.
    /// Providing an explicit assignment is unnecessary and can be removed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class Foo {
    ///   constructor(public name: unknown) {
    ///     this.name = name;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class Foo {
    ///   constructor(public name: unknown) {}
    /// }
    /// ```
    NoUnnecessaryParameterPropertyAssignment,
    typescript,
    nursery, // TODO: import tests from typescript-eslint, fix them and change back to correctness
    pending,
);

impl Rule for NoUnnecessaryParameterPropertyAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };
        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }
        for param in &method.value.params.items {
            if !is_parameter_property(param) {
                continue;
            }
            let Some(ident) = param.pattern.get_binding_identifier() else {
                continue;
            };
            for reference in ctx.symbol_references(ident.symbol_id()) {
                // check if the reference is being read which would indicate an assignment
                if !reference.is_read() {
                    continue;
                }

                // is the reference inside an assignment
                let Some(assignment_expr) =
                    find_parent_assignment_expression(ctx, reference.node_id())
                else {
                    continue;
                };

                if !is_unnecessary_assignment_operator(assignment_expr.operator) {
                    continue;
                }

                let Some(this_property_name) = get_this_property_name(&assignment_expr.left) else {
                    continue;
                };

                if this_property_name != ident.name {
                    continue;
                }

                ctx.diagnostic(no_unnecessary_parameter_property_assignment_diagnostic(
                    assignment_expr.span,
                ));
            }
        }
    }
}

/// TypeScript offers special syntax for turning a constructor parameter into a class property with the same name and value.
/// These are called parameter properties and are created by prefixing a constructor argument with one of the visibility modifiers public, private, protected, or readonly
///
/// https://www.typescriptlang.org/docs/handbook/2/classes.html#parameter-properties
fn is_parameter_property(param: &FormalParameter) -> bool {
    param.accessibility.is_some() || param.readonly
}

fn find_parent_assignment_expression<'a>(
    ctx: &LintContext<'a>,
    node_id: NodeId,
) -> Option<&'a AssignmentExpression<'a>> {
    for ancestor_kind in ctx.nodes().ancestor_kinds(node_id).skip(1) {
        match ancestor_kind {
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSInstantiationExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_) => continue,
            AstKind::AssignmentExpression(expr) => return Some(expr),
            _ => break,
        }
    }
    None
}

fn is_unnecessary_assignment_operator(operator: AssignmentOperator) -> bool {
    matches!(
        operator,
        AssignmentOperator::Assign
            | AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalNullish
    )
}

fn get_this_property_name<'a>(assignment_target: &AssignmentTarget<'a>) -> Option<Atom<'a>> {
    match assignment_target {
        AssignmentTarget::StaticMemberExpression(expr)
            if matches!(&expr.object, Expression::ThisExpression(_)) =>
        {
            // this.property
            Some(expr.property.name)
        }
        AssignmentTarget::ComputedMemberExpression(expr)
            if matches!(&expr.object, Expression::ThisExpression(_)) =>
        {
            // this["property"]
            if let Expression::StringLiteral(str) = &expr.expression {
                Some(str.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
        class Foo {
          constructor(public name: unknown) {}
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown, other: unknown) {
            this.other = other;
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            this.other = name;
          }
        }
        ",
        r"
        class Foo {
          constructor(name: unknown) {
            this.name = name;
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            this.name = 'other';
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            this.name = name + 'edited';
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            if (maybeTrue) {
              this.name = name + 'edited';
            }
          }
        }
        ",
        "
        class Foo {
          constructor(foo: string) {}
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {}
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.foo = bar;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: any) {
            this.foo = foo.bar;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.foo = this.bar;
          }
        }
        ",
        "
        class Foo {
          foo: string;
          constructor(foo: string) {
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          bar: string;
          constructor(private foo: string) {
            this.bar = foo;
          }
        }
        ",
        // "
        //     class Foo {
        //       constructor(private foo: string) {
        //         this.bar = () => {
        //           this.foo = foo;
        //         };
        //       }
        //     }
        // ",
        "
        class Foo {
          constructor(private foo: string) {
            this[`${foo}`] = foo;
          }
        }
        ",
        "
        function Foo(foo) {
          this.foo = foo;
        }
        ",
        "
        const foo = 'foo';
        this.foo = foo;
        ",
        "
        class Foo {
          constructor(public foo: number) {
            this.foo += foo;
            this.foo -= foo;
            this.foo *= foo;
            this.foo /= foo;
            this.foo %= foo;
            this.foo **= foo;
          }
        }
        ",
        // "
        //     class Foo {
        //       constructor(public foo: number) {
        //         this.foo += 1;
        //         this.foo = foo;
        //       }
        //     }
        // ",
        // "
        //     class Foo {
        //       constructor(
        //         public foo: number,
        //         bar: boolean,
        //       ) {
        //         if (bar) {
        //           this.foo += 1;
        //         } else {
        //           this.foo = foo;
        //         }
        //       }
        //     }
        // ",
        // "
        //     class Foo {
        //       constructor(public foo: number) {
        //         this.foo = foo;
        //       }
        //       init = (this.foo += 1);
        //     }
        // ",
        "
        class Foo {
          constructor(public foo: number) {
            {
              const foo = 1;
              this.foo = foo;
            }
          }
        }
        ",
        "
        declare const name: string;
        class Foo {
          constructor(public foo: number) {
            this[name] = foo;
          }
        }
        ",
        "
        declare const name: string;
        class Foo {
          constructor(public foo: number) {
            Foo.foo = foo;
          }
        }
        ",
        // "
        //     class Foo {
        //       constructor(public foo: number) {
        //         this.foo = foo;
        //       }
        //       init = (() => {
        //         this.foo += 1;
        //       })();
        //     }
        // ",
        "
        declare const name: string;
        class Foo {
          constructor(public foo: number) {
            this[name] = foo;
          }
          init = (this[name] = 1);
          init2 = (Foo.foo = 1);
        }
        ",
    ];

    let fail = vec![
        r"
        class Foo {
          constructor(public name: unknown) {
            this.name = name;
          }
        }
        ",
        r"
        class Foo {
          constructor(other: unknown, public name: unknown) {
            this.other = other;
            this.name = name;
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            this.name = name;
            this.name = name;
            this.name = name;
          }
        }
        ",
        r"
        class Foo {
          constructor(public name: unknown) {
            if (maybeTrue) {
              this.name = name;
            } else {
              this.name = name + 'edited';
            }
          }
        }
        ",
        "
        class Foo {
          constructor(public foo: string) {
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo?: string) {
            this.foo = foo!;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo?: string) {
            this.foo = foo as any;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo = '') {
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo = '') {
            this.foo = foo;
            this.foo += 'foo';
          }
        }
        ",
        "
        class Foo {
          constructor(public foo: string) {
            this.foo ||= foo;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo: string) {
            this.foo ??= foo;
          }
        }
        ",
        "
        class Foo {
          constructor(public foo: string) {
            this.foo &&= foo;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this['foo'] = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            function bar() {
              this.foo = foo;
            }
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.bar = () => {
              this.foo = foo;
            };
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            class Bar {
              constructor(private foo: string) {
                this.foo = foo;
              }
            }
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.foo = foo;
          }
          bar = () => {
            this.foo = 'foo';
          };
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.foo = foo;
          }
          init = foo => {
            this.foo = foo;
          };
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            this.foo = foo;
          }
          init = class Bar {
            constructor(private foo: string) {
              this.foo = foo;
            }
          };
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            {
              this.foo = foo;
            }
          }
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            (() => {
              this.foo = foo;
            })();
          }
        }
        ",
    ];

    Tester::new(
        NoUnnecessaryParameterPropertyAssignment::NAME,
        NoUnnecessaryParameterPropertyAssignment::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
