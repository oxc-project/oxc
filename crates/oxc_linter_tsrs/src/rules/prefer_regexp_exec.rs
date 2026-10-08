// Port of internal/rules/prefer_regexp_exec/prefer_regexp_exec.go.

/// Port of the github.com/dlclark/regexp2/v2 parser (also used by prefer-includes).
pub(crate) mod regexp2_syntax;
mod regexp2_tables;

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, NodeFlags, Symbol};
use tsrs_checker::Type;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor};
use crate::utils;

fn build_reg_exp_exec_over_string_match_message() -> RuleMessage {
    RuleMessage::new("regExpExecOverStringMatch", "Use the `RegExp#exec()` method instead.")
}

const ARGUMENT_TYPE_OTHER: i32 = 0;
// Go: argumentTypeString = 1 << iota (iota == 1), argumentTypeRegExp = 1 << 2.
const ARGUMENT_TYPE_STRING: i32 = 1 << 1;
const ARGUMENT_TYPE_REG_EXP: i32 = 1 << 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StaticArgumentValueKind {
    Unknown,
    String,
    RegExp,
    Other,
}

struct StaticArgumentValue {
    kind: StaticArgumentValueKind,
    reg_exp_flags: &'static str,
}

impl StaticArgumentValue {
    fn of(kind: StaticArgumentValueKind) -> StaticArgumentValue {
        StaticArgumentValue { kind, reg_exp_flags: "" }
    }
}

pub struct PreferRegexpExec;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(PreferRegexpExec))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for PreferRegexpExec {
    fn name(&self) -> &'static str {
        "prefer-regexp-exec"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_node_parenthesized(node: P<Node>) -> bool {
    node.parent().is_some_and(|parent| {
        ast::is_parenthesized_expression(parent) && parent.expression() == Some(node)
    })
}

fn is_weak_precedence_parent(node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        Kind::PostfixUnaryExpression
        | Kind::PrefixUnaryExpression
        | Kind::BinaryExpression
        | Kind::ConditionalExpression
        | Kind::AwaitExpression => return true,
        _ => {}
    }
    if ast::is_property_access_expression(parent) || ast::is_element_access_expression(parent) {
        return parent.expression() == Some(node);
    }
    if ast::is_call_expression(parent) || ast::is_new_expression(parent) {
        return parent.expression() == Some(node);
    }
    if ast::is_tagged_template_expression(parent) {
        return parent.as_tagged_template_expression().tag == node;
    }
    false
}

fn get_node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    let (pos, end) = ctx.trim(node);
    &ctx.text()[pos as usize..end as usize]
}

fn build_wrapping_fix(
    ctx: &Ctx,
    node: P<Node>,
    inner_nodes: &[P<Node>],
    wrap: &dyn Fn(&[String]) -> String,
) -> RuleFix {
    let inner_codes: Vec<String> = inner_nodes
        .iter()
        .map(|&inner_node| {
            let code = get_node_text(ctx, inner_node);
            if !utils::is_strong_precedence_node(inner_node) {
                format!("({code})")
            } else {
                code.to_string()
            }
        })
        .collect();
    let mut code = wrap(&inner_codes);
    if is_weak_precedence_parent(node) && !is_node_parenthesized(node) {
        code = format!("({code})");
    }
    ctx.fix_replace(node, code)
}

fn extract_regex_literal_flags(node: P<Node>) -> Option<&'static str> {
    if node.kind() != Kind::RegularExpressionLiteral {
        return None;
    }
    let text = node.text();
    let bytes = text.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'/' {
        return None;
    }
    let is_escaped = |idx: usize| -> bool {
        let mut backslashes = 0;
        let mut i = idx as isize - 1;
        while i >= 0 && bytes[i as usize] == b'\\' {
            backslashes += 1;
            i -= 1;
        }
        backslashes % 2 == 1
    };
    let closing_slash = (1..bytes.len()).rev().find(|&i| bytes[i] == b'/' && !is_escaped(i))?;
    Some(&text[closing_slash + 1..])
}

fn is_string_literal(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| n.kind() == Kind::StringLiteral)
}

fn is_reg_exp_constructor_call(node: P<Node>) -> bool {
    if !ast::is_call_expression(node) && !ast::is_new_expression(node) {
        return false;
    }
    let callee = node.expression().unwrap();
    ast::is_identifier(callee) && callee.text() == "RegExp"
}

fn definitely_does_not_contain_global_flag(node: P<Node>) -> bool {
    let node = ast::skip_parentheses(node);
    if !is_reg_exp_constructor_call(node) {
        return false;
    }
    let arguments = node.arguments();
    if arguments.len() < 2 {
        return true;
    }
    let flags = ast::skip_parentheses(arguments[1]);
    if utils::is_undefined_literal(Some(flags)) {
        return true;
    }
    if !is_string_literal(Some(flags)) {
        return false;
    }
    !flags.text().contains('g')
}

fn get_static_argument_value(
    ctx: &mut Ctx,
    node: P<Node>,
    visited: &mut FxHashSet<P<Symbol>>,
) -> StaticArgumentValue {
    use StaticArgumentValueKind as K;
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::StringLiteral => return StaticArgumentValue::of(K::String),
        Kind::RegularExpressionLiteral => {
            return match extract_regex_literal_flags(node) {
                Some(flags) => StaticArgumentValue { kind: K::RegExp, reg_exp_flags: flags },
                None => StaticArgumentValue::of(K::Unknown),
            };
        }
        Kind::NoSubstitutionTemplateLiteral
        | Kind::NumericLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword => return StaticArgumentValue::of(K::Other),
        Kind::Identifier => {
            let Some(symbol) = ctx.checker.get_symbol_at_location_exported(node) else {
                return StaticArgumentValue::of(K::Unknown);
            };
            if visited.contains(&symbol) {
                return StaticArgumentValue::of(K::Unknown);
            }
            visited.insert(symbol);
            let result = (|| {
                let Some(decl) = symbol.value_declaration() else {
                    return StaticArgumentValue::of(K::Unknown);
                };
                if !ast::is_variable_declaration(decl) {
                    return StaticArgumentValue::of(K::Unknown);
                }
                let Some(initializer) = decl.initializer() else {
                    return StaticArgumentValue::of(K::Unknown);
                };
                // Keep this conservative and match getStaticValue behavior for mutable bindings.
                let parent = decl.parent().unwrap();
                if !ast::is_variable_declaration_list(parent)
                    || !parent.flags().intersects(NodeFlags::Const)
                {
                    return StaticArgumentValue::of(K::Unknown);
                }
                get_static_argument_value(ctx, initializer, visited)
            })();
            visited.remove(&symbol);
            return result;
        }
        Kind::AsExpression | Kind::TypeAssertionExpression | Kind::NonNullExpression => {
            return get_static_argument_value(ctx, node.expression().unwrap(), visited);
        }
        _ => {}
    }
    if is_reg_exp_constructor_call(node) {
        let arguments = node.arguments();
        if !arguments.is_empty() && !is_string_literal(Some(arguments[0])) {
            return StaticArgumentValue::of(K::Unknown);
        }
        let mut flags = "";
        if arguments.len() > 1 {
            if !is_string_literal(Some(arguments[1])) {
                return StaticArgumentValue::of(K::Unknown);
            }
            flags = arguments[1].text();
        }
        if !arguments.is_empty() && !regexp2_syntax::compiles_ecmascript(arguments[0].text()) {
            return StaticArgumentValue::of(K::Unknown);
        }
        return StaticArgumentValue { kind: K::RegExp, reg_exp_flags: flags };
    }
    StaticArgumentValue::of(K::Unknown)
}

fn collect_argument_types(ctx: &mut Ctx, types: &[P<Type>]) -> i32 {
    let mut result = ARGUMENT_TYPE_OTHER;
    for &t in types {
        match utils::get_type_name(ctx.checker, t).as_str() {
            "RegExp" => result |= ARGUMENT_TYPE_REG_EXP,
            "string" => result |= ARGUMENT_TYPE_STRING,
            _ => {}
        }
    }
    result
}

fn build_string_to_reg_exp_literal(pattern: &str) -> String {
    format!("/{}/", pattern.replace('/', "\\/"))
}

fn report_with_fix(
    ctx: &mut Ctx,
    report_node: P<Node>,
    call_node: P<Node>,
    object_node: P<Node>,
    argument_node: P<Node>,
    build_expression: &dyn Fn(&str, &str) -> String,
) {
    ctx.report_node_with_fixes(
        report_node,
        build_reg_exp_exec_over_string_match_message(),
        |ctx| {
            vec![build_wrapping_fix(ctx, call_node, &[object_node, argument_node], &|code| {
                build_expression(&code[0], &code[1])
            })]
        },
    );
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let args = node.arguments();
        if args.len() != 1 {
            return;
        }
        let callee = node.expression().unwrap();
        if !ast::is_property_access_expression(callee) && !ast::is_element_access_expression(callee)
        {
            return;
        }
        let (property_name, ok) = ctx.checker.get_accessed_property_name(callee);
        if !ok || property_name != "match" {
            return;
        }
        let object_node = callee.expression().unwrap();
        let object_type = ctx.checker.get_type_at_location(object_node);
        if utils::get_type_name(ctx.checker, object_type) != "string" {
            return;
        }
        let argument_node = args[0];
        let static_argument =
            get_static_argument_value(ctx, argument_node, &mut FxHashSet::default());
        if static_argument.kind == StaticArgumentValueKind::RegExp
            && static_argument.reg_exp_flags.contains('g')
        {
            return;
        }
        let report_node = if ast::is_property_access_expression(callee) {
            callee.name().unwrap()
        } else {
            callee.as_element_access_expression().argument_expression
        };
        if is_string_literal(Some(argument_node)) {
            let pattern = argument_node.text();
            if !regexp2_syntax::compiles_ecmascript(pattern) {
                return;
            }
            let reg_exp_literal = build_string_to_reg_exp_literal(pattern);
            report_with_fix(
                ctx,
                report_node,
                node,
                object_node,
                argument_node,
                &|object_code, _| format!("{reg_exp_literal}.exec({object_code})"),
            );
            return;
        }
        let argument_type = ctx.checker.get_type_at_location(argument_node);
        let argument_types = collect_argument_types(ctx, &utils::union_type_parts(argument_type));
        if static_argument.kind == StaticArgumentValueKind::Unknown
            && argument_types & ARGUMENT_TYPE_REG_EXP != 0
            && !definitely_does_not_contain_global_flag(argument_node)
        {
            return;
        }
        match argument_types {
            ARGUMENT_TYPE_REG_EXP => report_with_fix(
                ctx,
                report_node,
                node,
                object_node,
                argument_node,
                &|object_code, argument_code| format!("{argument_code}.exec({object_code})"),
            ),
            ARGUMENT_TYPE_STRING => report_with_fix(
                ctx,
                report_node,
                node,
                object_node,
                argument_node,
                &|object_code, argument_code| {
                    format!("RegExp({argument_code}).exec({object_code})")
                },
            ),
            _ => {}
        }
    }
}
