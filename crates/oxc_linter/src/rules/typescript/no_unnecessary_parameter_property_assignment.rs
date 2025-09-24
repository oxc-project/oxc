use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentOperator, AssignmentTarget, ClassElement, Expression,
        FormalParameter, Function, MethodDefinitionKind, Statement,
    },
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{Atom, Span};
use rustc_hash::FxHashSet;

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
    correctness,
    suggestion,
);

impl Rule for NoUnnecessaryParameterPropertyAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };
        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }

        let parameter_properties: Vec<_> =
            method.value.params.items.iter().filter(|param| param.has_modifier()).collect();

        if parameter_properties.is_empty() {
            return;
        }

        let AstKind::ClassBody(class_body) = ctx.semantic().nodes().parent_kind(node.id()) else {
            return;
        };

        let mut assigned_before_constructor = FxHashSet::default();
        for statement in &class_body.body {
            let ClassElement::PropertyDefinition(property_definition) = statement else {
                continue;
            };
            let Some(expression) = &property_definition.value else {
                continue;
            };
            let assignments = get_assignments_inside_expression(expression);
            for assignment in assignments {
                if let Some(this_property_name) = get_property_name(&assignment.left) {
                    assigned_before_constructor.insert(this_property_name);
                }
            }
        }

        let Some(function_body) = &method.value.body else {
            return;
        };

        let mut visitor = AssignmentVisitor {
            ctx,
            parameter_properties,
            assigned_before_unnecessary: FxHashSet::default(),
            assigned_before_constructor,
        };
        visitor.visit_function_body(function_body);
    }
}

struct AssignmentVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    parameter_properties: Vec<&'b FormalParameter<'a>>,
    assigned_before_unnecessary: FxHashSet<Atom<'a>>,
    assigned_before_constructor: FxHashSet<Atom<'a>>,
}

impl<'a> Visit<'a> for AssignmentVisitor<'a, '_> {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        // don't continue walking into functions as they have a different scoped "this"
    }

    fn visit_assignment_expression(&mut self, assignment_expr: &AssignmentExpression<'a>) {
        let Some(this_property_name) = get_property_name(&assignment_expr.left) else {
            return;
        };
        // assigning to a property of this

        if !is_unnecessary_assignment_operator(assignment_expr.operator) {
            self.assigned_before_unnecessary.insert(this_property_name);
            return;
        }
        // operator could be unnecessary

        let Expression::Identifier(right_identifier) = assignment_expr.right.get_inner_expression()
        else {
            return;
        };
        // the right side of the assignment is an identifier

        if this_property_name != right_identifier.name {
            return;
        }
        // the property of this matches the identifier name on the right

        for param in &self.parameter_properties {
            let Some(binding_identifier) = param.pattern.get_binding_identifier() else {
                continue;
            };
            if binding_identifier.name != this_property_name {
                continue;
            }
            // name of property parameter matches the name of the assigned property

            let right_reference = self.ctx.scoping().get_reference(right_identifier.reference_id());
            if !self
                .ctx
                .symbol_references(binding_identifier.symbol_id())
                .any(|reference| reference.node_id() == right_reference.node_id())
            {
                continue;
            }
            // property parameter is same symbol as identifier on the right of assignment

            if self.assigned_before_unnecessary.contains(&this_property_name) {
                continue; // there already was an assignment inside the constructor
            }

            if self.assigned_before_constructor.contains(&this_property_name) {
                continue; // there already was an assignment outside the constructor
            }

            self.ctx.diagnostic_with_suggestion(
                no_unnecessary_parameter_property_assignment_diagnostic(assignment_expr.span),
                |fixer| {
                    fixer.delete_range(Span::new(
                        assignment_expr.span.start,
                        assignment_expr.span.end + 1,
                    ))
                },
            );
        }
    }
}

fn get_assignments_inside_expression<'a>(
    expression: &'a Expression,
) -> Vec<&'a AssignmentExpression<'a>> {
    let mut assignments: Vec<&AssignmentExpression> = Vec::new();

    match expression.without_parentheses() {
        Expression::CallExpression(call) => {
            // Immediately Invoked Function Expression (IIFE)

            let function_body = match call.callee.without_parentheses() {
                Expression::ArrowFunctionExpression(expr) => Some(&expr.body),
                Expression::FunctionExpression(expr) => expr.body.as_ref(),
                _ => None,
            };

            if let Some(function_body) = function_body {
                for statement in &function_body.statements {
                    if let Statement::ExpressionStatement(expr) = statement
                        && let Expression::AssignmentExpression(assignment) = &expr.expression
                    {
                        assignments.push(assignment);
                    }
                }
            }
        }
        Expression::AssignmentExpression(assignment) => {
            assignments.push(assignment);
        }
        _ => (),
    }

    assignments
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

fn get_property_name<'a>(assignment_target: &AssignmentTarget<'a>) -> Option<Atom<'a>> {
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
        "
        class Foo {
          constructor(public name: unknown) {}
        }
        ",
        "
        class Foo {
          constructor(public name: unknown, other: unknown) {
            this.other = other;
          }
        }
        ",
        "
        class Foo {
          constructor(public name: unknown) {
            this.other = name;
          }
        }
        ",
        "
        class Foo {
          constructor(name: unknown) {
            this.name = name;
          }
        }
        ",
        "
        class Foo {
          constructor(public name: unknown) {
            this.name = 'other';
          }
        }
        ",
        "
        class Foo {
          constructor(public name: unknown) {
            this.name = name + 'edited';
          }
        }
        ",
        "
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
        "
        class Foo {
          constructor(private foo: string) {
            this.bar = () => {
              this.foo = foo;
            };
          }
        }
        ",
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
        "
        class Foo {
          constructor(public foo: number) {
            this.foo += 1;
            this.foo = foo;
          }
        }
        ",
        "
        class Foo {
          constructor(
            public foo: number,
            bar: boolean,
          ) {
            if (bar) {
              this.foo += 1;
            } else {
              this.foo = foo;
            }
          }
        }
        ",
        "
        class Foo {
          constructor(public foo: number) {
            this.foo = foo;
          }
          init = (this.foo += 1);
        }
        ",
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
        "
        class Foo {
          constructor(public foo: number) {
            this.foo = foo;
          }
          init = (() => {
            this.foo += 1;
          })();
        }
        ",
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
        "
        class Foo {
          constructor(public foo: number) {
            this.foo = foo;
          }
          init = (function() {
            console.log('hi');
            this.foo += 1;
          })();
        }
        ",
        "
        class Foo {
          constructor(private foo: string) {
            function bar() {
              this.foo = foo;
            }
          }
        }
        ",
    ];

    let fail = vec![
        "
        class Foo {
          constructor(public name: unknown) {
            this.name = name;
          }
        }
        ",
        "
        class Foo {
          constructor(other: unknown, public name: unknown) {
            this.other = other;
            this.name = name;
          }
        }
        ",
        "
        class Foo {
          constructor(public name: unknown) {
            this.name = name;
            this.name = name;
            this.name = name;
          }
        }
        ",
        "
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

    let fix = vec![
        (
            "
            class Foo {
              constructor(public name: unknown) {
                this.name = name;
              }
            }
            ",
            "
            class Foo {
              constructor(public name: unknown) {
                
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(other: unknown, public name: unknown) {
                this.other = other;
                this.name = name;
              }
            }
            ",
            "
            class Foo {
              constructor(other: unknown, public name: unknown) {
                this.other = other;
                
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(public name: unknown) {
                this.name = name;
                this.name = name;
                this.name = name;
              }
            }
            ",
            "
            class Foo {
              constructor(public name: unknown) {
                
                
                
              }
            }
            ",
        ),
        (
            "
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
              constructor(public name: unknown) {
                if (maybeTrue) {
                  
                } else {
                  this.name = name + 'edited';
                }
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(public foo: string) {
                this.foo = foo;
              }
            }
            ",
            "
            class Foo {
              constructor(public foo: string) {
                
              }
            }
            ",
        ),
        (
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
                
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(public foo?: string) {
                this.foo = foo as any;
              }
            }
            ",
            "
            class Foo {
              constructor(public foo?: string) {
                
              }
            }
            ",
        ),
        (
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
                
              }
            }
            ",
        ),
        (
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
              constructor(public foo = '') {
                
                this.foo += 'foo';
              }
            }
            ",
        ),
        (
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
                
              }
            }
            ",
        ),
        (
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
                
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(public foo: string) {
                this.foo &&= foo;
              }
            }
            ",
            "
            class Foo {
              constructor(public foo: string) {
                
              }
            }
            ",
        ),
        (
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
                
              }
            }
            ",
        ),
        (
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
                function bar() {
                  this.foo = foo;
                }
                
              }
            }
            ",
        ),
        (
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
                this.bar = () => {
                  this.foo = foo;
                };
                
              }
            }
            ",
        ),
        (
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
                class Bar {
                  constructor(private foo: string) {
                    
                  }
                }
                
              }
            }
            ",
        ),
        (
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
                
              }
              bar = () => {
                this.foo = 'foo';
              };
            }
            ",
        ),
        (
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
                
              }
              init = foo => {
                this.foo = foo;
              };
            }
            ",
        ),
        (
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
                
              }
              init = class Bar {
                constructor(private foo: string) {
                  
                }
              };
            }
            ",
        ),
        (
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
                {
                  
                }
              }
            }
            ",
        ),
        (
            "
            class Foo {
              constructor(private foo: string) {
                (() => {
                  this.foo = foo;
                })();
              }
            }
            ",
            "
            class Foo {
              constructor(private foo: string) {
                (() => {
                  
                })();
              }
            }
            ",
        ),
    ];

    Tester::new(
        NoUnnecessaryParameterPropertyAssignment::NAME,
        NoUnnecessaryParameterPropertyAssignment::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
