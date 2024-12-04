use oxc_allocator;
use oxc_ast::ast::{
    AssignmentOperator, ClassBody, ClassElement, Expression, LogicalOperator, MethodDefinitionKind,
    Statement,
};
use oxc_ast::{match_member_expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn constructor_super_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ConstructorSuper,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else { return };

        if class.super_class.is_some() {
            let super_class = &class.super_class.as_ref().unwrap().without_parentheses();
            let has_super_constructor = is_possible_constructor(super_class);

            if has_super_constructor {
                if check_for_super_call(&class.body) {
                    ctx.diagnostic(constructor_super_diagnostic(class.span));
                }
            } else {
                if check_for_non_super_call(&class.body) {
                    ctx.diagnostic(constructor_super_diagnostic(class.span));
                }
            }
        } else {
            if check_for_non_super_call(&class.body) {
                ctx.diagnostic(constructor_super_diagnostic(class.span));
            }
        }
    }
}

fn is_possible_constructor(expression: &Expression<'_>) -> bool {
    if matches!(
        expression,
        Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ThisExpression(_)
            | Expression::CallExpression(_)
            | Expression::NewExpression(_)
            | Expression::ChainExpression(_)
            | Expression::YieldExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::MetaProperty(_)
            | match_member_expression!(Expression)
    ) {
        return true;
    }

    if matches!(expression, Expression::Identifier(identifier) if identifier.name != "undefined") {
        return true;
    }

    if let Expression::AssignmentExpression(assignment) = expression {
        if matches!(
            assignment.operator,
            AssignmentOperator::Assign | AssignmentOperator::LogicalAnd
        ) {
            return is_possible_constructor(&assignment.right);
        }

        if matches!(
            assignment.operator,
            AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
        ) {
            // ToDo check if left or right side
            return true;
        }

        return false;
    }

    if let Expression::LogicalExpression(logical) = expression {
        if logical.operator == LogicalOperator::And {
            return is_possible_constructor(&logical.right);
        }

        return is_possible_constructor(&logical.left) || is_possible_constructor(&logical.right);
    }

    if let Expression::ConditionalExpression(conditional) = expression {
        return is_possible_constructor(&conditional.alternate)
            || is_possible_constructor(&conditional.consequent);
    }

    if let Expression::SequenceExpression(sequence) = expression {
        return sequence.expressions.last().is_some_and(is_possible_constructor);
    }

    false
}

fn check_for_super_call(_class: &ClassBody<'_>) -> bool {
    false
}

fn check_for_non_super_call(class: &ClassBody<'_>) -> bool {
    let Some(statements) = get_constructor_statements(class) else {
        return false;
    };

    for statement in statements {
        let Statement::ExpressionStatement(expression) = statement else {
            continue;
        };

        if expression.expression.is_super_call_expression() {
            return true;
        }
    }

    false
}

fn get_constructor_statements<'a>(
    class: &'a ClassBody<'a>,
) -> Option<&'a oxc_allocator::Vec<'a, Statement<'a>>> {
    if class.body.len() == 0 {
        return None;
    }

    let constructor = class.body.iter().find(|part| matches!(part, ClassElement::MethodDefinition(method) if method.kind == MethodDefinitionKind::Constructor));

    if constructor.is_none() {
        return None;
    }

    // we already checked it, only for the compiler
    let ClassElement::MethodDefinition(method) = constructor.unwrap() else {
        return None;
    };

    let Some(func_body) = &method.value.body else {
        return None;
    };

    Some(&func_body.statements)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { }",
"class A { constructor() { } }",
"class A extends null { }",
"class A extends B { }",
"class A extends B { constructor() { super(); } }",
"class A extends B { constructor() { if (true) { super(); } else { super(); } } }",
"class A extends (class B {}) { constructor() { super(); } }",
"class A extends (B = C) { constructor() { super(); } }",
"class A extends (B &&= C) { constructor() { super(); } }",
"class A extends (B ||= C) { constructor() { super(); } }",
"class A extends (B ??= C) { constructor() { super(); } }",
"class A extends (B ||= 5) { constructor() { super(); } }",
"class A extends (B ??= 5) { constructor() { super(); } }",
"class A extends (B || C) { constructor() { super(); } }",
"class A extends (5 && B) { constructor() { super(); } }",
"class A extends (false && B) { constructor() { super(); } }",
"class A extends (B || 5) { constructor() { super(); } }",
"class A extends (B ?? 5) { constructor() { super(); } }",
"class A extends (a ? B : C) { constructor() { super(); } }",
"class A extends (B, C) { constructor() { super(); } }",
"class A { constructor() { class B extends C { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C { constructor() { } } } }",
"class A extends B { constructor() { a ? super() : super(); } }",
"class A extends B { constructor() { if (a) super(); else super(); } }",
"class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
"class A extends B { constructor() { try {} finally { super(); } } }",
"class A extends B { constructor() { if (a) throw Error(); super(); } }",
"class A extends B { constructor() { if (true) return a; super(); } }",
"class A extends null { constructor() { return a; } }",
"class A { constructor() { return a; } }",
"class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
"class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",
"class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0; i++);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (; i < 0; i++);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;; i++) {
			            if (foo) break;
			        }
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0;);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;;) {
			            if (foo) break;
			        }
			    }
			}
			",
"
			            class A extends B {
			                constructor(props) {
			                    super(props);
			
			                    try {
			                        let arr = [];
			                        for (let a of arr) {
			                        }
			                    } catch (err) {
			                    }
			                }
			            }
			        ",
"class A extends obj?.prop { constructor() { super(); } }",
"
			            class A extends Base {
			                constructor(list) {
			                    for (const a of list) {
			                        if (a.foo) {
			                            super(a);
			                            return;
			                        }
			                    }
			                    super();
			                }
			            }
			        "
    ];

    let fail = vec![
        "class A extends null { constructor() { super(); } }",
"class A extends null { constructor() { } }",
"class A extends 100 { constructor() { super(); } }",
"class A extends 'test' { constructor() { super(); } }",
"class A extends (B = 5) { constructor() { super(); } }",
"class A extends (B && 5) { constructor() { super(); } }",
"class A extends (B &&= 5) { constructor() { super(); } }",
"class A extends (B += C) { constructor() { super(); } }",
"class A extends (B -= C) { constructor() { super(); } }",
"class A extends (B **= C) { constructor() { super(); } }",
"class A extends (B |= C) { constructor() { super(); } }",
"class A extends (B &= C) { constructor() { super(); } }",
"class A extends B { constructor() { } }",
"class A extends B { constructor() { for (var a of b) super.foo(); } }",
"class A extends B { constructor() { for (var i = 1; i < 10; i++) super.foo(); } }",
"class A extends B { constructor() { var c = class extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { var c = () => super(); } }",
"class A extends B { constructor() { class C extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { var C = class extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C extends D { constructor() { } } } }",
"class A extends B { constructor() { super(); var C = class extends D { constructor() { } } } }",
"class A extends B { constructor() { if (a) super(); } }",
"class A extends B { constructor() { if (a); else super(); } }",
"class A extends B { constructor() { a && super(); } }",
"class A extends B { constructor() { switch (a) { case 0: super(); } } }",
"class A extends B { constructor() { switch (a) { case 0: break; default: super(); } } }",
"class A extends B { constructor() { try { super(); } catch (err) {} } }",
"class A extends B { constructor() { try { a; } catch (err) { super(); } } }",
"class A extends B { constructor() { if (a) return; super(); } }",
"class A extends B { constructor() { super(); super(); } }",
"class A extends B { constructor() { super() || super(); } }",
"class A extends B { constructor() { if (a) super(); super(); } }",
"class A extends B { constructor() { switch (a) { case 0: super(); default: super(); } } }",
"class A extends B { constructor(a) { while (a) super(); } }",
"class A extends B { constructor() { return; super(); } }",
"class Foo extends Bar {
			                constructor() {
			                    for (a in b) for (c in d);
			                }
			            }",
"class C extends D {
			
			                constructor() {
			                    do {
			                        something();
			                    } while (foo);
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    for (let i = 1;;i++) {
			                        if (bar) {
			                            break;
			                        }
			                    }
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    do {
			                        super();
			                    } while (foo);
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    while (foo) {
			                        if (bar) {
			                            super();
			                            break;
			                        }
			                    }
			                }
			
			            }"
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::CATEGORY, pass, fail).test_and_snapshot();
}
