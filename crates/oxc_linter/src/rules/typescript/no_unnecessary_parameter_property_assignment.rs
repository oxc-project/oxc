use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, AssignmentTarget, Expression, MethodDefinitionKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

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
            let is_parameter_property = param.accessibility.is_some() || param.readonly;
            if !is_parameter_property {
                continue;
            }
            let Some(ident) = param.pattern.get_binding_identifier() else {
                continue;
            };
            for reference in ctx.symbol_references(ident.symbol_id()) {
                // check if the param is being read which would indicate an assignment
                if !reference.is_read() {
                    continue;
                }

                let Some(AstKind::AssignmentExpression(assignment_expr)) =
                    ctx.nodes().parent_kind(reference.node_id())
                else {
                    continue;
                };

                if !matches!(
                    assignment_expr.operator,
                    AssignmentOperator::Assign
                        | AssignmentOperator::LogicalOr
                        | AssignmentOperator::LogicalAnd
                        | AssignmentOperator::LogicalNullish
                ) {
                    continue;
                }

                // check for assigning to this: this.x = ?
                let AssignmentTarget::StaticMemberExpression(static_member_expr) =
                    &assignment_expr.left
                else {
                    continue;
                };
                if !matches!(&static_member_expr.object, Expression::ThisExpression(_)) {
                    continue;
                }
                let assignment_name = static_member_expr.property.name;

                // check both sides of assignment have the same name: this.x = x
                let Expression::Identifier(assignment_target_ident) = &assignment_expr.right else {
                    continue;
                };
                if assignment_target_ident.name != assignment_name {
                    continue;
                }

                ctx.diagnostic(no_unnecessary_parameter_property_assignment_diagnostic(
                    assignment_expr.span,
                ));
            }
        }
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
        // "
        //     class Foo {
        //       constructor(public foo?: string) {
        //         this.foo = foo!;
        //       }
        //     }
        // ",
        // "
        //     class Foo {
        //       constructor(public foo?: string) {
        //         this.foo = foo as any;
        //       }
        //     }
        // ",
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
        // "
        //     class Foo {
        //       constructor(private foo: string) {
        //         this['foo'] = foo;
        //       }
        //     }
        // ",
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
