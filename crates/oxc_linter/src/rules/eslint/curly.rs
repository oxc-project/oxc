use crate::fixer::{RuleFix, RuleFixer};
use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::{AstKind, ast::IfStatement, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn curly_diagnostic(span: Span, keyword: &str, expected: bool) -> OxcDiagnostic {
    let condition_if_needed =
        matches!(keyword, "if" | "while" | "for").then_some(" condition").unwrap_or("");
    let prefix = if expected { "Expected" } else { "Unexpected" };
    let message = format!("{prefix} {{ after '{keyword}'{condition_if_needed}.");

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CurlyType {
    #[default]
    All,
    Multi,
    MultiLine,
    MultiOrNest,
}

impl CurlyType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "multi" => Self::Multi,
            "multi-line" => Self::MultiLine,
            "multi-or-nest" => Self::MultiOrNest,
            _ => Self::All,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Curly(CurlyConfig);

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CurlyConfig {
    /// Which type of curly brace enforcement to use.
    ///
    /// - `"all"`: require braces in all cases
    /// - `"multi"`: require braces only for multi-statement blocks
    /// - `"multi-line"`: require braces only for multi-line blocks
    /// - `"multi-or-nest"`: require braces for multi-line blocks or when nested
    curly_type: CurlyType,
    /// Whether to enforce consistent use of curly braces in if-else chains.
    consistent: bool,
}

impl Default for CurlyConfig {
    fn default() -> Self {
        Self { curly_type: CurlyType::All, consistent: false }
    }
}

struct IfBranch<'a> {
    statement: &'a Statement<'a>,
    is_else: bool,
    should_have_braces: Option<bool>,
    has_braces: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces the use of curly braces `{}` for all control statements
    /// (`if`, `else`, `for`, `while`, `do`, `with`).
    /// It ensures that all blocks are enclosed in curly braces to improve code clarity and maintainability.
    ///
    /// ### Why is this bad?
    ///
    /// Omitting curly braces can reduce code readability and increase the likelihood of errors, especially in deeply nested or indented code.
    /// It can also lead to bugs if additional statements are added later without properly enclosing them in braces.
    /// Using curly braces consistently makes the code safer and easier to modify.
    ///
    /// ### Examples
    ///
    /// #### `"all"` (default)
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* curly: ["error", "all"] */
    ///
    /// if (foo) foo++;
    /// while (bar) bar--;
    /// do foo();
    /// while (bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* curly: ["error", "all"] */
    ///
    /// if (foo) {
    ///   foo++;
    /// }
    /// while (bar) {
    ///   bar--;
    /// }
    /// do { foo(); } while (bar);
    /// ```
    ///
    /// #### `"multi"`
    /// Examples of **incorrect** code for this rule with the `"multi"` option:
    /// ```js
    /// /* curly: ["error", "multi"] */
    ///
    /// if (foo) foo();
    /// else { bar(); baz(); }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"multi"` option:
    /// ```js
    /// /* curly: ["error", "multi"] */
    ///
    /// if (foo) foo();
    /// else bar();
    /// ```
    ///
    /// #### `"multi-line"`
    /// Examples of **incorrect** code for this rule with the `"multi-line"` option:
    /// ```js
    /// /* curly: ["error", "multi-line"] */
    ///
    /// if (foo) foo()
    /// else
    ///   bar();
    ///
    /// while (foo)
    ///   foo()
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"multi-line"` option:
    /// ```js
    /// /* curly: ["error", "multi-line"] */
    ///
    /// if (foo) foo();
    /// else bar();
    ///
    /// while (foo) foo();
    ///
    /// while (true) {
    ///    doSomething();
    ///    doSomethingElse();
    /// }
    /// ```
    ///
    /// #### `"multi-or-nest"`
    /// Examples of **incorrect** code for this rule with the `"multi-or-nest"` option:
    /// ```js
    /// /* curly: ["error", "multi-or-nest"] */
    ///
    /// if (foo)
    ///   if (bar) bar();
    ///
    /// while (foo)
    ///   while (bar) bar();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"multi-or-nest"` option:
    /// ```js
    /// /* curly: ["error", "multi-or-nest"] */
    ///
    /// if (foo) {
    ///   if (bar) bar();
    /// }
    ///
    /// while (foo) {
    ///   while (bar) bar();
    /// }
    /// ```
    ///
    /// #### `{ "consistent": true }`
    ///
    /// When enabled, `consistent: true` enforces consistent use of braces within an `if-else` chain.
    /// If one branch of the chain uses braces, then all branches must use braces, even if not strictly required by the first option.
    ///
    /// Examples of **incorrect** code with `"multi"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi", "consistent"] */
    ///
    /// if (foo) {
    ///   bar();
    ///   baz();
    /// } else qux();
    ///
    /// if (foo) bar();
    /// else {
    ///   baz();
    ///   qux();
    /// }
    /// ```
    ///
    /// Examples of **correct** code with `"multi"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi", "consistent"] */
    ///
    /// if (foo) {
    ///   bar();
    ///   baz();
    /// } else {
    ///   qux();
    /// }
    ///
    /// if (foo) {
    ///   bar();
    /// } else {
    ///   baz();
    ///   qux();
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code with `"multi-line"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi-line", "consistent"] */
    ///
    /// if (foo) {
    ///   bar();
    /// } else
    ///   baz();
    /// ```
    ///
    /// Examples of **correct** code with `"multi-line"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi-line", "consistent"] */
    ///
    /// if (foo) {
    ///   bar();
    /// } else {
    ///   baz();
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code with `"multi-or-nest"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi-or-nest", "consistent"] */
    ///
    /// if (foo) {
    ///   if (bar) baz();
    /// } else qux();
    /// ```
    ///
    /// Examples of **correct** code with `"multi-or-nest"` and `consistent: true`:
    /// ```js
    /// /* curly: ["error", "multi-or-nest", "consistent"] */
    ///
    /// if (foo) {
    ///   if (bar) baz();
    /// } else {
    ///   qux();
    /// }
    /// ```
    Curly,
    eslint,
    style,
    fix,
    config = CurlyConfig,
);

impl Rule for Curly {
    fn from_configuration(value: Value) -> Self {
        let curly_type =
            value.get(0).and_then(Value::as_str).map(CurlyType::from).unwrap_or_default();

        let consistent =
            value.get(1).and_then(Value::as_str).is_some_and(|value| value == "consistent");

        Self(CurlyConfig { curly_type, consistent })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(stmt) => self.run_for_if_statement(stmt, ctx),
            AstKind::ForStatement(stmt) => self.run_for_loop("for", &stmt.body, ctx),
            AstKind::ForInStatement(stmt) => self.run_for_loop("for-in", &stmt.body, ctx),
            AstKind::ForOfStatement(stmt) => self.run_for_loop("for-of", &stmt.body, ctx),
            AstKind::WhileStatement(stmt) => self.run_for_loop("while", &stmt.body, ctx),
            AstKind::DoWhileStatement(stmt) => self.run_for_loop("do", &stmt.body, ctx),
            _ => {}
        }
    }
}

impl<'a> Curly {
    fn run_for_if_statement(&self, stmt: &'a IfStatement<'a>, ctx: &LintContext<'a>) {
        let branches = get_if_branches_from_statement(stmt, &self.0.curly_type, ctx);
        let does_any_branch_need_braces =
            branches.iter().any(|b| b.should_have_braces.unwrap_or(b.has_braces));

        for branch in &branches {
            let should_have_braces = if self.0.consistent {
                Some(does_any_branch_need_braces)
            } else {
                branch.should_have_braces
            };

            report_if_needed(
                ctx,
                branch.statement,
                get_if_else_keyword(branch.is_else),
                branch.has_braces,
                should_have_braces,
            );
        }
    }

    fn run_for_loop(&self, keyword: &str, body: &Statement<'a>, ctx: &LintContext<'a>) {
        let has_braces = has_braces(body);
        let should_have_braces = should_have_braces(&self.0.curly_type, body, ctx);
        report_if_needed(ctx, body, keyword, has_braces, should_have_braces);
    }
}

fn get_if_branches_from_statement<'a>(
    stmt: &'a IfStatement<'a>,
    curly_type: &CurlyType,
    ctx: &LintContext<'a>,
) -> Vec<IfBranch<'a>> {
    let mut branches = vec![IfBranch {
        statement: &stmt.consequent,
        is_else: false,
        should_have_braces: should_have_braces(curly_type, &stmt.consequent, ctx),
        has_braces: has_braces(&stmt.consequent),
    }];

    let mut current_statement = &stmt.alternate;

    while let Some(statement) = current_statement {
        if let Statement::IfStatement(node) = statement {
            branches.push(IfBranch {
                statement: &node.consequent,
                is_else: false,
                should_have_braces: should_have_braces(curly_type, &node.consequent, ctx),
                has_braces: has_braces(&node.consequent),
            });
            current_statement = &node.alternate;
        } else {
            branches.push(IfBranch {
                statement,
                is_else: true,
                should_have_braces: should_have_braces(curly_type, statement, ctx),
                has_braces: has_braces(statement),
            });
            break;
        }
    }

    branches
}

fn get_node_by_statement<'a>(statement: &'a Statement, ctx: &'a LintContext) -> &'a AstNode<'a> {
    let span = statement.span();

    ctx.nodes().iter().find(|n| n.span() == span).expect("Failed to get node by statement")
}

fn get_if_else_keyword(is_else: bool) -> &'static str {
    if is_else { "else" } else { "if" }
}

fn has_braces(body: &Statement) -> bool {
    matches!(body, Statement::BlockStatement(_))
}

fn should_have_braces<'a>(
    curly_type: &CurlyType,
    body: &Statement<'a>,
    ctx: &LintContext<'a>,
) -> Option<bool> {
    let braces_necessary = are_braces_necessary(body, ctx);

    if let Statement::BlockStatement(block) = body
        && (block.body.len() != 1 || braces_necessary)
    {
        return Some(true);
    }

    match curly_type {
        CurlyType::Multi => Some(false),
        CurlyType::MultiLine => {
            if is_collapsed_one_liner(body, ctx) {
                None
            } else {
                Some(true)
            }
        }
        CurlyType::MultiOrNest => {
            let Statement::BlockStatement(block) = body else {
                return Some(!is_one_liner(body, ctx));
            };

            let stmt = block.body.first();
            let body_start = body.span().start;
            let stmt_start = stmt.map_or(body_start, |s| s.span().start);
            let has_comment =
                ctx.comments_range(body_start..stmt_start.saturating_sub(1)).next().is_some();

            Some(stmt.is_none_or(|s| !is_one_liner(s, ctx) || has_comment))
        }
        CurlyType::All => Some(true),
    }
}

fn apply_rule_fix<'a>(
    fixer: &RuleFixer<'_, 'a>,
    body: &Statement<'a>,
    should_have_braces: bool,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let source = ctx.source_range(body.span());

    let fixed = if should_have_braces {
        format!("{{{source}}}")
    } else {
        let mut trimmed = source.trim_matches(|c| c == '{' || c == '}').to_string();
        if matches!(
            ctx.nodes().parent_kind(get_node_by_statement(body, ctx).id()),
            AstKind::DoWhileStatement(_)
        ) {
            trimmed.insert(0, ' ');
        }
        trimmed
    };

    fixer.replace(body.span(), fixed)
}

fn report_if_needed<'a>(
    ctx: &LintContext<'a>,
    body: &Statement<'a>,
    keyword: &str,
    has_braces: bool,
    should_have_braces: Option<bool>,
) {
    let Some(should_have_braces) = should_have_braces else {
        return;
    };
    if should_have_braces == has_braces {
        return;
    }

    ctx.diagnostic_with_fix(curly_diagnostic(body.span(), keyword, should_have_braces), |fixer| {
        apply_rule_fix(&fixer, body, should_have_braces, ctx)
    });
}

#[expect(clippy::cast_possible_truncation)]
fn is_collapsed_one_liner(node: &Statement, ctx: &LintContext) -> bool {
    let node = get_node_by_statement(node, ctx);
    let span = node.span();
    let node_string = ctx.source_range(span);

    let trimmed = node_string.trim_end_matches(|c: char| c.is_whitespace() || c == ';');
    let trimmed_len: u32 = match trimmed.len().try_into() {
        Ok(val) => val,
        Err(_) => return false, // length too large for u32
    };

    let before_node_span = get_token_before(node, ctx).map_or_else(
        || {
            let parent = ctx.nodes().parent_node(node.id());

            if parent.span().start < span.start {
                Span::empty(parent.span().start)
            } else {
                Span::empty(0)
            }
        },
        oxc_span::GetSpan::span,
    );

    let Some(next_char_offset) = get_next_char_offset(before_node_span, ctx) else {
        return true;
    };

    let end_offset =
        span.end.saturating_sub((node_string.len() as u32).saturating_sub(trimmed_len));
    let text = ctx.source_range(Span::new(next_char_offset, end_offset));

    !text.contains('\n')
}

fn is_one_liner(node: &Statement, ctx: &LintContext) -> bool {
    if matches!(node, Statement::EmptyStatement(_)) {
        return true;
    }

    let source = ctx.source_range(node.span());
    let trimmed = source.trim_end_matches(|c: char| c.is_whitespace() || c == ';');

    !trimmed.contains('\n')
}

fn get_token_before<'a>(node: &AstNode, ctx: &'a LintContext) -> Option<&'a AstNode<'a>> {
    let span_start = node.span().start;

    ctx.nodes().iter().filter(|n| n.span().end < span_start).max_by_key(|n| n.span().end)
}

pub fn are_braces_necessary(node: &Statement, ctx: &LintContext) -> bool {
    let Statement::BlockStatement(block) = node else {
        return false;
    };

    let Some(first_body_statement) = block.body.first() else {
        return false;
    };

    is_lexical_declaration(first_body_statement)
        || (has_unsafe_if(first_body_statement) && is_followed_by_else_keyword(node, ctx))
}

fn is_lexical_declaration(node: &Statement) -> bool {
    match node {
        Statement::VariableDeclaration(decl) => decl.kind.is_lexical(),
        Statement::FunctionDeclaration(_) | Statement::ClassDeclaration(_) => true,
        _ => false,
    }
}

#[expect(clippy::cast_possible_truncation)]
fn get_next_char_offset(span: Span, ctx: &LintContext) -> Option<u32> {
    let src = ctx.source_text();
    let start = span.end as usize;
    if start >= src.len() {
        return None;
    }

    if let Some(tail) = src.get(start..)
        && (tail.starts_with("\r\n") || tail.starts_with("\n\r"))
    {
        return Some(span.end + 2);
    }

    src[start..].chars().next().map(|c| span.end + c.len_utf8() as u32)
}

fn is_followed_by_else_keyword(node: &Statement, ctx: &LintContext) -> bool {
    let Some(next_char_offset) = get_next_char_offset(node.span(), ctx) else {
        return false;
    };

    let start = next_char_offset;
    let end: u32 = match ctx.source_text().len().try_into() {
        Ok(val) => val,
        Err(_) => return false, // length too large for u32
    };

    if start > end {
        return false;
    }

    ctx.source_range(Span::new(start, end))
        .trim_start()
        .trim_start_matches("else")
        .starts_with([' ', ';', '{'])
}

fn has_unsafe_if(node: &Statement) -> bool {
    match node {
        Statement::IfStatement(if_stmt) => {
            if_stmt.alternate.as_ref().is_none_or(|alt| has_unsafe_if(alt))
        }
        Statement::ForStatement(for_stmt) => has_unsafe_if(&for_stmt.body),
        Statement::ForInStatement(for_in_stmt) => has_unsafe_if(&for_in_stmt.body),
        Statement::ForOfStatement(for_of_stmt) => has_unsafe_if(&for_of_stmt.body),
        Statement::LabeledStatement(labeled_stmt) => has_unsafe_if(&labeled_stmt.body),
        Statement::WithStatement(with_stmt) => has_unsafe_if(&with_stmt.body),
        Statement::WhileStatement(while_stmt) => has_unsafe_if(&while_stmt.body),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("if (foo) { bar() }", None),
        ("if (foo) { bar() } else if (foo2) { baz() }", None),
        ("while (foo) { bar() }", None),
        ("do { bar(); } while (foo)", None),
        ("for (;foo;) { bar() }", None),
        ("for (var foo in bar) { console.log(foo) }", None),
        ("for (var foo of bar) { console.log(foo) }", None), // { "ecmaVersion": 6 },
        ("for (;foo;) bar()", Some(serde_json::json!(["multi"]))),
        ("if (foo) bar()", Some(serde_json::json!(["multi"]))),
        ("if (a) { b; c; }", Some(serde_json::json!(["multi"]))),
        ("for (var foo in bar) console.log(foo)", Some(serde_json::json!(["multi"]))),
        (
            "for (var foo in bar) { console.log(1); console.log(2) }",
            Some(serde_json::json!(["multi"])),
        ),
        ("for (var foo of bar) console.log(foo)", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar) { console.log(1); console.log(2) }",
            Some(serde_json::json!(["multi"])),
        ), // { "ecmaVersion": 6 },
        ("if (foo) bar()", Some(serde_json::json!(["multi-line"]))),
        (
            "if (foo) bar()
			",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("if (foo) bar(); else baz()", Some(serde_json::json!(["multi-line"]))),
        (
            "if (foo) bar();
			 else baz()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) bar()
			 else if (foo) bar()
			 else baz()",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("do baz(); while (foo)", Some(serde_json::json!(["multi-line"]))),
        ("if (foo) { bar() }", Some(serde_json::json!(["multi-line"]))),
        ("for (var foo in bar) console.log(foo)", Some(serde_json::json!(["multi-line"]))),
        (
            "for (var foo in bar) {
			 console.log(1);
			 console.log(2);
			 }",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("for (var foo of bar) console.log(foo)", Some(serde_json::json!(["multi-line"]))), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar) {
			 console.log(1);
			 console.log(2);
			 }",
            Some(serde_json::json!(["multi-line"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 bar();
			 baz();
			 }",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "do bar()
			 while (foo)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) {
			 quz = {
			 bar: baz,
			 qux: foo
			 };
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true) {
			 if (foo)
			 doSomething();
			 else
			 doSomethingElse();
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo)
			 quz = true;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) {
			 // line of comment
			 quz = true;
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "// line of comment
			 if (foo)
			 quz = true;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true)
			 doSomething();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++)
			 doSomething();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) {
			 if(bar)
			 doSomething();
			 } else
			 doSomethingElse();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar)
			 console.log(foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) {
			 if (foo) console.log(1);
			 else console.log(2)
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar)
			 console.log(foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar) {
			 if (foo) console.log(1);
			 else console.log(2)
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        ("if (foo) { const bar = 'baz'; }", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("while (foo) { let bar = 'baz'; }", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("for(;;) { function foo() {} }", Some(serde_json::json!(["multi"]))),
        ("for (foo in bar) { class Baz {} }", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("if (foo) { let bar; } else { baz(); }", Some(serde_json::json!(["multi", "consistent"]))), // { "ecmaVersion": 6 },
        (
            "if (foo) { bar(); } else { const baz = 'quux'; }",
            Some(serde_json::json!(["multi", "consistent"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 const bar = 'baz';
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 let bar = 'baz';
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 function bar() {}
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 class bar {}
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) doSomething()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else if (bar) doSomethingElse()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else doSomethingElse()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else if (bar) doSomethingElse();
			 else doAnotherThing()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++) doSomething()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) console.log(foo)
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar) console.log(foo)
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "while (foo) doSomething()
			 ;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "do doSomething()
			 ;while (foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else if (bar)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			 else if (bar) doSomethingElse();
			 else
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "while (foo)
			;",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "do
			;while (foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        // クリア
        (
            "if (true) { if (false) console.log(1) } else console.log(2)",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { if (b) console.log(1); else if (c) console.log(2) } else console.log(3)",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (true) { while(false) if (true); } else;", Some(serde_json::json!(["multi"]))),
        ("if (true) { label: if (false); } else;", Some(serde_json::json!(["multi"]))),
        ("if (true) { with(0) if (false); } else;", Some(serde_json::json!(["multi"]))),
        (
            "if (true) { while(a) if(b) while(c) if (d); else; } else;",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (true) foo(); else { bar(); baz(); }", Some(serde_json::json!(["multi"]))),
        (
            "if (true) { foo(); } else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) { foo(); } else if (true) { faa(); } else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) { foo(); faa(); } else { bar(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) foo()
			;[1, 2, 3].bar()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (x) for (var i in x) { if (i > 0) console.log(i); } else console.log('whoops');",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (a) { if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        ("if (a) { if (b) foo(); } else bar();", Some(serde_json::json!(["multi-or-nest"]))),
        (
            "if (a) { if (b) foo(); } else { bar(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (a) { if (b) foo(); } else { bar(); }",
            Some(serde_json::json!(["multi-or-nest", "consistent"])),
        ),
        ("if (a) { if (b) { foo(); bar(); } } else baz();", Some(serde_json::json!(["multi"]))),
        (
            "if (a) foo(); else if (b) { if (c) bar(); } else baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { if (b) foo(); else if (c) bar(); } else baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) if (b) foo(); else { if (c) bar(); } else baz();",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (a) { lbl:if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        ("if (a) { lbl1:lbl2:if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        ("if (a) { for (;;) if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { for (key in obj) if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { for (elem of arr) if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ), // { "ecmaVersion": 2015 },
        ("if (a) { with (obj) if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        ("if (a) { while (cond) if (b) foo(); } else bar();", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { while (cond) for (;;) for (key in obj) if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) { for (;;) for (key in obj) if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) for (;;) { for (key in obj) if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) for (;;) for (key in obj) { if (b) foo(); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "  const isIterable = (obj: any) : obj is Iterable<IgnoreRule> => {
                if (obj === null) return false;
                else if (typeof obj === 'string') return false;
                else return typeof value[Symbol.iterator] === 'function';
            };",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const isIterable = (obj: any): obj is Iterable<IgnoreRule> => {\r\n    if (obj === null) return false;\r\n    else if (typeof obj === 'string') return false;\r\n    else return typeof value[Symbol.iterator] === 'function';\r\n};\r\n",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "  const isIterable = (obj: any) : obj is Iterable<IgnoreRule> => {
                if (obj === null) return false;
                else if (typeof obj === 'string') return false;
                else return typeof value[Symbol.iterator] === 'function';
            };",
            Some(serde_json::json!(["multi"])),
        ),
    ];

    let fail = vec![
        ("if (foo) bar()", None),
        (
            "if (foo)
			 bar()",
            None,
        ),
        ("if (foo) { bar() } else baz()", None),
        ("if (foo) { bar() } else if (faa) baz()", None),
        ("while (foo) bar()", None),
        (
            "while (foo)
			 bar()",
            None,
        ),
        ("do bar(); while (foo)", None),
        (
            "do
			 bar(); while (foo)",
            None,
        ),
        ("for (;foo;) bar()", None),
        ("for (var foo in bar) console.log(foo)", None),
        ("for (var foo of bar) console.log(foo)", None), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar)
			 console.log(foo)",
            None,
        ), // { "ecmaVersion": 6 },
        ("for (a;;) console.log(foo)", None),            // { "ecmaVersion": 6 },
        (
            "for (a;;)
			 console.log(foo)",
            None,
        ), // { "ecmaVersion": 6 },
        ("for (var foo of bar) {console.log(foo)}", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("do{foo();} while(bar);", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("for (;foo;) { bar() }", Some(serde_json::json!(["multi"]))),
        (
            "for (;foo;)
			 bar()",
            None,
        ),
        ("if (foo) { bar() }", Some(serde_json::json!(["multi"]))),
        ("if (foo) if (bar) { baz() }", Some(serde_json::json!(["multi"]))),
        (
            "if (foo) if (bar) baz(); else if (quux) { quuux(); }",
            Some(serde_json::json!(["multi"])),
        ),
        ("while (foo) { bar() }", Some(serde_json::json!(["multi"]))),
        ("if (foo) baz(); else { bar() }", Some(serde_json::json!(["multi"]))),
        ("if (foo) if (bar); else { baz() }", Some(serde_json::json!(["multi"]))),
        ("if (true) { if (false) console.log(1) }", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { if (b) console.log(1); else console.log(2) } else console.log(3)",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (0)
			    console.log(0)
			else if (1) {
			    console.log(1)
			    console.log(1)
			} else {
			    if (2)
			        console.log(2)
			    else
			        console.log(3)
			}",
            Some(serde_json::json!(["multi"])),
        ),
        ("for (var foo in bar) { console.log(foo) }", Some(serde_json::json!(["multi"]))),
        ("for (var foo of bar) { console.log(foo) }", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        (
            "if (foo)
			 baz()",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("if (foo) baz()", None),
        (
            "while (foo)
			 baz()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (;foo;)
			 bar()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "while (bar &&
			 baz)
			 foo()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) bar(baz,
			 baz)",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("do foo(); while (bar)", Some(serde_json::json!(["all"]))),
        (
            "do
			 foo();
			 while (bar)",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("for (var foo in bar) {console.log(foo)}", Some(serde_json::json!(["multi"]))),
        (
            "for (var foo in bar)
			 console.log(foo)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo in bar)
			 console.log(1);
			 console.log(2)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo of bar)
			 console.log(foo)",
            Some(serde_json::json!(["multi-line"])),
        ), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar)
			 console.log(1);
			 console.log(2)",
            Some(serde_json::json!(["multi-line"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo)
			 quz = {
			 bar: baz,
			 qux: foo
			 };",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true)
			 if (foo)
			 doSomething();
			 else
			 doSomethingElse();
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) {
			 quz = true;
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        ("if (foo) { var bar = 'baz'; }", Some(serde_json::json!(["multi"]))),
        ("if (foo) { let bar; } else baz();", Some(serde_json::json!(["multi", "consistent"]))), // { "ecmaVersion": 6 },
        (
            "if (foo) bar(); else { const baz = 'quux' }",
            Some(serde_json::json!(["multi", "consistent"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) {
			 var bar = 'baz';
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true) {
			 doSomething();
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++) {
			 doSomething();
			 }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) if (foo) console.log(1); else console.log(2);",
            Some(serde_json::json!(["all"])),
        ),
        (
            "for (var foo in bar)
			 if (foo) console.log(1);
			 else console.log(2);",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) { if (foo) console.log(1) }",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar)
			 if (foo) console.log(1);
			 else console.log(2);",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "for (var foo of bar) { if (foo) console.log(1) }",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (true) foo();
			 else {
			 bar();
			 baz();
			 }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) { foo(); faa(); }
			 else bar();",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        ("if (true) foo(); else { baz(); }", Some(serde_json::json!(["multi", "consistent"]))),
        (
            "if (true) foo(); else if (true) faa(); else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) if (true) foo(); else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        ("do{foo();} while (bar)", Some(serde_json::json!(["multi"]))),
        (
            "do
			{foo();} while (bar)",
            Some(serde_json::json!(["multi"])),
        ),
        ("while (bar) { foo(); }", Some(serde_json::json!(["multi"]))),
        (
            "while (bar)
			{
			 foo(); }",
            Some(serde_json::json!(["multi"])),
        ),
        ("for (;;) { foo(); }", Some(serde_json::json!(["multi"]))),
        ("do{[1, 2, 3].map(bar);} while (bar)", Some(serde_json::json!(["multi"]))),
        ("if (foo) {bar()} baz()", Some(serde_json::json!(["multi"]))),
        ("do {foo();} while (bar)", Some(serde_json::json!(["multi"]))),
        (
            "if (foo) { bar }
			++baz;",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar; }
			++baz;",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar++ }
			baz;",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar }
			[1, 2, 3].map(foo);",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar }
			(1).toString();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar }
			/regex/.test('foo');",
            Some(serde_json::json!(["multi"])),
        ), // { "ecmaVersion": 6 },
        (
            "if (foo) { bar }
			Baz();",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (foo) { while (bar) {} } else {}", Some(serde_json::json!(["multi"]))),
        ("if (foo) { var foo = () => {} } else {}", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        ("if (foo) { var foo = function() {} } else {}", Some(serde_json::json!(["multi"]))),
        ("if (foo) { var foo = function*() {} } else {}", Some(serde_json::json!(["multi"]))), // { "ecmaVersion": 6 },
        (
            "if (true)
			foo()
			;[1, 2, 3].bar()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) {
			doSomething()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			else if (bar) {
			doSomethingElse()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			else {
			doSomethingElse()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++) {
			doSomething()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) {
			doSomething()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar) {
			doSomething()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ), // { "ecmaVersion": 6 },
        (
            "while (foo) {
			doSomething()
			;
			}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "do {
			doSomething()
			;
			} while (foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        ("if (a) { if (b) foo(); }", Some(serde_json::json!(["multi"]))),
        ("if (a) { if (b) foo(); else bar(); }", Some(serde_json::json!(["multi"]))),
        ("if (a) { if (b) foo(); else bar(); } baz();", Some(serde_json::json!(["multi"]))),
        ("if (a) { while (cond) if (b) foo(); }", Some(serde_json::json!(["multi"]))),
        ("if (a) while (cond) { if (b) foo(); }", Some(serde_json::json!(["multi"]))),
        ("if (a) while (cond) { if (b) foo(); else bar(); }", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { while (cond) { if (b) foo(); } bar(); baz() } else quux();",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (a) { if (b) foo(); } bar();", Some(serde_json::json!(["multi"]))),
        (
            "if(a) { if (b) foo(); } if (c) bar(); else baz();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (a) { do if (b) foo(); while (cond); } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) do { if (b) foo(); } while (cond); else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        ("if (a) { if (b) foo(); else bar(); } else baz();", Some(serde_json::json!(["multi"]))),
        ("if (a) while (cond) { bar(); } else baz();", Some(serde_json::json!(["multi"]))),
        ("if (a) { for (;;); } else bar();", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { while (cond) if (b) foo() } else bar();",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (a)  while (cond) if (b) foo()
			else
			 {bar();}",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (a) foo()
			else
			 bar();",
            None,
        ),
        ("if (a) { while (cond) if (b) foo() } ", Some(serde_json::json!(["multi", "consistent"]))),
        (
            "if(a) { if (b) foo(); } if (c) bar(); else if(foo){bar();}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (true) [1, 2, 3]
			.bar()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for(
			;
			;
			) {foo()}",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for(
			;
			;
			)
			foo()
			",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (a) { while (cond) { if (b) foo(); } } else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for(;;)foo()
			",
            None,
        ),
        (
            "for(var
			i
			 in
			 z)foo()
			",
            None,
        ),
        (
            "for(var i of
			 z)
			foo()
			",
            None,
        ), // { "ecmaVersion": 6 }
    ];

    let fix = vec![
        ("if (foo) bar()", "if (foo) {bar()}", None),
        (
            "if (foo)
			 bar()",
            "if (foo)
			 {bar()}",
            None,
        ),
        ("if (foo) { bar() } else baz()", "if (foo) { bar() } else {baz()}", None),
        (
            "if (foo) { bar() } else if (faa) baz()",
            "if (foo) { bar() } else if (faa) {baz()}",
            None,
        ),
        ("while (foo) bar()", "while (foo) {bar()}", None),
        (
            "while (foo)
			 bar()",
            "while (foo)
			 {bar()}",
            None,
        ),
        ("do bar(); while (foo)", "do {bar();} while (foo)", None),
        (
            "do
			 bar(); while (foo)",
            "do
			 {bar();} while (foo)",
            None,
        ),
        ("for (;foo;) bar()", "for (;foo;) {bar()}", None),
        ("for (var foo in bar) console.log(foo)", "for (var foo in bar) {console.log(foo)}", None),
        ("for (var foo of bar) console.log(foo)", "for (var foo of bar) {console.log(foo)}", None),
        (
            "for (var foo of bar)
			 console.log(foo)",
            "for (var foo of bar)
			 {console.log(foo)}",
            None,
        ),
        ("for (a;;) console.log(foo)", "for (a;;) {console.log(foo)}", None),
        (
            "for (a;;)
			 console.log(foo)",
            "for (a;;)
			 {console.log(foo)}",
            None,
        ),
        (
            "for (var foo of bar) {console.log(foo)}",
            "for (var foo of bar) console.log(foo)",
            Some(serde_json::json!(["multi"])),
        ),
        ("do{foo();} while(bar);", "do foo(); while(bar);", Some(serde_json::json!(["multi"]))),
        ("for (;foo;) { bar() }", "for (;foo;)  bar() ", Some(serde_json::json!(["multi"]))),
        (
            "for (;foo;)
			 bar()",
            "for (;foo;)
			 {bar()}",
            None,
        ),
        ("if (foo) { bar() }", "if (foo)  bar() ", Some(serde_json::json!(["multi"]))),
        (
            "if (foo) if (bar) { baz() }",
            "if (foo) if (bar)  baz() ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) if (bar) baz(); else if (quux) { quuux(); }",
            "if (foo) if (bar) baz(); else if (quux)  quuux(); ",
            Some(serde_json::json!(["multi"])),
        ),
        ("while (foo) { bar() }", "while (foo)  bar() ", Some(serde_json::json!(["multi"]))),
        (
            "if (foo) baz(); else { bar() }",
            "if (foo) baz(); else  bar() ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) if (bar); else { baz() }",
            "if (foo) if (bar); else  baz() ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (true) { if (false) console.log(1) }",
            "if (true)  if (false) console.log(1) ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for (var foo in bar) { console.log(foo) }",
            "for (var foo in bar)  console.log(foo) ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for (var foo of bar) { console.log(foo) }",
            "for (var foo of bar)  console.log(foo) ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo)
			 baz()",
            "if (foo)
			 {baz()}",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("if (foo) baz()", "if (foo) {baz()}", None),
        (
            "while (foo)
			 baz()",
            "while (foo)
			 {baz()}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (;foo;)
			 bar()",
            "for (;foo;)
			 {bar()}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "while (bar &&
			 baz)
			 foo()",
            "while (bar &&
			 baz)
			 {foo()}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) bar(baz,
			 baz)",
            "if (foo) {bar(baz,
			 baz)}",
            Some(serde_json::json!(["multi-line"])),
        ),
        ("do foo(); while (bar)", "do {foo();} while (bar)", Some(serde_json::json!(["all"]))),
        (
            "do
			 foo();
			 while (bar)",
            "do
			 {foo();}
			 while (bar)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo in bar) {console.log(foo)}",
            "for (var foo in bar) console.log(foo)",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for (var foo in bar)
			 console.log(foo)",
            "for (var foo in bar)
			 {console.log(foo)}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo in bar)
			 console.log(1);
			 console.log(2)",
            "for (var foo in bar)
			 {console.log(1);}
			 console.log(2)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo of bar)
			 console.log(foo)",
            "for (var foo of bar)
			 {console.log(foo)}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for (var foo of bar)
			 console.log(1);
			 console.log(2)",
            "for (var foo of bar)
			 {console.log(1);}
			 console.log(2)",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo)
			 quz = {
			 bar: baz,
			 qux: foo
			 };",
            "if (foo)
			 {quz = {
			 bar: baz,
			 qux: foo
			 };}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true)
			 if (foo)
			 doSomething();
			 else
			 doSomethingElse();
			",
            "while (true)
			 {if (foo)
			 doSomething();
			 else
			 doSomethingElse();}
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) {
			 quz = true;
			 }",
            "if (foo) 
			 quz = true;
			 ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) { var bar = 'baz'; }",
            "if (foo)  var bar = 'baz'; ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { let bar; } else baz();",
            "if (foo) { let bar; } else {baz();}",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (foo) bar(); else { const baz = 'quux' }",
            "if (foo) {bar();} else { const baz = 'quux' }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (foo) {
			 var bar = 'baz';
			 }",
            "if (foo) 
			 var bar = 'baz';
			 ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (true) {
			 doSomething();
			 }",
            "while (true) 
			 doSomething();
			 ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++) {
			 doSomething();
			 }",
            "for (var i = 0; foo; i++) 
			 doSomething();
			 ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) if (foo) console.log(1); else console.log(2);",
            "for (var foo in bar) {if (foo) console.log(1); else console.log(2);}",
            Some(serde_json::json!(["all"])),
        ),
        (
            "for (var foo in bar)
			 if (foo) console.log(1);
			 else console.log(2);",
            "for (var foo in bar)
			 {if (foo) console.log(1);
			 else console.log(2);}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) { if (foo) console.log(1) }",
            "for (var foo in bar)  if (foo) console.log(1) ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar)
			 if (foo) console.log(1);
			 else console.log(2);",
            "for (var foo of bar)
			 {if (foo) console.log(1);
			 else console.log(2);}",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar) { if (foo) console.log(1) }",
            "for (var foo of bar)  if (foo) console.log(1) ",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (true) foo();
			 else {
			 bar();
			 baz();
			 }",
            "if (true) {foo();}
			 else {
			 bar();
			 baz();
			 }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) { foo(); faa(); }
			 else bar();",
            "if (true) { foo(); faa(); }
			 else {bar();}",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) foo(); else { baz(); }",
            "if (true) foo(); else  baz(); ",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) foo(); else if (true) faa(); else { bar(); baz(); }",
            "if (true) {foo();} else if (true) {faa();} else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (true) if (true) foo(); else { bar(); baz(); }",
            "if (true) if (true) {foo();} else { bar(); baz(); }",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        ("do{foo();} while (bar)", "do foo(); while (bar)", Some(serde_json::json!(["multi"]))),
        (
            "do
			{foo();} while (bar)",
            "do
			 foo(); while (bar)",
            Some(serde_json::json!(["multi"])),
        ),
        ("while (bar) { foo(); }", "while (bar)  foo(); ", Some(serde_json::json!(["multi"]))),
        (
            "while (bar)
			{
			 foo(); }",
            "while (bar)\n\t\t\t
			 foo(); ",
            Some(serde_json::json!(["multi"])),
        ),
        ("for (;;) { foo(); }", "for (;;)  foo(); ", Some(serde_json::json!(["multi"]))),
        (
            "do{[1, 2, 3].map(bar);} while (bar)",
            "do [1, 2, 3].map(bar); while (bar)",
            Some(serde_json::json!(["multi"])),
        ),
        ("do {foo();} while (bar)", "do  foo(); while (bar)", Some(serde_json::json!(["multi"]))),
        (
            "if (foo) { bar; }
			++baz;",
            "if (foo)  bar; 
			++baz;",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { bar }
			Baz();",
            "if (foo)  bar 
			Baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (foo) { while (bar) {} } else {}",
            "if (foo)  while (bar) {}  else {}",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (true)
			foo()
			;[1, 2, 3].bar()",
            "if (true)
			{foo()
			;}[1, 2, 3].bar()",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (foo) {
			doSomething()
			;
			}",
            "if (foo) 
			doSomething()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			else if (bar) {
			doSomethingElse()
			;
			}",
            "if (foo) doSomething();
			else if (bar) 
			doSomethingElse()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (foo) doSomething();
			else {
			doSomethingElse()
			;
			}",
            "if (foo) doSomething();
			else 
			doSomethingElse()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var i = 0; foo; i++) {
			doSomething()
			;
			}",
            "for (var i = 0; foo; i++) 
			doSomething()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo in bar) {
			doSomething()
			;
			}",
            "for (var foo in bar) 
			doSomething()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "for (var foo of bar) {
			doSomething()
			;
			}",
            "for (var foo of bar) 
			doSomething()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "while (foo) {
			doSomething()
			;
			}",
            "while (foo) 
			doSomething()
			;
			",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "do {
			doSomething()
			;
			} while (foo)",
            "do  
			doSomething()
			;
			 while (foo)",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        ("if (a) { if (b) foo(); }", "if (a)  if (b) foo(); ", Some(serde_json::json!(["multi"]))),
        (
            "if (a) { if (b) foo(); else bar(); }",
            "if (a)  if (b) foo(); else bar(); ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { if (b) foo(); else bar(); } baz();",
            "if (a)  if (b) foo(); else bar();  baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { while (cond) if (b) foo(); }",
            "if (a)  while (cond) if (b) foo(); ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) { if (b) foo(); }",
            "if (a) while (cond)  if (b) foo(); ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) { if (b) foo(); else bar(); }",
            "if (a) while (cond)  if (b) foo(); else bar(); ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { while (cond) { if (b) foo(); } bar(); baz() } else quux();",
            "if (a) { while (cond)  if (b) foo();  bar(); baz() } else quux();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { if (b) foo(); } bar();",
            "if (a)  if (b) foo();  bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if(a) { if (b) foo(); } if (c) bar(); else baz();",
            "if(a)  if (b) foo();  if (c) bar(); else baz();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (a) { do if (b) foo(); while (cond); } else bar();",
            "if (a)  do if (b) foo(); while (cond);  else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) do { if (b) foo(); } while (cond); else bar();",
            "if (a) do   if (b) foo();  while (cond); else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { if (b) foo(); else bar(); } else baz();",
            "if (a)  if (b) foo(); else bar();  else baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) while (cond) { bar(); } else baz();",
            "if (a) while (cond)  bar();  else baz();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { for (;;); } else bar();",
            "if (a)  for (;;);  else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "if (a) { while (cond) if (b) foo() } else bar();",
            "if (a) { while (cond) if (b) foo() } else {bar();}",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (a)  while (cond) if (b) foo()
			else
			 {bar();}",
            "if (a)  while (cond) if (b) foo()
			else
			 bar();",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if (a) foo()
			else
			 bar();",
            "if (a) {foo()}
			else
			 {bar();}",
            None,
        ),
        (
            "if (a) { while (cond) if (b) foo() } ",
            "if (a)  while (cond) if (b) foo()  ",
            Some(serde_json::json!(["multi", "consistent"])),
        ),
        (
            "if(a) { if (b) foo(); } if (c) bar(); else if(foo){bar();}",
            "if(a)  if (b) foo();  if (c) bar(); else if(foo)bar();",
            Some(serde_json::json!(["multi-or-nest"])),
        ),
        (
            "if (true) [1, 2, 3]
			.bar()",
            "if (true) {[1, 2, 3]
			.bar()}",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "for(
			;
			;
			) {foo()}",
            "for(
			;
			;
			) foo()",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for(
			;
			;
			)
			foo()
			",
            "for(
			;
			;
			)
			{foo()}
			",
            Some(serde_json::json!(["multi-line"])),
        ),
        (
            "if (a) { while (cond) { if (b) foo(); } } else bar();",
            "if (a)  while (cond) { if (b) foo(); }  else bar();",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "for(;;)foo()
			",
            "for(;;){foo()}
			",
            None,
        ),
        (
            "for(var
			i
			 in
			 z)foo()
			",
            "for(var
			i
			 in
			 z){foo()}
			",
            None,
        ),
        (
            "for(var i of
			 z)
			foo()
			",
            "for(var i of
			 z)
			{foo()}
			",
            None,
        ),
        ("if(I){if(t)s}þ", "if(I){if(t){s}}þ", None),
    ];
    Tester::new(Curly::NAME, Curly::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
