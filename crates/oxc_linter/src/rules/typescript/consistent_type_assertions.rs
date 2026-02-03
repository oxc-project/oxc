use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{Expression, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::outermost_paren_parent,
    context::{ContextHost, LintContext},
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn use_angle_bracket_diagnostic(cast: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `<{cast}>` instead of `as {cast}`.")).with_label(span)
}

fn use_as_diagnostic(cast: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `as {cast}` instead of `<{cast}>`.")).with_label(span)
}

fn never_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use any type assertions.").with_label(span)
}

fn unexpected_object_type_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Always prefer `const x: T = { ... }`.").with_label(span)
}

fn unexpected_array_type_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Always prefer `const x: T[] = [ ... ]`.").with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum AssertionStyle {
    As,
    AngleBracket,
    Never,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum AssertionStyleNonNever {
    #[default]
    As,
    AngleBracket,
}

impl From<AssertionStyleNonNever> for AssertionStyle {
    fn from(value: AssertionStyleNonNever) -> Self {
        match value {
            AssertionStyleNonNever::As => Self::As,
            AssertionStyleNonNever::AngleBracket => Self::AngleBracket,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum AssertionStyleNever {
    Never,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum LiteralAssertionOption {
    #[default]
    Allow,
    AllowAsParameter,
    Never,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConsistentTypeAssertionsNeverConfig {
    assertion_style: AssertionStyleNever,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConsistentTypeAssertionsStyleConfig {
    /// Which assertion syntax is enforced when type assertions are allowed.
    ///
    /// - `"as"` (default)
    /// - `"angle-bracket"`
    #[serde(default)]
    assertion_style: AssertionStyleNonNever,
    /// Whether object literal type assertions are allowed, allowed only as parameters, or disallowed.
    ///
    /// - `"allow"` (default)
    /// - `"allow-as-parameter"`
    /// - `"never"`
    #[serde(default)]
    object_literal_type_assertions: LiteralAssertionOption,
    /// Whether array literal type assertions are allowed, allowed only as parameters, or disallowed.
    ///
    /// - `"allow"` (default)
    /// - `"allow-as-parameter"`
    /// - `"never"`
    #[serde(default)]
    array_literal_type_assertions: LiteralAssertionOption,
}

impl Default for ConsistentTypeAssertionsStyleConfig {
    fn default() -> Self {
        Self {
            assertion_style: AssertionStyleNonNever::As,
            object_literal_type_assertions: LiteralAssertionOption::Allow,
            array_literal_type_assertions: LiteralAssertionOption::Allow,
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ConsistentTypeAssertionsConfig {
    Never(ConsistentTypeAssertionsNeverConfig),
    Style(ConsistentTypeAssertionsStyleConfig),
}

impl Default for ConsistentTypeAssertionsConfig {
    fn default() -> Self {
        Self::Style(ConsistentTypeAssertionsStyleConfig::default())
    }
}

impl ConsistentTypeAssertionsConfig {
    fn assertion_style(&self) -> AssertionStyle {
        match self {
            Self::Never(config) => match config.assertion_style {
                AssertionStyleNever::Never => AssertionStyle::Never,
            },
            Self::Style(config) => config.assertion_style.into(),
        }
    }

    fn object_literal_type_assertions(&self) -> LiteralAssertionOption {
        match self {
            Self::Style(config) => config.object_literal_type_assertions,
            Self::Never(_) => LiteralAssertionOption::Allow,
        }
    }

    fn array_literal_type_assertions(&self) -> LiteralAssertionOption {
        match self {
            Self::Style(config) => config.array_literal_type_assertions,
            Self::Never(_) => LiteralAssertionOption::Allow,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentTypeAssertions(Box<ConsistentTypeAssertionsConfig>);

impl Deref for ConsistentTypeAssertions {
    type Target = ConsistentTypeAssertionsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent usage of TypeScript type assertions.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing assertion styles (`as` vs angle-bracket) makes code harder to read and maintain.
    /// In some codebases, type assertions are banned in favor of safer alternatives like
    /// type annotations or `satisfies`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (default: `assertionStyle: "as"`):
    /// ```ts
    /// const value = <Foo>bar;
    /// ```
    ///
    /// Examples of **correct** code for this rule (default: `assertionStyle: "as"`):
    /// ```ts
    /// const value = bar as Foo;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `assertionStyle: "angle-bracket"`:
    /// ```ts
    /// const value = bar as Foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule with `assertionStyle: "angle-bracket"`:
    /// ```ts
    /// const value = <Foo>bar;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `assertionStyle: "never"`:
    /// ```ts
    /// const value = bar as Foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule with `assertionStyle: "never"`:
    /// ```ts
    /// const value: Foo = bar;
    /// const value = bar satisfies Foo;
    /// ```
    ///
    /// When object/array literal assertions are disallowed, prefer annotations or `satisfies`:
    /// ```ts
    /// // incorrect (when `objectLiteralTypeAssertions: "never"`)
    /// const obj = { a: 1 } as Foo;
    ///
    /// // correct
    /// const obj: Foo = { a: 1 };
    /// const obj = { a: 1 } satisfies Foo;
    /// ```
    ConsistentTypeAssertions,
    typescript,
    style,
    conditional_fix_suggestion,
    config = ConsistentTypeAssertionsConfig,
);

impl Rule for ConsistentTypeAssertions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSAsExpression(as_expression) => match self.assertion_style() {
                AssertionStyle::As => {
                    check_expression_for_object_assertion(
                        node,
                        &as_expression.expression,
                        &as_expression.type_annotation,
                        ctx,
                        self,
                    );
                    check_expression_for_array_assertion(
                        node,
                        &as_expression.expression,
                        &as_expression.type_annotation,
                        ctx,
                        self,
                    );
                }
                AssertionStyle::AngleBracket => {
                    let cast = ctx.source_range(as_expression.type_annotation.span());
                    ctx.diagnostic(use_angle_bracket_diagnostic(cast, as_expression.span));
                }
                AssertionStyle::Never => {
                    if is_const(&as_expression.type_annotation) {
                        return;
                    }
                    ctx.diagnostic(never_diagnostic(as_expression.span));
                }
            },
            AstKind::TSTypeAssertion(type_assertion) => match self.assertion_style() {
                AssertionStyle::AngleBracket => {
                    check_expression_for_object_assertion(
                        node,
                        &type_assertion.expression,
                        &type_assertion.type_annotation,
                        ctx,
                        self,
                    );
                    check_expression_for_array_assertion(
                        node,
                        &type_assertion.expression,
                        &type_assertion.type_annotation,
                        ctx,
                        self,
                    );
                }
                AssertionStyle::As => {
                    let cast = ctx.source_range(type_assertion.type_annotation.span());
                    let is_parenthesized = is_parenthesized(node, ctx);
                    let needs_parentheses = !is_parenthesized && needs_parens_for_parent(node, ctx);
                    ctx.diagnostic_with_fix(
                        use_as_diagnostic(cast, type_assertion.span),
                        |fixer| {
                            let mut expression_text = expression_text_for_as(
                                &type_assertion.expression,
                                fixer.source_text(),
                            );
                            if !needs_parentheses
                                && !matches!(
                                    type_assertion.expression,
                                    Expression::ParenthesizedExpression(_)
                                )
                                && matches!(
                                    type_assertion.expression.without_parentheses(),
                                    Expression::ObjectExpression(_)
                                )
                                && needs_parens_for_object_literal_replacement(node, ctx)
                            {
                                expression_text = format!("({expression_text})");
                            }
                            let type_text =
                                fixer.source_range(type_assertion.type_annotation.span());
                            let mut replacement = format!("{expression_text} as {type_text}");
                            if needs_parentheses {
                                replacement = format!("({replacement})");
                            }
                            fixer.replace(type_assertion.span, replacement)
                        },
                    );
                }
                AssertionStyle::Never => {
                    if is_const(&type_assertion.type_annotation) {
                        return;
                    }
                    ctx.diagnostic(never_diagnostic(type_assertion.span));
                }
            },
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn is_const(type_annotation: &TSType) -> bool {
    if let TSType::TSTypeReference(type_reference) = type_annotation
        && let TSTypeName::IdentifierReference(ident) = &type_reference.type_name
    {
        return ident.name.as_str() == "const";
    }

    false
}

fn check_type(type_annotation: &TSType) -> bool {
    match type_annotation {
        TSType::TSAnyKeyword(_) | TSType::TSUnknownKeyword(_) => false,
        TSType::TSTypeReference(type_reference) => {
            if let TSTypeName::IdentifierReference(ident) = &type_reference.type_name {
                return ident.name.as_str() != "const";
            }
            true
        }
        _ => true,
    }
}

fn expression_text_for_as(expression: &Expression, source_text: &str) -> String {
    let text = expression.span().source_text(source_text);
    if matches!(expression, Expression::ParenthesizedExpression(_)) {
        return text.to_string();
    }
    if expression_needs_parens_for_as(expression) { format!("({text})") } else { text.to_string() }
}

fn expression_needs_parens_for_as(expression: &Expression) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::SequenceExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::YieldExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::TSAsExpression(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::TSTypeAssertion(_)
            | Expression::TSInstantiationExpression(_)
    )
}

fn is_parenthesized(node: &AstNode, ctx: &LintContext) -> bool {
    matches!(ctx.nodes().parent_kind(node.id()), AstKind::ParenthesizedExpression(_))
}

fn needs_parens_for_parent(node: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let node_span = node.kind().span();

    match parent.kind() {
        AstKind::BinaryExpression(_)
        | AstKind::LogicalExpression(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::UnaryExpression(_)
        | AstKind::UpdateExpression(_)
        | AstKind::AwaitExpression(_)
        | AstKind::YieldExpression(_)
        | AstKind::AssignmentExpression(_)
        | AstKind::SequenceExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_)
        | AstKind::TSTypeAssertion(_) => true,
        AstKind::CallExpression(call_expr) => call_expr.callee.span() == node_span,
        AstKind::NewExpression(new_expr) => new_expr.callee.span() == node_span,
        AstKind::StaticMemberExpression(member) => member.object.span() == node_span,
        AstKind::ComputedMemberExpression(member) => member.object.span() == node_span,
        AstKind::TaggedTemplateExpression(tagged) => tagged.tag.span() == node_span,
        AstKind::ArrowFunctionExpression(arrow) => {
            arrow.get_expression().is_some_and(|expr| expr.span() == node_span)
        }
        AstKind::ExpressionStatement(expr_stmt) => {
            let mut ancestor = ctx.nodes().parent_node(parent.id());
            if matches!(ancestor.kind(), AstKind::FunctionBody(_)) {
                ancestor = ctx.nodes().parent_node(ancestor.id());
            }
            if let AstKind::ArrowFunctionExpression(arrow) = ancestor.kind() {
                return arrow
                    .get_expression()
                    .is_some_and(|expr| expr.span() == expr_stmt.expression.span());
            }
            false
        }
        _ => false,
    }
}

fn is_as_parameter<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx.semantic()) else {
        return false;
    };

    match parent.kind() {
        AstKind::NewExpression(_)
        | AstKind::CallExpression(_)
        | AstKind::ThrowStatement(_)
        | AstKind::AssignmentPattern(_)
        | AstKind::FormalParameter(_)
        | AstKind::JSXExpressionContainer(_) => true,
        AstKind::TemplateLiteral(_) => {
            outermost_paren_parent(parent, ctx.semantic()).is_some_and(|grandparent| {
                matches!(grandparent.kind(), AstKind::TaggedTemplateExpression(_))
            })
        }
        _ => false,
    }
}

fn expression_text_for_replacement<'a>(
    node: &AstNode<'a>,
    expression: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> String {
    let text = ctx.source_range(expression.span());
    if matches!(expression, Expression::ParenthesizedExpression(_)) {
        return text.to_string();
    }
    if matches!(expression.without_parentheses(), Expression::ObjectExpression(_))
        && needs_parens_for_object_literal_replacement(node, ctx)
    {
        return format!("({text})");
    }
    text.to_string()
}

fn needs_parens_for_object_literal_replacement<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx.semantic()) else {
        return false;
    };
    match parent.kind() {
        AstKind::ExpressionStatement(_) => true,
        AstKind::ArrowFunctionExpression(arrow) => {
            let node_span = node.kind().span();
            arrow.get_expression().is_some_and(|expr| expr.span() == node_span)
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy)]
enum LiteralKind {
    Object,
    Array,
}

impl LiteralKind {
    fn example(self) -> &'static str {
        match self {
            Self::Object => "{ ... }",
            Self::Array => "[ ... ]",
        }
    }
}

fn get_suggestions<'a>(
    node: &AstNode<'a>,
    expression: &Expression<'a>,
    type_annotation: &TSType<'a>,
    ctx: &LintContext<'a>,
    literal_kind: LiteralKind,
) -> Vec<RuleFix> {
    let type_text = ctx.source_range(type_annotation.span());
    let expression_text = expression_text_for_replacement(node, expression, ctx);
    let mut suggestions = Vec::new();
    let literal = literal_kind.example();

    if let Some(parent) = outermost_paren_parent(node, ctx.semantic())
        && let AstKind::VariableDeclarator(var_decl) = parent.kind()
        && var_decl.type_annotation.is_none()
    {
        let fixer = RuleFixer::new(FixKind::Suggestion, ctx).for_multifix();
        let mut fix = fixer.new_fix_with_capacity(2);
        fix.push(fixer.insert_text_after(&var_decl.id, format!(": {type_text}")));
        fix.push(fixer.replace(node.kind().span(), expression_text.clone()));
        suggestions
            .push(fix.with_message(format!("Use const x: {type_text} = {literal} instead.")));
    }

    let fixer = RuleFixer::new(FixKind::Suggestion, ctx).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(fixer.replace(node.kind().span(), expression_text));
    fix.push(fixer.insert_text_after_range(node.kind().span(), format!(" satisfies {type_text}")));
    suggestions
        .push(fix.with_message(format!("Use const x = {literal} satisfies {type_text} instead.")));

    suggestions
}

fn check_expression_for_object_assertion<'a>(
    node: &AstNode<'a>,
    expression: &Expression<'a>,
    type_annotation: &TSType<'a>,
    ctx: &LintContext<'a>,
    config: &ConsistentTypeAssertionsConfig,
) {
    if matches!(config.assertion_style(), AssertionStyle::Never)
        || matches!(config.object_literal_type_assertions(), LiteralAssertionOption::Allow)
    {
        return;
    }

    if !matches!(expression.without_parentheses(), Expression::ObjectExpression(_)) {
        return;
    }

    if matches!(config.object_literal_type_assertions(), LiteralAssertionOption::AllowAsParameter)
        && is_as_parameter(node, ctx)
    {
        return;
    }

    if check_type(type_annotation) {
        let suggestions =
            get_suggestions(node, expression, type_annotation, ctx, LiteralKind::Object);
        ctx.diagnostic_with_suggestions(
            unexpected_object_type_assertion_diagnostic(node.kind().span()),
            suggestions,
        );
    }
}

fn check_expression_for_array_assertion<'a>(
    node: &AstNode<'a>,
    expression: &Expression<'a>,
    type_annotation: &TSType<'a>,
    ctx: &LintContext<'a>,
    config: &ConsistentTypeAssertionsConfig,
) {
    if matches!(config.assertion_style(), AssertionStyle::Never)
        || matches!(config.array_literal_type_assertions(), LiteralAssertionOption::Allow)
    {
        return;
    }

    if !matches!(expression.without_parentheses(), Expression::ArrayExpression(_)) {
        return;
    }

    if matches!(config.array_literal_type_assertions(), LiteralAssertionOption::AllowAsParameter)
        && is_as_parameter(node, ctx)
    {
        return;
    }

    if check_type(type_annotation) {
        let suggestions =
            get_suggestions(node, expression, type_annotation, ctx, LiteralKind::Array);
        ctx.diagnostic_with_suggestions(
            unexpected_array_type_assertion_diagnostic(node.kind().span()),
            suggestions,
        );
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::{TestCase, Tester};

    let pass = vec![
        (
            "const x = new Generic<int>() as Foo;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = b as A;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = [1] as readonly number[];",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = 'string' as a | b;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = !'string' as A;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = (a as A) + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = new Generic<string>() as Foo;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = new (Generic<string> as Foo)();",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = new (Generic<string> as Foo)('string');",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = () => ({ bar: 5 }) as Foo;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = () => bar as Foo;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = bar<string>`${'baz'}` as Foo;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = { key: 'value' } as const;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <Foo>new Generic<int>();",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <A>b;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <readonly number[]>[1];",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <a | b>'string';",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <A>!'string';",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <A>a + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <Foo>new Generic<string>();",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = new (<Foo>Generic<string>)();",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = new (<Foo>Generic<string>)('string');",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = () => <Foo>{ bar: 5 };",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = () => <Foo>bar;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <Foo>bar<string>`${'baz'}`;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <const>{ key: 'value' };",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = {} as Foo<int>;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "const x = {} as a | b;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "const x = ({} as A) + b;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "new print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "
            function foo() {
              throw { bar: 5 } as Foo;
            }
                  ",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "function b(x = {} as Foo.Bar) {}",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "function c(x = {} as Foo) {}",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "print?.({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "print?.call({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "print`${{ bar: 5 } as Foo}`;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "allow" }]),
            ),
        ),
        (
            "const x = <Foo<int>>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <a | b>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "const x = <A>{} + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "new print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw <Foo>{ bar: 5 };
            }
                  ",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "print?.(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "print?.call(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "print`${<Foo>{ bar: 5 }}`;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow", }, ]),
            ),
        ),
        (
            "print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "new print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw { bar: 5 } as Foo;
            }
                  ",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "function b(x = {} as Foo.Bar) {}",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "function c(x = {} as Foo) {}",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print?.({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print?.call({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print`${{ bar: 5 } as Foo}`;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "new print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw <Foo>{ bar: 5 };
            }
                  ",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print?.(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print?.call(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "print`${<Foo>{ bar: 5 }}`;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        ("const x = [] as string[];", Some(serde_json::json!([ { "assertionStyle": "as", }, ]))),
        (
            "const x = ['a'] as Array<string>;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const x = <string[]>[];",
            Some(serde_json::json!([ { "assertionStyle": "angle-bracket", }, ])),
        ),
        (
            "const x = <Array<string>>[];",
            Some(serde_json::json!([ { "assertionStyle": "angle-bracket", }, ])),
        ),
        (
            "print([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw [5] as Foo;
            }
                  ",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "function b(x = [5] as Foo.Bar) {}",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "print?.([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "print?.call([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "print`${[5] as Foo}`;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "new Print([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "print(<Foo>[5]);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw <Foo>[5];
            }
                  ",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "function b(x = <Foo.Bar>[5]) {}",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "print?.(<Foo>[5]);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "print?.call(<Foo>[5]);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "print`${<Foo>[5]}`;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "new Print(<Foo>[5]);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        ("const x = <const>[1];", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        ("const x = [1] as const;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "
            const x = { key: 'value' } as any;
                  ",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "
            const x = { key: 'value' } as unknown;
                  ",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
    ];

    let mut pass: Vec<TestCase> = pass.into_iter().map(Into::into).collect();
    pass.push(
        (
            "const bar = <Foo style={[5] as Bar} />;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
            None,
            Some(PathBuf::from("consistent_type_assertions.tsx")),
        )
            .into(),
    );
    pass.push(
        (
            "const bar = <Foo style={{ bar: 5 } as Bar} />;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
            None,
            Some(PathBuf::from("consistent_type_assertions.tsx")),
        )
            .into(),
    );

    let fail = vec![
        (
            "const x = new Generic<int>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        ("const x = b as A;", Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }]))),
        (
            "const x = [1] as readonly number[];",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = 'string' as a | b;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = !'string' as A;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = (a as A) + b;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = new Generic<string>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = new (Generic<string> as Foo)();",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = new (Generic<string> as Foo)('string');",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = () => ({ bar: 5 }) as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = () => bar as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = bar<string>`${'baz'}` as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = { key: 'value' } as const;",
            Some(serde_json::json!([{ "assertionStyle": "angle-bracket" }])),
        ),
        (
            "const x = <Foo>new Generic<int>();",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        ("const x = <A>b;", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        (
            "const x = <readonly number[]>[1];",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        ("const x = <a | b>'string';", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        ("const x = <A>!'string';", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        ("const x = <A>a + b;", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        (
            "const x = <Foo>new Generic<string>();",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)();",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)('string');",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        ("const x = () => <Foo>{ bar: 5 };", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        ("const x = () => <Foo>bar;", Some(serde_json::json!([{ "assertionStyle": "as" }]))),
        (
            "const x = <Foo>bar<string>`${'baz'}`;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <const>{ key: 'value' };",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = new Generic<int>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = b as A;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = [1] as readonly number[];",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = 'string' as a | b;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        ("const x = !'string' as A;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        ("const x = (a as A) + b;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = new Generic<string>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = new (Generic<string> as Foo)();",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = new (Generic<string> as Foo)('string');",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = () => ({ bar: 5 }) as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = () => bar as Foo;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = bar<string>`${'baz'}` as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = <Foo>new Generic<int>();",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = <A>b;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = <readonly number[]>[1];",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = <a | b>'string';", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        ("const x = <A>!'string';", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        ("const x = <A>a + b;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = <Foo>new Generic<string>();",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)();",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)('string');",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = () => <Foo>{ bar: 5 };",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        ("const x = () => <Foo>bar;", Some(serde_json::json!([{ "assertionStyle": "never" }]))),
        (
            "const x = <Foo>bar<string>`${'baz'}`;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
        ),
        (
            "const x = {} as Foo<int>;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = {} as a | b;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = ({} as A) + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "as", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = <Foo<int>>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = <a | b>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = <A>{} + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "allow-as-parameter", }, ]),
            ),
        ),
        (
            "const x = {} as Foo<int>;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "const x = {} as a | b;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "const x = ({} as A) + b;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "new print({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "
            function foo() {
              throw { bar: 5 } as Foo;
            }
                  ",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "function b(x = {} as Foo.Bar) {}",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "function c(x = {} as Foo) {}",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "print?.({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "print?.call({ bar: 5 } as Foo);",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "print`${{ bar: 5 } as Foo}`;",
            Some(
                serde_json::json!([{ "assertionStyle": "as", "objectLiteralTypeAssertions": "never" }]),
            ),
        ),
        (
            "const x = <Foo<int>>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "const x = <a | b>{};",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "const x = <A>{} + b;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "new print(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw <Foo>{ bar: 5 };
            }
                  ",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "print?.(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "print?.call(<Foo>{ bar: 5 });",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        (
            "print`${<Foo>{ bar: 5 }}`;",
            Some(
                serde_json::json!([ { "assertionStyle": "angle-bracket", "objectLiteralTypeAssertions": "never", }, ]),
            ),
        ),
        ("const a = <any>(b, c);", Some(serde_json::json!([ { "assertionStyle": "as", }, ]))),
        ("const f = <any>(() => {});", Some(serde_json::json!([ { "assertionStyle": "as", }, ]))),
        (
            "const f = <any>function () {};",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const f = <any>(async () => {});",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "
            function* g() {
              const y = <any>(yield a);
            }
                  ",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "
            declare let x: number, y: number;
            const bs = <any>(x <<= y);
                  ",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const ternary = <any>(true ? x : y);",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        ("const x = [] as string[];", Some(serde_json::json!([ { "assertionStyle": "never", }, ]))),
        ("const x = <string[]>[];", Some(serde_json::json!([ { "assertionStyle": "never", }, ]))),
        (
            "const x = [] as string[];",
            Some(serde_json::json!([ { "assertionStyle": "angle-bracket", }, ])),
        ),
        ("const x = <string[]>[];", Some(serde_json::json!([ { "assertionStyle": "as", }, ]))),
        (
            "const x = [] as string[];",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "const x = <string[]>[];",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "print([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "new print([5] as Foo);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "function b(x = [5] as Foo.Bar) {}",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw [5] as Foo;
            }
                  ",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "print`${[5] as Foo}`;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "const foo = () => [5] as Foo;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "as", }, ]),
            ),
        ),
        (
            "new print(<Foo>[5]);",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "function b(x = <Foo.Bar>[5]) {}",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "
            function foo() {
              throw <Foo>[5];
            }
                  ",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "print`${<Foo>[5]}`;",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "never", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
        (
            "const foo = <Foo>[5];",
            Some(
                serde_json::json!([ { "arrayLiteralTypeAssertions": "allow-as-parameter", "assertionStyle": "angle-bracket", }, ]),
            ),
        ),
    ];
    let mut fail: Vec<TestCase> = fail.into_iter().map(Into::into).collect();
    fail.push(
        (
            "const foo = <Foo style={{ bar: 5 } as Bar} />;",
            Some(serde_json::json!([{ "assertionStyle": "never" }])),
            None,
            Some(PathBuf::from("consistent_type_assertions.tsx")),
        )
            .into(),
    );

    let fix = vec![
        (
            "const x = <Foo>new Generic<int>();",
            "const x = new Generic<int>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <A>b;",
            "const x = b as A;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <readonly number[]>[1];",
            "const x = [1] as readonly number[];",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <a | b>'string';",
            "const x = 'string' as a | b;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <A>!'string';",
            "const x = !'string' as A;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <A>a + b;",
            "const x = (a as A) + b;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <Foo>new Generic<string>();",
            "const x = new Generic<string>() as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)();",
            "const x = new ((Generic<string>) as Foo)();",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = new (<Foo>Generic<string>)('string');",
            "const x = new ((Generic<string>) as Foo)('string');",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = () => <Foo>{ bar: 5 };",
            "const x = () => ({ bar: 5 } as Foo);",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = () => <Foo>bar;",
            "const x = () => (bar as Foo);",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <Foo>bar<string>`${'baz'}`;",
            "const x = bar<string>`${'baz'}` as Foo;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const x = <const>{ key: 'value' };",
            "const x = { key: 'value' } as const;",
            Some(serde_json::json!([{ "assertionStyle": "as" }])),
        ),
        (
            "const a = <any>(b, c);",
            "const a = (b, c) as any;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const f = <any>(() => {});",
            "const f = (() => {}) as any;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const f = <any>function () {};",
            "const f = function () {} as any;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const f = <any>(async () => {});",
            "const f = (async () => {}) as any;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "
            function* g() {
              const y = <any>(yield a);
            }
                  ",
            "
            function* g() {
              const y = (yield a) as any;
            }
                  ",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "
            declare let x: number, y: number;
            const bs = <any>(x <<= y);
                  ",
            "
            declare let x: number, y: number;
            const bs = (x <<= y) as any;
                  ",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const ternary = <any>(true ? x : y);",
            "const ternary = (true ? x : y) as any;",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
        (
            "const x = <string[]>[];",
            "const x = [] as string[];",
            Some(serde_json::json!([ { "assertionStyle": "as", }, ])),
        ),
    ];

    Tester::new(ConsistentTypeAssertions::NAME, ConsistentTypeAssertions::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
