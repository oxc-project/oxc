// Port of internal/rules/require_await/require_await.go.

use tsrs_ast::{self as ast, FunctionFlags, Kind, Node, NodeList, SourceFile};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor,
};
use crate::utils;

fn missing_await() -> RuleMessage {
    RuleMessage::new("missingAwait", "Function has no 'await' expression.")
}
fn remove_async() -> RuleMessage {
    RuleMessage::new("removeAsync", "Remove 'async'.")
}

struct ScopeInfo {
    has_await: bool,
    is_async_yield: bool,
    function_flags: FunctionFlags,
}

fn is_method_like(node: P<Node>) -> bool {
    ast::is_method_or_accessor(node) || ast::is_constructor_declaration(node)
}

/// Whether the node immediately preceding `node` in `list` is a method, constructor or
/// accessor.
fn previous_sibling_is_method_like(list: Option<P<NodeList>>, node: P<Node>) -> bool {
    let Some(list) = list else { return false };
    let nodes = list.nodes();
    for (i, &m) in nodes.iter().enumerate() {
        if m == node {
            return i > 0 && is_method_like(nodes[i - 1]);
        }
    }
    false
}

/// Whether inserting a `[` or `(` at `node`'s head would require a preceding semicolon (the
/// ASI hazard).
fn needs_preceding_semicolon(source_file: P<SourceFile>, node: P<Node>) -> bool {
    let pos = tsrs_scanner::get_token_pos_of_node(node, source_file, false);
    let text = source_file.text().as_bytes();
    let mut last_pos: i64 = -1;
    let mut i = pos as i64 - 1;
    while i >= 0 {
        let c = text[i as usize];
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
            i -= 1;
            continue;
        }
        last_pos = i;
        break;
    }
    if last_pos < 0 {
        return false;
    }
    let last_pos = last_pos as usize;
    let ch = text[last_pos];
    if matches!(ch, b';' | b':' | b'{' | b',' | b'(' | b'[') {
        return false;
    }
    if last_pos >= 1 {
        let two = &text[last_pos - 1..last_pos + 1];
        if two == b"=>" || two == b"++" || two == b"--" {
            return false;
        }
    }
    // A closing `}` that ends a class/object-literal member is self-delimiting.
    if ch == b'}' {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                Kind::ClassDeclaration | Kind::ClassExpression | Kind::InterfaceDeclaration
                    if previous_sibling_is_method_like(parent.member_list(), node) =>
                {
                    return false;
                }
                Kind::ObjectLiteralExpression
                    if previous_sibling_is_method_like(
                        Some(parent.as_object_literal_expression().properties),
                        node,
                    ) =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }
    true
}

/// Whether `type_ref` names the built-in type `name`, as a plain identifier or as a direct
/// globalThis qualifier.
fn is_type_reference_named(type_ref: P<Node>, name: &str) -> bool {
    let tn = type_ref.as_type_reference_node().type_name;
    if tn.kind() == Kind::Identifier {
        return tn.text() == name;
    }
    if !ast::is_qualified_name(tn) {
        return false;
    }
    let q = tn.as_qualified_name();
    q.right.text() == name && q.left.kind() == Kind::Identifier && q.left.text() == "globalThis"
}

fn build_remove_async_fixes(
    source_file: P<SourceFile>,
    node: P<Node>,
    async_token: P<Node>,
    is_generator: bool,
) -> Vec<RuleFix> {
    let mut fixes = Vec::with_capacity(3);
    let text = source_file.text();
    let async_start = tsrs_scanner::get_token_pos_of_node(async_token, source_file, false);
    // Remove the `async` keyword plus trailing whitespace, but preserve trailing comments.
    let remove_end = tsrs_scanner::skip_trivia_ex(
        text,
        async_token.end(),
        Some(&tsrs_scanner::SkipTriviaOptions { stop_at_comments: true, ..Default::default() }),
    );
    let mut add_semicolon = false;
    let next_token = tsrs_scanner::scan_token_at_position(source_file, async_token.end());
    if matches!(
        next_token,
        Kind::OpenParenToken
            | Kind::OpenBracketToken
            | Kind::NoSubstitutionTemplateLiteral
            | Kind::TemplateHead
    ) && (is_at_start_of_expression_statement(node) || is_method_like(node))
        && needs_preceding_semicolon(source_file, node)
    {
        add_semicolon = true;
    }
    fixes.push(RuleFix {
        text: if add_semicolon { ";" } else { "" }.to_string(),
        pos: async_start,
        end: remove_end,
    });

    let Some(return_type) = node.type_node() else {
        return fixes;
    };
    if return_type.kind() != Kind::TypeReference {
        return fixes;
    }
    let type_name = return_type.as_type_reference_node().type_name;
    let type_name_start = tsrs_scanner::get_token_pos_of_node(type_name, source_file, false);
    if is_generator && is_type_reference_named(return_type, "AsyncGenerator") {
        fixes.push(RuleFix {
            text: "Generator".to_string(),
            pos: type_name_start,
            end: type_name.end(),
        });
    } else if !is_generator && is_type_reference_named(return_type, "Promise") {
        if let Some(type_arguments) = return_type.as_type_reference_node().type_arguments() {
            if !type_arguments.nodes().is_empty() {
                // Unwrap `Promise<T>` to `T` by deleting `Promise<` and the trailing `>`.
                let open_angle_pos = type_arguments.pos() - 1;
                let close_angle_pos = type_arguments.end();
                fixes.push(RuleFix {
                    text: String::new(),
                    pos: close_angle_pos,
                    end: close_angle_pos + 1,
                });
                fixes.push(RuleFix {
                    text: String::new(),
                    pos: type_name_start,
                    end: open_angle_pos + 1,
                });
            }
        }
    }
    fixes
}

/// Whether `node` is at the same source position as an ancestor ExpressionStatement.
fn is_at_start_of_expression_statement(node: P<Node>) -> bool {
    let start = node.pos();
    let mut a = node.parent();
    while let Some(n) = a {
        if n.pos() != start {
            break;
        }
        if n.kind() == Kind::ExpressionStatement {
            return true;
        }
        a = n.parent();
    }
    false
}

pub struct RequireAwait;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(RequireAwait))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Exit(Kind::FunctionDeclaration),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Exit(Kind::MethodDeclaration),
    Listener::Enter(Kind::Constructor),
    Listener::Exit(Kind::Constructor),
    Listener::Enter(Kind::GetAccessor),
    Listener::Exit(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Exit(Kind::SetAccessor),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Exit(Kind::FunctionExpression),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Exit(Kind::ArrowFunction),
    Listener::Enter(Kind::AwaitExpression),
    Listener::Enter(Kind::ForOfStatement),
    Listener::Enter(Kind::VariableDeclarationList),
    Listener::Enter(Kind::YieldExpression),
    Listener::Enter(Kind::ReturnStatement),
];

impl Rule for RequireAwait {
    fn name(&self) -> &'static str {
        "require-await"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { scopes: Vec::new() })
    }
}

struct Visitor {
    scopes: Vec<ScopeInfo>,
}

impl Visitor {
    fn enter_function(&mut self, node: P<Node>) {
        let mut scope = ScopeInfo {
            has_await: false,
            is_async_yield: false,
            function_flags: FunctionFlags::Normal,
        };
        if let Some(body) = node.body() {
            if !ast::is_block(body) || !body.statements().is_empty() {
                scope.function_flags = ast::get_function_flags(Some(node));
            }
        }
        self.scopes.push(scope);
    }

    fn exit_function(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let scope = self.scopes.pop().unwrap();
        let is_async = scope.function_flags.intersects(FunctionFlags::Async);
        let is_gen = scope.function_flags.intersects(FunctionFlags::Generator);
        if is_async && !scope.has_await && !(is_gen && scope.is_async_yield) {
            // `is_async` guarantees the node has an `async` modifier.
            let async_token = utils::find_modifier(node, Kind::AsyncKeyword).unwrap();
            let (pos, end) = utils::get_function_head_loc(ctx.file, node);
            ctx.report_diagnostic_with_suggestions(
                RuleDiagnostic { pos, end, message: missing_await(), labeled_ranges: Vec::new() },
                |ctx| {
                    vec![RuleSuggestion {
                        message: remove_async(),
                        fixes: build_remove_async_fixes(ctx.file, node, async_token, is_gen),
                    }]
                },
            );
        }
    }

    fn mark_as_has_await(&mut self) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.has_await = true;
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        if let Listener::Exit(_) = listener {
            self.exit_function(ctx, node);
            return;
        }
        match node.kind() {
            Kind::FunctionDeclaration
            | Kind::MethodDeclaration
            | Kind::Constructor
            | Kind::GetAccessor
            | Kind::SetAccessor
            | Kind::FunctionExpression => self.enter_function(node),
            Kind::ArrowFunction => {
                self.enter_function(node);
                // check body-less async arrow function.
                // ignore `async () => await foo` because it's obviously correct
                if !self.scopes.last().unwrap().function_flags.intersects(FunctionFlags::Async) {
                    return;
                }
                let body = ast::skip_parentheses(node.body().unwrap());
                if ast::is_block(body) || ast::is_await_expression(body) {
                    return;
                }
                let t = ctx.checker.get_type_at_location(body);
                if utils::is_thenable_type(ctx.checker, body, Some(t)) {
                    self.mark_as_has_await();
                }
            }
            Kind::AwaitExpression => self.mark_as_has_await(),
            Kind::ForOfStatement if node.as_for_in_or_of_statement().await_modifier.is_some() => {
                self.mark_as_has_await()
            }
            Kind::VariableDeclarationList if ast::is_var_await_using(node) => {
                self.mark_as_has_await()
            }
            // Mark `is_async_yield` if it 1) delegates async generator function or 2) yields
            // thenable type.
            Kind::YieldExpression => {
                let Some(scope) = self.scopes.last() else {
                    return;
                };
                if scope.is_async_yield {
                    return;
                }
                let argument = node.expression();
                let Some(argument) = argument else { return };
                if !scope.function_flags.intersects(FunctionFlags::Generator) {
                    return;
                }
                if ast::is_literal_expression(argument) {
                    // ignoring this as for literals we don't need to check the definition
                    // eg : async function* run() { yield* 1 }
                    return;
                }
                let t = ctx.checker.get_type_at_location(argument);
                let is_async_yield = if node.as_yield_expression().asterisk_token.is_none() {
                    utils::is_thenable_type(ctx.checker, argument, Some(t))
                } else {
                    utils::type_recurser(t, &mut |t| {
                        utils::get_well_known_symbol_property_of_type(
                            t,
                            "asyncIterator",
                            ctx.checker,
                        )
                        .is_some()
                    })
                };
                if is_async_yield {
                    self.scopes.last_mut().unwrap().is_async_yield = true;
                }
            }
            Kind::ReturnStatement => {
                let Some(scope) = self.scopes.last() else {
                    return;
                };
                if scope.has_await || !scope.function_flags.intersects(FunctionFlags::Async) {
                    return;
                }
                if let Some(expr) = node.expression() {
                    let t = ctx.checker.get_type_at_location(expr);
                    if utils::is_thenable_type(ctx.checker, expr, Some(t)) {
                        self.mark_as_has_await();
                    }
                }
            }
            _ => {}
        }
    }
}
