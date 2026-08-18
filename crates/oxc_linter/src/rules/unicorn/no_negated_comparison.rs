use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression, UnaryExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_identifier_part,
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
    precedence::{GetPrecedence, Precedence},
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, outermost_paren_parent},
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_negated_comparison_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated comparison.")
        .with_help("Use the opposite comparison operator instead of negating the whole comparison.")
        .with_label(span)
}

fn no_negated_logical_comparison_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated logical expression of comparisons.")
        .with_help(
            "Apply De Morgan's law: negate each comparison and swap `&&`/`||` instead of negating the whole expression.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoNegatedComparison {
    /// Also check logical (`&&`/`||`) expressions that only contain equality comparisons.
    check_logical_expressions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow negating the result of an equality comparison, such as `!(a === b)`.
    ///
    /// ### Why is this bad?
    ///
    /// It is easier to read the opposite comparison operator (`!==` instead of
    /// `!(... === ...)`) than to negate the whole comparison.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (!(a === b)) {}
    /// const isDifferent = !(a === b);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (a !== b) {}
    /// const isDifferent = a !== b;
    /// ```
    ///
    /// Relational comparisons (`<`, `<=`, `>`, `>=`) are never checked, since the
    /// opposite operator is not equivalent when an operand is not a comparable
    /// number (e.g. `NaN`):
    ///
    /// ```js
    /// // Not flagged: `!(a >= b)` also catches the case where `a` is `NaN`.
    /// if (!(a >= b)) {}
    /// ```
    ///
    /// ### Options
    ///
    /// #### checkLogicalExpressions
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// When `true`, also reports negated logical (`&&`/`||`) expressions that only
    /// contain equality comparisons, applying De Morgan's laws.
    ///
    /// Example of **incorrect** code for this rule with `checkLogicalExpressions: true`:
    /// ```js
    /// const isDifferent = !(a === b && c === d);
    /// ```
    ///
    /// Example of **correct** code for this rule with `checkLogicalExpressions: true`:
    /// ```js
    /// const isDifferent = a !== b || c !== d;
    /// ```
    NoNegatedComparison,
    unicorn,
    pedantic,
    conditional_fix,
    config = NoNegatedComparison,
    version = "next",
    short_description = "Disallow negated comparisons.",
);

impl Rule for NoNegatedComparison {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(unary) = node.kind() else {
            return;
        };

        if unary.operator != UnaryOperator::LogicalNot {
            return;
        }

        let argument = unary.argument.without_parentheses();

        // Reject the common non-candidates (`!flag`, `!foo()`, ...) before the
        // ancestor walk below, which is far more expensive than matching the
        // argument. `only_equality_comparisons` recurses, so it stays behind it.
        let is_candidate = match argument {
            Expression::BinaryExpression(comparison) => comparison.operator.is_equality(),
            Expression::LogicalExpression(_) => self.check_logical_expressions,
            _ => false,
        };
        if !is_candidate {
            return;
        }

        // The inner `!` of a double negation is left alone: `!!(a === b)` is the
        // idiomatic boolean coercion, and the outer `!` is not reported either,
        // since its argument is a `UnaryExpression` rather than a comparison.
        // Parentheses are skipped when looking for that outer `!`, so
        // `!(!(a === b))` counts too.
        if outermost_paren_parent(node, ctx.semantic()).is_some_and(|parent| {
            matches!(parent.kind(), AstKind::UnaryExpression(outer) if outer.operator == UnaryOperator::LogicalNot)
        }) {
            return;
        }

        match argument {
            Expression::BinaryExpression(comparison) if comparison.operator.is_equality() => {
                check_comparison(node, unary, comparison, ctx);
            }
            Expression::LogicalExpression(_)
                if self.check_logical_expressions && only_equality_comparisons(argument) =>
            {
                check_logical(node, unary, argument, ctx);
            }
            _ => {}
        }
    }
}

/// A relational comparison (`<`, `<=`, ...), `in`, `instanceof`, `??`, or any
/// other kind of operand disqualifies the whole tree, since De Morgan's law only
/// distributes negation over `&&`/`||`.
fn only_equality_comparisons(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::BinaryExpression(comparison) => comparison.operator.is_equality(),
        Expression::LogicalExpression(logical) => {
            matches!(logical.operator, LogicalOperator::And | LogicalOperator::Or)
                && only_equality_comparisons(&logical.left)
                && only_equality_comparisons(&logical.right)
        }
        _ => false,
    }
}

fn check_comparison<'a>(
    node: &AstNode<'a>,
    unary: &UnaryExpression<'a>,
    comparison: &BinaryExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let diagnostic = no_negated_comparison_diagnostic(unary.span);

    // A comment in the "wrapper" region we discard - between `!` and the
    // comparison, or between the comparison and the closing paren, e.g.
    // `!/* c */(a === b)` - would be silently deleted. Comments *inside* the
    // comparison survive, since `flipped_comparison_text` copies the source.
    if has_wrapper_comment(unary, comparison.span, ctx) {
        ctx.diagnostic(diagnostic);
        return;
    }

    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        // This path copies the comparison's source verbatim, so whatever
        // parentheses it already had are still shielding in the replacement.
        let replacement = Replacement {
            text: flipped_comparison_text(comparison, ctx),
            precedence: Precedence::Equals,
            exposes_in: comparison_exposes_in(comparison),
        };
        apply_bang_removal_fix(
            fixer,
            node,
            &replacement,
            "Use the opposite comparison operator instead of negating the comparison.",
            ctx,
        )
    });
}

fn has_wrapper_comment(unary: &UnaryExpression, comparison_span: Span, ctx: &LintContext) -> bool {
    ctx.semantic().has_comments_between(Span::new(unary.span.start, comparison_span.start))
        || ctx.semantic().has_comments_between(Span::new(comparison_span.end, unary.span.end))
}

fn check_logical<'a>(
    node: &AstNode<'a>,
    unary: &UnaryExpression<'a>,
    logical_argument: &Expression<'a>,
    ctx: &LintContext<'a>,
) {
    let diagnostic = no_negated_logical_comparison_diagnostic(unary.span);

    // `flipped_logical_text` rebuilds the expression from its parts, silently
    // dropping any comments inside it, so withhold the fix instead.
    if ctx.semantic().has_comments_between(unary.span) {
        ctx.diagnostic(diagnostic);
        return;
    }

    let Some(replacement) = flipped_logical_text(logical_argument, ctx, None) else {
        ctx.diagnostic(diagnostic);
        return;
    };

    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        apply_bang_removal_fix(
            fixer,
            node,
            &replacement,
            "Apply De Morgan's law instead of negating the whole expression.",
            ctx,
        )
    });
}

/// Parent contexts that always want the replacement grouped, because the
/// operand position binds tighter than any comparison or logical operator
/// (e.g. `+!(a === b)` -> `+(a !== b)`, never `+a !== b`).
///
/// `PrivateInExpression` is listed separately from `BinaryExpression`: `#x in y`
/// is its own node, but its right operand binds exactly like the one of a plain
/// `in`, so `#x in !(a === b)` must not become `#x in a !== b`.
///
/// `TSNonNullExpression` is deliberately absent: a postfix `!` binds tighter
/// than a prefix one, so `!(a === b)!` parses as `!((a === b)!)` and a non-null
/// assertion can never be the direct parent of a reported node.
///
/// `SpreadElement` and `YieldExpression` are absent for the opposite reason:
/// both take an `AssignmentExpression` operand and consume it greedily, so
/// `foo(...a !== b)` and `yield a !== b` already mean the right thing.
fn parent_needs_grouped_comparison(parent_kind: AstKind) -> bool {
    matches!(
        parent_kind,
        AstKind::AwaitExpression(_)
            | AstKind::BinaryExpression(_)
            | AstKind::PrivateInExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::UnaryExpression(_)
    )
}

/// The init of a `for` statement is parsed with the `[~In]` grammar parameter,
/// so a top-level `in` there would be read as the `in` of a `for-in` head. The
/// discarded parentheses were the only thing shielding it, e.g.
/// `for (!(a in b === c);;)` -> `for (a in b !== c;;)` no longer parses.
///
/// Nothing between the node and the `for` restores `[+In]` here: crossing a
/// call argument or a nested paren would, but grouping those too only costs a
/// redundant pair of parentheses in source that is already pathological.
fn is_in_for_statement_init(node: &AstNode, ctx: &LintContext) -> bool {
    let span = node.span();
    ctx.nodes().ancestors(node.id()).any(|ancestor| {
        matches!(ancestor.kind(), AstKind::ForStatement(for_stmt)
            if for_stmt.init.as_ref().is_some_and(|init| init.span().contains_inclusive(span)))
    })
}

/// The text the `!` is replaced with, and what the caller needs to know about it
/// to decide on grouping.
///
/// `exposes_in` is carried rather than recomputed from the AST because only the
/// code that builds the text knows which parentheses survived into it: the
/// equality path copies source verbatim, while [`flipped_logical_text`] rebuilds
/// each leaf from `comparison.span` and so drops the leaf's own parentheses.
struct Replacement {
    text: String,
    precedence: Precedence,
    /// Whether `text` renders an `in` that no parentheses shield.
    exposes_in: bool,
}

/// [`exposes_in_operator`] for a comparison the caller already destructured.
///
/// The comparison's own operator is always an equality one here, so only its
/// operands can contribute an `in`.
fn comparison_exposes_in(comparison: &BinaryExpression) -> bool {
    exposes_in_operator(&comparison.left) || exposes_in_operator(&comparison.right)
}

/// Whether `expr` renders with an `in` that no parentheses shield, assuming its
/// source text is copied verbatim.
///
/// Only the operators that can hold an unparenthesized `in` need walking: an
/// `in` binds at the relational level, so anywhere below that (a unary operand,
/// a call argument, ...) it is already bracketed. Everything else - a
/// `ParenthesizedExpression` above all - falls through to `false`: parentheses
/// in the source are copied verbatim into the replacement, so they keep
/// shielding whatever is inside them.
fn exposes_in_operator(expr: &Expression) -> bool {
    match expr {
        Expression::PrivateInExpression(_) => true,
        Expression::BinaryExpression(binary) => {
            binary.operator == BinaryOperator::In
                || exposes_in_operator(&binary.left)
                || exposes_in_operator(&binary.right)
        }
        Expression::LogicalExpression(logical) => {
            exposes_in_operator(&logical.left) || exposes_in_operator(&logical.right)
        }
        Expression::TSAsExpression(expr) => exposes_in_operator(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => exposes_in_operator(&expr.expression),
        Expression::TSNonNullExpression(expr) => exposes_in_operator(&expr.expression),
        _ => false,
    }
}

/// Whether dropping the `!` would let a `&&`/`||`/`??` parent re-associate the
/// replacement, or produce outright invalid syntax.
///
/// `parent_needs_grouped_comparison` cannot answer this: a logical parent is
/// fine for an equality replacement (`x && a !== b`) but not for a lower
/// precedence logical one (`x && (a !== b || c !== d)`).
fn logical_parent_needs_parens(parent_kind: AstKind, replacement: Precedence) -> bool {
    let AstKind::LogicalExpression(parent) = parent_kind else {
        return false;
    };

    // `??` cannot be mixed with `&&`/`||` without explicit parentheses at all,
    // regardless of precedence - doing so is a SyntaxError.
    if parent.operator == LogicalOperator::Coalesce {
        return matches!(replacement, Precedence::LogicalOr | Precedence::LogicalAnd);
    }

    replacement < parent.operator.precedence()
}

/// An object, function, or class literal as the leftmost token re-parses as a
/// block or a declaration once it is no longer shielded by the `!`, e.g.
/// `!({} === b)` -> `{} !== b`. Rather than work out every position where that
/// can happen (statement start, arrow body, ...), always group these; they are
/// vanishingly rare as the left operand of a comparison.
fn starts_with_brace_or_declaration_keyword(text: &str) -> bool {
    if text.starts_with('{') {
        return true;
    }

    // `let [` is the lookahead that turns a statement into a lexical
    // declaration, so only a computed member access on `let` is a hazard:
    // `let.a !== b` and a bare `let !== b` both stay expression statements.
    if let Some(rest) = strip_keyword(text, "let") {
        return skip_trivia(rest).starts_with('[');
    }

    // `async function () {}` re-parses as an (unnamed, therefore invalid) async
    // function declaration just like `function () {}` does.
    let text = strip_keyword(text, "async").map_or(text, skip_trivia);

    strip_keyword(text, "function").is_some() || strip_keyword(text, "class").is_some()
}

/// Strips `keyword` from the front of `text`, but only when it really is a
/// keyword token rather than the start of a longer identifier.
fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
    text.strip_prefix(keyword).filter(|rest| !rest.starts_with(continues_identifier))
}

/// Skips whitespace and comments, so a keyword can be matched against the token
/// that actually follows it. Line terminators are skipped too: the `let [`
/// lookahead ignores them, and a line break before `function` only demotes
/// `async` to an identifier in source that cannot parse anyway.
fn skip_trivia(mut text: &str) -> &str {
    loop {
        text = text.trim_start();
        if let Some(rest) = text.strip_prefix("//") {
            text = rest.find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or("", |end| &rest[end..]);
        } else if let Some(rest) = text.strip_prefix("/*") {
            let Some((_, after)) = rest.split_once("*/") else { return text };
            text = after;
        } else {
            return text;
        }
    }
}

/// Whether `c` can continue (or start) an identifier. A unicode escape belongs
/// to the identifier it appears in even though the backslash that introduces it
/// is not itself an identifier character, so an escaped `a` written directly
/// after `return` forms the single identifier `returna` instead of the `return`
/// keyword.
fn continues_identifier(c: char) -> bool {
    is_identifier_part(c) || c == '\\'
}

/// Deletes the leading `!`, replaces the rest of the unary expression with
/// `replacement`, and handles the parentheses/spacing/ASI fallout of doing so.
fn apply_bang_removal_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    node: &AstNode<'a>,
    replacement: &Replacement,
    message: &'static str,
    ctx: &LintContext<'a>,
) -> RuleFix {
    // `node` is the reported `!`, so its span is the whole unary expression.
    let unary_span = node.span();
    let parent_kind = ctx.nodes().parent_kind(node.id());

    let needs_parens = parent_needs_grouped_comparison(parent_kind)
        || logical_parent_needs_parens(parent_kind, replacement.precedence)
        || starts_with_brace_or_declaration_keyword(&replacement.text)
        || (replacement.exposes_in && is_in_for_statement_init(node, ctx));

    let text =
        if needs_parens { format!("({})", replacement.text) } else { replacement.text.clone() };
    let first_char = text.chars().next().expect("replacement text is never empty");

    // Merge hazard: deleting `!` right after a keyword/identifier that isn't
    // separated by whitespace, e.g. `case!(a === b)` -> `casea !== b`. The
    // replacement side counts a leading backslash too, since it can only be the
    // start of a unicode escape within an identifier.
    let bang_start = unary_span.start as usize;
    let prev_char = ctx.source_text()[..bang_start].chars().next_back();
    let needs_space_before =
        prev_char.is_some_and(is_identifier_part) && continues_identifier(first_char);

    // ASI hazard: at the start of an `ExpressionStatement`, a replacement whose
    // first token can also continue an expression is parsed as a continuation of
    // the previous statement once the `!` is gone. Besides the operators, that
    // covers `(`/`[` (call and member access), a template literal (tagged
    // template), `<` (a JSX element, or a `<T>expr` type assertion, either of
    // which reads as a comparison) and a `.5`-style number.
    let needs_semicolon = matches!(first_char, '(' | '[' | '`' | '+' | '-' | '/' | '<' | '.')
        && could_be_asi_hazard(node, ctx);

    let fixer = fixer.for_multifix();
    let mut fixes = fixer.new_fix_with_capacity(2);

    if needs_semicolon {
        fixes.push(fixer.insert_text_before_range(unary_span, ";"));
    } else if needs_space_before {
        fixes.push(fixer.insert_text_before_range(unary_span, " "));
    }

    fixes.push(fixer.replace(unary_span, text));

    fixes.with_message(message)
}

/// Returns the source text of `comparison` with its (outermost) operator
/// replaced by its inverse, preserving everything else verbatim (whitespace,
/// comments, and any nested sub-expressions such as `a === b === c`).
fn flipped_comparison_text(comparison: &BinaryExpression, ctx: &LintContext) -> String {
    let inverse = comparison
        .operator
        .equality_inverse_operator()
        .expect("caller only invokes this for equality operators");

    let span = comparison.span;
    let text = ctx.source_range(span);
    let op_str = comparison.operator.as_str();
    let offset = ctx
        .find_next_token_within(comparison.left.span().end, comparison.right.span().start, op_str)
        .expect("the operator token must appear between its operands");

    let op_start = (comparison.left.span().end + offset - span.start) as usize;
    let op_end = op_start + op_str.len();

    format!("{}{}{}", &text[..op_start], inverse.as_str(), &text[op_end..])
}

/// Rebuilds `expr` with De Morgan's law applied, adding parentheses only where
/// required to keep the new (swapped) operators binding the way the old ones did
/// - e.g. a `||` that ends up nested inside a `&&` always needs them.
///
/// Returns the rewritten text, or [`None`] for any shape
/// [`only_equality_comparisons`] would have rejected. The two functions must
/// agree, but this one refuses to guess rather than panicking on, or silently
/// mistranslating, source it does not recognise.
///
/// Rebuilding from the parts means original line breaks and inner spacing
/// between the operands are not preserved. It also means a leaf's own
/// parentheses are dropped - the leaf is re-emitted from `comparison.span` -
/// so an `in` they were shielding becomes exposed and is reported through
/// [`Replacement::exposes_in`].
fn flipped_logical_text(
    expr: &Expression,
    ctx: &LintContext,
    parent_new_operator: Option<LogicalOperator>,
) -> Option<Replacement> {
    match expr.without_parentheses() {
        Expression::BinaryExpression(comparison) if comparison.operator.is_equality() => {
            Some(Replacement {
                text: flipped_comparison_text(comparison, ctx),
                precedence: Precedence::Equals,
                exposes_in: comparison_exposes_in(comparison),
            })
        }
        Expression::LogicalExpression(logical) => {
            let new_operator = match logical.operator {
                LogicalOperator::And => LogicalOperator::Or,
                LogicalOperator::Or => LogicalOperator::And,
                // De Morgan's law does not distribute over `??`.
                LogicalOperator::Coalesce => return None,
            };

            let left = flipped_logical_text(&logical.left, ctx, Some(new_operator))?;
            let right = flipped_logical_text(&logical.right, ctx, Some(new_operator))?;
            let text = format!("{} {} {}", left.text, new_operator.as_str(), right.text);
            let exposes_in = left.exposes_in || right.exposes_in;

            let needs_parens = parent_new_operator == Some(LogicalOperator::And)
                && new_operator == LogicalOperator::Or;

            if needs_parens {
                // The parentheses added here shield everything inside them.
                Some(Replacement {
                    text: format!("({text})"),
                    precedence: Precedence::Prefix,
                    exposes_in: false,
                })
            } else {
                Some(Replacement { text, precedence: new_operator.precedence(), exposes_in })
            }
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("a === b", None),
        ("a !== b", None),
        ("a > b", None),
        ("!Array.isArray(value)", None),
        ("!(a || b)", None),
        ("!(key in object)", None),
        ("!(value instanceof Class)", None),
        ("!foo", None),
        ("!!(a === b)", None),
        // A parenthesised double negation is still a double negation.
        ("!(!(a === b))", None),
        ("!(!!(a === b))", None),
        ("!((!(a === b)))", None),
        ("!foo === bar", None),
        ("if (!(chr >= 0)) {}", None),
        ("!(a > b)", None),
        ("!(a >= b)", None),
        ("!(a < b)", None),
        ("!(a <= b)", None),
        ("!(null > undefined)", None),
        ("!(a?.b > c)", None),
        ("async function foo() { return await !(a > b); }", None),
        ("const foo = a + !(b > c);", None),
        ("!((a?.b as number) > c)", None),
        ("!(a && b)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a === b && c)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a === b && foo())", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "!(a === b && (c === d || foo()))",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        ("!(a === b ?? c)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a > b && c === d)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a === b || c <= d)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "!(a === b && (c === d || e <= f))",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!((a || b) > c && d === e)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        ("!(a in b && c === d)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "!(a instanceof B && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Only the default (non-logical) behavior is active without the option.
        ("!(a === b && c === d)", None),
        ("!(a !== b || c != d)", None),
        // A third `!` does not make the innermost one visible again: every `!`
        // here either sits under another `!` or has a `UnaryExpression` argument.
        ("!!!(a === b)", None),
        // The argument must be the comparison itself, not merely contain one.
        ("!(a === b, c === d)", None),
        ("!(a === b ? c : d)", None),
        ("!(a = b)", None),
        // Only `!` negates; `~`/`-` are not the rule's business.
        ("~(a === b)", None),
        ("-(a === b)", None),
        // De Morgan's law does not distribute over `??`, so a coalesce anywhere in
        // the tree disqualifies it even when every leaf is an equality comparison.
        ("!((a === b) ?? c)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a === b && (c ?? d))", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        // A conditional is not a logical expression, even when its parts are
        // comparisons.
        (
            "!(a === b && c === d ? e : f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    let fail = vec![
        ("!(a === b)", None),
        ("!(a !== b)", None),
        ("!(a == b)", None),
        ("!(a != b)", None),
        ("!(a === b === c)", None),
        (r#"!(typeof value === "undefined")"#, None),
        ("!((a === b))", None),
        ("(!(a === b)).toString()", None),
        ("foo(!(a === b))", None),
        ("!(a /* comment */ === b)", None),
        ("!/* comment */(a === b)", None),
        ("foo\n!([a] === b)", None),
        ("function foo() {\n  return!\n    (a === b);\n}", None),
        ("function foo() {\n  throw!\n    (a === b);\n}", None),
        ("function foo() { return!(a === b); }", None),
        ("function foo() { throw!(a === b); }", None),
        ("switch (foo) { case!(a === b): break; }", None),
        ("async function foo() { return await !(a === b); }", None),
        ("async function foo() { return await!(a === b); }", None),
        ("const foo = void !(a === b);", None),
        ("const foo = typeof !(a === b);", None),
        ("const foo = +!(a === b);", None),
        ("const foo = a + !(b === c);", None),
        ("function * foo() { yield !(a === b); }", None),
        ("function * foo() { yield!(a === b); }", None),
        ("foo(...!(a === b));", None),
        ("foo\n!(a === b) + c", None),
        ("const foo = [a]\n!(b === c) + d", None),
        // An unbraced statement body is never an ASI hazard - the `)`/keyword before
        // it ends the statement head, so there is no expression to continue. A
        // semicolon here would empty the body instead.
        ("if (x) !([a] === b)", None),
        ("if (x) {} else !([a] === b)", None),
        ("while (x) !([a] === b)", None),
        ("for (;;) !([a] === b)", None),
        ("for (const k in o) !([a] === b)", None),
        ("for (const k of o) !([a] === b)", None),
        ("do !([a] === b); while (x)", None),
        ("lbl: !([a] === b)", None),
        ("!(a?.b === c)", None),
        ("!(a?.b !== c)", None),
        ("!(a?.() == b)", None),
        // An object/function/class literal as the left operand must stay grouped,
        // otherwise it re-parses as a block or a declaration.
        ("!({} === b);", None),
        ("!(function () {} === b);", None),
        ("!(class {} === b);", None),
        ("!(async function () {} === b);", None),
        ("!(async /* comment */ function () {} === b);", None),
        ("!(async === b);", None),
        // `let [` at statement start is the lookahead for a lexical declaration.
        ("!(let[0] === b);", None),
        ("!(let.a === b);", None),
        ("!(let === b);", None),
        ("() => !({} === b)", None),
        // A leading `<` or `.5` continues the previous statement just like `(`.
        ("foo\n!(<div/> === b)", None),
        ("foo\n!(.5 === b)", None),
        // An escaped identifier would merge with the keyword the `!` separates
        // it from, turning `return` into the identifier `return\u0061`.
        (r"function foo() { return!(\u0061 === b); }", None),
        (r"switch (foo) { case!(\u0061 === b): break; }", None),
        // Operand positions that bind tighter than a comparison, reached without
        // whitespace so the `!` is also the token separator.
        ("const foo = typeof!(a === b);", None),
        ("const foo = void!(a === b);", None),
        // Grouping and the keyword-merge check must not fight each other: `(` does
        // not continue an identifier, so `case(...)` needs no separating space.
        ("switch (foo) { case!({} === b): break; }", None),
        ("function foo() { return!({} === b); }", None),
        // Parents that leave an equality comparison alone.
        ("!(a === b) ? x : y", None),
        ("x = !(a === b)", None),
        ("(!(a === b), c)", None),
        ("`${!(a === b)}`", None),
        ("new C(!(a === b))", None),
        ("o[!(a === b)]", None),
        ("class A { x = !(a === b); }", None),
        // A binary parent re-associates, so the comparison must be grouped.
        ("!(a === b) instanceof C", None),
        ("2 ** !(a === b)", None),
        // `#x in <expr>` binds like a relational operator, so an ungrouped
        // comparison would re-associate to `(#x in a) !== b`.
        ("class A { m() { return #x in !(a === b); } }", None),
        // Every leading token that can continue the previous expression.
        ("foo\n!({} === b)", None),
        ("foo\n!(`t` === b)", None),
        ("foo\n!(/re/ === b)", None),
        ("foo\n!(-1 === b)", None),
        ("foo\n!(+1 === b)", None),
        // The `let [` lookahead has to be found past intervening trivia.
        ("!(let /* comment */ [0] === b);", None),
        // A comment in the discarded wrapper region suppresses the fix; unlike
        // `!/* comment */(a === b)` this one sits after the comparison.
        ("!(a === b /* comment */)", None),
        ("!((a === b) /* comment */)", None),
        // A `for` init is parsed with `[~In]`, so an `in` the parentheses were
        // shielding must stay grouped.
        ("for (!(a in b === c);;);", None),
        ("for (!(a === b in c);;);", None),
        ("for (x = !(a in b === c);;);", None),
        ("for (var x = !(a in b === c);;);", None),
        ("class A { m() { for (!(#x in y === z);;); } }", None),
        // ... while an `in` that keeps its own parentheses, or one outside a `for`
        // init, needs no extra grouping.
        ("for (!((a in b) === c);;);", None),
        ("!(a in b === c);", None),
        ("for (;!(a in b === c););", None),
        ("for (;;!(a in b === c));", None),
        // A logical parent must not be allowed to re-associate the comparison.
        ("x && !(a === b)", None),
        ("x ?? !(a === b)", None),
        ("!(foo! === bar)", None),
        ("!((foo as string) === bar)", None),
        ("const foo = !(a === b) as boolean;", None),
        ("const foo = !(a === b) satisfies boolean;", None),
        ("const foo = (!(a === b))!;", None),
        ("!(a === b && c === d)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "if (!(a === b && c === d)) {}",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        ("!(a !== b || c != d)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        ("!(a?.b === c && d === e)", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "!(a === b && (c === d || e === f))",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!((a === b && c === d) && e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d || e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b || c === d && e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!((a || b) === c && d === e)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!/* comment */(a === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a /* comment */ === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && /* comment */ c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "foo\n!(a === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() {\n  return!\n    (a === b && c === d);\n}",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() { return!(a === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() { throw!(a === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "switch (foo) { case!(a === b && c === d): break; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "async function foo() { return await!(a === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function * foo() { yield!(a === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // A `&&`/`||`/`??` parent must not be allowed to re-associate the
        // rewritten logical expression.
        (
            "x && !(a === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "x || !(a === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "x ?? !(a === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d) && x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(async function () {} === b && c === d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            r"function foo() { return!(\u0061 === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // The same grouping/ASI/parent hazards as the equality cases above, but
        // with a logical replacement, whose lower precedence is easier to lose.
        ("!({} === b && c === d);", Some(serde_json::json!([{"checkLogicalExpressions": true}]))),
        (
            "!(let[0] === b && c === d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d) ?? x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d) instanceof C",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "const foo = typeof !(a === b && c === d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "foo(...!(a === b && c === d));",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d) ? x : y",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "class A { x = !(a === b && c === d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "class A { m() { return #x in !(a === b && c === d); } }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "foo\n!([a] === b && c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Shapes where the `&&`/`||` swap changes which operator binds tighter.
        (
            "!(a === b || c === d) && x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "x || !(a === b || c === d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b || c === d || e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d && e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!((a === b || c === d) && e === f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && (c === d || (e === f && g === h)))",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // A trailing comment is inside the rebuilt span, so the fix is withheld.
        (
            "!(a === b && c === d /* comment */)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // `flipped_logical_text` copies each leaf verbatim, so the `for` init
        // hazard reaches the logical path through any leaf.
        (
            "for (!(a in b === c && d === e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "for (!(a === b && c in d === e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Rebuilding drops each leaf's own parentheses, so an `in` they were
        // shielding becomes exposed and the whole replacement needs grouping.
        (
            "for (!((a in b === c) && d === e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "for (!(d === e && (a in b === c));;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // ... but parentheses *inside* a leaf are copied verbatim and still shield.
        (
            "for (!((a in b) === c && d === e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    let fix = vec![
        ("!(a === b)", "a !== b", None),
        ("!(a !== b)", "a === b", None),
        ("!(a == b)", "a != b", None),
        ("!(a != b)", "a == b", None),
        ("!(a === b === c)", "a === b !== c", None),
        (r#"!(typeof value === "undefined")"#, r#"typeof value !== "undefined""#, None),
        ("!((a === b))", "a !== b", None),
        ("(!(a === b)).toString()", "(a !== b).toString()", None),
        ("foo(!(a === b))", "foo(a !== b)", None),
        ("!(a /* comment */ === b)", "a /* comment */ !== b", None),
        ("function foo() { return!(a === b); }", "function foo() { return a !== b; }", None),
        ("function foo() { throw!(a === b); }", "function foo() { throw a !== b; }", None),
        // The line break lives inside the replaced span, so it goes away with the
        // `!` and cannot turn into an ASI hazard after `return`/`throw`.
        (
            "function foo() {\n  return!\n    (a === b);\n}",
            "function foo() {\n  return a !== b;\n}",
            None,
        ),
        (
            "function foo() {\n  throw!\n    (a === b);\n}",
            "function foo() {\n  throw a !== b;\n}",
            None,
        ),
        ("switch (foo) { case!(a === b): break; }", "switch (foo) { case a !== b: break; }", None),
        (
            "async function foo() { return await !(a === b); }",
            "async function foo() { return await (a !== b); }",
            None,
        ),
        (
            "async function foo() { return await!(a === b); }",
            "async function foo() { return await(a !== b); }",
            None,
        ),
        ("const foo = void !(a === b);", "const foo = void (a !== b);", None),
        ("const foo = typeof !(a === b);", "const foo = typeof (a !== b);", None),
        ("const foo = +!(a === b);", "const foo = +(a !== b);", None),
        ("const foo = a + !(b === c);", "const foo = a + (b !== c);", None),
        // `yield` and `...` both take an `AssignmentExpression`, so no grouping -
        // but `yield!` still needs the space the `!` was providing.
        ("function * foo() { yield !(a === b); }", "function * foo() { yield a !== b; }", None),
        ("function * foo() { yield!(a === b); }", "function * foo() { yield a !== b; }", None),
        ("foo(...!(a === b));", "foo(...a !== b);", None),
        ("foo\n!(a === b) + c", "foo\n;(a !== b) + c", None),
        ("const foo = [a]\n!(b === c) + d", "const foo = [a]\n;(b !== c) + d", None),
        ("if (x) !([a] === b)", "if (x) [a] !== b", None),
        ("if (x) {} else !([a] === b)", "if (x) {} else [a] !== b", None),
        ("while (x) !([a] === b)", "while (x) [a] !== b", None),
        ("for (;;) !([a] === b)", "for (;;) [a] !== b", None),
        ("for (const k in o) !([a] === b)", "for (const k in o) [a] !== b", None),
        ("for (const k of o) !([a] === b)", "for (const k of o) [a] !== b", None),
        ("do !([a] === b); while (x)", "do [a] !== b; while (x)", None),
        ("lbl: !([a] === b)", "lbl: [a] !== b", None),
        ("!(a?.b === c)", "a?.b !== c", None),
        ("!(a?.b !== c)", "a?.b === c", None),
        ("!(a?.() == b)", "a?.() != b", None),
        ("!({} === b);", "({} !== b);", None),
        ("!(function () {} === b);", "(function () {} !== b);", None),
        ("!(class {} === b);", "(class {} !== b);", None),
        ("!(async function () {} === b);", "(async function () {} !== b);", None),
        (
            "!(async /* comment */ function () {} === b);",
            "(async /* comment */ function () {} !== b);",
            None,
        ),
        // `async` on its own is just an identifier, so it needs no grouping.
        ("!(async === b);", "async !== b;", None),
        ("!(let[0] === b);", "(let[0] !== b);", None),
        // Only `let [` is restricted, so these two stay ungrouped.
        ("!(let.a === b);", "let.a !== b;", None),
        ("!(let === b);", "let !== b;", None),
        ("() => !({} === b)", "() => ({} !== b)", None),
        ("foo\n!(<div/> === b)", "foo\n;<div/> !== b", None),
        ("foo\n!(.5 === b)", "foo\n;.5 !== b", None),
        (
            r"function foo() { return!(\u0061 === b); }",
            r"function foo() { return \u0061 !== b; }",
            None,
        ),
        (
            r"switch (foo) { case!(\u0061 === b): break; }",
            r"switch (foo) { case \u0061 !== b: break; }",
            None,
        ),
        // A unary operand reached without whitespace: `(` cannot continue the
        // keyword, so the grouping doubles as the separator.
        ("const foo = typeof!(a === b);", "const foo = typeof(a !== b);", None),
        ("const foo = void!(a === b);", "const foo = void(a !== b);", None),
        (
            "switch (foo) { case!({} === b): break; }",
            "switch (foo) { case({} !== b): break; }",
            None,
        ),
        ("function foo() { return!({} === b); }", "function foo() { return({} !== b); }", None),
        ("!(a === b) ? x : y", "a !== b ? x : y", None),
        ("x = !(a === b)", "x = a !== b", None),
        ("(!(a === b), c)", "(a !== b, c)", None),
        ("`${!(a === b)}`", "`${a !== b}`", None),
        ("new C(!(a === b))", "new C(a !== b)", None),
        ("o[!(a === b)]", "o[a !== b]", None),
        ("class A { x = !(a === b); }", "class A { x = a !== b; }", None),
        ("!(a === b) instanceof C", "(a !== b) instanceof C", None),
        ("2 ** !(a === b)", "2 ** (a !== b)", None),
        (
            "class A { m() { return #x in !(a === b); } }",
            "class A { m() { return #x in (a !== b); } }",
            None,
        ),
        // The `(` that grouping itself introduces is a hazard too.
        ("foo\n!({} === b)", "foo\n;({} !== b)", None),
        ("foo\n!(`t` === b)", "foo\n;`t` !== b", None),
        ("foo\n!(/re/ === b)", "foo\n;/re/ !== b", None),
        ("foo\n!(-1 === b)", "foo\n;-1 !== b", None),
        ("foo\n!(+1 === b)", "foo\n;+1 !== b", None),
        ("foo\n!([a] === b)", "foo\n;[a] !== b", None),
        ("!(let /* comment */ [0] === b);", "(let /* comment */ [0] !== b);", None),
        // An equality replacement binds tighter than any logical operator, so it
        // needs no grouping here.
        ("for (!(a in b === c);;);", "for ((a in b !== c);;);", None),
        ("for (!(a === b in c);;);", "for ((a !== b in c);;);", None),
        ("for (x = !(a in b === c);;);", "for (x = (a in b !== c);;);", None),
        ("for (var x = !(a in b === c);;);", "for (var x = (a in b !== c);;);", None),
        (
            "class A { m() { for (!(#x in y === z);;); } }",
            "class A { m() { for ((#x in y !== z);;); } }",
            None,
        ),
        // The `in` keeps the parentheses it already had, so no extra pair.
        ("for (!((a in b) === c);;);", "for ((a in b) !== c;;);", None),
        ("!(a in b === c);", "a in b !== c;", None),
        ("for (;!(a in b === c););", "for (;a in b !== c;);", None),
        ("for (;;!(a in b === c));", "for (;;a in b !== c);", None),
        ("x && !(a === b)", "x && a !== b", None),
        ("x ?? !(a === b)", "x ?? a !== b", None),
        ("!(foo! === bar)", "foo! !== bar", None),
        ("!((foo as string) === bar)", "(foo as string) !== bar", None),
        ("const foo = !(a === b) as boolean;", "const foo = (a !== b) as boolean;", None),
        (
            "const foo = !(a === b) satisfies boolean;",
            "const foo = (a !== b) satisfies boolean;",
            None,
        ),
        ("const foo = (!(a === b))!;", "const foo = (a !== b)!;", None),
        (
            "!(a === b && c === d)",
            "a !== b || c !== d",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "if (!(a === b && c === d)) {}",
            "if (a !== b || c !== d) {}",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a !== b || c != d)",
            "a === b && c == d",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && (c === d || e === f))",
            "a !== b || c !== d && e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!((a === b && c === d) && e === f)",
            "a !== b || c !== d || e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d || e === f)",
            "(a !== b || c !== d) && e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b || c === d && e === f)",
            "a !== b && (c !== d || e !== f)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() { return!(a === b && c === d); }",
            "function foo() { return a !== b || c !== d; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() {\n  return!\n    (a === b && c === d);\n}",
            "function foo() {\n  return a !== b || c !== d;\n}",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function foo() { throw!(a === b && c === d); }",
            "function foo() { throw a !== b || c !== d; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "switch (foo) { case!(a === b && c === d): break; }",
            "switch (foo) { case a !== b || c !== d: break; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "async function foo() { return await!(a === b && c === d); }",
            "async function foo() { return await(a !== b || c !== d); }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "function * foo() { yield!(a === b && c === d); }",
            "function * foo() { yield a !== b || c !== d; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // `||` binds looser than the `&&` parent, so it must be grouped.
        (
            "x && !(a === b && c === d)",
            "x && (a !== b || c !== d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d) && x",
            "(a !== b || c !== d) && x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // `||` binds tighter than the `||` parent, so grouping is unnecessary.
        (
            "x || !(a === b && c === d)",
            "x || a !== b || c !== d",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Mixing `??` with `||` is a SyntaxError without parentheses.
        (
            "x ?? !(a === b && c === d)",
            "x ?? (a !== b || c !== d)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(async function () {} === b && c === d);",
            "(async function () {} !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            r"function foo() { return!(\u0061 === b && c === d); }",
            r"function foo() { return \u0061 !== b || c !== d; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!({} === b && c === d);",
            "({} !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(let[0] === b && c === d);",
            "(let[0] !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "foo\n!([a] === b && c === d)",
            "foo\n;[a] !== b || c !== d",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // `??` on either side still needs the parentheses.
        (
            "!(a === b && c === d) ?? x",
            "(a !== b || c !== d) ?? x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Parents that bind tighter than `||` and so must group it.
        (
            "!(a === b && c === d) instanceof C",
            "(a !== b || c !== d) instanceof C",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "const foo = typeof !(a === b && c === d);",
            "const foo = typeof (a !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "foo(...!(a === b && c === d));",
            "foo(...a !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "class A { m() { return #x in !(a === b && c === d); } }",
            "class A { m() { return #x in (a !== b || c !== d); } }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Parents that bind looser than `||` and so leave it alone.
        (
            "!(a === b && c === d) ? x : y",
            "a !== b || c !== d ? x : y",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "class A { x = !(a === b && c === d); }",
            "class A { x = a !== b || c !== d; }",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Swapping `||` to `&&` raises the replacement's precedence, so these
        // logical parents need no parentheses.
        (
            "!(a === b || c === d) && x",
            "a !== b && c !== d && x",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "x || !(a === b || c === d)",
            "x || a !== b && c !== d",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // A flat chain of one operator collapses to a flat chain of the other.
        (
            "!(a === b || c === d || e === f)",
            "a !== b && c !== d && e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "!(a === b && c === d && e === f)",
            "a !== b || c !== d || e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // A nested `||` becomes a `&&`, which binds tighter than the surrounding
        // `||` and so loses its parentheses.
        (
            "!((a === b || c === d) && e === f)",
            "a !== b && c !== d || e !== f",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // ... while at three levels deep the innermost group is needed again.
        (
            "!(a === b && (c === d || (e === f && g === h)))",
            "a !== b || c !== d && (e !== f || g !== h)",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "for (!(a in b === c && d === e);;);",
            "for ((a in b !== c || d !== e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "for (!(a === b && c in d === e);;);",
            "for ((a !== b || c in d !== e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // The leaf's own parentheses do not survive the rebuild, so the `in` they
        // shielded needs the grouping added back around the whole replacement.
        (
            "for (!((a in b === c) && d === e);;);",
            "for ((a in b !== c || d !== e);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        (
            "for (!(d === e && (a in b === c));;);",
            "for ((d !== e || a in b !== c);;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
        // Parentheses inside a leaf are copied verbatim, so they still shield.
        (
            "for (!((a in b) === c && d === e);;);",
            "for ((a in b) !== c || d !== e;;);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    Tester::new(NoNegatedComparison::NAME, NoNegatedComparison::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();

    // `<T>expr` type assertions only parse in non-JSX TypeScript, so they cannot
    // be covered by the default `.tsx` tester above - there they are a parse
    // error, which would make a `fail` case pass without ever running the rule.
    let ts_pass = vec![
        ("const foo = <boolean>!(a > b);", None),
        (
            "const foo = <boolean>!(a > b && c === d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    let ts_fail = vec![
        ("const foo = <boolean>!(a === b);", None),
        ("foo\n!(<boolean>a === b)", None),
        (
            "const foo = <boolean>!(a === b && c === d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    let ts_fix = vec![
        ("const foo = <boolean>!(a === b);", "const foo = <boolean>(a !== b);", None),
        // Without the `;` this reads as `foo < boolean > a !== b`.
        ("foo\n!(<boolean>a === b)", "foo\n;<boolean>a !== b", None),
        (
            "const foo = <boolean>!(a === b && c === d);",
            "const foo = <boolean>(a !== b || c !== d);",
            Some(serde_json::json!([{"checkLogicalExpressions": true}])),
        ),
    ];

    Tester::new(NoNegatedComparison::NAME, NoNegatedComparison::PLUGIN, ts_pass, ts_fail)
        .change_rule_path_extension("ts")
        .expect_fix(ts_fix)
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}
