use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_break_in_nested_loop_diagnostic(span: Span, keyword: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Move this nested loop or switch into a function instead of using `{keyword}` here."
    ))
    .with_label(span)
}

fn switch_continue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "An unlabeled `continue` inside a `switch` continues the surrounding loop, not the next \
         `case`. Use a labeled `continue` if that is intentional.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoBreakInNestedLoop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `break` and `continue` in nested loops and switches inside loops.
    ///
    /// ### Why is this bad?
    ///
    /// Control flow statements that affect nested loops make it harder to understand which loop
    /// will resume or terminate. Moving the nested control flow into a function makes the exit
    /// explicit.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///     for (const child of item.children) {
    ///         if (child.done) {
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///     if (item.done) {
    ///         break;
    ///     }
    /// }
    /// ```
    NoBreakInNestedLoop,
    unicorn,
    style,
    none,
    version = "next",
    short_description = "Disallow `break` and `continue` in nested loops and switches inside loops.",
);

impl Rule for NoBreakInNestedLoop {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (jump_kind, span) = match node.kind() {
            AstKind::BreakStatement(statement) if statement.label.is_none() => {
                (JumpKind::Break, statement.span)
            }
            AstKind::ContinueStatement(statement) if statement.label.is_none() => {
                (JumpKind::Continue, statement.span)
            }
            _ => return,
        };

        match classify_jump(jump_kind, node.id(), ctx) {
            Some(Violation::NestedControlFlow) => {
                ctx.diagnostic(no_break_in_nested_loop_diagnostic(span, jump_kind.keyword()))
            }
            Some(Violation::ContinueInSwitch) => {
                ctx.diagnostic(switch_continue_diagnostic(span));
            }
            None => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JumpKind {
    Break,
    Continue,
}

impl JumpKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::Break => "break",
            Self::Continue => "continue",
        }
    }

    fn targets(self, ancestor: AstKind<'_>) -> bool {
        ancestor.is_iteration_statement()
            || (self == Self::Break && matches!(ancestor, AstKind::SwitchStatement(_)))
    }
}

enum Violation {
    NestedControlFlow,
    ContinueInSwitch,
}

fn classify_jump(jump_kind: JumpKind, node_id: NodeId, ctx: &LintContext<'_>) -> Option<Violation> {
    let mut has_target = false;
    let mut has_switch_before_target = false;

    for ancestor in ctx.nodes().ancestor_kinds(node_id) {
        if ancestor.is_function_like() {
            return None;
        }

        if has_target {
            if ancestor.is_iteration_statement() {
                return Some(Violation::NestedControlFlow);
            }
            continue;
        }

        if jump_kind.targets(ancestor) {
            if jump_kind == JumpKind::Continue && has_switch_before_target {
                return Some(Violation::ContinueInSwitch);
            }
            has_target = true;
        } else if matches!(ancestor, AstKind::SwitchStatement(_)) {
            has_switch_before_target = true;
        }
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (const item of items) {
                if (item.done) {
                    break;
                }
            }",
        "for (const item of items) {
                if (!item.visible) {
                    continue;
                }
            }",
        "for (const item of items) {
                for (const child of item.children) {
                    check(child);
                }
            }",
        "outer: for (const item of items) {
                for (const child of item.children) {
                    break outer;
                }
            }",
        "outer: for (const item of items) {
                for (const child of item.children) {
                    continue outer;
                }
            }",
        "for (const item of items) {
                inner: for (const child of item.children) {
                    break inner;
                }
            }",
        "label: {
                for (const item of items) {
                    switch (item.type) {
                        case 'child':
                            break label;
                    }
                }
            }",
        "outer: for (const item of items) {
                switch (item.type) {
                    case 'child':
                        continue outer;
                }
            }",
        "switch (value) {
                case 1:
                    break;
            }",
        "switch (value) {
                case 1:
                    for (const item of items) {
                        break;
                    }
            }",
        "switch (value) {
                case 1:
                    switch (otherValue) {
                        case 2:
                            break;
                    }
            }",
        "function processItem(item) {
                for (const child of item.children) {
                    break;
                }
            }
            for (const item of items) {
                processItem(item);
            }",
        "for (const item of items) {
                function processItem() {
                    for (const child of item.children) {
                        break;
                    }
                }
                processItem();
            }",
        "for (const item of items) {
                const processItem = () => {
                    for (const child of item.children) {
                        break;
                    }
                };
                processItem();
            }",
        "for (const item of items) {
                const processItem = function () {
                    while (item.pending) {
                        continue;
                    }
                };
                processItem();
            }",
    ];

    let fail = vec![
        "for (const item of items) {
                for (const child of item.children) {
                    break;
                }
            }",
        "for (const item of items) {
                while (item.children.pop()) {
                    continue;
                }
            }",
        "for (const item of items) {
                switch (item.type) {
                    case 'child':
                        break;
                }
            }",
        "for (const item of items) {
                switch (item.type) {
                    case 'child':
                        continue;
                }
            }",
        "for (const item of items) {
                for (const child of item.children) {
                    switch (child.type) {
                        case 'child':
                            continue;
                    }
                }
            }",
        "for (const item of items) {
                switch (item.type) {
                    case 'child':
                        while (item.pending) {
                            continue;
                        }
                }
            }",
        "for (let index = 0; index < items.length; index++) {
                for (const child of items[index].children) {
                    break;
                }
            }",
        "for (const key in items) {
                do {
                    continue;
                } while (items[key].pending);
            }",
        "const processItem = () => {
                for (const item of items) {
                    while (item.pending) {
                        break;
                    }
                }
            }",
    ];

    Tester::new(NoBreakInNestedLoop::NAME, NoBreakInNestedLoop::PLUGIN, pass, fail)
        .test_and_snapshot();
}
