use std::cell::{Cell, RefCell};

use lazy_static::lazy_static;
use oxc_ast::{
    ast::{Expression, IdentifierReference, StaticMemberExpression},
    AstKind, CommentKind,
};
use oxc_semantic::{AstNode, NodeId, SymbolId};
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator, UnaryOperator};
use rustc_hash::FxHashSet;

use crate::LintContext;

mod pure_functions;

pub struct NodeListenerOptions<'a, 'b> {
    pub checked_mutated_nodes: RefCell<FxHashSet<SymbolId>>,
    pub checked_called_nodes: RefCell<FxHashSet<SymbolId>>,
    pub ctx: &'b LintContext<'a>,
    pub has_valid_this: Cell<bool>,
    pub called_with_new: Cell<bool>,
    pub whitelist_modules: Option<&'b Vec<WhitelistModule>>,
    pub whitelist_functions: Option<&'b Vec<String>>,
}

impl<'a, 'b> NodeListenerOptions<'a, 'b> {
    pub fn new(ctx: &'b LintContext<'a>) -> Self {
        Self {
            checked_mutated_nodes: RefCell::new(FxHashSet::default()),
            checked_called_nodes: RefCell::new(FxHashSet::default()),
            ctx,
            has_valid_this: Cell::new(false),
            called_with_new: Cell::new(false),
            whitelist_modules: None,
            whitelist_functions: None,
        }
    }

    pub fn with_whitelist_modules(self, whitelist_modules: &'b Vec<WhitelistModule>) -> Self {
        Self { whitelist_modules: Some(whitelist_modules), ..self }
    }

    pub fn with_whitelist_functions(self, whitelist_functions: &'b Vec<String>) -> Self {
        Self { whitelist_functions: Some(whitelist_functions), ..self }
    }

    pub fn insert_mutated_node(&self, symbol_id: SymbolId) -> bool {
        self.checked_mutated_nodes.borrow_mut().insert(symbol_id)
    }

    pub fn insert_called_node(&self, symbol_id: SymbolId) -> bool {
        self.checked_called_nodes.borrow_mut().insert(symbol_id)
    }
}

#[derive(Debug, Default, Clone)]
pub struct WhitelistModule {
    pub name: String,
    pub functions: ModuleFunctions,
}

#[derive(Debug, Clone, Default)]
pub enum ModuleFunctions {
    #[default]
    All,
    Specific(Vec<String>),
}

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

    /// If the value is a boolean, return the negation of the boolean, otherwise return `None`.
    pub fn neg_bool(&self) -> Option<Value> {
        match self {
            Value::Boolean(boolean) => Some(Value::Boolean(!*boolean)),
            _ => None,
        }
    }

    pub fn to_bool(self) -> Option<bool> {
        match self {
            Value::Boolean(boolean) => Some(boolean),
            Value::Number(num) => Some(num != 0.0),
            Value::String(str) => Some(!matches!(str, StringValue::Empty)),
            Value::Unknown => None,
        }
    }
}

pub fn get_write_expr<'a, 'b>(
    node_id: NodeId,
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

lazy_static! {
    static ref PURE_FUNCTIONS_SET: FxHashSet<&'static str> = {
        let mut set = FxHashSet::default();
        set.extend(pure_functions::PURE_FUNCTIONS);

        set
    };
}

pub enum FunctionName<'a> {
    Identifier(&'a IdentifierReference<'a>),
    StaticMemberExpr(&'a StaticMemberExpression<'a>),
}

impl<'a> FunctionName<'a> {
    fn from_expression(expr: &'a Expression<'a>) -> Option<Self> {
        match expr {
            Expression::Identifier(ident) => Some(FunctionName::Identifier(ident)),
            Expression::StaticMemberExpression(member_expr) => {
                Some(FunctionName::StaticMemberExpr(member_expr))
            }
            _ => None,
        }
    }
}

impl GetSpan for FunctionName<'_> {
    fn span(&self) -> Span {
        match self {
            FunctionName::Identifier(ident) => ident.span,
            FunctionName::StaticMemberExpr(member_expr) => member_expr.span,
        }
    }
}

pub fn is_pure_function(function_name: &FunctionName, options: &NodeListenerOptions) -> bool {
    if has_pure_notation(function_name.span(), options.ctx) {
        return true;
    }
    let name = flatten_member_expr_if_possible(function_name);

    if options.whitelist_functions.is_some_and(|whitelist| whitelist.contains(&name.to_string())) {
        return true;
    }

    PURE_FUNCTIONS_SET.contains(name.as_str())
}

fn flatten_member_expr_if_possible(function_name: &FunctionName) -> CompactStr {
    match function_name {
        FunctionName::StaticMemberExpr(static_member_expr) => {
            let Some(parent_name) = FunctionName::from_expression(&static_member_expr.object)
            else {
                return CompactStr::from("");
            };
            let flattened_parent = flatten_member_expr_if_possible(&parent_name);
            CompactStr::from(format!("{}.{}", flattened_parent, static_member_expr.property.name))
        }
        FunctionName::Identifier(ident) => ident.name.to_compact_str(),
    }
}

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
    let Some(comment) = ctx.semantic().comments_range(..span.start).next_back() else {
        return false;
    };
    let raw = comment.span.source_text(ctx.semantic().source_text());

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
    let comment = ctx.semantic().comments_range(..span.start).next_back()?;

    let comment_text = comment.span.source_text(ctx.source_text());

    if !is_tree_shaking_comment(comment_text) {
        return None;
    }

    // If there are non-whitespace characters between the `comment`` and the `span`,
    // we treat the `comment` not belongs to the `span`.
    let only_whitespace = ctx.source_text()[comment.span.end as usize..span.start as usize]
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

    if comment.span.end < current_line_start {
        let previous_line =
            ctx.source_text()[..comment.span.end as usize].lines().next_back().unwrap_or("");
        let nothing_before_comment = previous_line
            .trim()
            .strip_prefix(if comment.kind == CommentKind::Line { "//" } else { "/*" })
            .is_some_and(|s| s.trim().is_empty());
        if !nothing_before_comment {
            return None;
        }
    }

    Some(comment_text)
}

pub fn is_local_variable_a_whitelisted_module(
    node: &AstNode,
    name: &str,
    options: &NodeListenerOptions,
) -> bool {
    let Some(AstKind::ImportDeclaration(parent)) = options.ctx.nodes().parent_kind(node.id())
    else {
        return false;
    };
    let module_name = parent.source.value.as_str();
    is_function_side_effect_free(name, module_name, options)
}

pub fn is_function_side_effect_free(
    name: &str,
    module_name: &str,
    options: &NodeListenerOptions,
) -> bool {
    let Some(whitelist_modules) = options.whitelist_modules else {
        return false;
    };
    for module in whitelist_modules {
        let is_module_match =
            module.name == module_name || module.name == "#local" && module_name.starts_with('.');

        if is_module_match {
            match &module.functions {
                ModuleFunctions::All => return true,
                ModuleFunctions::Specific(functions) => {
                    return functions.contains(&name.to_string());
                }
            }
        }
    }

    false
}

/// Port from <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/463fa1f0bef7caa2b231a38b9c3557051f506c92/src/rules/no-side-effects-in-initialization.ts#L136-L161>
/// <https://tc39.es/ecma262/#sec-evaluatestringornumericbinaryexpression>
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
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
        BinaryOperator::Multiplication => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => Value::Unknown,
        },
        BinaryOperator::Division => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => Value::Unknown,
        },
        // <https://tc39.es/ecma262/#sec-islessthan>
        BinaryOperator::LessThan => match (left, right) {
            // <https://tc39.es/ecma262/#sec-numeric-types-number-lessThan>
            (Value::Unknown, Value::Number(_)) | (Value::Number(_), Value::Unknown) => {
                Value::Boolean(false)
            }
            (Value::Number(a), Value::Number(b)) => Value::Boolean(a < b),
            _ => Value::Unknown,
        },
        BinaryOperator::GreaterEqualThan => {
            calculate_binary_operation(BinaryOperator::LessThan, left, right)
                .neg_bool()
                .unwrap_or(Value::Unknown)
        }
        BinaryOperator::Equality => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean((a - b).abs() < f64::EPSILON),
            _ => Value::Unknown,
        },
        BinaryOperator::Inequality => {
            calculate_binary_operation(BinaryOperator::Equality, left, right)
                .neg_bool()
                .unwrap_or(Value::Unknown)
        }
        BinaryOperator::StrictEquality => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean((a - b).abs() < f64::EPSILON),
            (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(a == b),
            _ => Value::Unknown,
        },
        BinaryOperator::StrictInequality => {
            calculate_binary_operation(BinaryOperator::StrictEquality, left, right)
                .neg_bool()
                .unwrap_or(Value::Unknown)
        }
        BinaryOperator::LessEqualThan => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Boolean(a <= b),
            _ => Value::Unknown,
        },
        BinaryOperator::GreaterThan => {
            calculate_binary_operation(BinaryOperator::LessEqualThan, left, right)
                .neg_bool()
                .unwrap_or(Value::Unknown)
        }

        BinaryOperator::ShiftRightZeroFill => match (left, right) {
            // <https://tc39.es/ecma262/#sec-numeric-types-number-unsignedRightShift>
            (Value::Number(a), Value::Number(b)) => {
                // Casting between two integers of the same size (e.g. i32 -> u32) is a no-op
                let a = a as i32;
                let b = b as i32;
                // 1. Let lNum be ! ToUint32(x).
                let l_num = a as u32;
                // 2. Let rNum be ! ToUint32(y).
                let r_num = b as u32;
                // 3. Let shiftCount be ℝ(rNum) modulo 32.
                let shift_count = r_num % 32;
                // 4. Return the result of performing a zero-filling right shift of lNum by shiftCount bits.
                Value::Number(f64::from(l_num >> shift_count))
            }
            _ => Value::Unknown,
        },
        BinaryOperator::Remainder => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
            _ => Value::Unknown,
        },
        BinaryOperator::BitwiseOR => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(f64::from(a as i32 | b as i32)),
            _ => Value::Unknown,
        },
        BinaryOperator::BitwiseXOR => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(f64::from(a as i32 ^ b as i32)),
            _ => Value::Unknown,
        },
        BinaryOperator::BitwiseAnd => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(f64::from(a as i32 & b as i32)),
            _ => Value::Unknown,
        },
        BinaryOperator::Exponential => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),
            _ => Value::Unknown,
        },
        BinaryOperator::ShiftLeft => match (left, right) {
            // <https://tc39.es/ecma262/#sec-numeric-types-number-leftShift>
            (Value::Number(a), Value::Number(b)) => {
                // 1. Let lNum be ! ToInt32(x).
                let l_num = a as i32;
                // 2. Let rNum be ! ToUint32(y).
                let r_num = b as i32;
                let r_num = r_num as u32;
                // 3. Let shiftCount be ℝ(rNum) modulo 32.
                let shift_count = r_num % 32;
                // 4. Return the result of left shifting lNum by shiftCount bits.
                Value::Number(f64::from(l_num << shift_count))
            }
            _ => Value::Unknown,
        },
        BinaryOperator::ShiftRight => match (left, right) {
            // <https://tc39.es/ecma262/#sec-numeric-types-number-signedRightShift>
            (Value::Number(a), Value::Number(b)) => {
                // 1. Let lNum be ! ToInt32(x).
                let l_num = a as i32;
                // 2. Let rNum be ! ToUint32(y).
                let r_num = b as i32;
                let r_num = r_num as u32;
                // 3. Let shiftCount be ℝ(rNum) modulo 32.
                let shift_count = r_num % 32;
                // 4. Return the result of performing a sign-extending right shift of lNum by shiftCount bits. The most significant bit is propagated. The mathematical value of the result is exactly representable as a 32-bit two's complement bit string.i
                Value::Number(f64::from(l_num >> shift_count))
            }
            _ => Value::Unknown,
        },
        BinaryOperator::In | BinaryOperator::Instanceof => Value::Unknown,
    }
}

/// <https://tc39.es/ecma262/#sec-binary-logical-operators-runtime-semantics-evaluation>
pub fn calculate_logical_operation(op: LogicalOperator, left: Value, right: Value) -> Value {
    match op {
        LogicalOperator::And => {
            let left = left.to_bool();
            let right = right.to_bool();

            match (left, right) {
                (Some(false), _) | (_, Some(false)) => Value::Boolean(false),
                (Some(true), Some(true)) => Value::Boolean(true),
                _ => Value::Unknown,
            }
        }
        LogicalOperator::Or => {
            let left = left.to_bool();
            let right = right.to_bool();

            match (left, right) {
                (Some(true), _) | (_, Some(true)) => Value::Boolean(true),
                (Some(false), Some(false)) => Value::Boolean(false),
                _ => Value::Unknown,
            }
        }
        LogicalOperator::Coalesce => Value::Unknown,
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn calculate_unary_operation(op: UnaryOperator, value: Value) -> Value {
    match op {
        UnaryOperator::UnaryNegation => match value {
            Value::Number(num) => Value::Number(-num),
            _ => Value::Unknown,
        },
        UnaryOperator::UnaryPlus => match value {
            Value::Number(num) => Value::Number(num),
            _ => Value::Unknown,
        },
        UnaryOperator::LogicalNot => match value {
            Value::Boolean(boolean) => Value::Boolean(!boolean),
            _ => Value::Unknown,
        },
        UnaryOperator::BitwiseNot => match value {
            Value::Number(num) => Value::Number(f64::from(!(num as i32))),
            _ => Value::Unknown,
        },
        UnaryOperator::Typeof => Value::String(StringValue::NonEmpty),
        UnaryOperator::Void | UnaryOperator::Delete => Value::Unknown,
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

    // "*",
    let op = BinaryOperator::Multiplication;
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(2.0),), Value::Number(8.0));

    // "/",
    let op = BinaryOperator::Division;
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(2.0),), Value::Number(2.0));

    // "<"
    let op = BinaryOperator::LessThan;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Unknown, Value::Number(2.0),), Value::Boolean(false));

    // ">=",
    let op = BinaryOperator::GreaterEqualThan;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(2.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Number(3.0), Value::Number(2.0),), Value::Boolean(true));

    // "==",
    let op = BinaryOperator::Equality;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(1.0),), Value::Boolean(true));

    // "!=",
    let op = BinaryOperator::Inequality;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(1.0),), Value::Boolean(false));

    // "===",
    let op = BinaryOperator::StrictEquality;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(1.0),), Value::Boolean(true));
    // "!==",
    let op = BinaryOperator::StrictInequality;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(1.0),), Value::Boolean(false));

    // "<=",
    let op = BinaryOperator::LessEqualThan;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Number(2.0), Value::Number(2.0),), Value::Boolean(true));
    assert_eq!(fun(op, Value::Number(3.0), Value::Number(2.0),), Value::Boolean(false));

    // ">",
    let op = BinaryOperator::GreaterThan;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(2.0), Value::Number(2.0),), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(3.0), Value::Number(2.0),), Value::Boolean(true));

    // "<<",
    let op = BinaryOperator::ShiftLeft;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(0.0),), Value::Number(1.0));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(4.0));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(31.0),), Value::Number(-2_147_483_648.0));
    assert_eq!(
        fun(op, Value::Number(1.0), Value::Number(f64::MAX),),
        Value::Number(-2_147_483_648.0)
    );
    assert_eq!(fun(op, Value::Number(-5.0), Value::Number(2.0),), Value::Number(-20.0));

    // ">>",
    let op = BinaryOperator::ShiftRight;
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(2.0),), Value::Number(1.0));
    // issue: https://github.com/oxc-project/oxc/issues/4914
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(49.0)), Value::Number(0.0));
    assert_eq!(fun(op, Value::Number(-5.0), Value::Number(2.0),), Value::Number(-2.0));

    // ">>>",
    let op = BinaryOperator::ShiftRightZeroFill;
    assert_eq!(fun(op, Value::Number(0.0), Value::Number(0.0),), Value::Number(0.0));
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(0.0),), Value::Number(1.0));
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(2.0),), Value::Number(1.0));
    assert_eq!(fun(op, Value::Number(2.0), Value::Number(4.0),), Value::Number(0.0));
    assert_eq!(fun(op, Value::Number(4096.0), Value::Number(4096.0)), Value::Number(4096.0));
    assert_eq!(fun(op, Value::Number(4096.0), Value::Number(1024.0)), Value::Number(4096.0));
    assert_eq!(fun(op, Value::Number(4096.0), Value::Number(33.0)), Value::Number(2048.0));
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(49.0)), Value::Number(0.0));
    assert_eq!(fun(op, Value::Number(-5.0), Value::Number(2.0)), Value::Number(1_073_741_822.0));

    // "%",
    let op = BinaryOperator::Remainder;
    assert_eq!(fun(op, Value::Number(4.0), Value::Number(2.0),), Value::Number(0.0));

    // "|",
    let op = BinaryOperator::BitwiseOR;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(3.0));

    // "^",
    let op = BinaryOperator::BitwiseXOR;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(3.0));

    // "&",
    let op = BinaryOperator::BitwiseAnd;
    assert_eq!(fun(op, Value::Number(1.0), Value::Number(2.0),), Value::Number(0.0));

    // "in",
    let op = BinaryOperator::In;
    assert_eq!(fun(op, Value::Unknown, Value::Number(2.0),), Value::Unknown);

    // "instanceof",
    let op = BinaryOperator::Instanceof;
    assert_eq!(fun(op, Value::Unknown, Value::Number(2.0),), Value::Unknown);

    // "**",
    let op = BinaryOperator::Exponential;
    assert_eq!(fun(op, Value::Number(2.0), Value::Number(3.0),), Value::Number(8.0));
}

#[test]
fn test_logical_operation() {
    use oxc_syntax::operator::LogicalOperator;

    let fun = calculate_logical_operation;

    // "&&"
    let op = LogicalOperator::And;
    assert_eq!(fun(op, Value::Boolean(true), Value::Boolean(true)), Value::Boolean(true));
    assert_eq!(fun(op, Value::Boolean(true), Value::Boolean(false)), Value::Boolean(false));
    assert_eq!(fun(op, Value::Boolean(false), Value::Boolean(true)), Value::Boolean(false));
    assert_eq!(fun(op, Value::Boolean(false), Value::Boolean(false)), Value::Boolean(false));
    assert_eq!(fun(op, Value::Unknown, Value::Boolean(true)), Value::Unknown);
    assert_eq!(fun(op, Value::Boolean(true), Value::Unknown), Value::Unknown);
    assert_eq!(fun(op, Value::Unknown, Value::Unknown), Value::Unknown);

    // "||"
    let op = LogicalOperator::Or;
    assert_eq!(fun(op, Value::Boolean(true), Value::Boolean(true)), Value::Boolean(true));
    assert_eq!(fun(op, Value::Boolean(true), Value::Boolean(false)), Value::Boolean(true));
    assert_eq!(fun(op, Value::Boolean(false), Value::Boolean(true)), Value::Boolean(true));
    assert_eq!(fun(op, Value::Boolean(false), Value::Boolean(false)), Value::Boolean(false));
    assert_eq!(fun(op, Value::Unknown, Value::Boolean(true)), Value::Boolean(true));
    assert_eq!(fun(op, Value::Boolean(true), Value::Unknown), Value::Boolean(true));
    assert_eq!(fun(op, Value::Unknown, Value::Unknown), Value::Unknown);

    // "??"
    let op = LogicalOperator::Coalesce;
    assert_eq!(fun(op, Value::Boolean(true), Value::Boolean(true)), Value::Unknown);
}

#[test]
fn test_unary_operation() {
    use oxc_syntax::operator::UnaryOperator;

    let fun = calculate_unary_operation;

    // "-"
    let op = UnaryOperator::UnaryNegation;
    assert_eq!(fun(op, Value::Number(1.0)), Value::Number(-1.0));
    assert_eq!(fun(op, Value::Boolean(true)), Value::Unknown);

    // "+"
    let op = UnaryOperator::UnaryPlus;
    assert_eq!(fun(op, Value::Number(1.0)), Value::Number(1.0));
    assert_eq!(fun(op, Value::Boolean(true)), Value::Unknown);

    // "!"
    let op = UnaryOperator::LogicalNot;
    assert_eq!(fun(op, Value::Boolean(true)), Value::Boolean(false));
    assert_eq!(fun(op, Value::Number(1.0)), Value::Unknown);

    // "~"
    let op = UnaryOperator::BitwiseNot;
    assert_eq!(fun(op, Value::Number(1.0)), Value::Number(-2.0));
    assert_eq!(fun(op, Value::Boolean(true)), Value::Unknown);

    // "typeof"
    let op = UnaryOperator::Typeof;
    assert_eq!(fun(op, Value::Number(1.0)), Value::String(StringValue::NonEmpty));
    assert_eq!(fun(op, Value::Boolean(true)), Value::String(StringValue::NonEmpty));
}
