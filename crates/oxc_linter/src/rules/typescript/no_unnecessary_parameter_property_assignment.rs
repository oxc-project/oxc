use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{
        AccessorProperty, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
        AssignmentTarget, Class, ClassElement, Expression, FormalParameter, Function,
        IdentifierReference, PropertyDefinition,
    },
};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, ScopeFlags};
use oxc_span::{GetSpan, Span};
use oxc_str::Str;

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
    version = "0.15.13",
    short_description = "Prevents unnecessary assignment of parameter properties.",
);

impl Rule for NoUnnecessaryParameterPropertyAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };
        if !method.kind.is_constructor() {
            return;
        }

        if !method.value.params.items.iter().any(FormalParameter::has_modifier) {
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
            collect_field_initializer_writes(expression, &mut assigned_before_constructor);
        }

        let Some(function_body) = &method.value.body else {
            return;
        };

        let mut visitor = AssignmentVisitor {
            ctx,
            parameters: &method.value.params.items,
            assigned_before_unnecessary: FxHashSet::default(),
            assigned_before_constructor,
            written_before: FxHashSet::default(),
        };
        visitor.visit_function_body(function_body);
    }
}

struct AssignmentVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    parameters: &'b [FormalParameter<'a>],
    assigned_before_unnecessary: FxHashSet<Str<'a>>,
    assigned_before_constructor: FxHashSet<Str<'a>>,
    /// Properties already written with a value that may differ from the parameter.
    written_before: FxHashSet<Str<'a>>,
}

impl<'a> VisitJs<'a> for AssignmentVisitor<'a, '_> {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        // don't continue walking into functions as they have a different scoped "this"
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let reference = self.ctx.scoping().get_reference(identifier.reference_id());
        if reference.is_write()
            && self.parameters.iter().any(|param| {
                param
                    .pattern
                    .get_binding_identifier()
                    .is_some_and(|binding| reference.symbol_id() == Some(binding.symbol_id()))
            })
            && self.is_in_constructor_scope(reference.node_id(), false)
        {
            // Any prior write may change the value copied by a later `this.x = x`.
            self.assigned_before_unnecessary.insert(identifier.name.into());
        }
    }

    fn visit_assignment_expression(&mut self, assignment_expr: &AssignmentExpression<'a>) {
        self.check_assignment(assignment_expr);
        walk_js::walk_assignment_expression(self, assignment_expr);
    }
}

impl<'a> AssignmentVisitor<'a, '_> {
    /// With `own_this`, the static blocks and field initializers of a nested class also end
    /// the constructor scope, because they have their own `this`.
    fn is_in_constructor_scope(&self, node_id: NodeId, own_this: bool) -> bool {
        let span = self.ctx.nodes().get_node(node_id).kind().span();
        // Match the constructor scope, allowing a single arrow IIFE as upstream does.
        let mut functions = self.ctx.nodes().ancestors(node_id).filter(|node| match node.kind() {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => true,
            AstKind::StaticBlock(_) => own_this,
            AstKind::PropertyDefinition(PropertyDefinition { value: Some(value), .. })
            | AstKind::AccessorProperty(AccessorProperty { value: Some(value), .. }) => {
                own_this && value.span().contains_inclusive(span)
            }
            _ => false,
        });
        let mut function = functions.next();
        if let Some(node) = function
            && matches!(node.kind(), AstKind::ArrowFunctionExpression(_))
            && let Some(AstKind::CallExpression(call)) = self
                .ctx
                .nodes()
                .ancestor_kinds(node.id())
                .find(|kind| !matches!(kind, AstKind::ParenthesizedExpression(_)))
            && call.callee.without_parentheses().node_id() == node.id()
        {
            function = functions.next();
        }
        function.is_some_and(|node| matches!(node.kind(), AstKind::Function(_)))
    }

    fn check_assignment(&mut self, assignment_expr: &AssignmentExpression<'a>) {
        if !self.is_in_constructor_scope(assignment_expr.node_id(), true) {
            return;
        }

        let Some(this_property_name) = get_property_name(&assignment_expr.left) else {
            return;
        };
        // assigning to a property of this

        if !is_unnecessary_assignment_operator(assignment_expr.operator) {
            self.assigned_before_unnecessary.insert(this_property_name);
            return;
        }
        // operator could be unnecessary

        if !get_identifier(&assignment_expr.right).is_some_and(|right_identifier| {
            right_identifier.name == this_property_name
                && self.is_parameter_property(right_identifier)
        }) {
            self.written_before.insert(this_property_name);
            return;
        }

        if self.assigned_before_unnecessary.contains(&this_property_name) {
            return; // there already was an assignment inside the constructor
        }

        if self.assigned_before_constructor.contains(&this_property_name) {
            return; // there already was an assignment outside the constructor
        }

        let diagnostic =
            no_unnecessary_parameter_property_assignment_diagnostic(assignment_expr.span);
        let statement_node = self.ctx.nodes().parent_node(assignment_expr.node_id());
        if let AstKind::ExpressionStatement(statement) = statement_node.kind()
            && !self.written_before.contains(&this_property_name)
        {
            self.ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                match self.ctx.nodes().parent_kind(statement_node.id()) {
                    AstKind::IfStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::DoWhileStatement(_)
                    | AstKind::ForStatement(_)
                    | AstKind::ForInStatement(_)
                    | AstKind::ForOfStatement(_)
                    | AstKind::LabeledStatement(_)
                    | AstKind::WithStatement(_) => fixer.replace(statement.span, "{}"),
                    _ => fixer.delete_range(statement.span),
                }
            });
        } else {
            // Nested assignments may supply a value to their surrounding expression.
            self.ctx.diagnostic(diagnostic);
        }
    }

    /// Whether `identifier` refers to a parameter property of the same name.
    fn is_parameter_property(&self, identifier: &IdentifierReference<'a>) -> bool {
        let reference = self.ctx.scoping().get_reference(identifier.reference_id());
        self.parameters
            .iter()
            .filter(|param| param.has_modifier())
            .filter_map(|param| param.pattern.get_binding_identifier())
            .any(|binding| {
                binding.name == identifier.name
                    && self
                        .ctx
                        .symbol_references(binding.symbol_id())
                        .any(|binding_reference| binding_reference.node_id() == reference.node_id())
            })
    }
}

/// Unwraps parentheses, `as` expressions and non-null assertions, but not other TypeScript
/// wrappers such as `satisfies`, which perform an explicit check.
fn get_identifier<'b, 'a>(expression: &'b Expression<'a>) -> Option<&'b IdentifierReference<'a>> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier),
        Expression::ParenthesizedExpression(expr) => get_identifier(&expr.expression),
        Expression::TSAsExpression(expr) => get_identifier(&expr.expression),
        Expression::TSNonNullExpression(expr) => get_identifier(&expr.expression),
        _ => None,
    }
}

/// Collects the `this` properties written by a field initializer, including inside an
/// initializer IIFE such as `init = (() => { this.x = 1; })()`. Writes inside other functions
/// and classes are skipped, because they have a different `this`.
fn collect_field_initializer_writes<'a>(
    expression: &Expression<'a>,
    names: &mut FxHashSet<Str<'a>>,
) {
    let mut collector = FieldInitializerWrites { names };
    if let Expression::CallExpression(call) = expression.without_parentheses() {
        match call.callee.without_parentheses() {
            Expression::ArrowFunctionExpression(arrow) => {
                collector.visit_arrow_function_body(&arrow.body);
            }
            Expression::FunctionExpression(function) => {
                if let Some(body) = &function.body {
                    collector.visit_function_body(body);
                }
            }
            callee => collector.visit_expression(callee),
        }
        collector.visit_arguments(&call.arguments);
    } else {
        collector.visit_expression(expression);
    }
}

struct FieldInitializerWrites<'n, 'a> {
    names: &'n mut FxHashSet<Str<'a>>,
}

impl<'a> VisitJs<'a> for FieldInitializerWrites<'_, 'a> {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _it: &Class<'a>) {}

    fn visit_assignment_expression(&mut self, assignment_expr: &AssignmentExpression<'a>) {
        if let Some(name) = get_property_name(&assignment_expr.left) {
            self.names.insert(name);
        }
        walk_js::walk_assignment_expression(self, assignment_expr);
    }
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

fn get_property_name<'a>(assignment_target: &AssignmentTarget<'a>) -> Option<Str<'a>> {
    match assignment_target {
        AssignmentTarget::StaticMemberExpression(expr)
            if matches!(expr.object.without_parentheses(), Expression::ThisExpression(_)) =>
        {
            // this.property
            Some(expr.property.name.into())
        }
        AssignmentTarget::ComputedMemberExpression(expr)
            if matches!(expr.object.without_parentheses(), Expression::ThisExpression(_)) =>
        {
            // this["property"]
            match expr.expression.without_parentheses() {
                Expression::StringLiteral(str) => str.value.as_arena_str(),
                Expression::TemplateLiteral(template) => template.single_quasi()?.as_arena_str(),
                _ => None,
            }
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class C { constructor(public x: number) { this.x = x satisfies number; } }",
        "class C { snapshot = [this.x = 0]; constructor(public x: number) { this.x = x; } }",
        "class C { init = (() => { if (ready) { this.x = 0; } })(); constructor(public x: number) { this.x = x; } }",
        "class C { constructor(public x: number) { this[`${x}`] = x; } }",
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
        "
        class User {
          constructor(public name: string) {
            name = name.trim();
            this.name = name;
          }
        }
        ",
        "
        class User {
          constructor(public name: string) {
            name += '!';
            this.name = name;
          }
        }
        ",
        "
        class User {
          constructor(public name: string, flag: boolean) {
            if (flag) {
              name = name.trim();
            }
            this.name = name;
          }
        }
        ",
        "
        class User {
          constructor(public name: string, public age: number) {
            name = name.trim();
            this.name = name;
          }
        }
        ",
        "class User { constructor(public name: string) { let local; local = (name = name.trim()); this.name = name; } }",
        "class User { constructor(public name = '') { name ||= 'other'; this.name = name; } }",
        "class User { constructor(public name: string) { (() => { name = 'other'; })(); this.name = name; } }",
        "class User { constructor(public age: number) { age++; this.age = age; } }",
        "class User { constructor(public age: number) { --age; this.age = age; } }",
        "class User { constructor(public name: string, names: string[]) { for (name of names) {} this.name = name; } }",
        "class User { constructor(public name: string, names: object) { for (name in names) {} this.name = name; } }",
        "class User { constructor(public name: string, values: string[]) { [name] = values; this.name = name; } }",
        "class User { constructor(public name: string, value: {name: string}) { ({name} = value); this.name = name; } }",
        "class User { constructor(public names: string[], values: string[]) { [...names] = values; this.names = names; } }",
        "class User { constructor(public age: number, flag: boolean) { if (flag) age++; this.age = age; } }",
    ];

    let fail = vec![
        "class C { constructor(public x: number) { this.x = (x as number); } }",
        "class C { constructor(public x: number) { this.x = x!; } }",
        "class C { constructor(public x: number) { this.x = 0; this.x = x; } }",
        "class C { constructor(public x: number) { if (x) this.x = x; else {} } }",
        "class Outer { constructor(public x: number) { class Inner { static x = 0; static { this.x += 1; } } this.x = x; } }",
        "class C { constructor(public x: number) { (this).x = x; this[(\"x\")] = x; this[`x`] = x; } }",
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
        "
        class User {
          constructor(public name: string) {
            this.name = name;
            name = name.trim();
          }
        }
        ",
        "
        class User {
          constructor(public name: string, public age: number) {
            name = name.trim();
            this.name = name;
            this.age = age;
          }
        }
        ",
        "
        class User {
          constructor(public name: string) {
            let local = name;
            local = local.trim();
            this.name = name;
          }
        }
        ",
        "class User { constructor(public name: string) { { let name = ''; name = 'other'; } this.name = name; } }",
        "class User { constructor(public name: string) { function f() { name = 'other'; } this.name = name; } }",
        "class User { constructor(public name: string) { const f = () => { name = 'other'; }; this.name = name; } }",
        "class User { constructor(public name: string) { let local; local = (this.name = name); } }",
        "class User { constructor(public name: string) { let local; local = this.name = name; } }",
        "class User { constructor(public name: string) { setTimeout(() => { name = 'later'; }); this.name = name; } }",
        "class User { constructor(public age: number) { this.age = age; age++; } }",
        "class User { constructor(public age: number) { { let age = 0; age++; } this.age = age; } }",
    ];

    let fix = vec![
        (
            "class C { constructor(public x: number) { this.x = 0; this.x = x; } }",
            "class C { constructor(public x: number) { this.x = 0; this.x = x; } }",
        ),
        (
            "class C { constructor(public x: number) { if (x) this.x = x; else {} } }",
            "class C { constructor(public x: number) { if (x) {} else {} } }",
        ),
        (
            "class C { constructor(public x: number) { (this).x = x; } }",
            "class C { constructor(public x: number) {  } }",
        ),
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
        (
            "class User { constructor(public name: string) { this.name = name; name = name.trim(); } }",
            "class User { constructor(public name: string) {  name = name.trim(); } }",
        ),
        (
            "class User { constructor(public name: string, public age: number) { name = name.trim(); this.name = name; this.age = age; } }",
            "class User { constructor(public name: string, public age: number) { name = name.trim(); this.name = name;  } }",
        ),
        (
            "class User { constructor(public name: string) { let local = name; local = local.trim(); this.name = name; } }",
            "class User { constructor(public name: string) { let local = name; local = local.trim();  } }",
        ),
        (
            "class User { constructor(public name: string) { let local; local = (this.name = name); } }",
            "class User { constructor(public name: string) { let local; local = (this.name = name); } }",
        ),
        (
            "class User { constructor(public name: string) { let local; local = this.name = name; } }",
            "class User { constructor(public name: string) { let local; local = this.name = name; } }",
        ),
        (
            "class User { constructor(public name: string) { this.name = name} }",
            "class User { constructor(public name: string) { } }",
        ),
        (
            "class User { constructor(public name: string) { setTimeout(() => { name = 'later'; }); this.name = name; } }",
            "class User { constructor(public name: string) { setTimeout(() => { name = 'later'; });  } }",
        ),
        (
            "class User { constructor(public age: number) { this.age = age; age++; } }",
            "class User { constructor(public age: number) {  age++; } }",
        ),
        (
            "class User { constructor(public age: number) { { let age = 0; age++; } this.age = age; } }",
            "class User { constructor(public age: number) { { let age = 0; age++; }  } }",
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
