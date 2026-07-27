use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingPattern, Expression, MemberExpression,
    VariableDeclarationKind, VariableDeclarator,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::TupleRuleConfig,
};

pub const DOCUMENTATION: &str = r"### What it does

Require destructuring from arrays and/or objects.

### Why is this bad?

With JavaScript ES2015, a new syntax was added for creating variables from an array index or object property,
called destructuring. This rule enforces usage of destructuring
instead of accessing a property through a member expression.

### Examples

Examples of **incorrect** code for this rule:
```js
// With `array` enabled
const foo = array[0];
bar.baz = array[0];
// With `object` enabled
const qux = object.qux;
const quux = object['quux'];
```

Examples of **correct** code for this rule:
```js
// With `array` enabled
const [ foo ] = array;
const arr = array[someIndex];
[bar.baz] = array;

// With `object` enabled
const { baz } = object;
const obj = object.bar;
```";

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
pub struct PreferDestructuringConfig(
    PreferDestructuringOption,
    PreferDestructuringEnforcementConfig,
);

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

/// Parsed state shared by the `eslint` and `typescript` variants of the rule.
#[derive(Debug, Default, Clone)]
pub struct PreferDestructuring {
    variable_declarator: PreferDestructuringTargetConfig,
    assignment_expression: PreferDestructuringTargetConfig,
    enforce_for_renamed_properties: bool,
    enforce_for_declaration_with_type_annotation: bool,
}

impl PreferDestructuring {
    pub fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<PreferDestructuringConfig>>(value)
            .map(|config| config.into_inner().into_rule())
    }

    pub fn run_on_assignment_expression<'a>(
        &self,
        assign_expr: &AssignmentExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(right) = assign_expr.right.without_parentheses().as_member_expression() else {
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
                    if self.enforce_for_renamed_properties && self.assignment_expression.object {
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

    pub fn run_on_variable_declarator<'a>(
        &self,
        declarator: &VariableDeclarator<'a>,
        ctx: &LintContext<'a>,
    ) {
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
                        if let Expression::StringLiteral(string_literal) = &comp_expr.expression
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
    fixer: &RuleFixer<'_, '_>,
    prop_span: Span,
    object_span: Span,
    replacement_span: Span,
) -> RuleFix {
    let prop = fixer.source_range(prop_span);
    let object_text = fixer.source_range(object_span);
    let replacement = format!("{{{prop}}} = {object_text}");
    fixer.replace(replacement_span, replacement)
}
