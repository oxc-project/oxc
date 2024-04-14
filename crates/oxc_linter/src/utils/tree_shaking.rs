use oxc_ast::{ast::Expression, AstKind};
use oxc_semantic::AstNodeId;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::LintContext;

#[derive(Copy, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(StringValue),
    Unknown,
}

// We only care if it is falsy value (empty string).
#[derive(Copy, Clone)]
pub enum StringValue {
    Empty,
    NonEmpty,
}

impl Value {
    pub fn new(expr: &Expression) -> Value {
        match expr {
            Expression::BooleanLiteral(bool_lit) => Value::Boolean(bool_lit.value),
            Expression::NumericLiteral(num_lit) => Value::Number(num_lit.value),
            Expression::StringLiteral(str_lit) => {
                if str_lit.value.is_empty() {
                    Value::String(StringValue::Empty)
                } else {
                    Value::String(StringValue::NonEmpty)
                }
            }
            Expression::TemplateLiteral(template_lit) => {
                if !template_lit.is_no_substitution_template() {
                    Value::Unknown
                } else if template_lit.quasi().is_some_and(|s| s == "") {
                    Value::String(StringValue::Empty)
                } else {
                    Value::String(StringValue::NonEmpty)
                }
            }
            _ => Value::Unknown,
        }
    }
    pub fn get_falsy_value(&self) -> Option<bool> {
        match &self {
            Value::Unknown => None,
            Value::Boolean(boolean) => Some(!*boolean),
            Value::Number(num) => Some(*num == 0.0),
            Value::String(str) => Some(matches!(str, StringValue::Empty)),
        }
    }
}

pub fn get_write_expr<'a, 'b>(
    node_id: AstNodeId,
    ctx: &'b LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let parent = ctx.nodes().parent_kind(node_id)?;
    match parent {
        AstKind::AssignmentExpression(assign_expr) => Some(&assign_expr.right),
        _ => None,
    }
}

pub fn no_effects() {}

/// Comments containing @__PURE__ or #__PURE__ mark a specific function call
/// or constructor invocation as side effect free.
///
/// Such an annotation is considered valid if it directly
/// precedes a function call or constructor invocation
/// and is only separated from the callee by white-space or comments.
///
/// The only exception are parentheses that wrap a call or invocation.
///
/// <https://rollupjs.org/configuration-options/#pure>
pub fn has_pure_notation(span: Span, ctx: &LintContext) -> bool {
    let Some((start, comment)) = ctx.semantic().trivias().comments_range(..span.start).next_back()
    else {
        return false;
    };
    let span = Span::new(*start, comment.end);
    let raw = span.source_text(ctx.semantic().source_text());

    raw.contains("@__PURE__") || raw.contains("#__PURE__")
}

/// Port from <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/463fa1f0bef7caa2b231a38b9c3557051f506c92/src/rules/no-side-effects-in-initialization.ts#L136-L161>
pub fn calculate_binary_operation(op: BinaryOperator, left: Value, right: Value) -> Value {
    match op {
        BinaryOperator::Addition => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(str1), Value::String(str2)) => {
                if matches!(str1, StringValue::Empty) && matches!(str2, StringValue::Empty) {
                    Value::String(StringValue::Empty)
                } else {
                    Value::String(StringValue::NonEmpty)
                }
            }
            _ => Value::Unknown,
        },
        BinaryOperator::Subtraction => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => Value::Unknown,
        },
        _ => Value::Unknown,
    }
}
