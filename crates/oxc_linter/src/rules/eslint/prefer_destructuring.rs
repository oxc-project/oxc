use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, BindingPattern, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_object_destructuring(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use Object destructuring.")
        .with_help("Use object destructuring rather than direct member access.")
        .with_label(span)
}

fn prefer_array_destructuring(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use Array destructuring.")
        .with_help("Use array destructuring rather than direct member access.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
struct Config {
    array: bool,
    object: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { array: true, object: true }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferDestructuring {
    /// Configuration for destructuring in variable declarations, configured for arrays and objects independently.
    #[serde(rename = "VariableDeclarator")]
    variable_declarator: Config,
    /// Configuration for destructuring in assignment expressions, configured for arrays and objects independently.
    #[serde(rename = "AssignmentExpression")]
    assignment_expression: Config,
    /// Determines whether the object destructuring rule applies to renamed variables.
    enforce_for_renamed_properties: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require destructuring from arrays and/or objects
    ///
    /// ### Why is this bad?
    ///
    /// With JavaScript ES2015, a new syntax was added for creating variables from an array index or object property,
    /// called destructuring. This rule enforces usage of destructuring
    /// instead of accessing a property through a member expression.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // With `array` enabled
    /// const foo = array[0];
    /// bar.baz = array[0];
    /// // With `object` enabled
    /// const qux = object.qux;
    /// const quux = object['quux'];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // With `array` enabled
    /// const [ foo ] = array;
    /// const arr = array[someIndex];
    /// [bar.baz] = array;
    ///
    /// // With `object` enabled
    /// const { baz } = object;
    /// const obj = object.bar;
    /// ```
    PreferDestructuring,
    eslint,
    style,
    fix,
    config = PreferDestructuring,
);

impl Rule for PreferDestructuring {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let (variable_declarator, assignment_expression) = if let Some(obj) = value.get(0) {
            let array = obj.get("array").and_then(Value::as_bool);
            let object = obj.get("object").and_then(Value::as_bool);
            if array.is_some() || object.is_some() {
                (
                    Config { array: array.unwrap_or(false), object: object.unwrap_or(false) },
                    Config { array: array.unwrap_or(false), object: object.unwrap_or(false) },
                )
            } else {
                let var_config = obj.get("VariableDeclarator").and_then(Value::as_object).map_or(
                    Config { array: false, object: false },
                    |conf| Config {
                        array: conf.get("array").and_then(Value::as_bool).unwrap_or(false),
                        object: conf.get("object").and_then(Value::as_bool).unwrap_or(false),
                    },
                );
                let assign_config = obj
                    .get("AssignmentExpression")
                    .and_then(Value::as_object)
                    .map_or(Config { array: false, object: false }, |conf| Config {
                        array: conf.get("array").and_then(Value::as_bool).unwrap_or(false),
                        object: conf.get("object").and_then(Value::as_bool).unwrap_or(false),
                    });
                (var_config, assign_config)
            }
        } else {
            (Config::default(), Config::default())
        };

        Ok(Self {
            variable_declarator,
            assignment_expression,
            enforce_for_renamed_properties: value
                .get(1)
                .and_then(|v| v.get("enforceForRenamedProperties"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign_expr) if assign_expr.operator.is_assign() => {
                let Some(right) = assign_expr.right.without_parentheses().as_member_expression()
                else {
                    return;
                };
                if !check_expr(right) {
                    return;
                }
                match right {
                    MemberExpression::ComputedMemberExpression(comp_expr) => {
                        if matches!(comp_expr.expression, Expression::TemplateLiteral(_)) {
                            return;
                        }
                        if matches!(comp_expr.expression, Expression::NumericLiteral(_)) {
                            if self.assignment_expression.array {
                                ctx.diagnostic(prefer_array_destructuring(assign_expr.span));
                            }
                        } else {
                            if self.enforce_for_renamed_properties
                                && self.assignment_expression.object
                            {
                                ctx.diagnostic(prefer_object_destructuring(assign_expr.span));
                            }
                            if let Expression::StringLiteral(string_literal) = &comp_expr.expression
                                && get_target_name(&assign_expr.left)
                                    .is_some_and(|v| v == string_literal.value)
                            {
                                ctx.diagnostic_with_fix(
                                    prefer_object_destructuring(assign_expr.span),
                                    |fixer| {
                                        generate_fix(
                                            &fixer,
                                            string_literal.span.shrink(1),
                                            get_object_span_without_redundant_parentheses(
                                                &comp_expr.object,
                                            ),
                                            assign_expr.span,
                                            true,
                                        )
                                    },
                                );
                            }
                        }
                    }
                    MemberExpression::StaticMemberExpression(static_expr)
                        if self.assignment_expression.object =>
                    {
                        if get_target_name(&assign_expr.left)
                            .is_some_and(|name| name == static_expr.property.name.as_str())
                        {
                            // Safe autofix for assignments: foo = object.foo; -> ({ foo } = object);
                            ctx.diagnostic_with_fix(
                                prefer_object_destructuring(assign_expr.span),
                                |fixer| {
                                    generate_fix(
                                        &fixer,
                                        static_expr.property.span,
                                        get_object_span_without_redundant_parentheses(
                                            &static_expr.object,
                                        ),
                                        assign_expr.span,
                                        true,
                                    )
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                if let Some(init) = &declarator.init
                    && let Some(right) = init.without_parentheses().as_member_expression()
                {
                    if !check_expr(right) {
                        return;
                    }
                    let name = if matches!(declarator.id, BindingPattern::BindingIdentifier(_)) {
                        declarator.id.get_identifier_name().map(|v| v.as_str())
                    } else {
                        None
                    };
                    match right {
                        MemberExpression::ComputedMemberExpression(comp_expr) => {
                            if matches!(comp_expr.expression, Expression::TemplateLiteral(_)) {
                                return;
                            }
                            if matches!(comp_expr.expression, Expression::NumericLiteral(_)) {
                                if self.variable_declarator.array {
                                    ctx.diagnostic(prefer_array_destructuring(init.span()));
                                }
                            } else {
                                if self.enforce_for_renamed_properties
                                    && self.variable_declarator.object
                                {
                                    ctx.diagnostic(prefer_object_destructuring(right.span()));
                                }
                                if let Expression::StringLiteral(string_literal) =
                                    &comp_expr.expression
                                    && self.variable_declarator.object
                                    && name.is_some_and(|v| v == string_literal.value)
                                {
                                    ctx.diagnostic_with_fix(
                                        prefer_object_destructuring(init.span()),
                                        |fixer| {
                                            generate_fix(
                                                &fixer,
                                                string_literal.span.shrink(1),
                                                get_object_span_without_redundant_parentheses(
                                                    &comp_expr.object,
                                                ),
                                                declarator.span(),
                                                false,
                                            )
                                        },
                                    );
                                }
                            }
                        }
                        MemberExpression::StaticMemberExpression(static_expr)
                            if self.variable_declarator.object =>
                        {
                            if self.enforce_for_renamed_properties {
                                ctx.diagnostic(prefer_object_destructuring(right.span()));
                            }
                            if name.is_some_and(|name| name == static_expr.property.name.as_str()) {
                                ctx.diagnostic_with_fix(
                                    prefer_object_destructuring(init.span()),
                                    |fixer| {
                                        generate_fix(
                                            &fixer,
                                            static_expr.property.span,
                                            get_object_span_without_redundant_parentheses(
                                                &static_expr.object,
                                            ),
                                            declarator.span(),
                                            false,
                                        )
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn get_target_name<'a>(target: &'a AssignmentTarget<'a>) -> Option<&'a str> {
    if let AssignmentTarget::AssignmentTargetIdentifier(ident) = target {
        return Some(ident.name.as_str());
    }
    None
}

fn check_expr(expr: &MemberExpression) -> bool {
    if matches!(expr, MemberExpression::PrivateFieldExpression(_))
        || matches!(expr.object(), Expression::Super(_))
    {
        return false;
    }
    true
}

/// Returns the span of the object expression, stripping redundant parentheses for expressions
/// where they are unnecessary in the destructuring context.
///
/// For example: `(bar[baz]).foo` -> uses span of `bar[baz]` (without parens)
/// But: `(a, b).foo` -> uses span of `(a, b)` (keeps parens, comma operator needs them)
fn get_object_span_without_redundant_parentheses(object: &Expression) -> Span {
    match object.without_parentheses() {
        Expression::CallExpression(_)
        | Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::ThisExpression(_) => object.without_parentheses().span(),
        _ => object.span(),
    }
}

/// Generate the fix for object destructuring.
/// `is_assignment` determines whether to wrap in parentheses for assignment expressions.
fn generate_fix(
    fixer: &crate::fixer::RuleFixer<'_, '_>,
    prop_span: Span,
    object_span: Span,
    replacement_span: Span,
    is_assignment: bool,
) -> crate::fixer::RuleFix {
    let prop = fixer.source_range(prop_span);
    let object_text = fixer.source_range(object_span);
    let replacement = if is_assignment {
        format!("({{ {prop} }} = {object_text})")
    } else {
        format!("{{{prop}}} = {object_text}")
    };
    fixer.replace(replacement_span, replacement)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var [foo] = array;", None),
        ("var { foo } = object;", None),
        (
            "a = b.c",
            Some(
                serde_json::json!([{ "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": true }]),
            ),
        ),
        (
            "let a = arr[0];",
            Some(
                serde_json::json!([{ "AssignmentExpression": { "object": true, "array": true } }]),
            ),
        ),
        (
            "var a = arr[0];",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": true } }])),
        ),
        ("a = arr[0];", Some(serde_json::json!([{ "AssignmentExpression": { "object": true } }]))),
        ("let a = arr[0];", Some(serde_json::json!([{ "object": true }]))),
        ("var foo;", None),
        (
            "var foo = object.bar;",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": true } }])),
        ),
        ("var foo = object.bar;", Some(serde_json::json!([{ "object": true }]))),
        (
            "var foo = object.bar;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "var foo = object.bar;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": false }])),
        ),
        (
            "var foo = object['bar'];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "var foo = object[bar];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": false }])),
        ),
        (
            "var { bar: foo } = object;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var { bar: foo } = object;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var { [bar]: foo } = object;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var { [bar]: foo } = object;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = array[0];",
            Some(serde_json::json!([{ "VariableDeclarator": { "array": false } }])),
        ),
        ("var foo = array[0];", Some(serde_json::json!([{ "array": false }]))),
        (
            "var foo = object.foo;",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": false } }])),
        ),
        (
            "var foo = object['foo'];",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": false } }])),
        ),
        ("({ foo } = object);", None),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(serde_json::json!([{ "array": false }, { "enforceForRenamedProperties": true }])),
        ),
        ("[foo] = array;", None),
        ("foo += array[0]", None),
        ("foo &&= array[0]", None), // { "ecmaVersion": 2021 },
        ("foo += bar.foo", None),
        ("foo ||= bar.foo", None),    // { "ecmaVersion": 2021 },
        ("foo ??= bar['foo']", None), // { "ecmaVersion": 2021 },
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": true } ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": false } ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false } }, { "enforceForRenamedProperties": true } ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(serde_json::json!([ { "AssignmentExpression": { "array": false } } ])),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false } } ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false }, "AssignmentExpression": { "array": true } } ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true }, "AssignmentExpression": { "object": false } } ]),
            ),
        ),
        (
            "var foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": false }, "AssignmentExpression": { "object": true } } ]),
            ),
        ),
        ("class Foo extends Bar { static foo() {var foo = super.foo} }", None),
        ("foo = bar[foo];", None),
        ("var foo = bar[foo];", None),
        ("var {foo: {bar}} = object;", Some(serde_json::json!([{ "object": true }]))),
        ("var {bar} = object.foo;", Some(serde_json::json!([{ "object": true }]))),
        ("var foo = array?.[0];", None),
        ("var foo = object?.foo;", None),
        ("class C { #x; foo() { const x = this.#x; } }", None),
        ("class C { #x; foo() { x = this.#x; } }", None),
        ("class C { #x; foo(a) { x = a.#x; } }", None),
        (
            "class C { #x; foo() { const x = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { const y = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { x = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { y = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo(a) { x = a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo(a) { y = a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { x = this.a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
    ];

    let fail = vec![
        ("var foo = array[0];", None),
        ("foo = array[0];", None),
        ("var foo = object.foo;", None),
        ("var foo = (a, b).foo;", None),
        ("var length = (() => {}).length;", None),
        ("var foo = (a = b).foo;", None),
        ("var foo = (a || b).foo;", None),
        ("var foo = (f()).foo;", None),
        ("var foo = object.bar.foo;", None),
        (
            "var foobar = object.bar;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foobar = object.bar;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = object[bar];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = object[bar];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = object[foo];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        ("var foo = object['foo'];", None),
        ("foo = object.foo;", None),
        ("foo = object['foo'];", None),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(serde_json::json!([{ "AssignmentExpression": { "array": true } }])),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false }, }, ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false }, "AssignmentExpression": { "array": true }, }, ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true, "object": false }, "AssignmentExpression": { "object": true }, }, ]),
            ),
        ),
        ("class Foo extends Bar { static foo() {var bar = super.foo.bar} }", None),
        ("var /* comment */ foo = object.foo;", None),
        ("var a, /* comment */foo = object.foo;", None),
        ("var foo /* comment */ = object.foo;", None),
        ("var a, foo /* comment */ = object.foo;", None),
        ("var foo /* comment */ = object.foo, a;", None),
        (
            "var foo // comment
			 = object.foo;",
            None,
        ),
        ("var foo = /* comment */ object.foo;", None),
        (
            "var foo = // comment
			 object.foo;",
            None,
        ),
        ("var foo = (/* comment */ object).foo;", None),
        ("var foo = (object /* comment */).foo;", None),
        ("var foo = bar(/* comment */).foo;", None),
        ("var foo = bar/* comment */.baz.foo;", None),
        (
            "var foo = bar[// comment
			baz].foo;",
            None,
        ),
        (
            "var foo // comment
			 = bar(/* comment */).foo;",
            None,
        ),
        ("var foo = bar/* comment */.baz/* comment */.foo;", None),
        (
            "var foo = object// comment
			.foo;",
            None,
        ),
        ("var foo = object./* comment */foo;", None),
        ("var foo = (/* comment */ object.foo);", None),
        ("var foo = (object.foo /* comment */);", None),
        ("var foo = object.foo/* comment */;", None),
        ("var foo = object.foo// comment", None),
        ("var foo = object.foo/* comment */, a;", None),
        (
            "var foo = object.foo// comment
			, a;",
            None,
        ),
        ("var foo = object.foo, /* comment */ a;", None),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        ("var foo = object.foo;", "var {foo} = object;", None),
        ("var foo = (a, b).foo;", "var {foo} = (a, b);", None),
        //     ("var length = (() => {}).length;", "var {length} = () => {};", None),
        //     ("var foo = (a = b).foo;", "var {foo} = a = b;", None),
        //     ("var foo = (a || b).foo;", "var {foo} = a || b;", None),
        ("var foo = (f()).foo;", "var {foo} = f();", None),
        ("var foo = object.bar.foo;", "var {foo} = object.bar;", None),
        (
            "class Foo extends Bar { static foo() {var bar = super.foo.bar} }",
            "class Foo extends Bar { static foo() {var {bar} = super.foo} }",
            None,
        ),
        ("var /* comment */ foo = object.foo;", "var /* comment */ {foo} = object;", None),
        ("var a, /* comment */foo = object.foo;", "var a, /* comment */{foo} = object;", None),
        ("var foo = bar(/* comment */).foo;", "var {foo} = bar(/* comment */);", None),
        ("var foo = bar/* comment */.baz.foo;", "var {foo} = bar/* comment */.baz;", None),
        (
            "var foo = bar[// comment
        		baz].foo;",
            "var {foo} = bar[// comment
        		baz];",
            None,
        ),
        ("var foo = (bar[baz]).foo;", "var {foo} = bar[baz];", None),
        ("var foo = object.foo/* comment */;", "var {foo} = object/* comment */;", None),
        ("var foo = object.foo// comment", "var {foo} = object// comment", None),
        ("var foo = object.foo/* comment */, a;", "var {foo} = object/* comment */, a;", None),
        (
            "var foo = object.foo// comment
        		, a;",
            "var {foo} = object// comment
        		, a;",
            None,
        ),
        ("var foo = object.foo, /* comment */ a;", "var {foo} = object, /* comment */ a;", None),
        ("var foo = object['foo'];", "var {foo} = object;", None),
        ("foo = object.foo;", "({ foo } = object);", None),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
