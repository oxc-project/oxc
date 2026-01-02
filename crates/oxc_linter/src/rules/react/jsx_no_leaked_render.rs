use oxc_ast::{
    AstKind,
    ast::{ConditionalExpression, Expression, JSXExpression, LogicalExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn leaked_render_diagnostic(span: Span, valid_strategies: &[ValidStrategy]) -> OxcDiagnostic {
    let help = match valid_strategies {
        [ValidStrategy::Ternary] => "Use a ternary expression: `condition ? <Element /> : null`",
        [ValidStrategy::Coerce] => "Coerce the condition to boolean with `!!` or `Boolean()`",
        _ => "Coerce the condition with `!!` or use a ternary: `condition ? <Element /> : null`",
    };

    OxcDiagnostic::warn(
        "Potential leaked value that might cause unintentionally rendered values or rendering crashes",
    )
    .with_help(help)
    .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidStrategy {
    #[default]
    Ternary,
    Coerce,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxNoLeakedRenderConfig {
    #[serde(default = "default_valid_strategies")]
    valid_strategies: Vec<ValidStrategy>,
}

fn default_valid_strategies() -> Vec<ValidStrategy> {
    vec![ValidStrategy::Ternary, ValidStrategy::Coerce]
}

impl Default for JsxNoLeakedRenderConfig {
    fn default() -> Self {
        Self { valid_strategies: default_valid_strategies() }
    }
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoLeakedRender(Box<JsxNoLeakedRenderConfig>);

impl std::ops::Deref for JsxNoLeakedRender {
    type Target = JsxNoLeakedRenderConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow problematic leaked values from being rendered.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `&&` operator to render JSX can cause unexpected values
    /// to appear in the DOM. Values like `0`, `NaN`, and empty strings
    /// are rendered by React, which may cause visual bugs or crashes
    /// in React Native.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// {count && <Items />}
    /// {items.length && <List />}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// {count > 0 && <Items />}
    /// {!!count && <Items />}
    /// {Boolean(count) && <Items />}
    /// {count ? <Items /> : null}
    /// ```
    JsxNoLeakedRender,
    react,
    correctness,
    conditional_fix,
    config = JsxNoLeakedRenderConfig,
);

impl Rule for JsxNoLeakedRender {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = value
            .get(0)
            .cloned()
            .map(serde_json::from_value::<JsxNoLeakedRenderConfig>)
            .transpose()?
            .unwrap_or_default();

        Ok(Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXExpressionContainer(container) = node.kind() else { return };

        match &container.expression {
            JSXExpression::LogicalExpression(logical) => {
                self.check_logical_expression(logical, ctx);
            }
            JSXExpression::ConditionalExpression(cond) => {
                self.check_conditional_expression(cond, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl JsxNoLeakedRender {
    /// Returns the first valid strategy to use for fixes
    fn fix_strategy(&self) -> ValidStrategy {
        self.valid_strategies.first().copied().unwrap_or_default()
    }

    /// Check if ternary strategy is allowed
    fn has_ternary_strategy(&self) -> bool {
        self.valid_strategies.contains(&ValidStrategy::Ternary)
    }

    /// Check if coerce strategy is allowed
    fn has_coerce_strategy(&self) -> bool {
        self.valid_strategies.contains(&ValidStrategy::Coerce)
    }

    /// Check LogicalExpression (&&) in JSX
    fn check_logical_expression<'a>(
        &self,
        logical: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if logical.operator != LogicalOperator::And {
            return;
        }

        // Check if React 18+ (empty string is safe in React 18+)
        let react_version = ctx.settings().react.version;
        let is_react_18_or_later = react_version.is_some_and(|v| v.major() >= 18);

        // In ternary-only mode, ALL && usage must be converted to ternary
        // Skip validity checks entirely
        if !self.has_coerce_strategy() {
            ctx.diagnostic_with_fix(
                leaked_render_diagnostic(logical.left.span(), &self.valid_strategies),
                |fixer| {
                    let left_text = ctx.source_range(logical.left.span());
                    let right_text = ctx.source_range(logical.right.span());
                    let condition = trim_double_negation(left_text);
                    fixer.replace(logical.span, format!("{condition} ? {right_text} : null"))
                },
            );
            return;
        }

        // Find invalid parts in the && chain for surgical coercion
        let invalid_parts = self.find_invalid_parts_in_chain_with_react_version(logical, is_react_18_or_later);
        if invalid_parts.is_empty() {
            return;
        }

        // Report on the first invalid part
        let first_invalid = invalid_parts[0];
        ctx.diagnostic_with_fix(
            leaked_render_diagnostic(first_invalid, &self.valid_strategies),
            |fixer| {
                match self.fix_strategy() {
                    ValidStrategy::Ternary => {
                        let left_text = ctx.source_range(logical.left.span());
                        let right_text = ctx.source_range(logical.right.span());
                        let condition = trim_double_negation(left_text);
                        fixer.replace(logical.span, format!("{condition} ? {right_text} : null"))
                    }
                    ValidStrategy::Coerce => {
                        // Surgical fix: only coerce parts that need it
                        let fixed = self.generate_surgical_fix(logical, ctx);
                        fixer.replace(logical.span, fixed)
                    }
                }
            },
        );
    }

    /// Check ConditionalExpression (ternary) in JSX - only in coerce-only mode
    fn check_conditional_expression<'a>(
        &self,
        cond: &ConditionalExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        // If ternary strategy is valid, ternary expressions are allowed
        if self.has_ternary_strategy() {
            return;
        }

        // Check if alternate is a valid render value
        // Original ESLint: isNonNullishLiteral(alternate) || JSX element/fragment
        // Valid alternates: literals that aren't null, OR JSX elements/fragments
        let alt = cond.alternate.without_parentheses();
        let is_valid_alternate = match alt {
            // JSX is always valid
            Expression::JSXElement(_) => true,
            // String, number, boolean, bigint literals are valid (even empty string)
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::BigIntLiteral(_) => true,
            // null is NOT valid
            Expression::NullLiteral(_) => false,
            // undefined identifier is NOT valid
            Expression::Identifier(ident) if ident.name == "undefined" => false,
            // Other identifiers and expressions are NOT valid (not a literal)
            _ => false,
        };

        if is_valid_alternate {
            return;
        }

        // Check for special case: `cond ? false : value`
        let is_false_consequent =
            matches!(&cond.consequent, Expression::BooleanLiteral(lit) if !lit.value);

        ctx.diagnostic_with_fix(
            leaked_render_diagnostic(cond.test.span(), &self.valid_strategies),
            |fixer| {
                let test_text = ctx.source_range(cond.test.span());

                if is_false_consequent {
                    // `cond ? false : value` -> `!cond && value` or coerce test
                    let alternate_text = ctx.source_range(cond.alternate.span());

                    // Check if test is a logical expression that needs coercion
                    if let Expression::LogicalExpression(test_logical) = &cond.test {
                        let coerced = self.generate_surgical_fix_all_parts(test_logical, ctx);
                        return fixer
                            .replace(cond.span, format!("{coerced} ? false : {alternate_text}"));
                    }

                    // Simple negation case
                    fixer.replace(cond.span, format!("!{test_text} && {alternate_text}"))
                } else {
                    // `cond ? value : null` -> `!!cond && value`
                    let consequent_text = ctx.source_range(cond.consequent.span());

                    if let Expression::LogicalExpression(test_logical) = &cond.test {
                        let coerced = self.generate_surgical_fix_all_parts(test_logical, ctx);
                        fixer.replace(cond.span, format!("{coerced} && {consequent_text}"))
                    } else if self.is_valid_left_side(&cond.test) {
                        fixer.replace(cond.span, format!("{test_text} && {consequent_text}"))
                    } else {
                        fixer.replace(cond.span, format!("!!{test_text} && {consequent_text}"))
                    }
                }
            },
        );
    }

    /// Find all invalid expression spans in an && chain (React 18+ aware)
    fn find_invalid_parts_in_chain_with_react_version(
        &self,
        logical: &LogicalExpression,
        is_react_18: bool,
    ) -> Vec<Span> {
        let mut invalid = Vec::new();
        self.collect_invalid_in_logical(logical, &mut invalid, true, is_react_18);
        invalid
    }

    fn collect_invalid_in_logical(
        &self,
        logical: &LogicalExpression,
        invalid: &mut Vec<Span>,
        is_outermost: bool,
        is_react_18: bool,
    ) {
        if logical.operator == LogicalOperator::And {
            // Check left side
            if let Expression::LogicalExpression(left_logical) = logical.left.without_parentheses()
            {
                if left_logical.operator == LogicalOperator::And {
                    self.collect_invalid_in_logical(left_logical, invalid, false, is_react_18);
                } else if !self.is_valid_left_side_with_react_version(&logical.left, is_react_18) {
                    // Left is a LogicalExpression but not && (e.g., ||)
                    invalid.push(logical.left.span());
                }
            } else if !self.is_valid_left_side_with_react_version(&logical.left, is_react_18) {
                invalid.push(logical.left.span());
            }

            // Check right side
            if let Expression::LogicalExpression(right_logical) = logical.right.without_parentheses()
            {
                if right_logical.operator == LogicalOperator::And {
                    // Right is another && chain - recurse
                    self.collect_invalid_in_logical(right_logical, invalid, is_outermost, is_react_18);
                } else if !is_outermost && !self.is_valid_left_side_with_react_version(&logical.right, is_react_18) {
                    // Right is non-&& LogicalExpression and we're nested - check it
                    invalid.push(logical.right.span());
                }
            } else if !is_outermost && !self.is_valid_left_side_with_react_version(&logical.right, is_react_18) {
                // Right is not a LogicalExpression and we're nested - check it
                invalid.push(logical.right.span());
            }
            // When is_outermost is true and right is not a LogicalExpression,
            // that's the final render value - don't check it
        }
    }

    /// Check if expression is valid (React 18+ aware - empty string is safe in React 18+)
    fn is_valid_left_side_with_react_version(&self, expr: &Expression, is_react_18: bool) -> bool {
        // In React 18+, empty string '' is safe (doesn't render)
        if is_react_18 {
            if let Expression::StringLiteral(lit) = expr.without_parentheses() {
                if lit.value.is_empty() {
                    return true;
                }
            }
        }
        self.is_valid_left_side(expr)
    }

    /// Generate surgical fix that only coerces parts that need it (for && && pattern)
    fn generate_surgical_fix<'a>(
        &self,
        logical: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) -> String {
        self.fix_logical_chain(logical, ctx, true)
    }

    /// Generate surgical fix that coerces ALL invalid parts (for ternary test conversion)
    fn generate_surgical_fix_all_parts<'a>(
        &self,
        logical: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) -> String {
        self.fix_logical_chain_all(logical, ctx)
    }

    fn fix_logical_chain<'a>(
        &self,
        logical: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
        is_top_level: bool,
    ) -> String {
        if logical.operator != LogicalOperator::And {
            let text = ctx.source_range(logical.span);
            return text.to_string();
        }

        let left_fixed = match logical.left.without_parentheses() {
            Expression::LogicalExpression(left_logical)
                if left_logical.operator == LogicalOperator::And =>
            {
                self.fix_logical_chain(left_logical, ctx, false)
            }
            _ => {
                let text = ctx.source_range(logical.left.span());
                if self.is_valid_left_side(&logical.left) {
                    text.to_string()
                } else {
                    format!("!!{text}")
                }
            }
        };

        let right_text = ctx.source_range(logical.right.span());

        // For the rightmost part in the top-level chain, don't coerce (it's the render value)
        // But for nested && parts, we need to check validity
        let right_fixed = if is_top_level {
            // Check if right side is another && chain
            match logical.right.without_parentheses() {
                Expression::LogicalExpression(right_logical)
                    if right_logical.operator == LogicalOperator::And =>
                {
                    // Recurse but this becomes top level for the nested chain
                    self.fix_logical_chain(right_logical, ctx, true)
                }
                _ => {
                    // Final render value - keep as-is
                    right_text.to_string()
                }
            }
        } else {
            // Nested: right side is part of the condition chain
            match logical.right.without_parentheses() {
                Expression::LogicalExpression(right_logical)
                    if right_logical.operator == LogicalOperator::And =>
                {
                    self.fix_logical_chain(right_logical, ctx, false)
                }
                _ => {
                    if self.is_valid_left_side(&logical.right) {
                        right_text.to_string()
                    } else {
                        format!("!!{right_text}")
                    }
                }
            }
        };

        format!("{left_fixed} && {right_fixed}")
    }

    /// Fix all parts of a logical chain (for ternary test conversion - coerce everything invalid)
    fn fix_logical_chain_all<'a>(
        &self,
        logical: &LogicalExpression<'a>,
        ctx: &LintContext<'a>,
    ) -> String {
        if logical.operator != LogicalOperator::And {
            let text = ctx.source_range(logical.span);
            return text.to_string();
        }

        let left_fixed = match logical.left.without_parentheses() {
            Expression::LogicalExpression(left_logical) => {
                self.fix_logical_chain_all(left_logical, ctx)
            }
            _ => {
                let text = ctx.source_range(logical.left.span());
                if self.is_valid_left_side(&logical.left) {
                    text.to_string()
                } else {
                    format!("!!{text}")
                }
            }
        };

        let right_fixed = match logical.right.without_parentheses() {
            Expression::LogicalExpression(right_logical) => {
                self.fix_logical_chain_all(right_logical, ctx)
            }
            _ => {
                let text = ctx.source_range(logical.right.span());
                if self.is_valid_left_side(&logical.right) {
                    text.to_string()
                } else {
                    format!("!!{text}")
                }
            }
        };

        format!("{left_fixed} && {right_fixed}")
    }

    /// Check if expression is already coerced to boolean (safe for rendering)
    fn is_valid_left_side(&self, expr: &Expression) -> bool {
        let expr = expr.without_parentheses();

        match expr {
            // UnaryExpression with ! operator (including !!)
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                // If only ternary strategy allowed, coerce (!!) is not valid
                if self.valid_strategies.len() == 1
                    && self.valid_strategies[0] == ValidStrategy::Ternary
                {
                    return false;
                }
                true
            }

            // BinaryExpression with comparison operators - result is always boolean
            Expression::BinaryExpression(binary) => matches!(
                binary.operator,
                BinaryOperator::LessThan
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::LessEqualThan
                    | BinaryOperator::GreaterEqualThan
                    | BinaryOperator::Equality
                    | BinaryOperator::Inequality
                    | BinaryOperator::StrictEquality
                    | BinaryOperator::StrictInequality
                    | BinaryOperator::Instanceof
                    | BinaryOperator::In
            ),

            // CallExpression to Boolean() - must have exactly one argument
            Expression::CallExpression(call) => {
                if call.callee.is_specific_id("Boolean") && call.arguments.len() == 1 {
                    // If only ternary strategy allowed, Boolean() coerce is not valid
                    if self.valid_strategies.len() == 1
                        && self.valid_strategies[0] == ValidStrategy::Ternary
                    {
                        return false;
                    }
                    true
                } else {
                    false
                }
            }

            // Nested LogicalExpression - check if it's a valid pattern
            Expression::LogicalExpression(logical) => {
                self.is_valid_nested_logical_expression(logical)
            }

            // ConditionalExpression (ternary) always returns the evaluated branch
            Expression::ConditionalExpression(_) => true,

            _ => false,
        }
    }

    /// Check nested logical expressions for validity
    /// Original ESLint: recursively checks both sides of ANY logical expression
    fn is_valid_nested_logical_expression(&self, logical: &LogicalExpression) -> bool {
        // Both sides must be valid for the expression to be valid
        // This applies to &&, ||, and ?? operators
        self.is_valid_left_side(&logical.left) && self.is_valid_left_side(&logical.right)
    }
}

/// Trim !! prefix from expression text for cleaner ternary conversion
/// Also removes surrounding parentheses if the !! was wrapping a parenthesized expression
fn trim_double_negation(text: &str) -> &str {
    let trimmed = text.trim_start();
    if let Some(rest) = trimmed.strip_prefix("!!") {
        let rest = rest.trim_start();
        // If the result starts with ( and ends with ), strip them too
        if rest.starts_with('(') && rest.ends_with(')') {
            &rest[1..rest.len() - 1]
        } else {
            rest
        }
    } else {
        text
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"const Component = () => {
  return <div>{customTitle || defaultTitle}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>There are {elements.length} elements</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{!count && 'No results found'}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{!!elements.length && <List elements={elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{Boolean(elements.length) && <List elements={elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length > 0 && <List elements={elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length ? <List elements={elements}/> : null}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{count ? <List elements={elements}/> : null}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{count ? <List elements={elements}/> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{!!count && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{count ? <List elements={elements}/> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{!!count && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r#"const Component = ({ elements, count }) => {
  return (
    <div>
      <div> {direction ? (direction === "down" ? "▼" : "▲") : ""} </div>
      <div>{ containerName.length > 0 ? "Loading several stuff" : "Loading" }</div>
    </div>
  )
}"#,
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r#"const Component = ({ elements, count }) => {
  return <div>{direction ? (direction === "down" ? "▼" : "▲") : ""}</div>
}"#,
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r#"const Component = ({ direction }) => {
  return (
    <div>
      <div>{!!direction && direction === "down" && "▼"}</div>
      <div>{direction === "down" && !!direction && "▼"}</div>
      <div>{direction === "down" || !!direction && "▼"}</div>
      <div>{(!display || display === DISPLAY.WELCOME) && <span>foo</span>}</div>
    </div>
  )
}"#,
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{count ? <List elements={elements}/> : <EmptyList />}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ elements, count }) => {
  return <div>{count ? <List elements={elements}/> : <EmptyList />}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        // Note: Tests for boolean literals (const isOpen = true/false) are not included
        // because variable type inference is not yet implemented. The original ESLint rule
        // checks if a variable is declared as a boolean literal and allows it in && patterns.
    ];

    let fail = vec![
        (
            r"const Example = () => {
  return (
    <>
      {0 && <Something/>}
      {'' && <Something/>}
      {NaN && <Something/>}
    </>
  )
}",
            None,
            Some(serde_json::json!({"settings":{"react":{"version":"17.999.999"}}})),
        ),
        (
            r"const Example = () => {
  return (
    <>
      {0 && <Something/>}
      {'' && <Something/>}
      {NaN && <Something/>}
    </>
  )
}",
            None,
            Some(serde_json::json!({"settings":{"react":{"version":"18.0.0"}}})),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            None,
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ someCondition, title }) => {
  return <div>{!someCondition && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{!!count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count > 0 && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{0 != count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, total, title }) => {
  return <div>{count < total && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title, somethingElse }) => {
  return <div>{!!(count && somethingElse) && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ connection, hasError, hasErrorUpdate}) => {
  return <div>{connection && (hasError || hasErrorUpdate)}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{!count ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ count, somethingElse, title }) => {
  return <div>{count && somethingElse ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const Component = ({ items, somethingElse, title }) => {
  return <div>{items.length > 0 && somethingElse && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  const items = []
  const breakpoint = { phones: true }

  return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  return <Something checked={isIndeterminate ? false : isChecked} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  return <Something checked={cond && isIndeterminate ? false : isChecked} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  return (
    <>
      {someCondition && (
        <div>
          <p>hello</p>
        </div>
      )}
    </>
  )
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const MyComponent = () => {
  return (
    <>
      {someCondition && (
        <SomeComponent
          prop1={val1}
          prop2={val2}
        />
      )}
    </>
  )
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
            None,
        ),
        (
            r"const isOpen = 0;
const Component = () => {
  return <Popover open={isOpen && items.length > 0} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
            None,
        ),
    ];

    let fix = vec![
        (
            r"const Example = () => {
  return (
    <>
      {0 && <Something/>}
      {'' && <Something/>}
      {NaN && <Something/>}
    </>
  )
}",
            r"const Example = () => {
  return (
    <>
      {0 ? <Something/> : null}
      {'' ? <Something/> : null}
      {NaN ? <Something/> : null}
    </>
  )
}",
            None,
        ),
        // Note: React 18 fix test (empty string unchanged) removed because fix tests
        // don't support settings yet. The fail test with settings still validates detection.
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{count ? title : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            r"const Component = ({ count }) => {
  return <div>{count ? <span>There are {count} results</span> : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{elements.length ? <List elements={elements}/> : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{elements[0] ? <List elements={elements}/> : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
}",
            None,
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            r"const Component = ({ numberA, numberB }) => {
  return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{count ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            r"const Component = ({ count }) => {
  return <div>{count ? <span>There are {count} results</span> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{elements.length ? <List elements={elements}/> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length ? <List elements={nestedCollection.elements}/> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{elements[0] ? <List elements={elements}/> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) ? <Results>{numberA+numberB}</Results> : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ someCondition, title }) => {
  return <div>{!someCondition && title}</div>
}",
            r"const Component = ({ someCondition, title }) => {
  return <div>{!someCondition ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{!!count && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{count ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count > 0 && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{count > 0 ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{0 != count && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{0 != count ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, total, title }) => {
  return <div>{count < total && title}</div>
}",
            r"const Component = ({ count, total, title }) => {
  return <div>{count < total ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, title, somethingElse }) => {
  return <div>{!!(count && somethingElse) && title}</div>
}",
            r"const Component = ({ count, title, somethingElse }) => {
  return <div>{count && somethingElse ? title : null}</div>
}",
            Some(serde_json::json!([{"validStrategies":["ternary"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count && title}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{!!count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ count }) => {
  return <div>{count && <span>There are {count} results</span>}</div>
}",
            r"const Component = ({ count }) => {
  return <div>{!!count && <span>There are {count} results</span>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements.length && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{!!elements.length && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ nestedCollection }) => {
  return <div>{nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            r"const Component = ({ nestedCollection }) => {
  return <div>{!!nestedCollection.elements.length && <List elements={nestedCollection.elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ elements }) => {
  return <div>{elements[0] && <List elements={elements}/>}</div>
}",
            r"const Component = ({ elements }) => {
  return <div>{!!elements[0] && <List elements={elements}/>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ numberA, numberB }) => {
  return <div>{(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            r"const Component = ({ numberA, numberB }) => {
  return <div>{!!(numberA || numberB) && <Results>{numberA+numberB}</Results>}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ connection, hasError, hasErrorUpdate}) => {
  return <div>{connection && (hasError || hasErrorUpdate)}</div>
}",
            r"const Component = ({ connection, hasError, hasErrorUpdate}) => {
  return <div>{!!connection && (hasError || hasErrorUpdate)}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{count ? title : null}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{!!count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ count, title }) => {
  return <div>{!count ? title : null}</div>
}",
            r"const Component = ({ count, title }) => {
  return <div>{!count && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ count, somethingElse, title }) => {
  return <div>{count && somethingElse ? title : null}</div>
}",
            r"const Component = ({ count, somethingElse, title }) => {
  return <div>{!!count && !!somethingElse && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const Component = ({ items, somethingElse, title }) => {
  return <div>{items.length > 0 && somethingElse && title}</div>
}",
            r"const Component = ({ items, somethingElse, title }) => {
  return <div>{items.length > 0 && !!somethingElse && title}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const MyComponent = () => {
  const items = []
  const breakpoint = { phones: true }

  return <div>{items.length > 0 && breakpoint.phones && <span />}</div>
}",
            r"const MyComponent = () => {
  const items = []
  const breakpoint = { phones: true }

  return <div>{items.length > 0 && !!breakpoint.phones && <span />}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
        ),
        (
            r"const MyComponent = () => {
  return <div>{maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
}",
            r"const MyComponent = () => {
  return <div>{!!maybeObject && (isFoo ? <Aaa /> : <Bbb />)}</div>
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const MyComponent = () => {
  return <Something checked={isIndeterminate ? false : isChecked} />
}",
            r"const MyComponent = () => {
  return <Something checked={!isIndeterminate && isChecked} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const MyComponent = () => {
  return <Something checked={cond && isIndeterminate ? false : isChecked} />
}",
            r"const MyComponent = () => {
  return <Something checked={!!cond && !!isIndeterminate ? false : isChecked} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
        (
            r"const MyComponent = () => {
  return (
    <>
      {someCondition && (
        <div>
          <p>hello</p>
        </div>
      )}
    </>
  )
}",
            r"const MyComponent = () => {
  return (
    <>
      {!!someCondition && (
        <div>
          <p>hello</p>
        </div>
      )}
    </>
  )
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
        ),
        (
            r"const MyComponent = () => {
  return (
    <>
      {someCondition && (
        <SomeComponent
          prop1={val1}
          prop2={val2}
        />
      )}
    </>
  )
}",
            r"const MyComponent = () => {
  return (
    <>
      {!!someCondition && (
        <SomeComponent
          prop1={val1}
          prop2={val2}
        />
      )}
    </>
  )
}",
            Some(serde_json::json!([{"validStrategies":["coerce","ternary"]}])),
        ),
        (
            r"const isOpen = 0;
const Component = () => {
  return <Popover open={isOpen && items.length > 0} />
}",
            r"const isOpen = 0;
const Component = () => {
  return <Popover open={!!isOpen && items.length > 0} />
}",
            Some(serde_json::json!([{"validStrategies":["coerce"]}])),
        ),
    ];

    Tester::new(JsxNoLeakedRender::NAME, JsxNoLeakedRender::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
