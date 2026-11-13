use oxc_ast::{
    AstKind,
    ast::{BlockStatement, IfStatement, LabeledStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_continue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`continue` statement at the end of block")
        .with_help("Remove the redundant `continue` statement at the end of block.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessContinue;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disable usage of a `continue` statement as the last statement of a loop
    ///
    /// ### Why is this bad?
    ///
    /// A `continue` statement is used in a `for` or a `while` loop to prematurely move to the next iteration.
    /// Such a statement at the end of the loop (or at the end of an ending `if`-block) is redundant since it was about to move to the next iteration anyway.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (const item of items) {
    /// 	process(item);
    /// 	continue;
    /// }
    ///
    /// for (const item of items) {
    /// 	if (shouldSkip(item)) {
    /// 		continue;
    /// 	}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (const item of items) {
    /// 	process(item);
    /// }
    ///
    /// for (const item of items) {
    /// 	if (!shouldSkip(item)) {
    /// 		// anything other than a continue statement
    /// 	}
    /// }
    ///
    /// for (const item of items) {
    ///     if (shouldSkip(item)) {
    ///         continue;
    ///     }
    ///     process(item);
    /// }
    /// ```
    NoUselessContinue,
    unicorn,
    pedantic,
    pending,
    version = "next",
    short_description = "Detects whether there is a `continue` statement at the end of a loop.",
);

impl Rule for NoUselessContinue {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::LabeledStatement(labeled_stmt) => {
                self.check_labeled_statement(labeled_stmt, ctx);
            }
            AstKind::ForStatement(for_stmt) => {
                self.check_body(&for_stmt.body, ctx);
            }
            AstKind::ForOfStatement(for_of_stmt) => {
                self.check_body(&for_of_stmt.body, ctx);
            }
            AstKind::ForInStatement(for_in_stmt) => {
                self.check_body(&for_in_stmt.body, ctx);
            }
            AstKind::DoWhileStatement(do_while_stmt) => {
                self.check_body(&do_while_stmt.body, ctx);
            }
            AstKind::WhileStatement(while_stmt) => {
                self.check_body(&while_stmt.body, ctx);
            }
            _ => {}
        }
    }
}

impl<'a> NoUselessContinue {
    fn check_labeled_statement(&self, labeled_stmt: &LabeledStatement<'a>, ctx: &LintContext<'a>) {
        if let Statement::BlockStatement(block) = &labeled_stmt.body
            && let [Statement::ContinueStatement(_)] = block.body.as_slice()
        {
            return;
        }

        self.check_body(&labeled_stmt.body, ctx);
    }

    /// `else if` is an [`IfStatement`] stored on `alternate`, not a separate node.
    fn check_if_statement(&self, if_stmt: &IfStatement<'a>, ctx: &LintContext<'a>) {
        self.check_if_branch(&if_stmt.consequent, ctx);
        if let Some(alternate) = &if_stmt.alternate {
            self.check_if_branch(alternate, ctx);
        }
    }

    fn check_if_branch(&self, statement: &Statement<'a>, ctx: &LintContext<'a>) {
        match statement {
            Statement::BlockStatement(block) => self.check_block(block, ctx),
            Statement::IfStatement(if_stmt) => self.check_if_statement(if_stmt, ctx),
            _ => {}
        }
    }

    fn check_block(&self, block: &BlockStatement<'a>, ctx: &LintContext<'a>) {
        if block.body.len() > 0 {
            match block.body.last() {
                Some(Statement::ContinueStatement(cont_stmt)) => {
                    if let None = cont_stmt.label {
                        ctx.diagnostic(no_useless_continue_diagnostic(cont_stmt.span));
                    }
                }
                Some(Statement::IfStatement(if_stmt)) => {
                    self.check_if_statement(if_stmt, ctx);
                }
                Some(Statement::BlockStatement(block_stmt)) => {
                    self.check_block(block_stmt, ctx);
                }
                _ => {}
            }
        }
    }

    fn check_body(&self, body: &Statement<'a>, ctx: &LintContext<'a>) {
        match body {
            Statement::ForStatement(for_stmt) => {
                self.check_body(&for_stmt.body, ctx);
            }
            Statement::ForOfStatement(for_of_stmt) => {
                self.check_body(&for_of_stmt.body, ctx);
            }
            Statement::WhileStatement(while_stmt) => {
                self.check_body(&while_stmt.body, ctx);
            }
            Statement::IfStatement(if_stmt) => {
                self.check_if_statement(if_stmt, ctx);
            }
            Statement::BlockStatement(block) => {
                self.check_block(block, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (const x of xs) {
            if (skip(x)) {
                continue;
            }
            process(x);
        }",
        "for (const x of xs) {
            if (a) {
                continue;
            }
            doMore();
        }",
        "for (const x of xs) {
            if (a) {
                continue;
            } else {
                doMore();
            }
            use(x);
        }",
        "while (cond) continue;",
        "for (;;) continue;",
        "for (const x of xs) continue;",
        "outer: for (const x of xs) {
            for (const y of ys) {
                continue outer;
            }
        }",
        "loop: for (const x of xs) {
            doX();
            continue loop;
        }",
        "for (const x of xs) {
            switch (x) {
                case 1:
                    continue;
            }
        }",
        "for (const x of xs) {
            try {
                continue;
            } finally {
                cleanup();
            }
        }",
        "for (const x of xs) {
            try {
                doX();
                continue;
            } catch {}
        }",
        "for (const x of xs) {
            try {
                doX();
            } catch {
                continue;
            }
        }",
        "for (const x of xs) {
            try {
                doX();
            } finally {
                continue;
            }
        }",
        "for (const x of xs) {
            for (const y of ys) {
                if (skip(y)) {
                    continue;
                }
                process(y);
            }
        }",
        "for (const x of xs) {
            continue;
            doX();
        }",
        "for (const x of xs) {
            continue;
            function f() {}
        }",
        "for (const x of xs) {
            if (a) {
                continue;
            }
            ;
        }",
        "for (const x of xs) {
            {
                continue;
            }
            doMore();
        }",
        "for (const x of xs) {
            block: {
                continue;
            }
        }",
        "for (const x of xs) {
            if (a) {
                if (b) {
                    continue;
                }
            }
            doMore();
        }",
    ];

    let fail = vec![
        "for (const x of xs) {
            process(x);
            continue;
        }",
        "for (const x of xs) {
            continue;
        }",
        "while (cond) {
            doX();
            continue;
        }",
        "do {
            doX();
            continue;
        } while (cond);",
        "for (const x in object) {
            doX();
            continue;
        }",
        "for (let i = 0; i < n; i++) {
            doX();
            continue;
        }",
        "for (const x of xs) if (a) { continue; }",
        "for (const x of xs) {
            if (a) {
                continue;
            }
        }",
        "for (const x of xs) {
            if (a) {
                if (b) {
                    continue;
                }
            }
        }",
        "for (const x of xs) {
            if (a) {
                doX();
                continue;
            } else {
                doY();
            }
        }",
        "for (const x of xs) {
            if (a) {
                doX();
            } else {
                doY();
                continue;
            }
        }",
        "for (const x of xs) {
            if (a) {
                doX();
            } else if (b) {
                doY();
                continue;
            }
        }",
        "async function run() {
            for await (const x of xs) {
                process(x);
                continue;
            }
        }",
        "for (const x of xs) {
            {
                continue;
            }
        }",
        "for (const x of xs) {
            for (const y of ys) {
                process(y);
                continue;
            }
        }",
        "for (const x of xs) {
            for (const y of ys) {
                continue;
            }
            continue;
        }",
        "outer: for (const x of xs) {
            doX();
            continue;
        }",
        "for (const x of xs) {
            continue;
            continue;
        }",
        "for (const x of xs) {
            if (a) {
                doX();
            } else {
                if (b) {
                    continue;
                }
            }
        }",
        "do {
            if (a) {
                continue;
            }
        } while (cond);",
        "for (const x of xs) {
            doX();
            continue; // trailing comment
        }",
        "for (const x of xs) {
            doX();
            // leading comment
            continue;
        }",
    ];

    Tester::new(NoUselessContinue::NAME, NoUselessContinue::PLUGIN, pass, fail).test_and_snapshot();
}
