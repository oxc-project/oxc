// Port of internal/rules/prefer_includes/prefer_includes.go.

use tsrs_ast::{self as ast, Kind, Node, Symbol};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor};
use crate::utils;

fn build_prefer_includes_message() -> RuleMessage {
    RuleMessage::new("preferIncludes", "Use 'includes()' method instead.")
}

fn build_prefer_string_includes_message() -> RuleMessage {
    RuleMessage::new(
        "preferStringIncludes",
        "Use `String#includes()` method with a string instead.",
    )
}

pub struct PreferIncludes;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(PreferIncludes))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::CallExpression), Listener::Enter(Kind::BinaryExpression)];

impl Rule for PreferIncludes {
    fn name(&self) -> &'static str {
        "prefer-includes"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

// Escape special characters for string literal
// The pattern from regex already has escape sequences, we only need to escape apostrophes
fn escape_string(s: &str) -> String {
    s.replace('\'', "\\'")
}

// Check if a pattern string contains only simple literal characters
// The TypeScript version uses a proper ECMAScript regex parser (@eslint-community/regexpp),
// but we just check for unescaped metacharacters and escaped special sequences.
fn is_simple_literal_pattern(pattern: &str) -> bool {
    let mut prev_rune = '\0';
    for ch in pattern.chars() {
        if prev_rune != '\\' {
            // Unescaped regex metacharacters
            if matches!(
                ch,
                '.' | '*' | '+' | '?' | '|' | '^' | '$' | '[' | ']' | '(' | ')' | '{' | '}'
            ) {
                return false;
            }
        } else {
            // Escaped sequences that are regex metacharacters (not simple literals)
            if matches!(ch, 'd' | 'D' | 'w' | 'W' | 's' | 'S' | 'b' | 'B' | 'c' | 'x' | 'u') {
                return false;
            }
        }
        prev_rune = ch;
    }
    true
}

// Extract pattern from regex literal: /bar/ -> "bar"
fn extract_regex_literal_pattern(node: P<Node>) -> Option<&'static str> {
    if node.kind() != Kind::RegularExpressionLiteral {
        return None;
    }
    let text = node.text(); // e.g., "/bar/" or "/bar/i"
    let bytes = text.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'/' {
        return None;
    }
    let last_slash = (1..bytes.len()).rev().find(|&i| bytes[i] == b'/')?;
    let pattern = &text[1..last_slash];
    let flags = &text[last_slash + 1..];
    // Reject patterns with any flags
    if !flags.is_empty() {
        return None;
    }
    // Reject non-literal patterns before compiling them.
    if !is_simple_literal_pattern(pattern)
        || !super::prefer_regexp_exec::regexp2_syntax::compiles_ecmascript(pattern)
    {
        return None;
    }
    non_empty(pattern)
}

fn non_empty(s: &'static str) -> Option<&'static str> {
    if s.is_empty() { None } else { Some(s) }
}

// Extract pattern from RegExp constructor: new RegExp('bar') -> "bar"
fn extract_reg_exp_constructor_pattern(node: P<Node>) -> Option<&'static str> {
    if node.kind() != Kind::NewExpression {
        return None;
    }
    let expression = node.expression().unwrap();
    if expression.kind() != Kind::Identifier || expression.text() != "RegExp" {
        return None;
    }
    let args = node.arguments();
    if args.is_empty() || args[0].kind() != Kind::StringLiteral {
        return None;
    }
    let pattern = args[0].text();
    // Reject non-literal patterns before compiling them.
    if !is_simple_literal_pattern(pattern)
        || !super::prefer_regexp_exec::regexp2_syntax::compiles_ecmascript(pattern)
    {
        return None;
    }
    non_empty(pattern)
}

// Resolve pattern from variable: const p = /bar/; p.test(...) -> "bar"
fn resolve_variable_pattern(ctx: &mut Ctx, node: P<Node>) -> Option<&'static str> {
    if !ast::is_identifier(node) {
        return None;
    }
    let symbol = ctx.checker.get_symbol_at_location_exported(node)?;
    let value_decl = symbol.value_declaration()?;
    if value_decl.kind() != Kind::VariableDeclaration {
        return None;
    }
    let initializer = value_decl.initializer()?;
    // Try regex literal: const p = /bar/
    extract_regex_literal_pattern(initializer)
        // Try RegExp constructor: const p = new RegExp('bar')
        .or_else(|| extract_reg_exp_constructor_pattern(initializer))
}

// Resolve a regex pattern from a node (direct regex literal or a variable reference).
fn resolve_regex_pattern(ctx: &mut Ctx, node: P<Node>) -> Option<&'static str> {
    extract_regex_literal_pattern(node).or_else(|| resolve_variable_pattern(ctx, node))
}

// Check if two function declarations have matching parameter signatures
// Compares the full text of each parameter (name, type annotation, and optionality)
fn has_same_parameters(decl_a: P<Node>, decl_b: P<Node>) -> bool {
    if !ast::is_function_like(decl_a) || !ast::is_function_like(decl_b) {
        return false;
    }
    let params_a = decl_a.parameters();
    let params_b = decl_b.parameters();
    if params_a.len() != params_b.len() {
        return false;
    }
    for (&param_a, &param_b) in params_a.iter().zip(params_b) {
        let (Some(sf_a), Some(sf_b)) =
            (ast::get_source_file_of_node(param_a), ast::get_source_file_of_node(param_b))
        else {
            return false;
        };
        let text_a = &sf_a.text()[param_a.pos() as usize..param_a.end() as usize];
        let text_b = &sf_b.text()[param_b.pos() as usize..param_b.end() as usize];
        if text_a != text_b {
            return false;
        }
    }
    true
}

// Check if the indexOf symbol has a compatible includes method
// Verifies that for every indexOf declaration, there exists an includes
// declaration on the same type with matching parameters
fn index_of_has_compatible_includes(ctx: &mut Ctx, index_of_symbol: P<Symbol>) -> bool {
    let declarations = index_of_symbol.declarations();
    if declarations.is_empty() {
        return false;
    }
    for &index_of_decl in declarations.iter() {
        let Some(type_decl) = index_of_decl.parent() else {
            return false;
        };
        let t = ctx.checker.get_type_at_location(type_decl);
        let Some(includes_symbol) = ctx.checker.get_property_of_type(t, "includes") else {
            return false;
        };
        let includes_decls = includes_symbol.declarations();
        if includes_decls.is_empty() {
            return false;
        }
        if !includes_decls
            .iter()
            .any(|&includes_decl| has_same_parameters(index_of_decl, includes_decl))
        {
            return false;
        }
    }
    true
}

// Check if the node is a number literal with specific value
// Handles both numeric literals (0) and prefix unary expressions (-1)
fn is_number_literal(node: P<Node>, value: i64) -> bool {
    if node.kind() == Kind::NumericLiteral {
        if let Ok(num) = node.text().parse::<i64>() {
            return num == value;
        }
    }
    if node.kind() == Kind::PrefixUnaryExpression {
        let prefix_expr = node.as_prefix_unary_expression();
        if prefix_expr.operator == Kind::MinusToken
            && prefix_expr.operand.kind() == Kind::NumericLiteral
        {
            if let Ok(num) = prefix_expr.operand.text().parse::<i64>() {
                return -num == value;
            }
        }
    }
    false
}

// Patterns: !== -1, != -1, > -1, >= 0
fn is_positive_check(binary_expr: P<Node>) -> bool {
    let b = binary_expr.as_binary_expression();
    let right = b.right.get();
    match b.operator_token.kind() {
        Kind::ExclamationEqualsEqualsToken
        | Kind::ExclamationEqualsToken
        | Kind::GreaterThanToken => is_number_literal(right, -1),
        Kind::GreaterThanEqualsToken => is_number_literal(right, 0),
        _ => false,
    }
}

// Patterns: === -1, == -1, <= -1, < 0
fn is_negative_check(binary_expr: P<Node>) -> bool {
    let b = binary_expr.as_binary_expression();
    let right = b.right.get();
    match b.operator_token.kind() {
        Kind::EqualsEqualsEqualsToken | Kind::EqualsEqualsToken | Kind::LessThanEqualsToken => {
            is_number_literal(right, -1)
        }
        Kind::LessThanToken => is_number_literal(right, 0),
        _ => false,
    }
}

impl Visitor {
    // Handle: /regex/.test(str) -> str.includes('literal')
    fn check_call_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let callee = node.expression().unwrap();
        if callee.kind() != Kind::PropertyAccessExpression {
            return;
        }
        let name_node = callee.name().unwrap();
        if !ast::is_identifier(name_node) || name_node.text() != "test" {
            return;
        }
        if node.arguments().len() != 1 {
            return;
        }
        let regex_node = callee.expression().unwrap();
        let Some(pattern) = resolve_regex_pattern(ctx, regex_node) else {
            return;
        };
        let argument = node.arguments()[0];
        let arg_type = utils::get_constrained_type_at_location(ctx.checker, argument);
        let Some(includes_symbol) = ctx.checker.get_property_of_type(arg_type, "includes") else {
            return;
        };
        if includes_symbol.declarations().is_empty() {
            return;
        }
        let needs_parens = !matches!(
            argument.kind(),
            Kind::Identifier
                | Kind::StringLiteral
                | Kind::NumericLiteral
                | Kind::NoSubstitutionTemplateLiteral
                | Kind::PropertyAccessExpression
                | Kind::CallExpression
                | Kind::ElementAccessExpression
                | Kind::ParenthesizedExpression
        );
        let (call_pos, call_end) = ctx.trim(node);
        let mut fixes: Vec<RuleFix> = Vec::new();
        fixes.push(ctx.fix_remove_range(call_pos, argument.pos()));
        fixes.push(ctx.fix_remove_range(argument.end(), call_end));
        if needs_parens {
            fixes.push(ctx.fix_replace_range(argument.pos(), argument.pos(), "("));
            fixes.push(ctx.fix_replace_range(argument.end(), argument.end(), ")"));
        }
        let escaped_pattern = escape_string(pattern);
        fixes.push(ctx.fix_replace_range(
            argument.end(),
            argument.end(),
            format!(".includes('{escaped_pattern}')"),
        ));
        ctx.report_node_with_fixes(node, build_prefer_string_includes_message(), |_| fixes);
    }

    // Handle: array.indexOf(item) !== -1 -> array.includes(item)
    fn check_binary_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let left = node.as_binary_expression().left;
        if left.kind() != Kind::CallExpression {
            return;
        }
        let callee = left.expression().unwrap();
        if callee.kind() != Kind::PropertyAccessExpression {
            return;
        }
        let name_node = callee.name().unwrap();
        if !ast::is_identifier(name_node) || name_node.text() != "indexOf" {
            return;
        }
        let is_positive = is_positive_check(node);
        let is_negative = is_negative_check(node);
        if !is_positive && !is_negative {
            return;
        }
        let Some(index_of_symbol) = ctx.checker.get_symbol_at_location_exported(name_node) else {
            return;
        };
        if !index_of_has_compatible_includes(ctx, index_of_symbol) {
            return;
        }
        let mut fixes: Vec<RuleFix> = Vec::new();
        fixes.push(ctx.fix_replace(name_node, "includes"));
        fixes.push(ctx.fix_remove_range(left.end(), node.end()));
        if is_negative {
            let (call_pos, _) = ctx.trim(left);
            fixes.push(ctx.fix_replace_range(call_pos, call_pos, "!"));
        }
        ctx.report_node_with_fixes(node, build_prefer_includes_message(), |_| fixes);
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::CallExpression) => self.check_call_expression(ctx, node),
            Listener::Enter(Kind::BinaryExpression) => self.check_binary_expression(ctx, node),
            _ => {}
        }
    }
}
