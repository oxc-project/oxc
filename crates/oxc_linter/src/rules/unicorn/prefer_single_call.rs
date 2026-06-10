use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::Fix,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_single_call_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not call `{description}` multiple times."))
        .with_help("Merge with the previous call.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PreferSingleCallConfig {
    /// Methods to ignore.
    #[serde(default)]
    ignore: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferSingleCall(Box<PreferSingleCallConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces combining multiple `Array#{push,unshift}()`,
    /// `Element#classList.{add,remove}()`, and `importScripts()` into a single call.
    ///
    /// Supersedes the deprecated `unicorn/no-array-push-push` rule.
    ///
    /// ### Why is this bad?
    ///
    /// Calling the same variadic method on the same receiver multiple times
    /// consecutively can be merged into a single call, which is more concise
    /// and can be marginally more performant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.push(1);
    /// foo.push(2);
    ///
    /// foo.unshift(1);
    /// foo.unshift(2);
    ///
    /// element.classList.add('foo');
    /// element.classList.add('bar');
    ///
    /// importScripts('foo.js');
    /// importScripts('bar.js');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo.push(1, 2);
    ///
    /// foo.unshift(2, 1);
    ///
    /// element.classList.add('foo', 'bar');
    ///
    /// importScripts('foo.js', 'bar.js');
    /// ```
    PreferSingleCall,
    unicorn,
    pedantic,
    pending,
    fix,
    config = PreferSingleCallConfig,
    version = "0.0.0",
);

/// Callee source-text patterns to ignore for `Array#{push,unshift}`.
const DEFAULT_ARRAY_MUTATION_IGNORE: &[&str] = &[
    "stream.push",
    "this.push",
    "this.stream.push",
    "process.stdin.push",
    "process.stdout.push",
    "process.stderr.push",
    "stream.unshift",
    "this.unshift",
    "this.stream.unshift",
    "process.stdin.unshift",
    "process.stdout.unshift",
    "process.stderr.unshift",
];

/// Information extracted from a matched call expression.
struct CallInfo<'a> {
    /// Human-readable description of the rule case (used in the message).
    description: &'static str,
    /// Source text of the call's receiver. For `Array#push` this is the
    /// object before `.push`; for `classList.{add,remove}` it is the element
    /// before `.classList`; for `importScripts` it is the literal string
    /// `"importScripts"`.
    receiver_text: &'a str,
    /// Span to attach the diagnostic label to (the method name identifier).
    diagnostic_span: Span,
    /// Whether to keep the second call and merge the first call's arguments
    /// into it. This is required for `unshift`, where call order affects the
    /// final array order.
    keep_second_call: bool,
}

/// Returns `true` if `expr` is a "stable" receiver — an expression whose
/// value cannot be changed by a function call side-effect between two
/// consecutive statements. We accept identifiers, `this`, and chains of
/// static (non-computed) member accesses built from those primitives.
fn is_stable_receiver(expr: &Expression<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::Identifier(_) | Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(m) if !m.optional => is_stable_receiver(&m.object),
        _ => false,
    }
}

/// If `call` matches one of the tracked patterns, return its [`CallInfo`];
/// otherwise return `None`.
fn classify_call<'a>(
    call: &'a CallExpression<'a>,
    src: &'a str,
    ignored_callees: &[String],
) -> Option<CallInfo<'a>> {
    if call.optional {
        return None;
    }

    match call.callee.without_parentheses() {
        // `receiver.push/unshift(...)`, `el.classList.add(...)`, `el.classList.remove(...)`
        Expression::StaticMemberExpression(member) => {
            if member.optional {
                return None;
            }
            let method = member.property.name.as_str();

            match method {
                "push" | "unshift" => {
                    if !is_stable_receiver(&member.object) {
                        return None;
                    }
                    let callee_text = call.callee.span().source_text(src);
                    if DEFAULT_ARRAY_MUTATION_IGNORE.contains(&callee_text)
                        || ignored_callees.iter().any(|ignored| ignored == callee_text)
                    {
                        return None;
                    }
                    let receiver_text = member.object.without_parentheses().span().source_text(src);
                    let (description, keep_second_call) = if method == "push" {
                        ("Array#push()", false)
                    } else {
                        ("Array#unshift()", true)
                    };
                    Some(CallInfo {
                        description,
                        receiver_text,
                        diagnostic_span: member.property.span,
                        keep_second_call,
                    })
                }

                "add" | "remove" => {
                    // Must be `<element>.classList.add/remove`
                    let obj = member.object.without_parentheses();
                    let obj_member = match obj {
                        Expression::StaticMemberExpression(m) if !m.optional => m,
                        _ => return None,
                    };
                    if obj_member.property.name.as_str() != "classList" {
                        return None;
                    }
                    if !is_stable_receiver(&obj_member.object) {
                        return None;
                    }
                    let receiver_text =
                        obj_member.object.without_parentheses().span().source_text(src);
                    let description = if method == "add" {
                        "Element#classList.add()"
                    } else {
                        "Element#classList.remove()"
                    };
                    Some(CallInfo {
                        description,
                        receiver_text,
                        diagnostic_span: member.property.span,
                        keep_second_call: false,
                    })
                }

                _ => None,
            }
        }

        // `importScripts(...)`
        Expression::Identifier(ident) if ident.name == "importScripts" => Some(CallInfo {
            description: "importScripts()",
            receiver_text: "importScripts",
            diagnostic_span: ident.span,
            keep_second_call: false,
        }),

        _ => None,
    }
}

/// Returns `true` when `a` and `b` represent the same mergeable call
/// (same method type, same stable receiver).
fn same_call<'a>(a: &CallInfo<'a>, b: &CallInfo<'a>) -> bool {
    a.description == b.description && a.receiver_text == b.receiver_text
}

/// Returns `true` if `receiver` appears as a standalone identifier (or
/// member-access chain) within `arg_src` — i.e. it is not a substring of a
/// longer identifier.
///
/// Used to detect arguments like `arr.length` in `arr.push(arr.length)`,
/// where merging two consecutive pushes would change the evaluation order of
/// the argument (the receiver is mutated by the first call).
fn arg_references_receiver(arg_src: &str, receiver: &str) -> bool {
    if receiver.is_empty() {
        return false;
    }
    let rbytes = receiver.as_bytes();
    let abytes = arg_src.as_bytes();
    let mut i = 0;
    while i + rbytes.len() <= abytes.len() {
        if abytes[i..].starts_with(rbytes) {
            let before_ok = i == 0 || !is_js_ident_continue(abytes[i - 1] as char);
            let after_ok = i + rbytes.len() >= abytes.len()
                || !is_js_ident_continue(abytes[i + rbytes.len()] as char);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[inline]
fn is_js_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

impl Rule for PreferSingleCall {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Match only expression statements, then look at the parent statement
        // list to find the immediate previous sibling. This keeps the rule off
        // non-expression-statement nodes in the generated runner.
        let AstKind::ExpressionStatement(curr_es) = node.kind() else {
            return;
        };
        let Expression::CallExpression(curr_call) = &curr_es.expression else {
            return;
        };

        let src = ctx.source_text();
        let Some(curr_info) = classify_call(curr_call, src, &self.0.ignore) else {
            return;
        };

        let parent = ctx.nodes().parent_node(node.id());
        let stmts: &[Statement<'a>] = match parent.kind() {
            AstKind::BlockStatement(b) => &b.body,
            AstKind::Program(p) => &p.body,
            AstKind::FunctionBody(b) => &b.statements,
            AstKind::StaticBlock(b) => &b.body,
            AstKind::SwitchCase(c) => &c.consequent,
            _ => return,
        };

        let Some(idx) = stmts.iter().position(|stmt| stmt.span() == curr_es.span) else {
            return;
        };
        let Some(prev_stmt) = idx.checked_sub(1).and_then(|idx| stmts.get(idx)) else {
            return;
        };

        let Statement::ExpressionStatement(prev_es) = prev_stmt else {
            return;
        };
        let Expression::CallExpression(prev_call) = &prev_es.expression else {
            return;
        };
        let Some(prev_info) = classify_call(prev_call, src, &self.0.ignore) else {
            return;
        };

        if !same_call(&prev_info, &curr_info) {
            return;
        }

        let first_call_span = prev_call.span;
        let second_call_span = curr_call.span;
        let first_stmt_start = prev_es.span.start;
        let first_stmt_end = prev_es.span.end;
        let second_stmt_start = curr_es.span.start;
        let second_stmt_end = curr_es.span.end;
        let keep_second_call = curr_info.keep_second_call;
        let (target_call, source_args) = if keep_second_call {
            (curr_call, &prev_call.arguments)
        } else {
            (prev_call, &curr_call.arguments)
        };
        let description = curr_info.description;
        let diag_span = curr_info.diagnostic_span;
        let receiver = prev_info.receiver_text;

        // P1: Skip autofix when a second-call argument references the
        // receiver — merging would change the evaluation order because the
        // first call mutates the receiver before those args are read.
        // e.g. `arr.push(1); arr.push(arr.length);`
        let has_state_dep_args = curr_call
            .arguments
            .iter()
            .any(|a| arg_references_receiver(a.span().source_text(src), receiver));

        // P2: Skip autofix when there are comments in the gap between the
        // two statements — deleting the gap would silently drop them.
        let removal_span = if keep_second_call {
            Span::new(first_stmt_start, second_stmt_start)
        } else {
            Span::new(first_stmt_end, second_stmt_end)
        };
        let has_gap_comments = ctx.comments_range(removal_span.start..removal_span.end).count() > 0;

        if has_state_dep_args || has_gap_comments {
            ctx.diagnostic(prefer_single_call_diagnostic(diag_span, description));
            return;
        }

        ctx.diagnostic_with_fix(prefer_single_call_diagnostic(diag_span, description), |fixer| {
            let mut fix = fixer
                .new_fix_with_capacity(2)
                .with_message(format!("Merge into previous `{description}` call"));

            // Insert the source call's arguments into the target call.
            if !source_args.is_empty() {
                let args_text = source_args
                    .iter()
                    .map(|a| a.span().source_text(src))
                    .collect::<Vec<_>>()
                    .join(", ");

                // Determine separator. Check whether the target call ends with a
                // trailing comma (like `push(a,)`) to avoid generating `push(a,, b)`.
                let target_src = target_call.span.source_text(src);
                let before_paren = target_src[..target_src.len().saturating_sub(1)].trim_end();
                let separator = if target_call.arguments.is_empty() {
                    ""
                } else if before_paren.ends_with(',') {
                    " "
                } else {
                    ", "
                };

                // Replace the closing ')' of the target call with `{sep}{args})`.
                let target_span = if keep_second_call { second_call_span } else { first_call_span };
                let close_paren = Span::new(target_span.end - 1, target_span.end);
                fix.push(Fix::new(format!("{separator}{args_text})"), close_paren));
            }

            // Delete the second statement (from end of first stmt to end of
            // second stmt, including any whitespace/newline in between).
            //
            // ASI handling: if the first statement has no semicolon but the
            // second does, preserve the semicolon so the result stays valid.
            let first_has_semi = prev_es.span.source_text(src).trim_end().ends_with(';');
            let second_has_semi = curr_es.span.source_text(src).trim_end().ends_with(';');
            if !keep_second_call && !first_has_semi && second_has_semi {
                fix.push(Fix::new(";", removal_span));
            } else {
                fix.push(Fix::delete(removal_span));
            }

            fix
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::tester::{TestCase, Tester};

    use super::*;

    #[test]
    fn test() {
        let pass = vec![
            // Already a single call.
            "foo.push(1, 2);",
            // Different methods.
            "foo.push(1); foo.pop();",
            // Different receivers.
            "foo.push(1); bar.push(2);",
            // Non-consecutive (something in between).
            "foo.push(1); console.log(x); foo.push(2);",
            // Optional call — skip.
            "foo.push?.(1); foo.push?.(2);",
            // Optional member — skip.
            "foo?.push(1); foo?.push(2);",
            // Ignored push callee patterns.
            "this.push(1); this.push(2);",
            "stream.push(1); stream.push(2);",
            "process.stdin.push(1); process.stdin.push(2);",
            "process.stdout.push(1); process.stdout.push(2);",
            "process.stderr.push(1); process.stderr.push(2);",
            "this.stream.push(1); this.stream.push(2);",
            "stream.unshift(1); stream.unshift(2);",
            "this.unshift(1); this.unshift(2);",
            "this.stream.unshift(1); this.stream.unshift(2);",
            "process.stdin.unshift(1); process.stdin.unshift(2);",
            "process.stdout.unshift(1); process.stdout.unshift(2);",
            "process.stderr.unshift(1); process.stderr.unshift(2);",
            // classList on different elements.
            "a.classList.add('x'); b.classList.add('y');",
            // add vs remove — different method.
            "el.classList.add('x'); el.classList.remove('y');",
            // importScripts with a statement in between.
            "importScripts('a.js'); doSomething(); importScripts('b.js');",
            // Unstable receiver (function call result may differ).
            "getArr().push(1); getArr().push(2);",
            // Computed member access — receiver identity not guaranteed.
            "a[0].push(1); a[0].push(2);",
            // .add that is not classList.add.
            "foo.add(1); foo.add(2);",
        ];

        let fail = vec![
            // Basic Array#push.
            "foo.push(1); foo.push(2);",
            // push with multiple args on each side.
            "foo.push(1, 2); foo.push(3, 4);",
            // Second call has no args (redundant call is removed).
            "foo.push(1); foo.push();",
            // First call has no args.
            "foo.push(); foo.push(1);",
            // Trailing comma in first call.
            "foo.push(1,); foo.push(2);",
            // Multi-line, both with semicolons.
            "foo.push(1);\nfoo.push(2);",
            // ASI: first statement lacks a semicolon.
            "foo.push(1)\nfoo.push(2);",
            // Spread argument.
            "foo.push(...a); foo.push(...b);",
            // Array#unshift keeps the second call and prepends the first call's args.
            "foo.unshift(1); foo.unshift(2);",
            "foo.unshift(1, 2); foo.unshift(3, 4);",
            "foo.unshift(1); foo.unshift();",
            "foo.unshift(); foo.unshift(1);",
            // classList.add.
            "el.classList.add('foo'); el.classList.add('bar');",
            // classList.remove.
            "el.classList.remove('foo'); el.classList.remove('bar');",
            // importScripts.
            "importScripts('a.js'); importScripts('b.js');",
            // Inside a function body.
            "function f() { foo.push(1); foo.push(2); }",
            // Deeply nested member receiver (stable chain).
            "a.b.push(1); a.b.push(2);",
            // this.prop.push (not in the ignore list).
            "this.arr.push(1); this.arr.push(2);",
            // Three consecutive calls — fires on each consecutive pair.
            "foo.push(1); foo.push(2); foo.push(3);",
            // Parenthesized receiver — normalised to same identity.
            "(foo).push(1); foo.push(2);",
            "foo.push(1); (foo).push(2);",
            // P1: second-call arg reads the receiver — merging would change
            // evaluation order (diagnostic fires, but no autofix offered).
            "arr.push(1); arr.push(arr.length);",
            "arr.push(1); arr.push(arr[0]);",
            // P2: comment in the gap — autofix would silently drop it.
            "foo.push(1); // keep this\nfoo.push(2);",
        ];

        let fix = vec![
            ("foo.push(1); foo.push(2);", "foo.push(1, 2);"),
            ("foo.push(1, 2); foo.push(3, 4);", "foo.push(1, 2, 3, 4);"),
            ("foo.push(1); foo.push();", "foo.push(1);"),
            ("foo.push(); foo.push(1);", "foo.push(1);"),
            ("foo.push(1,); foo.push(2);", "foo.push(1, 2);"),
            ("foo.push(1);\nfoo.push(2);", "foo.push(1, 2);"),
            // ASI: semicolon from second statement is preserved.
            ("foo.push(1)\nfoo.push(2);", "foo.push(1, 2);"),
            ("foo.push(...a); foo.push(...b);", "foo.push(...a, ...b);"),
            ("foo.unshift(1); foo.unshift(2);", "foo.unshift(2, 1);"),
            ("foo.unshift(1, 2); foo.unshift(3, 4);", "foo.unshift(3, 4, 1, 2);"),
            ("foo.unshift(1); foo.unshift();", "foo.unshift(1);"),
            ("foo.unshift(); foo.unshift(1);", "foo.unshift(1);"),
            (
                "el.classList.add('foo'); el.classList.add('bar');",
                "el.classList.add('foo', 'bar');",
            ),
            (
                "el.classList.remove('foo'); el.classList.remove('bar');",
                "el.classList.remove('foo', 'bar');",
            ),
            ("importScripts('a.js'); importScripts('b.js');", "importScripts('a.js', 'b.js');"),
            ("function f() { foo.push(1); foo.push(2); }", "function f() { foo.push(1, 2); }"),
            ("a.b.push(1); a.b.push(2);", "a.b.push(1, 2);"),
            ("this.arr.push(1); this.arr.push(2);", "this.arr.push(1, 2);"),
            // Three calls: first fix merges calls 1+2; the third call is left for the next pass.
            ("foo.push(1); foo.push(2); foo.push(3);", "foo.push(1, 2); foo.push(3);"),
            // Parenthesized receiver — keeps the paren form of whichever call is first.
            ("(foo).push(1); foo.push(2);", "(foo).push(1, 2);"),
            ("foo.push(1); (foo).push(2);", "foo.push(1, 2);"),
        ];

        let mut pass = pass.into_iter().map(TestCase::from).collect::<Vec<_>>();
        pass.extend([
            ("foo.push(1); foo.push(2);", Some(serde_json::json!([{ "ignore": ["foo.push"] }])))
                .into(),
            (
                "foo.unshift(1); foo.unshift(2);",
                Some(serde_json::json!([{ "ignore": ["foo.unshift"] }])),
            )
                .into(),
        ]);
        let fail = fail.into_iter().map(TestCase::from).collect::<Vec<_>>();

        Tester::new(PreferSingleCall::NAME, PreferSingleCall::PLUGIN, pass, fail)
            .expect_fix(fix)
            .test_and_snapshot();
    }
}
