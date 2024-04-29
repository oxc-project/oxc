use oxc_ast::{ast::Expression, AstKind, CommentKind};
use oxc_semantic::AstNodeId;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::LintContext;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(StringValue),
    Unknown,
}

// We only care if it is falsy value (empty string).
#[derive(Copy, Clone, Debug, PartialEq)]
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
            Value::String(str) => Some(!matches!(str, StringValue::Empty)),
        }
    }
}

pub fn get_write_expr<'a, 'b>(
    node_id: AstNodeId,
    ctx: &'b LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let parent = ctx.nodes().parent_node(node_id)?;
    match parent.kind() {
        AstKind::SimpleAssignmentTarget(_) | AstKind::AssignmentTarget(_) => {
            get_write_expr(parent.id(), ctx)
        }
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

const TREE_SHAKING_COMMENT_ID: &str = "tree-shaking";
const COMMENT_NO_SIDE_EFFECT_WHEN_CALLED: &str = "no-side-effects-when-called";

fn is_tree_shaking_comment(comment: &str) -> bool {
    comment.trim_start().starts_with(TREE_SHAKING_COMMENT_ID)
}

/// check if the `span` has a leading comment for opening side effect check.
/// e.g. `export default /* tree-shaking no-side-effects-when-called */ ext`
pub fn has_comment_about_side_effect_check(span: Span, ctx: &LintContext) -> bool {
    get_leading_tree_shaking_comment(span, ctx)
        .is_some_and(|comment| comment.contains(COMMENT_NO_SIDE_EFFECT_WHEN_CALLED))
}

/// Get the nearest comment before the `span`, return `None` if no leading comment is founded.
///
///  # Examples
/// ```javascript
/// /* valid comment for `a`  */ let a = 1;
///
/// // valid comment for `b`
/// let b = 1;
///
/// // valid comment for `c`
///
///
/// let c = 1;
///
/// let d = 1; /* invalid comment for `e` */
/// let e = 2
/// ```
pub fn get_leading_tree_shaking_comment<'a>(span: Span, ctx: &LintContext<'a>) -> Option<&'a str> {
    let (start, comment) = ctx.semantic().trivias().comments_range(..span.start).next_back()?;

    let comment_text = {
        let span = Span::new(*start, comment.end);
        span.source_text(ctx.source_text())
    };

    if !is_tree_shaking_comment(comment_text) {
        return None;
    }

    // If there are non-whitespace characters between the `comment`` and the `span`,
    // we treat the `comment` not belongs to the `span`.
    let only_whitespace = ctx.source_text()[comment.end as usize..span.start as usize]
        .strip_prefix("*/") // for multi-line comment
        .is_some_and(|s| s.trim().is_empty());

    if !only_whitespace {
        return None;
    }

    // Next step, we need make sure it's not the trailing comment of the previous line.
    let mut current_line_start = span.start as usize;
    for c in ctx.source_text()[..span.start as usize].chars().rev() {
        if c == '\n' {
            break;
        }

        current_line_start -= c.len_utf8();
    }
    let Ok(current_line_start) = u32::try_from(current_line_start) else {
        return None;
    };

    if comment.end < current_line_start {
        let previous_line =
            ctx.source_text()[..comment.end as usize].lines().next_back().unwrap_or("");
        let nothing_before_comment = previous_line
            .trim()
            .strip_prefix(if comment.kind == CommentKind::SingleLine { "//" } else { "/*" })
            .is_some_and(|s| s.trim().is_empty());
        if !nothing_before_comment {
            return None;
        }
    }

    Some(comment_text)
}

/// Port from <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/463fa1f0bef7caa2b231a38b9c3557051f506c92/src/rules/no-side-effects-in-initialization.ts#L136-L161>
/// <https://tc39.es/ecma262/#sec-evaluatestringornumericbinaryexpression>
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
        // <https://tc39.es/ecma262/#sec-islessthan>
        #[allow(clippy::single_match)]
        BinaryOperator::LessThan => match (left, right) {
            // <https://tc39.es/ecma262/#sec-numeric-types-number-lessThan>
            (Value::Unknown, Value::Number(_)) | (Value::Number(_), Value::Unknown) => {
                Value::Boolean(false)
            }
            (Value::Number(a), Value::Number(b)) => Value::Boolean(a < b),
            _ => Value::Unknown,
        },
        _ => Value::Unknown,
    }
}

#[test]
fn test_calculate_binary_operation() {
    use oxc_syntax::operator::BinaryOperator;

    let fun = calculate_binary_operation;

    // "+"
    let op = BinaryOperator::Addition;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(3.0));
    assert_eq!(
        fun(op, Value::String(StringValue::Empty), Value::String(StringValue::Empty)),
        Value::String(StringValue::Empty)
    );
    assert_eq!(
        fun(op, Value::String(StringValue::Empty), Value::String(StringValue::NonEmpty)),
        Value::String(StringValue::NonEmpty)
    );

    assert_eq!(
        fun(op, Value::String(StringValue::NonEmpty), Value::String(StringValue::NonEmpty)),
        Value::String(StringValue::NonEmpty)
    );

    // "-"
    let op = BinaryOperator::Subtraction;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(-1.0));

    // "<"
    let op = BinaryOperator::LessThan;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Unknown, Value::Number(2.0),), Value::Boolean(false));
}
