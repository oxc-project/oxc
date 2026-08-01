use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, BindingPattern, Expression, MemberExpression, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
};

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
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferDestructuringTargetConfig {
    array: bool,
    object: bool,
}

impl Default for PreferDestructuringTargetConfig {
    fn default() -> Self {
        Self { array: true, object: true }
    }
}

impl PreferDestructuringTargetConfig {
    fn disabled() -> Self {
        Self { array: false, object: false }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreferDestructuringTargetOption {
    array: Option<bool>,
    object: Option<bool>,
}

impl PreferDestructuringTargetOption {
    fn enabled_by_default() -> Self {
        Self { array: Some(true), object: Some(true) }
    }

    fn into_config(self) -> PreferDestructuringTargetConfig {
        PreferDestructuringTargetConfig {
            array: self.array.unwrap_or(false),
            object: self.object.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
struct PreferDestructuringAssignmentConfig {
    variable_declarator: Option<PreferDestructuringTargetOption>,
    assignment_expression: Option<PreferDestructuringTargetOption>,
}

impl PreferDestructuringAssignmentConfig {
    fn into_configs(self) -> (PreferDestructuringTargetConfig, PreferDestructuringTargetConfig) {
        let variable_declarator = self.variable_declarator.map_or_else(
            PreferDestructuringTargetConfig::disabled,
            PreferDestructuringTargetOption::into_config,
        );
        let assignment_expression = self.assignment_expression.map_or_else(
            PreferDestructuringTargetConfig::disabled,
            PreferDestructuringTargetOption::into_config,
        );

        (variable_declarator, assignment_expression)
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(untagged)]
enum PreferDestructuringOption {
    Target(PreferDestructuringTargetOption),
    Assignment(PreferDestructuringAssignmentConfig),
}

impl Default for PreferDestructuringOption {
    fn default() -> Self {
        Self::Target(PreferDestructuringTargetOption::enabled_by_default())
    }
}

impl PreferDestructuringOption {
    fn into_configs(self) -> (PreferDestructuringTargetConfig, PreferDestructuringTargetConfig) {
        match self {
            Self::Target(config) => {
                let config = config.into_config();
                (config.clone(), config)
            }
            Self::Assignment(config) => config.into_configs(),
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferDestructuringEnforcementConfig {
    enforce_for_renamed_properties: bool,
    enforce_for_declaration_with_type_annotation: bool,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
struct PreferDestructuringConfig(PreferDestructuringOption, PreferDestructuringEnforcementConfig);

impl PreferDestructuringConfig {
    fn into_rule(self) -> PreferDestructuring {
        let (variable_declarator, assignment_expression) = self.0.into_configs();

        PreferDestructuring {
            variable_declarator,
            assignment_expression,
            enforce_for_renamed_properties: self.1.enforce_for_renamed_properties,
            enforce_for_declaration_with_type_annotation: self
                .1
                .enforce_for_declaration_with_type_annotation,
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferDestructuring {
    /// Configuration for destructuring in variable declarations, configured for arrays and objects independently.
    #[serde(rename = "VariableDeclarator")]
    variable_declarator: PreferDestructuringTargetConfig,
    /// Configuration for destructuring in assignment expressions, configured for arrays and objects independently.
    #[serde(rename = "AssignmentExpression")]
    assignment_expression: PreferDestructuringTargetConfig,
    /// Determines whether the object destructuring rule applies to renamed variables.
    enforce_for_renamed_properties: bool,
    /// Determines whether the rule applies to variable declarations with type annotations.
    enforce_for_declaration_with_type_annotation: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require destructuring from arrays and/or objects.
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
    conditional_fix,
    config = PreferDestructuringConfig,
    version = "1.10.0",
    short_description = "Require destructuring from arrays and/or objects.",
);

impl Rule for PreferDestructuring {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<PreferDestructuringConfig>>(value)
            .map(|config| config.into_inner().into_rule())
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
                                ctx.diagnostic(prefer_object_destructuring(assign_expr.span));
                            }
                        }
                    }
                    MemberExpression::StaticMemberExpression(static_expr)
                        if self.assignment_expression.object
                            && get_target_name(&assign_expr.left)
                                .is_some_and(|name| name == static_expr.property.name.as_str()) =>
                    {
                        ctx.diagnostic(prefer_object_destructuring(assign_expr.span));
                    }
                    _ => {}
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                let has_type_annotation = declarator.type_annotation.is_some();
                if has_type_annotation && !self.enforce_for_declaration_with_type_annotation {
                    return;
                }

                // Skip `using` and `await using` declarations - destructuring doesn't apply to them
                if matches!(
                    declarator.kind,
                    VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
                ) {
                    return;
                }
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
                            } else if self.variable_declarator.object {
                                if let Expression::StringLiteral(string_literal) =
                                    &comp_expr.expression
                                    && name.is_some_and(|v| v == string_literal.value)
                                {
                                    if has_type_annotation {
                                        ctx.diagnostic(prefer_object_destructuring(init.span()));
                                    } else {
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
                                                )
                                            },
                                        );
                                    }
                                } else if self.enforce_for_renamed_properties {
                                    ctx.diagnostic(prefer_object_destructuring(right.span()));
                                }
                            }
                        }
                        MemberExpression::StaticMemberExpression(static_expr)
                            if self.variable_declarator.object =>
                        {
                            if name.is_some_and(|name| name == static_expr.property.name.as_str()) {
                                if has_type_annotation {
                                    ctx.diagnostic(prefer_object_destructuring(init.span()));
                                } else {
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
                                            )
                                        },
                                    );
                                }
                            } else if self.enforce_for_renamed_properties {
                                ctx.diagnostic(prefer_object_destructuring(right.span()));
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

/// Generate the fix for object destructuring in a variable declaration.
fn generate_fix(
    fixer: &crate::fixer::RuleFixer<'_, '_>,
    prop_span: Span,
    object_span: Span,
    replacement_span: Span,
) -> crate::fixer::RuleFix {
    let prop = fixer.source_range(prop_span);
    let object_text = fixer.source_range(object_span);
    let replacement = format!("{{{prop}}} = {object_text}");
    fixer.replace(replacement_span, replacement)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var [foo] = array;", None),
        ("var { foo } = object;", None),
        ("const foo: string = object.foo;", None),
        ("const foo: string = object['foo'];", None),
        ("const foo: string = array[0];", None),
        (
            "const object = { foo: 'value' as const }; const foo: string = object.foo;",
            Some(serde_json::json!([
                {
                    "VariableDeclarator": { "array": false, "object": true },
                    "AssignmentExpression": { "array": false, "object": false }
                },
                {
                    "enforceForDeclarationWithTypeAnnotation": false,
                    "enforceForRenamedProperties": false
                }
            ])),
        ),
        (
            "const foo: string = object.foo;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
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
        ("using foo = array[0];", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("using foo = object.foo;", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("await using foo = array[0];", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("await using foo = object.foo;", None), // { "sourceType": "module", "ecmaVersion": 2026, }
    ];

    let fail = vec![
        ("var foo = array[0];", None),
        ("foo = array[0];", None),
        ("var foo = object.foo;", None),
        (
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object['foo'];",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = array[0];",
            Some(
                serde_json::json!([{ "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object.foo;",
            Some(serde_json::json!([
                { "object": true },
                {
                    "enforceForDeclarationWithTypeAnnotation": true,
                    "enforceForRenamedProperties": true
                }
            ])),
        ),
        (
            "var foo: string = object['foo'];",
            Some(serde_json::json!([
                { "object": true },
                {
                    "enforceForDeclarationWithTypeAnnotation": true,
                    "enforceForRenamedProperties": true
                }
            ])),
        ),
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
        // ("var length = (() => {}).length;", "var {length} = () => {};", None),
        // ("var foo = (a = b).foo;", "var {foo} = a = b;", None),
        // ("var foo = (a || b).foo;", "var {foo} = a || b;", None),
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
        (
            "var foo: string = object.foo;",
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object['foo'];",
            "var foo: string = object['foo'];",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        ("foo = object.foo;", "foo = object.foo;", None),
        ("foo = object['foo'];", "foo = object['foo'];", None),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
