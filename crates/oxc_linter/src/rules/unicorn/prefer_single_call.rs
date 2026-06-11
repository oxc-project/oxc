use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression, ExpressionStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_same_expression,
};

/// Callee patterns that are always ignored for `push` and `unshift`.
/// These cover Node.js streams, where `push(null)` signals end-of-stream.
const DEFAULT_IGNORE: &[&str] = &[
    "stream.push",
    "stream.unshift",
    "this.push",
    "this.unshift",
    "this.stream.push",
    "this.stream.unshift",
    "process.stdin.push",
    "process.stdin.unshift",
    "process.stdout.push",
    "process.stdout.unshift",
    "process.stderr.push",
    "process.stderr.unshift",
];

/// Which mergeable call pattern a given [`CallExpression`] matches.
#[derive(Clone, Copy, Debug)]
enum CallKind {
    /// `array.push(...args)`
    Push,
    /// `array.unshift(...args)` — semantically the second call becomes the target
    Unshift,
    /// `element.classList.add(...args)`
    ClassListAdd,
    /// `element.classList.remove(...args)`
    ClassListRemove,
    /// `importScripts(...args)` — standalone global function
    ImportScripts,
}

impl CallKind {
    fn description(self) -> &'static str {
        match self {
            CallKind::Push => "Array#push()",
            CallKind::Unshift => "Array#unshift()",
            CallKind::ClassListAdd => "Element#classList.add()",
            CallKind::ClassListRemove => "Element#classList.remove()",
            CallKind::ImportScripts => "importScripts()",
        }
    }

    /// For `unshift`, keep the *second* call as the target and prepend the
    /// first call's arguments. Everything else keeps the first call.
    fn keep_second_call(self) -> bool {
        matches!(self, CallKind::Unshift)
    }
}

fn prefer_single_call_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not call `{description}` multiple times."))
        .with_help("Merge with the previous call.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferSingleCall {
    /// Additional callee patterns (e.g. `"foo.push"`) to ignore on top of the
    /// built-in stream/this/process defaults.
    ignore: Vec<String>,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces combining multiple `Array#{push,unshift}()`,
    /// `Element#classList.{add,remove}()`, and `importScripts()` into one call.
    ///
    /// ### Why is this bad?
    ///
    /// Multiple consecutive calls to these methods can always be merged into a
    /// single call with multiple arguments, which is shorter and avoids the
    /// overhead of repeated function invocations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo.push(1);
    /// foo.push(2);
    ///
    /// foo.classList.add('a');
    /// foo.classList.add('b');
    ///
    /// importScripts('foo.js');
    /// importScripts('bar.js');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo.push(1, 2);
    ///
    /// foo.classList.add('a', 'b');
    ///
    /// importScripts('foo.js', 'bar.js');
    /// ```
    PreferSingleCall,
    unicorn,
    style,
    conditional_fix_suggestion,
    config = PreferSingleCall,
    version = "next",
);

impl Rule for PreferSingleCall {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        value
            .get(0)
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map(|opt| opt.unwrap_or_default())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(expr_stmt) = node.kind() else { return };

        // Locate this statement inside its parent's statement list.
        let parent = ctx.nodes().parent_node(node.id());
        let statements: &[Statement<'a>] = match parent.kind() {
            AstKind::BlockStatement(block) => &block.body,
            AstKind::Program(program) => &program.body,
            AstKind::FunctionBody(body) => &body.statements,
            AstKind::StaticBlock(block) => &block.body,
            AstKind::SwitchCase(case) => &case.consequent,
            _ => return,
        };

        let Some(idx) = statements.iter().position(|s| s.span() == expr_stmt.span) else {
            return;
        };
        if idx == 0 {
            return;
        }

        let prev_stmt = &statements[idx - 1];
        let curr_stmt = &statements[idx];

        let Some(first_call) = get_call_from_stmt(prev_stmt) else { return };
        let Some(second_call) = get_call_from_stmt(curr_stmt) else { return };

        // Both calls must match the same mergeable kind.
        let Some(kind) = classify_call(second_call) else { return };
        if classify_call(first_call).map(|k| k.description()) != Some(kind.description()) {
            return;
        }

        // Skip if the callee is in the built-in or user-provided ignore list.
        if is_callee_ignored(second_call, &self.ignore) {
            return;
        }

        // Both calls must target the exact same receiver.
        let first_callee = first_call.callee.get_inner_expression();
        let second_callee = second_call.callee.get_inner_expression();
        if !is_same_expression(first_callee, second_callee, ctx) {
            return;
        }

        let diag_span = diagnostic_span(second_call);
        let description = kind.description();
        let keep_second = kind.keep_second_call();
        let first_span = prev_stmt.span();
        let second_span = curr_stmt.span();

        // `unshift` uses a suggestion because merging reverses evaluation order
        // of arguments when either call has side-effecting args.
        // All other cases are safe to auto-fix.
        if keep_second {
            ctx.diagnostic_with_suggestion(
                prefer_single_call_diagnostic(diag_span, description),
                |fixer| {
                    merge_calls(
                        fixer,
                        first_call,
                        second_call,
                        first_span,
                        second_span,
                        keep_second,
                        ctx,
                    )
                },
            );
        } else {
            ctx.diagnostic_with_fix(
                prefer_single_call_diagnostic(diag_span, description),
                |fixer| {
                    merge_calls(
                        fixer,
                        first_call,
                        second_call,
                        first_span,
                        second_span,
                        keep_second,
                        ctx,
                    )
                },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Call extraction
// ---------------------------------------------------------------------------

/// Extract the [`CallExpression`] from an [`ExpressionStatement`], handling
/// the case where the whole expression is wrapped in a [`ChainExpression`]
/// (e.g. `foo?.push(1)` at statement level).
fn get_call_from_stmt<'a>(stmt: &'a Statement<'a>) -> Option<&'a CallExpression<'a>> {
    let Statement::ExpressionStatement(expr_stmt) = stmt else { return None };
    get_call_from_expr_stmt(expr_stmt)
}

fn get_call_from_expr_stmt<'a>(
    expr_stmt: &'a ExpressionStatement<'a>,
) -> Option<&'a CallExpression<'a>> {
    match &expr_stmt.expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ChainExpression(chain) => {
            if let ChainElement::CallExpression(call) = &chain.expression {
                Some(call)
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Call classification
// ---------------------------------------------------------------------------

/// Return the [`CallKind`] for a call expression, or `None` if it is not one
/// of the patterns this rule cares about.
fn classify_call(call: &CallExpression) -> Option<CallKind> {
    // `importScripts` has no optional-call restriction (importScripts?.() still fires).
    if matches!(call.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "importScripts")
    {
        return Some(CallKind::ImportScripts);
    }

    // For all method calls, `push?.()` / `add?.()` style optional *calls* are excluded.
    if call.optional {
        return None;
    }

    let member = call.callee.get_member_expr()?;

    match member.static_property_name()? {
        "push" => Some(CallKind::Push),
        "unshift" => Some(CallKind::Unshift),
        method @ ("add" | "remove") => {
            // For classList methods the member access to `add`/`remove` must
            // also not be optional (e.g. `classList?.add()` is excluded).
            if member.optional() {
                return None;
            }
            // The object of `add`/`remove` must be a `.classList` access.
            let object = member.object().get_inner_expression();
            let obj_member = object.get_member_expr()?;
            if obj_member.static_property_name() != Some("classList") {
                return None;
            }
            if method == "add" {
                Some(CallKind::ClassListAdd)
            } else {
                Some(CallKind::ClassListRemove)
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Ignore list
// ---------------------------------------------------------------------------

/// Returns `true` if the call's callee matches any built-in or user-supplied
/// ignore pattern.
fn is_callee_ignored(call: &CallExpression, user_ignore: &[String]) -> bool {
    let callee = call.callee.get_inner_expression();
    DEFAULT_IGNORE
        .iter()
        .copied()
        .chain(user_ignore.iter().map(String::as_str))
        .any(|pattern| callee_matches_pattern(callee, pattern))
}

/// Checks whether `callee` matches a dot-separated pattern such as
/// `"this.push"`, `"stream.push"`, or `"process.stdin.push"`.
///
/// Walks the member-expression chain from right to left, accepting
/// `this` as the terminal keyword.
fn callee_matches_pattern(callee: &Expression, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split('.').collect();
    let mut expr = callee;

    for (i, &part) in parts.iter().enumerate().rev() {
        match expr.get_inner_expression() {
            Expression::StaticMemberExpression(mem) if mem.property.name == part => {
                expr = &mem.object;
            }
            Expression::Identifier(id) if id.name == part && i == 0 => return true,
            Expression::ThisExpression(_) if part == "this" && i == 0 => return true,
            _ => return false,
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

/// Span of the method name (or function identifier) to highlight.
fn diagnostic_span(call: &CallExpression) -> Span {
    if let Some(member) = call.callee.get_inner_expression().as_member_expression() {
        if let Some((span, _)) = member.static_property_info() {
            return span;
        }
    }
    if let Some(ident) = call.callee.get_inner_expression().get_identifier_reference() {
        return ident.span;
    }
    call.span
}

// ---------------------------------------------------------------------------
// Fixer
// ---------------------------------------------------------------------------

/// Produce a fix that merges two consecutive calls into one.
///
/// - `keep_second = false` (push / classList / importScripts): append the
///   second call's arguments to the first call, then delete the second statement.
/// - `keep_second = true` (unshift): append the first call's arguments to the
///   second call, then delete the first statement.
fn merge_calls<'a>(
    fixer: RuleFixer<'_, 'a>,
    first_call: &CallExpression<'a>,
    second_call: &CallExpression<'a>,
    first_span: Span,
    second_span: Span,
    keep_second: bool,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2).with_message("Merge into a single call.");

    let (target, source) =
        if keep_second { (second_call, first_call) } else { (first_call, second_call) };

    // Insert the source arguments into the target call.
    if let Some(insertion) = args_insertion(&fixer, target, source, ctx) {
        fix.push(insertion);
    }

    // Delete the now-redundant source statement.
    let delete_span = if keep_second {
        Span::new(first_span.start, second_span.start)
    } else {
        Span::new(first_span.end, second_span.end)
    };
    fix.push(fixer.delete_range(delete_span));

    fix
}

/// Build the [`RuleFix`] that inserts `source`'s arguments into `target`.
///
/// Returns `None` when `source` has no arguments (nothing to insert).
fn args_insertion<'a>(
    fixer: &RuleFixer<'_, 'a>,
    target: &CallExpression<'a>,
    source: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<RuleFix> {
    let first_src = source.arguments.first()?;
    let last_src = source.arguments.last()?;
    let src_text = ctx.source_range(Span::new(first_src.span().start, last_src.span().end));

    if target.arguments.is_empty() {
        // Insert before the closing `)` of the target.
        let close = Span::new(target.span.end - 1, target.span.end);
        return Some(fixer.insert_text_before_range(close, src_text.to_string()));
    }

    let last_tgt = target.arguments.last().unwrap();
    let source_text = ctx.source_text();
    let after_end = last_tgt.span().end as usize;
    let close_pos = (target.span.end - 1) as usize;

    // Detect a trailing comma already present after the last target argument.
    if after_end <= close_pos {
        let between = &source_text[after_end..close_pos];
        if let Some(comma_off) = between.find(',') {
            let insert_pos = last_tgt.span().end + u32::try_from(comma_off).unwrap_or(0) + 1;
            return Some(fixer.insert_text_after_range(
                Span::new(insert_pos, insert_pos),
                format!(" {src_text}"),
            ));
        }
    }

    // No trailing comma — append ", src_text" after the last argument.
    Some(fixer.insert_text_after_range(last_tgt.span(), format!(", {src_text}")))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "foo.forEach(fn);
            foo.forEach(fn);",
            None,
        ),
        ("foo.push(1);", None),
        (
            "foo.push(1);
            foo.unshift(2);",
            None,
        ),
        (
            r#"foo.push(1);; // <- there is a "EmptyStatement" between
            foo.push(2);"#,
            None,
        ),
        (
            "foo.push(1);
            bar.push(2);",
            None,
        ),
        ("foo.push(1);push(2)", None),
        ("push(1);foo.push(2)", None),
        ("new foo.push(1);foo.push(2)", None),
        ("foo.push(1);new foo.push(2)", None),
        ("foo[push](1);foo.push(2)", None),
        ("foo.push(1);foo[push](2)", None),
        ("foo.push(foo.push(1));", None),
        (
            "const length = foo.push(1);
            foo.push(2);",
            None,
        ),
        (
            "foo.push(1);
            const length = foo.push(2);",
            None,
        ),
        (
            "foo().push(1);
            foo().push(2);",
            None,
        ),
        (
            "foo().bar.push(1);
            foo().bar.push(2);",
            None,
        ),
        (
            "const stream = new Readable();
            stream.push('one string');
            stream.push('another string');",
            None,
        ),
        (
            "class FooReadable extends Readable {
                pushAndEnd(chunk) {
                    this.push(chunk);
                    this.push(null);
                }
            }",
            None,
        ),
        (
            "class Foo {
                pushAndEnd(chunk) {
                    this.stream.push(chunk);
                    this.stream.push(null);
                }
            }",
            None,
        ),
        (
            "process.stdin.push(chunk);
            process.stdin.push(null);",
            None,
        ),
        (
            "process.stdout.push(chunk);
            process.stdout.push(null);",
            None,
        ),
        (
            "process.stderr.push(chunk);
            process.stderr.push(null);",
            None,
        ),
        (
            "foo.push(1);
            foo.push(2);
            foo.bar.push(1);
            foo.bar.push(2);",
            Some(serde_json::json!([ { "ignore": ["foo.push", "foo.bar.push"], }, ])),
        ),
        ("for (const _ of []) foo.push(bar);", None),
        (
            "function bar() {}
            foo.push(bindEvents);",
            None,
        ),
        (
            "foo.push?.(1);
            foo.push?.(2);",
            None,
        ),
        (
            "foo.push(1);
            foo.push?.(2);",
            None,
        ),
        (
            "foo.push?.(1);
            foo.push(2);",
            None,
        ),
        ("foo.unshift(1);", None),
        (
            "foo.push(1);
            foo.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1);
            foo.push(2);",
            None,
        ),
        (
            r#"foo.unshift(1);; // <- there is a "EmptyStatement" between
            foo.unshift(2);"#,
            None,
        ),
        (
            "foo.unshift(1);
            bar.unshift(2);",
            None,
        ),
        ("foo.unshift(1);unshift(2)", None),
        ("unshift(1);foo.unshift(2)", None),
        ("new foo.unshift(1);foo.unshift(2)", None),
        ("foo.unshift(1);new foo.unshift(2)", None),
        ("foo[unshift](1);foo.unshift(2)", None),
        ("foo.unshift(1);foo[unshift](2)", None),
        ("foo.unshift(foo.unshift(1));", None),
        (
            "const length = foo.unshift(1);
            foo.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1);
            const length = foo.unshift(2);",
            None,
        ),
        (
            "foo().unshift(1);
            foo().unshift(2);",
            None,
        ),
        (
            "foo().bar.unshift(1);
            foo().bar.unshift(2);",
            None,
        ),
        (
            "const stream = new Readable();
            stream.unshift('one string');
            stream.unshift('another string');",
            None,
        ),
        (
            "class FooReadable extends Readable {
                unshiftAndEnd(chunk) {
                    this.unshift(chunk);
                    this.unshift(null);
                }
            }",
            None,
        ),
        (
            "class Foo {
                unshiftAndEnd(chunk) {
                    this.stream.unshift(chunk);
                    this.stream.unshift(null);
                }
            }",
            None,
        ),
        (
            "process.stdin.unshift(chunk);
            process.stdin.unshift(null);",
            None,
        ),
        (
            "process.stdout.unshift(chunk);
            process.stdout.unshift(null);",
            None,
        ),
        (
            "process.stderr.unshift(chunk);
            process.stderr.unshift(null);",
            None,
        ),
        (
            "foo.unshift(1);
            foo.unshift(2);
            foo.bar.unshift(1);
            foo.bar.unshift(2);",
            Some(serde_json::json!([ { "ignore": ["foo.unshift", "foo.bar.unshift"], }, ])),
        ),
        ("for (const _ of []) foo.unshift(bar);", None),
        (
            "function bar() {}
            foo.unshift(bindEvents);",
            None,
        ),
        (
            "foo.unshift?.(1);
            foo.unshift?.(2);",
            None,
        ),
        (
            "foo.unshift(1);
            foo.unshift?.(2);",
            None,
        ),
        (
            "foo.unshift?.(1);
            foo.unshift(2);",
            None,
        ),
        (
            "foo.classList.toggle('foo');
            foo.classList.toggle('bar');",
            None,
        ),
        (r#"foo.classList.add("foo");"#, None),
        (
            r#"foo.classList.add("foo");
            foo.classList.remove("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");; // <- there is a "EmptyStatement" between
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            bar.classList.add("bar");"#,
            None,
        ),
        (r#"foo.classList.add("foo");add("bar")"#, None),
        (r#"add("foo");foo.classList("bar")"#, None),
        (r#"new foo.classList.add("foo");foo.classList.add("bar")"#, None),
        (r#"foo.classList.add("foo");new foo.classList.add("bar")"#, None),
        (r#"foo.classList[add]("foo");foo.classList.add("bar")"#, None),
        (r#"foo.classList.add("foo");foo.classList[add]("bar");"#, None),
        (r#"foo.classList.add(foo.classList.add("foo"));"#, None),
        (
            r#"foo.classList.add("foo");
            foo[classList].add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            (new foo.classList).add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            foo.classList.add?.("bar");"#,
            None,
        ),
        (
            r#"foo.notClassList.add("foo");
            foo.notClassList.add("bar");"#,
            None,
        ),
        (
            r#"classList.add("foo");
            classList.add("bar");"#,
            None,
        ),
        (
            r#"const _ = foo.classList.add("foo");
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            const _ = foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo().classList.add("foo");
            foo().classList.add("bar");"#,
            None,
        ),
        (
            r#"foo().bar.classList.add("foo");
            foo().bar.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList?.add("foo");
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            foo.classList?.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add?.("foo");
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            foo.classList.add?.("bar");"#,
            None,
        ),
        (
            r#"foo.classList?.remove("foo");
            foo.classList.remove("bar");"#,
            None,
        ),
        (
            r#"foo.classList.remove("foo");
            foo.classList?.remove("bar");"#,
            None,
        ),
        (
            r#"foo.classList.remove?.("foo");
            foo.classList.remove("bar");"#,
            None,
        ),
        (
            r#"foo.classList.remove("foo");
            foo.classList.remove?.("bar");"#,
            None,
        ),
        (
            "importScripts('foo.js');
            notImportScripts('bar.js');",
            None,
        ),
        (r#"importScripts("foo.js");"#, None),
        (
            r#"importScripts("foo.js");; // <- there is a "EmptyStatement" between
            importScripts("bar.js");"#,
            None,
        ),
        (r#"new importScripts("foo.js");importScripts("bar.js")"#, None),
        (r#"importScripts("foo.js");new importScripts("bar.js")"#, None),
        (
            r#"const _ = importScripts("foo.js");
            importScripts("bar.js");"#,
            None,
        ),
        (
            r#"importScripts("foo.js");
            const _ = importScripts("bar.js");"#,
            None,
        ),
        (
            r#"importScripts("foo.js");
            importScripts("bar.js");"#,
            Some(serde_json::json!([ { "ignore": ["importScripts"], }, ])),
        ),
        (
            "'1'.someMagicPropertyReturnsAnArray.push(1);
            (1).someMagicPropertyReturnsAnArray.push(2);
            /a/i.someMagicPropertyReturnsAnArray.push(1);
            /b/g.someMagicPropertyReturnsAnArray.push(2);
            1n.someMagicPropertyReturnsAnArray.push(1);
            2n.someMagicPropertyReturnsAnArray.push(2);
            (true).someMagicPropertyReturnsAnArray.push(1);
            (false).someMagicPropertyReturnsAnArray.push(2);",
            None,
        ),
    ];

    let fail = vec![
        (
            "foo.push(1);
            foo.push(2);",
            None,
        ),
        (
            "(foo.push)(1);
            (foo.push)(2);",
            None,
        ),
        (
            "foo.bar.push(1);
            foo.bar.push(2);",
            None,
        ),
        (
            "foo.push(1);
            (foo).push(2);",
            None,
        ),
        (
            "foo.push();
            foo.push();",
            None,
        ),
        (
            "foo.push(1);
            foo.push();",
            None,
        ),
        (
            "foo.push();
            foo.push(2);",
            None,
        ),
        (
            "foo.push(1, 2);
            foo.push((3), (4));",
            None,
        ),
        (
            "foo.push(1, 2,);
            foo.push(3, 4);",
            None,
        ),
        (
            "foo.push(1, 2);
            foo.push(3, 4,);",
            None,
        ),
        (
            "foo.push(1, 2,);
            foo.push(3, 4,);",
            None,
        ),
        (
            "foo.push(1, 2, ...a,);
            foo.push(...b,);",
            None,
        ),
        (
            "foo.push(bar());
            foo.push(1);",
            None,
        ),
        (
            "foo.push(1);
            foo.push(bar());",
            None,
        ),
        (
            "foo.push(1,);
            foo.push(2,);
            foo.push(3,);",
            None,
        ),
        (
            "if (a) {
                foo.push(1);
                foo.push(2);
            }",
            None,
        ),
        (
            "switch (a) {
                default:
                    foo.push(1);
                    foo.push(2);
            }",
            None,
        ),
        (
            "function a() {
                foo.push(1);
                foo.push(2);
            }",
            None,
        ),
        (
            "foo.push(1)
            foo.push(2)
            ;[foo].forEach(bar)",
            None,
        ),
        (
            "foo.bar.push(1);
            (foo)['bar'].push(2);",
            None,
        ),
        (
            "foo.push(1);
            foo.push(2);
            stream.push(1);
            stream.push(2);",
            None,
        ),
        (
            "foo.bar.push(1);
            foo.bar.push(2);
            foo.push(1);
            foo.push(2);
            bar.foo.push(1);
            bar.foo.push(2);",
            Some(serde_json::json!([ { "ignore": ["foo", "foo.bar"], }, ])),
        ),
        (
            "foo.push(1);
            foo?.push(2);",
            None,
        ),
        (
            "foo?.push(1);
            foo.push(2);",
            None,
        ),
        (
            "foo?.push(1);
            foo?.push(2);",
            None,
        ),
        (
            "foo?.bar.push(1);
            foo?.bar.push(2);",
            None,
        ),
        (
            "(foo as any[]).push(1);
            (foo as any[]).push(2);",
            None,
        ), // {"parser": parsers.typescript},
        (
            "foo!.push(1);
            foo!.push(2);",
            None,
        ), // {"parser": parsers.typescript},
        (
            "foo.unshift(1);
            foo.unshift(2);",
            None,
        ),
        (
            "(foo.unshift)(1);
            (foo.unshift)(2);",
            None,
        ),
        (
            "foo.bar.unshift(1);
            foo.bar.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1);
            (foo).unshift(2);",
            None,
        ),
        (
            "bar()
            foo.unshift(1);
            (foo).unshift(2);",
            None,
        ),
        (
            "foo.unshift();
            foo.unshift();",
            None,
        ),
        (
            "foo.unshift(1);
            foo.unshift();",
            None,
        ),
        (
            "foo.unshift();
            foo.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1, 2);
            foo.unshift((3), (4));",
            None,
        ),
        (
            "foo.unshift(1, 2,);
            foo.unshift(3, 4);",
            None,
        ),
        (
            "foo.unshift(1, 2);
            foo.unshift(3, 4,);",
            None,
        ),
        (
            "foo.unshift(1, 2,);
            foo.unshift(3, 4,);",
            None,
        ),
        (
            "foo.unshift(1, 2, ...a,);
            foo.unshift(...b,);",
            None,
        ),
        (
            "foo.unshift(bar());
            foo.unshift(1);",
            None,
        ),
        (
            "foo.unshift(1);
            foo.unshift(bar());",
            None,
        ),
        (
            "foo.unshift(x);
            foo.unshift(foo.length);",
            None,
        ),
        (
            "foo.unshift(1);
            // Keep this comment
            foo.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1,);
            foo.unshift(2,);
            foo.unshift(3,);",
            None,
        ),
        (
            "if (a) {
                foo.unshift(1);
                foo.unshift(2);
            }",
            None,
        ),
        (
            "switch (a) {
                default:
                    foo.unshift(1);
                    foo.unshift(2);
            }",
            None,
        ),
        (
            "function a() {
                foo.unshift(1);
                foo.unshift(2);
            }",
            None,
        ),
        (
            "foo.unshift(1)
            foo.unshift(2)
            ;[foo].forEach(bar)",
            None,
        ),
        (
            "foo.bar.unshift(1);
            (foo)['bar'].unshift(2);",
            None,
        ),
        (
            "foo.unshift(1);
            foo?.unshift(2);",
            None,
        ),
        (
            "foo.unshift(1);
            foo?.unshift(2,);",
            None,
        ),
        (
            "foo?.unshift(1);
            foo.unshift(2);",
            None,
        ),
        (
            "foo?.unshift(1);
            foo?.unshift(2);",
            None,
        ),
        (
            "foo?.bar.unshift(1);
            foo?.bar.unshift(2);",
            None,
        ),
        (
            "(foo as any[]).unshift(1);
            (foo as any[]).unshift(2);",
            None,
        ), // {"parser": parsers.typescript},
        (
            "foo!.unshift(1);
            foo!.unshift(2);",
            None,
        ), // {"parser": parsers.typescript},
        (
            r#"foo.classList.add("foo");
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.remove("foo");
            foo.classList.remove("bar");"#,
            None,
        ),
        (
            r#"(foo.classList.add)("foo");
            (foo.classList.add)("bar");"#,
            None,
        ),
        (
            r#"foo.bar.classList.add("foo");
            foo.bar.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            (foo).classList.add("bar");"#,
            None,
        ),
        (
            "foo.classList.add();
            foo.classList.add();",
            None,
        ),
        (
            r#"foo.classList.add("foo");
            foo.classList.add();"#,
            None,
        ),
        (
            "foo.classList.add();
            foo.classList.add(2);",
            None,
        ),
        (
            "foo.classList.add(a, b);
            foo.classList.add((c), (d));",
            None,
        ),
        (
            "foo.classList.add.push(a, b,);
            foo.classList.add.push(c, d);",
            None,
        ),
        (
            "foo.classList.add(a, b);
            foo.classList.add(c, d,);",
            None,
        ),
        (
            "foo.classList.add(a, b,);
            foo.classList.add(c, d,);",
            None,
        ),
        (
            "foo.classList.add(a, b, ...c,);
            foo.classList.add(...d,);",
            None,
        ),
        (
            r#"foo.classList.add(bar());
            foo.classList.add("foo");"#,
            None,
        ),
        (
            "foo.classList.add(a);
            foo.classList.add(bar());",
            None,
        ),
        (
            "foo.classList.add(a,);
            foo.classList.add(b,);
            foo.classList.add(c,);",
            None,
        ),
        (
            "if (a) {
                foo.classList.add(a);
                foo.classList.add(b);
            }",
            None,
        ),
        (
            "switch (a) {
                default:
                    foo.classList.add(a);
                    foo.classList.add(b);
            }",
            None,
        ),
        (
            "function _() {
                foo.classList.add(a);
                foo.classList.add(b);
            }",
            None,
        ),
        (
            "foo.classList.add(a)
            foo.classList.add(b)
            ;[foo].forEach(bar)",
            None,
        ),
        (
            "foo.bar.classList.add(a);
            (foo)['bar'].classList.add(b);",
            None,
        ),
        (
            r#"foo?.classList.add("foo");
            foo.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo.classList.add("foo");
            foo?.classList.add("bar");"#,
            None,
        ),
        (
            r#"foo?.classList.add("foo");
            foo?.classList.add("bar");"#,
            None,
        ),
        (
            r#"importScripts("foo.js");
            importScripts("bar.js");"#,
            None,
        ),
        (
            r#"(importScripts)("foo.js");
            (importScripts)("bar.js");"#,
            None,
        ),
        (
            "importScripts();
            importScripts();",
            None,
        ),
        (
            r#"importScripts("foo.js");
            importScripts();"#,
            None,
        ),
        (
            "importScripts();
            importScripts(2);",
            None,
        ),
        (
            "importScripts(a, b);
            importScripts((c), (d));",
            None,
        ),
        (
            "importScripts(a, b,);
            importScripts(c, d);",
            None,
        ),
        (
            "importScripts(a, b);
            importScripts(c, d,);",
            None,
        ),
        (
            "importScripts(a, b,);
            importScripts(c, d,);",
            None,
        ),
        (
            "foo.classList.add(a, b, ...c,);
            foo.classList.add(...d,);",
            None,
        ),
        (
            r#"importScripts(bar());
            importScripts("foo.js");"#,
            None,
        ),
        (
            "importScripts(a);
            importScripts(bar());",
            None,
        ),
        (
            "importScripts(a,);
            importScripts(b,);
            importScripts(c,);",
            None,
        ),
        (
            "if (a) {
                importScripts(a);
                importScripts(b);
            }",
            None,
        ),
        (
            "switch (a) {
                default:
                    importScripts(a);
                    importScripts(b);
            }",
            None,
        ),
        (
            "function _() {
                importScripts(a);
                importScripts(b);
            }",
            None,
        ),
        (
            "importScripts(a)
            importScripts(b)
            ;[foo].forEach(bar)",
            None,
        ),
        (
            r#"importScripts?.("foo.js");
            importScripts("bar.js");"#,
            None,
        ),
        (
            r#"importScripts("foo.js");
            importScripts?.("bar.js");"#,
            None,
        ),
        (
            r#"importScripts?.("foo.js");
            importScripts?.("bar.js");"#,
            None,
        ),
        (
            "class A extends B {
                foo() {
                    this.x.push(1);
                    this.x.push(2);
                    super.x.push(1);
                    super.x.push(2);
                    ((a?.x).y).push(1);
                    (a.x?.y).push(1);
                    ((a?.x.y).z).push(1);
                    ((a.x?.y).z).push(1);
                    a[null].push(1);
                    a['null'].push(1);
                    '1'.someMagicPropertyReturnsAnArray.push(1);
                    '1'.someMagicPropertyReturnsAnArray.push(2);
                    /a/i.someMagicPropertyReturnsAnArray.push(1);
                    /a/i.someMagicPropertyReturnsAnArray.push(2);
                    1n.someMagicPropertyReturnsAnArray.push(1);
                    1n.someMagicPropertyReturnsAnArray.push(2);
                    (true).someMagicPropertyReturnsAnArray.push(1);
                    (true).someMagicPropertyReturnsAnArray.push(2);
                }
            }",
            None,
        ),
        (
            "a[x].push(1);
            a[x].push(2);",
            None,
        ),
    ];

    let fix = vec![
        (
            "class A extends B {
                foo() {
                    this.x.push(1);
                    this.x.push(2);
                    super.x.push(1);
                    super.x.push(2);
                    ((a?.x).y).push(1);
                    (a.x?.y).push(1);
                    ((a?.x.y).z).push(1);
                    ((a.x?.y).z).push(1);
                    a[null].push(1);
                    a['null'].push(1);
                    '1'.someMagicPropertyReturnsAnArray.push(1);
                    '1'.someMagicPropertyReturnsAnArray.push(2);
                    /a/i.someMagicPropertyReturnsAnArray.push(1);
                    /a/i.someMagicPropertyReturnsAnArray.push(2);
                    1n.someMagicPropertyReturnsAnArray.push(1);
                    1n.someMagicPropertyReturnsAnArray.push(2);
                    (true).someMagicPropertyReturnsAnArray.push(1);
                    (true).someMagicPropertyReturnsAnArray.push(2);
                }
            }",
            "class A extends B {
                foo() {
                    this.x.push(1, 2);
                    super.x.push(1, 2);
                    ((a?.x).y).push(1, 1);
                    ((a?.x.y).z).push(1, 1);
                    a[null].push(1, 1);
                    '1'.someMagicPropertyReturnsAnArray.push(1, 2);
                    /a/i.someMagicPropertyReturnsAnArray.push(1, 2);
                    1n.someMagicPropertyReturnsAnArray.push(1, 2);
                    (true).someMagicPropertyReturnsAnArray.push(1, 2);
                }
            }",
        ),
        (
            "a[x].push(1);
            a[x].push(2);",
            "a[x].push(1, 2);",
        ),
    ];

    Tester::new(PreferSingleCall::NAME, PreferSingleCall::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
