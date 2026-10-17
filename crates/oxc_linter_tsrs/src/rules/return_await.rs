// Port of internal/rules/return_await/return_await.go.

use tsrs_ast::{self as ast, Kind, ModifierFlags, Node, NodeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor};
use crate::utils::{self, TypeAwaitable};

fn disallowed_promise_await() -> RuleMessage {
    RuleMessage::new(
        "disallowedPromiseAwait",
        "Returning an awaited promise is not allowed in this context.",
    )
}
fn disallowed_promise_await_suggestion() -> RuleMessage {
    RuleMessage::new(
        "disallowedPromiseAwaitSuggestion",
        "Remove `await` before the expression. Use caution as this may impact control flow.",
    )
}
fn non_promise_await() -> RuleMessage {
    RuleMessage::new(
        "nonPromiseAwait",
        "Returning an awaited value that is not a promise is not allowed.",
    )
}
fn required_promise_await() -> RuleMessage {
    RuleMessage::new(
        "requiredPromiseAwait",
        "Returning an awaited promise is required in this context.",
    )
}
fn required_promise_await_suggestion() -> RuleMessage {
    RuleMessage::new(
        "requiredPromiseAwaitSuggestion",
        "Add `await` before the expression. Use caution as this may impact control flow.",
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Always,
    ErrorHandlingCorrectnessOnly,
    InTryCatch,
    Never,
}

pub struct ReturnAwait {
    mode: Mode,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let v = match options {
        Some(serde_json::Value::Array(a)) if a.len() == 1 => a.first(),
        other => other,
    };
    let mode = match v.and_then(|v| v.as_str()) {
        None => Mode::InTryCatch,
        Some("always") => Mode::Always,
        Some("error-handling-correctness-only") => Mode::ErrorHandlingCorrectnessOnly,
        Some("in-try-catch") => Mode::InTryCatch,
        Some("never") => Mode::Never,
        Some(other) => return Err(format!("return-await: invalid option {other:?}")),
    };
    Ok(Box::new(ReturnAwait { mode }))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WhetherToAwait {
    DontCare,
    Await,
    NoAwait,
}

fn get_whether_to_await(affects_error_handling: bool, mode: Mode) -> WhetherToAwait {
    match mode {
        Mode::Always => WhetherToAwait::Await,
        Mode::ErrorHandlingCorrectnessOnly => {
            if affects_error_handling {
                WhetherToAwait::Await
            } else {
                WhetherToAwait::DontCare
            }
        }
        Mode::InTryCatch => {
            if affects_error_handling {
                WhetherToAwait::Await
            } else {
                WhetherToAwait::NoAwait
            }
        }
        Mode::Never => WhetherToAwait::NoAwait,
    }
}

const FUNCTION_KINDS: &[Kind] = &[
    Kind::ArrowFunction,
    Kind::FunctionDeclaration,
    Kind::FunctionExpression,
    Kind::MethodDeclaration,
    Kind::Constructor,
    Kind::GetAccessor,
    Kind::SetAccessor,
];

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::Constructor),
    Listener::Enter(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Exit(Kind::ArrowFunction),
    Listener::Exit(Kind::FunctionDeclaration),
    Listener::Exit(Kind::FunctionExpression),
    Listener::Exit(Kind::MethodDeclaration),
    Listener::Exit(Kind::Constructor),
    Listener::Exit(Kind::GetAccessor),
    Listener::Exit(Kind::SetAccessor),
    Listener::Enter(Kind::ReturnStatement),
];

impl Rule for ReturnAwait {
    fn name(&self) -> &'static str {
        "return-await"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { mode: self.mode, scopes: Vec::new() })
    }
}

struct Scope {
    has_async: bool,
    owning_func: P<Node>,
}

struct Visitor {
    mode: Mode,
    scopes: Vec<Scope>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TryBlock {
    Catch,
    Finally,
    Try,
}

fn find_containing_try_statement(node: P<Node>) -> Option<(TryBlock, P<Node>)> {
    let mut child = node;
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if ast::is_function_like(a) {
            break;
        }
        if !ast::is_try_statement(a) {
            child = a;
            ancestor = a.parent();
            continue;
        }
        let s = a.as_try_statement();
        // Go's switch leaves the zero value (Catch) when no case matches.
        let block = if Some(child) == Some(s.try_block) {
            TryBlock::Try
        } else if Some(child) == s.catch_clause {
            TryBlock::Catch
        } else if Some(child) == s.finally_block {
            TryBlock::Finally
        } else {
            TryBlock::Catch
        };
        return Some((block, a));
    }
    None
}

fn affects_explicit_error_handling(mut node: P<Node>) -> bool {
    loop {
        let Some((block, try_statement)) = find_containing_try_statement(node) else {
            return false;
        };
        match block {
            TryBlock::Catch => {
                if try_statement.as_try_statement().finally_block.is_some() {
                    return true;
                }
                node = try_statement;
            }
            TryBlock::Finally => node = try_statement,
            TryBlock::Try => return true,
        }
    }
}

impl Visitor {
    fn affects_explicit_resource_management(&self, node: P<Node>) -> bool {
        let scope = self.scopes.last().unwrap();
        if !scope.owning_func.body().is_some_and(ast::is_block) {
            return false;
        }
        let mut declaration_scope = ast::get_enclosing_block_scope_container(node);
        while let Some(ds) = declaration_scope {
            if let Some(locals) = ds.locals() {
                let mut found = false;
                locals.for_each(|_, local| {
                    if let Some(decl) = local.value_declaration() {
                        if ast::is_variable_declaration(decl)
                            && decl.parent().is_some_and(|p| p.flags().intersects(NodeFlags::Using))
                            && decl.pos() < node.pos()
                        {
                            found = true;
                        }
                    }
                });
                if found {
                    return true;
                }
            }
            if scope.owning_func == ds {
                break;
            }
            declaration_scope = ast::get_enclosing_block_scope_container(ds);
        }
        false
    }

    fn insert_await_fix(ctx: &Ctx, node: P<Node>, is_high_precedence: bool) -> Vec<RuleFix> {
        if is_high_precedence {
            return vec![ctx.fix_insert_before(node, "await ")];
        }
        vec![ctx.fix_insert_before(node, "await ("), ctx.fix_insert_after(node, ")")]
    }

    fn remove_await_fix(ctx: &Ctx, node: P<Node>) -> RuleFix {
        let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
        ctx.fix_remove_range(r.pos(), r.end())
    }

    fn test(&self, ctx: &mut Ctx, node: P<Node>) {
        let is_await = ast::is_await_expression(node);
        let child = if is_await { node.expression().unwrap() } else { node };
        let t = ctx.checker.get_type_at_location(child);
        let certainty = utils::needs_to_be_awaited(ctx.checker, node, t);
        if certainty != TypeAwaitable::Always {
            if is_await {
                if certainty == TypeAwaitable::May {
                    return;
                }
                ctx.report_node_with_fixes(node, non_promise_await(), |ctx| {
                    vec![Self::remove_await_fix(ctx, node)]
                });
            }
            return;
        }
        let affects_error_handling = affects_explicit_error_handling(node)
            || self.affects_explicit_resource_management(node);
        let use_auto_fix = !affects_error_handling;
        match get_whether_to_await(affects_error_handling, self.mode) {
            WhetherToAwait::Await => {
                if !is_await {
                    let fixes = Self::insert_await_fix(
                        ctx,
                        node,
                        utils::is_higher_precedence_than_await(node),
                    );
                    ctx.report_node_with_fixes_or_suggestions(
                        node,
                        use_auto_fix,
                        required_promise_await(),
                        required_promise_await_suggestion(),
                        fixes,
                    );
                }
            }
            WhetherToAwait::DontCare => {}
            WhetherToAwait::NoAwait => {
                if is_await {
                    let fix = Self::remove_await_fix(ctx, node);
                    ctx.report_node_with_fixes_or_suggestions(
                        node,
                        use_auto_fix,
                        disallowed_promise_await(),
                        disallowed_promise_await_suggestion(),
                        vec![fix],
                    );
                }
            }
        }
    }

    fn test_each_possibly_returned_node(&self, ctx: &mut Ctx, node: P<Node>) {
        let node = ast::skip_parentheses(node);
        if node.kind() == Kind::ConditionalExpression {
            let e = node.as_conditional_expression();
            self.test_each_possibly_returned_node(ctx, e.when_false);
            self.test_each_possibly_returned_node(ctx, e.when_true);
        } else {
            self.test(ctx, node);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::ReturnStatement) => {
                let Some(expr) = node.expression() else {
                    return;
                };
                if !self.scopes.last().is_some_and(|s| s.has_async) {
                    return;
                }
                self.test_each_possibly_returned_node(ctx, expr);
            }
            Listener::Enter(k) if FUNCTION_KINDS.contains(&k) => {
                self.scopes.push(Scope {
                    has_async: ast::has_syntactic_modifier(node, ModifierFlags::Async),
                    owning_func: node,
                });
            }
            Listener::Exit(Kind::ArrowFunction) => {
                if let Some(body) = node.body() {
                    if !ast::is_block(body) && self.scopes.last().is_some_and(|s| s.has_async) {
                        self.test_each_possibly_returned_node(ctx, body);
                    }
                }
                self.scopes.pop();
            }
            Listener::Exit(_) => {
                self.scopes.pop();
            }
            _ => {}
        }
    }
}
