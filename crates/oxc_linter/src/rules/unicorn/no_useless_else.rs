use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, CallExpression, Expression, LogicalOperator, Statement,
        SwitchStatement, TryStatement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_else_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `else` after a statement that exits.")
        .with_help(
            "Remove the `else` block and move its contents outside of the `if` statement, since the preceding branch already always exits.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessElse;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `else` blocks after `if` blocks that always exit, whether
    /// through a `return`, `throw`, `break`, or `continue` statement, or a
    /// call to `process.exit()`.
    ///
    /// ### Why is this bad?
    ///
    /// When an `if` block always exits, the `else` block becomes unnecessary
    /// and its contents can be placed outside the `if` statement, reducing
    /// nesting and improving readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo() {
    ///     if (a) {
    ///         return 1;
    ///     } else {
    ///         return 2;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function foo() {
    ///     if (a) {
    ///         return 1;
    ///     }
    ///
    ///     return 2;
    /// }
    /// ```
    NoUselessElse,
    unicorn,
    style,
    pending,
    version = "next",
    short_description = "Disallow `else` blocks after `if` blocks that always exit.",
);

/// Whether a piece of control flow definitely diverts execution away from
/// the statements that follow it, and if so, in which way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    /// Execution may continue past this point; nothing is proven.
    Fallthrough,
    /// Execution definitely leaves the nearest enclosing `switch` via an
    /// unlabeled `break`, without necessarily exiting any further.
    BreaksSwitch,
    /// Execution definitely diverts away, via `return`, `throw`, a labeled
    /// `break`/`continue`, an unlabeled `continue`, or a call to
    /// `process.exit()`.
    Exits,
}

impl Flow {
    fn rank(self) -> u8 {
        match self {
            Flow::Fallthrough => 0,
            Flow::BreaksSwitch => 1,
            Flow::Exits => 2,
        }
    }
}

/// The flow that holds when either of two independent paths may be taken:
/// the weaker (less certain) of the two, since both paths must exit for the
/// combination to be considered a guaranteed exit.
fn weaker(a: Flow, b: Flow) -> Flow {
    if a.rank() <= b.rank() { a } else { b }
}

fn always_exits(stmt: &Statement, ctx: &LintContext) -> bool {
    analyze_stmt(stmt, false, false, ctx) == Flow::Exits
        || analyze_stmt(stmt, false, true, ctx) == Flow::Exits
}

fn analyze_sequence(
    stmts: &[Statement],
    in_switch: bool,
    hard_only: bool,
    ctx: &LintContext,
) -> Flow {
    // In `hard_only` mode we're proving that a call to `process.exit()` is
    // reached unconditionally, in order to bypass a `catch` block that
    // doesn't itself guarantee an exit (since `process.exit()` can't be
    // caught). That guarantee doesn't hold past the first statement: any
    // earlier statement might itself throw before we get there, and if it
    // does, the (non-exiting) `catch` would run instead. So only the
    // sequence's first statement is eligible to prove a hard exit.
    if hard_only {
        return stmts
            .first()
            .map_or(Flow::Fallthrough, |stmt| analyze_stmt(stmt, in_switch, true, ctx));
    }

    for stmt in stmts {
        let flow = analyze_stmt(stmt, in_switch, false, ctx);
        if flow != Flow::Fallthrough {
            return flow;
        }
    }
    Flow::Fallthrough
}

fn analyze_stmt(stmt: &Statement, in_switch: bool, hard_only: bool, ctx: &LintContext) -> Flow {
    match stmt {
        Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::ContinueStatement(_) => {
            if hard_only {
                Flow::Fallthrough
            } else {
                Flow::Exits
            }
        }
        Statement::BreakStatement(break_stmt) => {
            if hard_only {
                Flow::Fallthrough
            } else if break_stmt.label.is_none() && in_switch {
                Flow::BreaksSwitch
            } else {
                Flow::Exits
            }
        }
        Statement::BlockStatement(block) => {
            analyze_sequence(&block.body, in_switch, hard_only, ctx)
        }
        Statement::LabeledStatement(labeled) => {
            analyze_stmt(&labeled.body, in_switch, hard_only, ctx)
        }
        Statement::WhileStatement(while_stmt) => {
            if is_bool_literal(&while_stmt.test, true) && !contains_break(&while_stmt.body) {
                Flow::Exits
            } else {
                Flow::Fallthrough
            }
        }
        Statement::DoWhileStatement(do_while) => {
            if is_bool_literal(&do_while.test, true) && !contains_break(&do_while.body) {
                Flow::Exits
            } else {
                Flow::Fallthrough
            }
        }
        Statement::ForStatement(for_stmt) => {
            let infinite_test =
                for_stmt.test.as_ref().is_none_or(|test| is_bool_literal(test, true));
            if infinite_test && !contains_break(&for_stmt.body) {
                Flow::Exits
            } else {
                Flow::Fallthrough
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if expr_always_calls_process_exit(&expr_stmt.expression, hard_only, ctx) {
                Flow::Exits
            } else {
                Flow::Fallthrough
            }
        }
        Statement::IfStatement(if_stmt) => {
            if is_bool_literal(&if_stmt.test, true) {
                return analyze_stmt(&if_stmt.consequent, in_switch, hard_only, ctx);
            }
            if is_bool_literal(&if_stmt.test, false) {
                return if_stmt
                    .alternate
                    .as_ref()
                    .map_or(Flow::Fallthrough, |alt| analyze_stmt(alt, in_switch, hard_only, ctx));
            }
            let Some(alternate) = &if_stmt.alternate else { return Flow::Fallthrough };
            let consequent_flow = analyze_stmt(&if_stmt.consequent, in_switch, hard_only, ctx);
            let alternate_flow = analyze_stmt(alternate, in_switch, hard_only, ctx);
            weaker(consequent_flow, alternate_flow)
        }
        Statement::TryStatement(try_stmt) => analyze_try(try_stmt, in_switch, hard_only, ctx),
        Statement::SwitchStatement(switch_stmt) => analyze_switch(switch_stmt, hard_only, ctx),
        _ => Flow::Fallthrough,
    }
}

fn analyze_try(
    try_stmt: &TryStatement,
    in_switch: bool,
    hard_only: bool,
    ctx: &LintContext,
) -> Flow {
    if hard_only {
        let try_flow = analyze_sequence(&try_stmt.block.body, in_switch, true, ctx);
        let finally_flow = try_stmt
            .finalizer
            .as_ref()
            .map_or(Flow::Fallthrough, |f| analyze_sequence(&f.body, in_switch, true, ctx));
        if try_flow == Flow::Exits || finally_flow == Flow::Exits {
            Flow::Exits
        } else {
            Flow::Fallthrough
        }
    } else {
        let finally_flow = try_stmt
            .finalizer
            .as_ref()
            .map_or(Flow::Fallthrough, |f| analyze_sequence(&f.body, in_switch, false, ctx));
        if finally_flow != Flow::Fallthrough {
            return finally_flow;
        }
        let try_flow = analyze_sequence(&try_stmt.block.body, in_switch, false, ctx);
        // Without a `catch`, an exception thrown from `try` always propagates,
        // so the absence of a handler behaves like a guaranteed exit.
        let catch_flow = try_stmt.handler.as_ref().map_or(Flow::Exits, |handler| {
            analyze_sequence(&handler.body.body, in_switch, false, ctx)
        });
        weaker(try_flow, catch_flow)
    }
}

fn analyze_switch(switch_stmt: &SwitchStatement, hard_only: bool, ctx: &LintContext) -> Flow {
    if !switch_stmt.cases.iter().any(|case| case.test.is_none()) {
        // No `default` clause: some values may not match any case.
        return Flow::Fallthrough;
    }

    let mut fallthrough_target = Flow::Fallthrough;
    let mut overall = Flow::Exits;
    for case in switch_stmt.cases.iter().rev() {
        let this_flow = analyze_sequence(&case.consequent, true, hard_only, ctx);
        let resolved = if this_flow == Flow::Fallthrough { fallthrough_target } else { this_flow };
        fallthrough_target = resolved;
        overall = weaker(overall, resolved);
    }

    // From the outside, a `break` only escapes this `switch`; it doesn't
    // propagate any further, so it's indistinguishable from falling through.
    if overall == Flow::Exits { Flow::Exits } else { Flow::Fallthrough }
}

fn is_bool_literal(expr: &Expression, value: bool) -> bool {
    matches!(expr.without_parentheses(), Expression::BooleanLiteral(b) if b.value == value)
}

/// Conservatively checks whether `stmt` contains a `break` statement
/// anywhere in its subtree (labeled or not), used to tell whether a loop
/// with a statically-true test could ever be exited. Over-detecting a
/// `break` (e.g. one that actually targets a differently-labeled outer
/// loop) is safe here: it just means an eternal loop goes unrecognized,
/// never a false positive.
fn contains_break(stmt: &Statement) -> bool {
    match stmt {
        Statement::BreakStatement(_) => true,
        Statement::BlockStatement(block) => block.body.iter().any(contains_break),
        Statement::IfStatement(if_stmt) => {
            contains_break(&if_stmt.consequent)
                || if_stmt.alternate.as_ref().is_some_and(contains_break)
        }
        Statement::TryStatement(try_stmt) => {
            try_stmt.block.body.iter().any(contains_break)
                || try_stmt
                    .handler
                    .as_ref()
                    .is_some_and(|handler| handler.body.body.iter().any(contains_break))
                || try_stmt
                    .finalizer
                    .as_ref()
                    .is_some_and(|finalizer| finalizer.body.iter().any(contains_break))
        }
        Statement::SwitchStatement(switch_stmt) => {
            switch_stmt.cases.iter().any(|case| case.consequent.iter().any(contains_break))
        }
        Statement::LabeledStatement(labeled) => contains_break(&labeled.body),
        Statement::WhileStatement(while_stmt) => contains_break(&while_stmt.body),
        Statement::DoWhileStatement(do_while) => contains_break(&do_while.body),
        Statement::ForStatement(for_stmt) => contains_break(&for_stmt.body),
        Statement::ForInStatement(for_in) => contains_break(&for_in.body),
        Statement::ForOfStatement(for_of) => contains_break(&for_of.body),
        _ => false,
    }
}

/// Whether evaluating `expr` as a standalone expression statement is
/// guaranteed to call `process.exit(...)`, taking short-circuiting
/// (`&&`, `||`, `??`, `?:`) into account.
///
/// When `require_safe_args` is set, the call's arguments must themselves be
/// incapable of throwing (only literals or bare identifiers), since a call
/// to `process.exit()` that never actually gets reached because one of its
/// arguments threw first can still be caught by an enclosing `catch`.
fn expr_always_calls_process_exit(
    expr: &Expression,
    require_safe_args: bool,
    ctx: &LintContext,
) -> bool {
    match expr.without_parentheses() {
        Expression::CallExpression(call) => {
            is_process_exit_call(call, ctx)
                && (!require_safe_args || call.arguments.iter().all(is_safe_argument))
        }
        Expression::LogicalExpression(logical) => match logical.operator {
            LogicalOperator::And => {
                is_bool_literal(&logical.left, true)
                    && expr_always_calls_process_exit(&logical.right, require_safe_args, ctx)
            }
            LogicalOperator::Or => {
                is_bool_literal(&logical.left, false)
                    && expr_always_calls_process_exit(&logical.right, require_safe_args, ctx)
            }
            LogicalOperator::Coalesce => false,
        },
        Expression::ConditionalExpression(cond) => {
            expr_always_calls_process_exit(&cond.consequent, require_safe_args, ctx)
                && expr_always_calls_process_exit(&cond.alternate, require_safe_args, ctx)
        }
        Expression::ArrayExpression(array) => array.elements.iter().any(|el| {
            el.as_expression()
                .is_some_and(|e| expr_always_calls_process_exit(e, require_safe_args, ctx))
                || matches!(
                    el,
                    ArrayExpressionElement::SpreadElement(spread)
                        if expr_always_calls_process_exit(&spread.argument, require_safe_args, ctx)
                )
        }),
        _ => false,
    }
}

fn is_safe_argument(arg: &Argument) -> bool {
    let Some(expr) = arg.as_expression() else { return false };
    matches!(
        expr.without_parentheses(),
        Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::Identifier(_)
    )
}

fn is_process_exit_call(call: &CallExpression, ctx: &LintContext) -> bool {
    if call.optional {
        return false;
    }
    let Expression::StaticMemberExpression(member) = &call.callee else { return false };
    if member.optional || member.property.name != "exit" {
        return false;
    }
    let Expression::Identifier(object) = &member.object else { return false };
    object.name == "process" && object.is_global_reference(ctx.scoping())
}

impl Rule for NoUselessElse {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };
        let Some(alternate) = &if_stmt.alternate else {
            return;
        };
        let siblings: &[Statement] = match ctx.nodes().parent_kind(node.id()) {
            AstKind::Program(program) => &program.body,
            AstKind::BlockStatement(block) => &block.body,
            AstKind::StaticBlock(block) => &block.body,
            AstKind::SwitchCase(case) => &case.consequent,
            AstKind::FunctionBody(body) => &body.statements,
            _ => return,
        };
        // Skip if this `if` statement is unreachable because an earlier
        // sibling in the same statement list always exits first.
        for sibling in siblings {
            if sibling.span() == if_stmt.span {
                break;
            }
            if always_exits(sibling, ctx) {
                return;
            }
        }

        if always_exits(&if_stmt.consequent, ctx) {
            let else_span = Span::new(if_stmt.consequent.span().end, alternate.span().start);
            ctx.diagnostic(no_useless_else_diagnostic(else_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function qux() {
                if (foo) {
                    return;
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        do {} while (maybeThrow(), process.exit(1));
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        throw error;
                        process.exit(1);
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        process.exit(maybeThrow());
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        process.exit((maybeThrow(), 1));
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        maybeThrow();
                        process.exit();
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "if (foo) {
                bar();
            } else {
                baz();
            }",
        "const MODE_GROUP = new Set(['foo', 'bar']);
            const args = new Map();
            const modes = new Set();
            if (args.has('modes')) {
                for (const mode of args.get('modes').split(',')) {
                    const trimmed = mode.trim();
                    if (MODE_GROUP.has(trimmed)) {
                        modes.add(trimmed);
                    }
                }
                if (!modes.size) {
                    throw new Error('No valid modes');
                }
            } else {
                for (const mode of MODE_GROUP) {
                    modes.add(mode);
                }
            }",
        "const modes = new Set(['foo']);
            modes.clear();
            if (condition) {
                modes.size ? process.exit() : doSomething();
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { return false; }});
            if (condition) {
                if (object.value) {
                    throw new Error();
                }
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    if (object.value && true) {
                        throw new Error();
                    }
                } catch {}
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    if (object?.value) {
                        throw new Error();
                    }
                } catch {}
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            const getterKey = 'value';
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    if (object[getterKey]) {
                        throw new Error();
                    }
                } catch {}
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    object.value(process.exit(1));
                } catch {}
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    object.value.foo(process.exit(1));
                } catch {}
            } else {
                doSomethingElse();
            }",
        "const object = {value: true};
            Object.defineProperty(object, 'value', {get() { throw new Error(); }});
            if (condition) {
                try {
                    (object?.value)[process.exit(1)];
                } catch {}
            } else {
                doSomethingElse();
            }",
        "let modes = new Set(['foo']);
            if (condition) {
                if ((modes = new Set()).size) {
                    throw new Error();
                }
            } else {
                doSomethingElse();
            }",
        "function qux() {
                const modes = new Set(['foo']);
                modes.clear();
                if (foo) {
                    if (modes.size) {
                        return;
                    }
                    bar();
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    if (bar) {
                        return;
                    }
                } else {
                    baz();
                }
            }",
        "const value = foo
                ? bar()
                : baz();",
        "function qux() {
                if (foo) {
                    try {
                        return bar();
                    } catch {}
                } else {
                    baz();
                }
            }",
        "if (foo) {
                try {
                    class Example {
                        static {
                            maybeThrow();
                            process.exit(1);
                        }
                    }
                } catch {}
            } else {
                baz();
            }",
        "if (foo) {
                while (bar) {
                    break;
                }
            } else {
                baz();
            }",
        "if (foo) {
                switch (bar) {
                    case baz:
                        break;
                }
            } else {
                qux();
            }",
        "function qux() {
                if (foo) {
                    switch (bar) {
                        case 1:
                            return;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (bar) {
                        case 1:
                            qux();
                            break;
                        default:
                            return;
                    }
                } else {
                    baz();
                }
            }",
        "for (const foo of bar)
                if (foo) {
                    continue;
                } else {
                    baz();
                }",
        "function qux() {
                if (foo) {
                    outer: for (const a of b) {
                        for (const c of d) {
                            break outer;
                        }
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (bar) {
                        case 1:
                            return;
                        default:
                            break;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (value) {
                        case 1:
                            break;
                    default:
                        break;
                    case process.exit(1):
                        {}
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (value) {
                        case 1:
                            return;
                    default:
                        break;
                    case process.exit(1):
                        {}
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        doSomething();
                    } catch {
                        handle();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    if (inner) {
                        return;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    do {
                        if (bar) {
                            break;
                        }
                    } while (true);
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    for (const x of xs) {
                        return x;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        return doSomething();
                    } catch {
                        handle();
                    } finally {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    const inner = () => {
                        return 1;
                    };
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    function inner() {
                        return 1;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                return;
                if (foo) {
                    return;
                } else {
                    baz();
                }
            }",
        "function qux() {
                return;
                if (foo) {
                    process.exit();
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    false && process.exit();
                } else {
                    baz();
                }
            }",
        "function qux(process) {
                if (foo) {
                    process.exit();
                } else {
                    baz();
                }
            }",
        "import process from 'node:process';
            if (foo) {
                process.exit();
            } else {
                baz();
            }",
        "if (foo) {
                process.exitCode = 1;
            } else {
                baz();
            }",
        "if (foo) {
                process?.exit();
            } else {
                baz();
            }",
        "if (foo) {
                process.exit?.();
            } else {
                baz();
            }",
        "if (foo) {
                callback?.(process.exit(1));
            } else {
                baz();
            }",
        "if (foo) {
                foo?.bar[process.exit(1)];
            } else {
                baz();
            }",
        "if (foo) {
                process['exit']();
            } else {
                baz();
            }",
        "if (foo) {
                lib.process.exit();
            } else {
                baz();
            }",
        "if (foo) {
                new process.exit();
            } else {
                baz();
            }",
        "function qux() {
                if (foo) {
                    if (bar) {
                        process.exit();
                    }
                } else {
                    baz();
                }
            }",
        "function qux(value) {
                if (foo) {
                    try {
                        switch (value) {
                        default:
                            break;
                        case process.exit(1):
                            {}
                        }
                    } catch {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
    ];

    let fail = vec![
        "function qux(condition) {
                if (foo) {
                    try {
                        condition ? process.exit(1) : process.exit(2);
                    } catch {}
                } else {
                    baz();
                }
            }",
        "function qux(condition) {
                if (foo) {
                    try {
                        if (condition) {
                            process.exit(1);
                        } else {
                            process.exit(2);
                        }
                    } catch {}
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        return bar();
                    } finally {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (bar) {
                        case 1:
                            return;
                        default:
                            throw new Error();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    while (true) {
                        doSomething();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    for (;;) {
                        doSomething();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    do {
                        doSomething();
                    } while (true);
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (bar) {
                        case 1:
                        case 2:
                            return;
                        default:
                            throw new Error();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        return doSomething();
                    } catch {
                        throw new Error();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    if (a) {
                        return;
                    } else if (b) {
                        throw new Error();
                    } else {
                        return;
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                outer: for (const a of b) {
                    if (foo) {
                        continue outer;
                    } else {
                        baz();
                    }
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        doSomething();
                    } finally {
                        return cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else { bar(); }
            }",
        "if (foo) {
                throw new Error();
            } else {
                bar();
            }",
        "function qux() {
                if (false) {
                    return;
                } else {
                    bar();
                }
            }",
        "function qux() {
                if (true) {
                    return;
                } else {
                    bar();
                }
            }",
        "while (foo) {
                if (bar) {
                    break;
                } else {
                    baz();
                }
            }",
        "while (foo) {
                if (bar) {
                    continue;
                } else {
                    baz();
                }
            }",
        "switch (foo) {
                case bar:
                    if (baz) {
                        break;
                    } else {
                        qux();
                    }
            }",
        "class Foo {
                static {
                    if (foo) {
                        throw new Error();
                    } else {
                        bar();
                    }
                }
            }",
        "function qux() {
                if (foo)
                    return;
                else
                    bar();
            }",
        "function qux() {
                if (foo) {
                    return;
                } else if (bar) {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    if (bar) {
                        return;
                    } else {
                        throw new Error();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    // Keep this comment.
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    /*
                    Keep this comment.
                    */
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    const bar = 1;
                    baz(bar);
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    let bar = 1;
                    baz(bar);
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    class Bar {}
                    baz(Bar);
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    function bar() {}
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else /* comment */ {
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } /* comment */ else {
                    bar();
                }
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    bar();
                } // trailing
                baz();
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    bar()
                } baz()
            }",
        "function qux() {
                if (foo) {
                    return;
                } else {
                    bar = function() {}
                }
                (baz)();
            }",
        "function qux() {
                if (foo)
                    return
                else
                    [bar].forEach(baz)
            }",
        "function qux() {
                if (foo) {
                    process.exit();
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    true && process.exit();
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    [...process.exit()];
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo)
                    process.exit(1);
                else
                    baz();
            }",
        "function qux() {
                if (foo) {
                    setup();
                    process.exit(1);
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    {
                        process.exit();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (value) {
                        case 1:
                            if (false) {
                                break;
                            }
                            process.exit(1);
                        default:
                            process.exit(2);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (value) {
                        case 1:
                            if (true) {
                                return;
                            }
                            break;
                        default:
                            process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    label: {
                        if (false) {
                            break label;
                        }
                        process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    process.exit(1);
                } else if (bar) {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        process.exit(1);
                        cleanup();
                    } finally {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    if (bar) {
                        return;
                    } else {
                        process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        throw error;
                    } catch {
                        process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    switch (value) {
                        case 1:
                            local: {
                                break local;
                            }
                            process.exit(1);
                        default:
                            process.exit(2);
                    }
                } else {
                    baz();
                }
            }",
        "outer: while (condition) {
                if (foo) {
                    switch (value) {
                        case 1:
                            break outer;
                    default:
                            process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "outer: while (condition) {
                if (foo) {
                    switch (value) {
                        case 1:
                            continue outer;
                    default:
                            process.exit(1);
                    }
                } else {
                    baz();
                }
            }",
        "function qux() {
                if (foo) {
                    try {
                        process.exit();
                    } finally {
                        cleanup();
                    }
                } else {
                    baz();
                }
            }",
    ];

    Tester::new(NoUselessElse::NAME, NoUselessElse::PLUGIN, pass, fail).test_and_snapshot();
}
