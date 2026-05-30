use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, fixer::Fix, rule::Rule};

fn prefer_single_call_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not call `{description}` multiple times."))
        .with_help("Merge with the previous call.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSingleCall;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces combining multiple `Array#push()`, `Element#classList.{add,remove}()`,
    /// and `importScripts()` into a single call.
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
    /// element.classList.add('foo', 'bar');
    ///
    /// importScripts('foo.js', 'bar.js');
    /// ```
    PreferSingleCall,
    unicorn,
    pedantic,
    pending,
    fix,
    version = "0.0.0",
);

/// Callee source-text patterns to ignore for `Array#push`.
const PUSH_IGNORE: &[&str] = &[
    "stream.push",
    "this.push",
    "this.stream.push",
    "process.stdin.push",
    "process.stdout.push",
    "process.stderr.push",
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
fn classify_call<'a>(call: &'a CallExpression<'a>, src: &'a str) -> Option<CallInfo<'a>> {
    if call.optional {
        return None;
    }

    match call.callee.without_parentheses() {
        // `receiver.push(...)`, `el.classList.add(...)`, `el.classList.remove(...)`
        Expression::StaticMemberExpression(member) => {
            if member.optional {
                return None;
            }
            let method = member.property.name.as_str();

            match method {
                "push" => {
                    if !is_stable_receiver(&member.object) {
                        return None;
                    }
                    let callee_text = call.callee.span().source_text(src);
                    if PUSH_IGNORE.contains(&callee_text) {
                        return None;
                    }
                    let receiver_text = member.object.without_parentheses().span().source_text(src);
                    Some(CallInfo {
                        description: "Array#push()",
                        receiver_text,
                        diagnostic_span: member.property.span,
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
                    let receiver_text = obj_member.object.without_parentheses().span().source_text(src);
                    let description = if method == "add" {
                        "Element#classList.add()"
                    } else {
                        "Element#classList.remove()"
                    };
                    Some(CallInfo {
                        description,
                        receiver_text,
                        diagnostic_span: member.property.span,
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
        }),

        _ => None,
    }
}

/// Returns `true` when `a` and `b` represent the same mergeable call
/// (same method type, same stable receiver).
fn same_call<'a>(a: &CallInfo<'a>, b: &CallInfo<'a>) -> bool {
    a.description == b.description && a.receiver_text == b.receiver_text
}

impl Rule for PreferSingleCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Run on block-level containers so we can iterate statement pairs in
        // O(n) with `windows(2)` rather than doing O(n) work per statement.
        let stmts: &[Statement<'a>] = match node.kind() {
            AstKind::BlockStatement(b) => &b.body,
            AstKind::Program(p) => &p.body,
            AstKind::FunctionBody(b) => &b.statements,
            AstKind::StaticBlock(b) => &b.body,
            AstKind::SwitchCase(c) => &c.consequent,
            _ => return,
        };

        let src = ctx.source_text();

        for window in stmts.windows(2) {
            let (prev_stmt, curr_stmt) = (&window[0], &window[1]);

            let Statement::ExpressionStatement(prev_es) = prev_stmt else { continue; };
            let Statement::ExpressionStatement(curr_es) = curr_stmt else { continue; };

            let Expression::CallExpression(prev_call) = &prev_es.expression else { continue; };
            let Expression::CallExpression(curr_call) = &curr_es.expression else { continue; };

            let Some(prev_info) = classify_call(prev_call, src) else { continue; };
            let Some(curr_info) = classify_call(curr_call, src) else { continue; };

            if !same_call(&prev_info, &curr_info) {
                continue;
            }

            let first_call_span = prev_call.span;
            let first_stmt_end = prev_es.span.end;
            let second_stmt_end = curr_es.span.end;
            let second_args = &curr_call.arguments;
            let description = curr_info.description;
            let diag_span = curr_info.diagnostic_span;

            ctx.diagnostic_with_fix(
                prefer_single_call_diagnostic(diag_span, description),
                |fixer| {
                    let mut fix = fixer
                        .new_fix_with_capacity(2)
                        .with_message(format!("Merge into previous `{description}` call"));

                    // If the second call has arguments, insert them into the first call.
                    if !second_args.is_empty() {
                        let args_text = second_args
                            .iter()
                            .map(|a| a.span().source_text(src))
                            .collect::<Vec<_>>()
                            .join(", ");

                        // Determine separator. Check whether the first call ends with a
                        // trailing comma (like `push(a,)`) to avoid generating `push(a,, b)`.
                        let first_src = first_call_span.source_text(src);
                        let before_paren =
                            first_src[..first_src.len().saturating_sub(1)].trim_end();
                        let separator = if prev_call.arguments.is_empty() {
                            ""
                        } else if before_paren.ends_with(',') {
                            " "
                        } else {
                            ", "
                        };

                        // Replace the closing ')' of the first call with `{sep}{args})`.
                        let close_paren =
                            Span::new(first_call_span.end - 1, first_call_span.end);
                        fix.push(Fix::new(format!("{separator}{args_text})"), close_paren));
                    }

                    // Delete the second statement (from end of first stmt to end of
                    // second stmt, including any whitespace/newline in between).
                    //
                    // ASI safety: if the first statement has no semicolon but the
                    // second does, preserve the semicolon so the result stays valid.
                    let first_has_semi =
                        prev_es.span.source_text(src).trim_end().ends_with(';');
                    let second_has_semi =
                        curr_es.span.source_text(src).trim_end().ends_with(';');
                    let gap_span = Span::new(first_stmt_end, second_stmt_end);
                    if !first_has_semi && second_has_semi {
                        fix.push(Fix::new(";", gap_span));
                    } else {
                        fix.push(Fix::delete(gap_span));
                    }

                    fix
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tester::Tester;

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
            ("el.classList.add('foo'); el.classList.add('bar');", "el.classList.add('foo', 'bar');"),
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

        Tester::new(PreferSingleCall::NAME, PreferSingleCall::PLUGIN, pass, fail)
            .expect_fix(fix)
            .test_and_snapshot();
    }
}
