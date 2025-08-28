use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn curly_diagnostic(span: Span, keyword: &str, expected: bool) -> OxcDiagnostic {
    let condition_if_needed =
        matches!(keyword, "if" | "while" | "for").then_some(" condition").unwrap_or("");

    let prefix = if expected { "Expected" } else { "Unexpected" };
    let message = format!("{prefix} {{ after '{keyword}'{condition_if_needed}.");
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq)]
enum CurlyType {
    #[default]
    All,
    Multi,
    MultiLine,
    MultiOrNest,
    Consistent,
}

impl CurlyType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "multi" => Self::Multi,
            "multi-line" => Self::MultiLine,
            "multi-or-nest" => Self::MultiOrNest,
            "consistent" => Self::Consistent,
            _ => Self::All,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Curly(Box<CurlyConfig>);

#[derive(Debug, Clone)]
pub struct CurlyConfig {
    options: Vec<CurlyType>,
}

impl Default for CurlyConfig {
    fn default() -> Self {
        Self { options: vec![CurlyType::All] }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces the use of curly braces `{}` for all control statements (`if`, `else`, `for`, `while`, `do`, etc.).
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
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (foo) foo++;
    ///
    /// for (let i = 0; i < 10; i++) doSomething(i);
    ///
    /// while (bar) bar--;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (foo) {
    ///     foo++;
    /// }
    ///
    /// for (let i = 0; i < 10; i++) {
    ///     doSomething(i);
    /// }
    ///
    /// while (bar) {
    ///     bar--;
    /// }
    /// ```
    Curly,
    eslint,
    style,
    fix
);

impl Rule for Curly {
    fn from_configuration(value: serde_json::Value) -> Self {
        let options = value.as_array().filter(|array| !array.is_empty()).map_or_else(
            || vec![CurlyType::All],
            |array| array.iter().filter_map(|v| v.as_str().map(CurlyType::from)).collect(),
        );

        Self(Box::new(CurlyConfig { options }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(stmt) => {
                struct StatementInfo<'a> {
                    statement: &'a Statement<'a>,
                    is_else: bool,
                    should_have_braces: Option<bool>,
                    has_braces: bool,
                }

                let mut infos = vec![StatementInfo {
                    statement: &stmt.consequent,
                    is_else: false,
                    should_have_braces: should_have_braces(&self.0.options, &stmt.consequent, ctx),
                    has_braces: has_braces(&stmt.consequent),
                }];

                let mut current_statement = &stmt.alternate;

                while let Some(statement) = current_statement {
                    if let Statement::IfStatement(node) = statement {
                        infos.push(StatementInfo {
                            statement: &node.consequent,
                            is_else: false,
                            should_have_braces: should_have_braces(
                                &self.0.options,
                                &node.consequent,
                                ctx,
                            ),
                            has_braces: has_braces(&node.consequent),
                        });
                        current_statement = &node.alternate;
                    } else {
                        infos.push(StatementInfo {
                            statement,
                            is_else: true,
                            should_have_braces: should_have_braces(&self.0.options, statement, ctx),
                            has_braces: has_braces(statement),
                        });
                        break;
                    }
                }

                let keyword = |is_else: bool| if is_else { "else" } else { "if" };

                if self.0.options.contains(&CurlyType::Consistent) {
                    let mut expected = Some(false);

                    for info in &infos {
                        expected = Some(info.should_have_braces.unwrap_or(info.has_braces));
                        if expected == Some(true) {
                            break;
                        }
                    }

                    for info in &infos {
                        report_if_needed(
                            ctx,
                            info.statement,
                            keyword(info.is_else),
                            info.has_braces,
                            expected,
                        );
                    }
                } else {
                    for info in &infos {
                        report_if_needed(
                            ctx,
                            info.statement,
                            keyword(info.is_else),
                            info.has_braces,
                            info.should_have_braces,
                        );
                    }
                }
            }
            AstKind::ForStatement(stmt) => {
                report_if_needed(
                    ctx,
                    &stmt.body,
                    "for",
                    has_braces(&stmt.body),
                    should_have_braces(&self.0.options, &stmt.body, ctx),
                );
            }
            AstKind::ForInStatement(stmt) => {
                report_if_needed(
                    ctx,
                    &stmt.body,
                    "for-in",
                    has_braces(&stmt.body),
                    should_have_braces(&self.0.options, &stmt.body, ctx),
                );
            }
            AstKind::ForOfStatement(stmt) => {
                report_if_needed(
                    ctx,
                    &stmt.body,
                    "for-of",
                    has_braces(&stmt.body),
                    should_have_braces(&self.0.options, &stmt.body, ctx),
                );
            }
            AstKind::WhileStatement(stmt) => {
                report_if_needed(
                    ctx,
                    &stmt.body,
                    "while",
                    has_braces(&stmt.body),
                    should_have_braces(&self.0.options, &stmt.body, ctx),
                );
            }
            AstKind::DoWhileStatement(stmt) => {
                report_if_needed(
                    ctx,
                    &stmt.body,
                    "do",
                    has_braces(&stmt.body),
                    should_have_braces(&self.0.options, &stmt.body, ctx),
                );
            }
            _ => {}
        }
    }
}

fn get_node_by_statement<'a>(statement: &'a Statement, ctx: &'a LintContext) -> &'a AstNode<'a> {
    let span = statement.span();
    ctx.nodes().iter().find(|n| n.span() == span).expect("Failed to get node by statement")
}

fn has_braces(body: &Statement) -> bool {
    matches!(body, Statement::BlockStatement(_))
}

fn should_have_braces<'a>(
    options: &[CurlyType],
    body: &Statement<'a>,
    ctx: &LintContext<'a>,
) -> Option<bool> {
    let is_block = matches!(body, Statement::BlockStatement(_));
    let is_not_single_statement = match body {
        Statement::BlockStatement(block) => block.body.len() != 1,
        _ => true,
    };
    let braces_necessary = are_braces_necessary(body, ctx);

    if is_block && (is_not_single_statement || braces_necessary) {
        Some(true)
    } else if options.contains(&CurlyType::Multi) {
        Some(false)
    } else if options.contains(&CurlyType::MultiLine) {
        if is_collapsed_one_liner(body, ctx) { None } else { Some(true) }
    } else if options.contains(&CurlyType::MultiOrNest) {
        Some(if is_block {
            let stmt = match body {
                Statement::BlockStatement(block) => block.body.first(),
                _ => None,
            };
            let body_start = body.span().start;
            let stmt_start = stmt.map_or(body_start, |stmt| stmt.span().start);
            let comments = ctx.comments_range(body_start..stmt_start - 1);

            stmt.is_none_or(|stmt| !is_one_liner(stmt, ctx) || comments.count() > 0)
        } else {
            !is_one_liner(body, ctx)
        })
    } else {
        Some(true)
    }
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
        if should_have_braces {
            let fixed = format!("{{{}}}", ctx.source_range(body.span()));
            fixer.replace(body.span(), fixed)
        } else {
            let needs_preceding_space = matches!(
                ctx.nodes().parent_kind(get_node_by_statement(body, ctx).id()),
                AstKind::DoWhileStatement(_)
            );
            let mut fixed = ctx.source_range(body.span()).to_string();

            if let Some(stripped) = fixed.strip_prefix(|c: char| c.is_whitespace() || c == '{') {
                fixed = stripped.to_string();
            }

            if let Some(stripped) = fixed.strip_suffix(|c: char| c.is_whitespace() || c == '}') {
                fixed = stripped.to_string();
            } else if fixed.ends_with('}') {
                fixed.pop();
            }

            if needs_preceding_space {
                fixed.insert(0, ' ');
            }
            fixer.replace(body.span(), fixed)
        }
    });
}

#[expect(clippy::cast_possible_truncation)] // for `as i32`
fn is_collapsed_one_liner(node: &Statement, ctx: &LintContext) -> bool {
    let node = get_node_by_statement(node, ctx);
    let span = node.span();
    let node_string = ctx.source_range(span);

    let trimmed_len =
        u32::try_from(node_string.trim_end_matches(|c: char| c.is_whitespace() || c == ';').len())
            .expect("Failed to convert to u32");

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

    let text = ctx.source_range(Span::new(
        next_char_offset,
        span.end - ((node_string.len() as u32) - trimmed_len),
    ));

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

    if let Some(tail) = src.get(start..) {
        if tail.starts_with("\r\n") || tail.starts_with("\n\r") {
            return Some(span.end + 2);
        }
    }

    src[start..].chars().next().map(|c| span.end + c.len_utf8() as u32)
}

#[expect(clippy::cast_possible_truncation)] // for `as i32`
fn is_followed_by_else_keyword(node: &Statement, ctx: &LintContext) -> bool {
    let Some(next_char_offset) = get_next_char_offset(node.span(), ctx) else {
        return false;
    };

    let start = next_char_offset;
    let end = ctx.source_text().len() as u32;

    if start > end {
        return false;
    }

    let followed_source = ctx.source_range(Span::new(start, end));
    followed_source.trim_start().starts_with("else")
        && followed_source.trim_start_matches("else").starts_with([' ', ';', '{'])
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
