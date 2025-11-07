use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentOperator, ClassElement, Expression, MemberExpression,
        MethodDefinitionKind, PropertyDefinitionType, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_class_fields_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer class field declaration over `this` assignment in constructor for static values.",
    )
    .with_help("Declare static values as class fields instead of assigning them to `this` in the constructor.")
    .with_label(span)
}

fn prefer_class_fields_suggestion(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Encountered same-named class field declaration and `this` assignment in constructor.",
    )
    .with_help("Replace the class field declaration with the value from `this` assignment.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferClassFields;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers class field declarations over `this` assignments in constructors for static values.
    ///
    /// ### Why is this bad?
    ///
    /// Class field declarations are more readable and less error-prone than assigning static
    /// values to `this` in the constructor. Using class fields keeps the constructor cleaner
    /// and makes the intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class Foo {
    ///     constructor() {
    ///         this.bar = 1;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class Foo {
    ///     bar = 1;
    /// }
    /// ```
    PreferClassFields,
    unicorn,
    style,
    conditional_fix_suggestion
);

impl Rule for PreferClassFields {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        // Find constructor
        let constructor = class.body.body.iter().find(|element| {
            matches!(
                element,
                ClassElement::MethodDefinition(method)
                    if method.kind == MethodDefinitionKind::Constructor
                        && !method.r#static
                        && !method.computed
            )
        });

        let Some(ClassElement::MethodDefinition(constructor)) = constructor else {
            return;
        };

        let Some(body) = &constructor.value.body else {
            return;
        };

        // Find first non-empty statement in constructor
        let first_statement =
            body.statements.iter().find(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));

        let Some(Statement::ExpressionStatement(expr_stmt)) = first_statement else {
            return;
        };

        // Check if it's a simple assignment to this.property = literal
        let Expression::AssignmentExpression(assign) = &expr_stmt.expression else {
            return;
        };

        if !is_simple_this_assignment_with_literal(assign) {
            return;
        }

        let Some(member) = assign.left.as_member_expression() else {
            return;
        };

        let Some(property_name) = get_property_name(member) else {
            return;
        };

        // Check if there's an existing property definition with the same name
        let existing_property = class.body.body.iter().find(|element| {
            if let ClassElement::PropertyDefinition(prop) = element {
                if prop.r#static || prop.computed {
                    return false;
                }

                if prop.r#type == PropertyDefinitionType::PropertyDefinition {
                    return prop.key.name().is_some_and(|name| name == property_name);
                }
            }
            false
        });

        // If there's an existing property with a value, show suggestion instead of fix
        if let Some(ClassElement::PropertyDefinition(prop)) = existing_property
            && prop.value.is_some()
        {
            ctx.diagnostic_with_suggestion(prefer_class_fields_suggestion(assign.span), |fixer| {
                let fixer = fixer.for_multifix();
                let mut fix = fixer.new_fix_with_capacity(2);

                // Remove the assignment from constructor
                fix.push(fixer.delete(&**expr_stmt));

                // Replace existing property value
                if let Some(value) = &prop.value {
                    let mut codegen = fixer.codegen();
                    codegen.print_expression(&assign.right);
                    fix.push(fixer.replace(value.span(), codegen.into_source_text()));
                }

                fix.with_message("Replace `this` assignment with class field declaration")
            });
            return;
        }

        // Otherwise, provide a fix
        ctx.diagnostic_with_fix(prefer_class_fields_diagnostic(assign.span), |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix = fixer.new_fix_with_capacity(2);

            fix.push(fixer.delete(&**expr_stmt));

            if let Some(ClassElement::PropertyDefinition(prop)) = existing_property {
                let mut codegen = fixer.codegen();
                codegen.print_str(" = ");
                codegen.print_expression(&assign.right);
                let insert_span =
                    prop.type_annotation.as_ref().map_or(prop.key.span(), |ty| ty.span);
                fix.push(fixer.insert_text_after_range(insert_span, codegen.into_source_text()));
            } else {
                let indent =
                    ctx.source_range(constructor.span).lines().next().map_or("\t", |line| {
                        let leading_whitespace = line.len() - line.trim_start().len();
                        &line[..leading_whitespace]
                    });

                let mut codegen = fixer.codegen();
                codegen.print_str(indent);
                codegen.print_str(property_name);
                codegen.print_str(" = ");
                codegen.print_expression(&assign.right);
                codegen.print_str(";\n");
                fix.push(fixer.insert_text_before(&**constructor, codegen.into_source_text()));
            }

            fix.with_message("Replace `this` assignment with class field declaration")
        });
    }
}

fn is_simple_this_assignment_with_literal(assign: &AssignmentExpression) -> bool {
    if assign.operator != AssignmentOperator::Assign {
        return false;
    }

    let Some(member) = assign.left.as_member_expression() else {
        return false;
    };

    // Check if it's this.property
    if !matches!(member.object(), Expression::ThisExpression(_)) {
        return false;
    }

    if member.is_computed() {
        return false;
    }

    // Check if the value is a literal
    matches!(
        &assign.right,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

fn get_property_name<'a>(member: &MemberExpression<'a>) -> Option<&'a str> {
    match member {
        MemberExpression::StaticMemberExpression(expr) => Some(expr.property.name.as_str()),
        MemberExpression::PrivateFieldExpression(expr) => Some(expr.field.name.as_str()),
        MemberExpression::ComputedMemberExpression(_) => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo {bar = 1}",
        "class Foo {static bar = 1}",
        "class Foo {#bar = 1}",
        "class Foo {static #bar = 1}",
        "class Foo {constructor() {this.bar += 1}}",
        "class Foo {constructor() {this[bar] = 1}}",
        "class Foo {constructor() {notThis.bar = 1}}",
        "class Foo {constructor() {notThis.bar = 1 + 2}}",
        "class Foo {
				constructor() {
					if (something) { return; }
					this.bar = 1;
				}
			}",
        "class Foo {
				foo: string = 'foo';
			}",
        "declare class Foo {
				constructor(foo?: string);
			}",
    ];

    let fail = vec![
        "class Foo {
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				constructor() {
					;
					this.bar = 1;
				}
			}",
        "class Foo {
				constructor() {
					this.bar = 1;
					this.baz = 2;
				}
			}",
        "class Foo {
				constructor() {
					this.bar = 1;
					this.bar = 2;
				}
			}",
        "class Foo {
				bar;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				#bar;
				constructor() {
					this.#bar = 1;
				}
			}",
        "class Foo {
				bar = 0;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				#bar = 0;
				constructor() {
					this.#bar = 1;
				}
			}",
        "class Foo {
				[bar];
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				[bar] = 0;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				static bar;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				static bar = 0;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				static [bar];
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
				static [bar] = 1;
				constructor() {
					this.bar = 1;
				}
			}",
        "class Foo {
			constructor() {
				this.bar = 1;
			}}",
        "class Foo {
			constructor() {
				this.bar = 1;
			}
			static}",
        "class Foo {
			constructor() {
				this.bar = 1;
			}
			static// comment;
			}",
        r#"class MyError extends Error {
				constructor(message: string) {
					this.name = "MyError";
				}
			}"#,
        r#"class MyError extends Error {
				name: string;
				constructor(message: string) {
					this.name = "MyError";
				}
			}"#,
    ];

    let fix = vec![
        (
            "class Foo { constructor() { this.bar = 1; } }",
            "class Foo { bar = 1;\nconstructor() {  } }",
            None,
        ),
        (
            "class Foo { bar; constructor() { this.bar = 1; } }",
            "class Foo { bar = 1; constructor() {  } }",
            None,
        ),
        (
            "class Foo { closed: boolean; constructor() { this.closed = false; } }",
            "class Foo { closed: boolean = false; constructor() {  } }",
            None,
        ),
    ];

    Tester::new(PreferClassFields::NAME, PreferClassFields::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
