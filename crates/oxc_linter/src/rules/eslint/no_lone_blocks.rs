use oxc_ast::ast::{Declaration, VariableDeclarationKind};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_lone_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Block is unnecessary.").with_label(span)
}

fn no_nested_lone_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Nested block is redundant.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLoneBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unnecessary standalone block statements.
    ///
    /// ### Why is this bad?
    ///
    /// Standalone blocks can be confusing as they do not provide any meaningful purpose when used unnecessarily.
    /// They may introduce extra nesting, reducing code readability, and can mislead readers about scope or intent.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// {
    ///   var x = 1;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (condition) {
    ///   var x = 1;
    /// }
    ///
    /// {
    ///   let x = 1; // Used to create a valid block scope.
    /// }
    /// ```
    NoLoneBlocks,
    eslint,
    style,
);

impl Rule for NoLoneBlocks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BlockStatement(stmt) = node.kind() else {
            return;
        };

        let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        if stmt.body.is_empty() {
            let is_comment_in_stmt =
                ctx.semantic().comments_range(stmt.span.start..stmt.span.end).last().is_some();

            if !is_comment_in_stmt
                && !matches!(parent_node.kind(), AstKind::TryStatement(_) | AstKind::CatchClause(_))
            {
                report(ctx, node, parent_node);
            }
            return;
        }

        let mut is_lone_blocks = is_lone_block(node, parent_node);

        if is_lone_blocks {
            for child in &stmt.body {
                match child.as_declaration() {
                    Some(Declaration::VariableDeclaration(decl))
                        if decl.kind != VariableDeclarationKind::Var =>
                    {
                        is_lone_blocks = false;
                    }
                    Some(
                        Declaration::ClassDeclaration(_) | Declaration::FunctionDeclaration(_),
                    ) => {
                        is_lone_blocks = false;
                    }
                    _ => {}
                }
            }

            if is_lone_blocks {
                report(ctx, node, parent_node);
                return;
            }
        }

        match parent_node.kind() {
            AstKind::FunctionBody(parent) => {
                if parent.statements.len() == 1 && stmt.body.len() == 1 {
                    report(ctx, node, parent_node);
                }
            }
            AstKind::BlockStatement(parent_statement) => {
                if parent_statement.body.len() == 1 {
                    report(ctx, node, parent_node);
                }
            }
            AstKind::StaticBlock(parent_statement) => {
                if parent_statement.body.len() == 1 {
                    report(ctx, node, parent_node);
                }
            }
            _ => {}
        }
    }
}

fn report(ctx: &LintContext, node: &AstNode, parent_node: &AstNode) {
    match parent_node.kind() {
        AstKind::BlockStatement(_) | AstKind::StaticBlock(_) => {
            ctx.diagnostic(no_nested_lone_blocks_diagnostic(node.span()));
        }
        _ => ctx.diagnostic(no_lone_blocks_diagnostic(node.span())),
    };
}

fn is_lone_block(node: &AstNode, parent_node: &AstNode) -> bool {
    match parent_node.kind() {
        AstKind::BlockStatement(_) | AstKind::StaticBlock(_) | AstKind::Program(_) => true,
        AstKind::SwitchCase(parent_node) => {
            let consequent = &parent_node.consequent;
            if consequent.len() != 1 {
                return true;
            }
            let node_span = node.span();
            let consequent_span = consequent[0].span();
            node_span.start != consequent_span.start || node_span.end != consequent_span.end
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (foo) { if (bar) { baz(); } }",
        "do { bar(); } while (foo)",
        "function foo() { while (bar) { baz() } }",
        "function test() { { console.log(6); console.log(6) } }",
        "{ let x = 1; }",                      // { "ecmaVersion": 6 },
        "{ const x = 1; }",                    // { "ecmaVersion": 6 },
        "'use strict'; { function bar() {} }", // { "ecmaVersion": 6 },
        "{ function bar() {} }", // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "impliedStrict": true } } },
        "{ class Bar {} }",      // { "ecmaVersion": 6 },
        "{ {let y = 1;} let x = 1; }", // { "ecmaVersion": 6 },
        "
            switch (foo) {
            case bar: {
                baz;
            }
            }
        ",
        "
            switch (foo) {
            case bar: {
                baz;
            }
            case qux: {
                boop;
            }
            }
        ",
        "
            switch (foo) {
            case bar:
            {
                baz;
            }
            }
        ",
        "function foo() { { const x = 4 } const x = 3 }", // { "ecmaVersion": 6 },
        "class C { static {} }",                          // { "ecmaVersion": 2022 },
        "class C { static { foo; } }",                    // { "ecmaVersion": 2022 },
        "class C { static { if (foo) { block; } } }",     // { "ecmaVersion": 2022 },
        "class C { static { lbl: { block; } } }",         // { "ecmaVersion": 2022 },
        "class C { static { { let block; } something; } }", // { "ecmaVersion": 2022 },
        "class C { static { something; { const block = 1; } } }", // { "ecmaVersion": 2022 },
        "class C { static { { function block(){} } something; } }", // { "ecmaVersion": 2022 },
        "class C { static { something; { class block {}  } } }", // { "ecmaVersion": 2022 },
        "
            {
            using x = makeDisposable();
            }
        ", // {                "parser": require(parser("typescript-parsers/no-lone-blocks/using")),                "ecmaVersion": 2022            },
        "
            {
            await using x = makeDisposable();
            }
        ", // {                "parser": require(parser("typescript-parsers/no-lone-blocks/await-using")),                "ecmaVersion": 2022            }
        // Issue: <https://github.com/oxc-project/oxc/issues/8515>
        "try {} catch {}",
        // Issue: https://github.com/oxc-project/oxc/issues/8697
        "
            if (foo) {
                // do nothing
            }
            else if (bar) {
                // do nothing again
            }
            else {
                // do nothing
            }
        ",
    ];

    let fail = vec![
        "{}",
        "{var x = 1;}",
        "foo(); {} bar();",
        "function test() { { console.log(6); } }",
        "if (foo) { bar(); {} baz(); }",
        "{
			{ } }",
        "function foo() { bar(); {} baz(); }",
        "while (foo) { {} }",
        // MEMO: Currently, this rule always analyzes in strict mode (as it cannot retrieve ecmaFeatures).
        // "{ function bar() {} }", // { "ecmaVersion": 6 },
        "{var x = 1;}", // { "ecmaVersion": 6 },
        "{
			{var x = 1;}
			let y = 2; } {let z = 1;}", // { "ecmaVersion": 6 },
        "{
			{let x = 1;}
			var y = 2; } {let z = 1;}", // { "ecmaVersion": 6 },
        "{
			{var x = 1;}
			var y = 2; }
			{var z = 1;}", // { "ecmaVersion": 6 },
        "
            switch (foo) {
            case 1:
                foo();
                {
                    bar;
                }
            }
        ",
        "
            switch (foo) {
                case 1:
                    {
                        bar;
                    }
                    foo();
            }
        ",
        "
            function foo () {
                {
                    const x = 4;
                }
            }
        ", // { "ecmaVersion": 6 },
        "
            function foo () {
                {
                    var x = 4;
                }
            }
        ",
        "
            class C {
                static {
                    if (foo) {
                        {
                            let block;
                        }
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    if (foo) {
                        {
                            block;
                        }
                        something;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        block;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        let block;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        const block = 1;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        function block() {}
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        class block {}
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        var block;
                    }
                    something;
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    something;
                    {
                        var block;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    {
                        block;
                    }
                    something;
                }
            }
        ", // { "ecmaVersion": 2022 },
        "
            class C {
                static {
                    something;
                    {
                        block;
                    }
                }
            }
        ", // { "ecmaVersion": 2022 }
    ];

    Tester::new(NoLoneBlocks::NAME, NoLoneBlocks::PLUGIN, pass, fail).test_and_snapshot();
}
