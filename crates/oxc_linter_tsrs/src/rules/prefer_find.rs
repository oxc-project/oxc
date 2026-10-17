// Port of internal/rules/prefer_find/prefer_find.go.

use std::fmt::Write;

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, NodeFlags, Symbol};
use tsrs_checker::Type;
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor};
use crate::utils;

fn build_prefer_find_message() -> RuleMessage {
    RuleMessage::new("preferFind", "Prefer .find(...) instead of .filter(...)[0].")
}

fn build_prefer_find_suggestion_message() -> RuleMessage {
    RuleMessage::new("preferFindSuggestion", "Use .find(...) instead of .filter(...)[0].")
}

/// Minimal arbitrary-precision integer standing in for Go's math/big.Int (only the operations
/// the rule needs: SetString, Neg, Sign, String and conversion to float64).
#[derive(Clone, Debug)]
struct BigInt {
    negative: bool,
    /// Little-endian base-1e9 limbs; empty means zero.
    limbs: Vec<u32>,
}

const LIMB_BASE: u64 = 1_000_000_000;

impl BigInt {
    /// big.Int.SetString(s, base) for base 0, 2, 8, 10 or 16.
    fn set_string(s: &str, base: u32) -> Option<BigInt> {
        let mut s = s;
        let mut negative = false;
        if let Some(rest) = s.strip_prefix('+') {
            s = rest;
        } else if let Some(rest) = s.strip_prefix('-') {
            s = rest;
            negative = true;
        }
        let mut base = base;
        let mut allow_underscores = false;
        if base == 0 {
            base = 10;
            allow_underscores = true;
            let b = s.as_bytes();
            if b.len() >= 2 && b[0] == b'0' {
                match b[1] {
                    b'x' | b'X' => {
                        base = 16;
                        s = &s[2..];
                    }
                    b'b' | b'B' => {
                        base = 2;
                        s = &s[2..];
                    }
                    b'o' | b'O' => {
                        base = 8;
                        s = &s[2..];
                    }
                    _ => {
                        base = 8;
                        s = &s[1..];
                    }
                }
            }
        }
        let mut limbs: Vec<u32> = Vec::new();
        let mut any = false;
        let mut prev_underscore = false;
        for (i, ch) in s.chars().enumerate() {
            if ch == '_' && allow_underscores {
                if prev_underscore || i == 0 {
                    return None;
                }
                prev_underscore = true;
                continue;
            }
            prev_underscore = false;
            let d = ch.to_digit(base)?;
            any = true;
            let mut carry = d as u64;
            for limb in limbs.iter_mut() {
                let v = *limb as u64 * base as u64 + carry;
                *limb = (v % LIMB_BASE) as u32;
                carry = v / LIMB_BASE;
            }
            while carry > 0 {
                limbs.push((carry % LIMB_BASE) as u32);
                carry /= LIMB_BASE;
            }
        }
        if prev_underscore {
            return None;
        }
        if !any {
            return None;
        }
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
        let negative = negative && !limbs.is_empty();
        Some(BigInt { negative, limbs })
    }

    fn neg(&self) -> BigInt {
        BigInt { negative: !self.negative && !self.limbs.is_empty(), limbs: self.limbs.clone() }
    }

    fn sign(&self) -> i32 {
        if self.limbs.is_empty() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }

    fn to_decimal_string(&self) -> String {
        if self.limbs.is_empty() {
            return "0".to_string();
        }
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        let mut iter = self.limbs.iter().rev();
        out.push_str(&iter.next().unwrap().to_string());
        for limb in iter {
            let _ = write!(out, "{limb:09}");
        }
        out
    }

    /// big.Float.SetInt(x).Float64(): round to nearest even, like parsing the exact decimal.
    fn to_f64(&self) -> f64 {
        self.to_decimal_string().parse::<f64>().unwrap()
    }
}

#[derive(Clone, Debug)]
enum StaticValue {
    String(&'static str),
    Number(f64),
    Boolean(bool),
    Null,
    BigInt(BigInt),
    Undefined,
    Symbol,
}

struct FilterExpressionData {
    filter_node: P<Node>,
    is_bracket_syntax_for_filter: bool,
}

/// Go strconv.FormatFloat(v, 'g', -1, 64) for finite, non-zero v.
fn go_format_float_g_shortest(v: f64) -> String {
    let sci = format!("{:e}", v.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let sign = if v < 0.0 { "-" } else { "" };
    if !(-4..6).contains(&exp) {
        let mut m = digits[..1].to_string();
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let esign = if exp < 0 { '-' } else { '+' };
        return format!("{sign}{m}e{esign}{:02}", exp.abs());
    }
    let dp = exp + 1;
    let n_digits = digits.len() as i32;
    let body = if dp <= 0 {
        format!("0.{}{}", "0".repeat((-dp) as usize), digits)
    } else if dp >= n_digits {
        format!("{}{}", digits, "0".repeat((dp - n_digits) as usize))
    } else {
        format!("{}.{}", &digits[..dp as usize], &digits[dp as usize..])
    };
    format!("{sign}{body}")
}

fn number_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v == f64::INFINITY {
        return "Infinity".to_string();
    }
    if v == f64::NEG_INFINITY {
        return "-Infinity".to_string();
    }
    if v == 0.0 {
        return "0".to_string();
    }
    go_format_float_g_shortest(v)
}

/// Go strconv.ParseFloat(s, 64); None where Go returns an error (including overflow to ±Inf).
fn go_parse_float(s: &str) -> Option<f64> {
    let v = s.parse::<f64>().ok()?;
    if v.is_infinite() {
        let lower = s.trim_start_matches(['+', '-']).to_ascii_lowercase();
        if lower != "inf" && lower != "infinity" {
            return None;
        }
    }
    Some(v)
}

fn to_number_from_string(s: &str) -> f64 {
    let s = s.trim();
    if s.is_empty() {
        return 0.0;
    }
    if s == "Infinity" || s == "+Infinity" {
        return f64::INFINITY;
    }
    if s == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    let mut base = 0;
    let mut trimmed = s;
    let mut sign = "";
    if trimmed.starts_with('+') || trimmed.starts_with('-') {
        sign = &trimmed[..1];
        trimmed = &trimmed[1..];
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        base = 16;
        trimmed = &trimmed[2..];
    } else if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
        base = 2;
        trimmed = &trimmed[2..];
    } else if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
        base = 8;
        trimmed = &trimmed[2..];
    }
    if base != 0 {
        if trimmed.is_empty() {
            return f64::NAN;
        }
        return match BigInt::set_string(&format!("{sign}{trimmed}"), base) {
            Some(int_val) => int_val.to_f64(),
            None => f64::NAN,
        };
    }
    go_parse_float(s).unwrap_or(f64::NAN)
}

fn to_number(value: &StaticValue) -> Option<f64> {
    match value {
        StaticValue::Number(n) => Some(*n),
        StaticValue::String(s) => Some(to_number_from_string(s)),
        StaticValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        StaticValue::Null => Some(0.0),
        StaticValue::Undefined => Some(f64::NAN),
        StaticValue::BigInt(_) | StaticValue::Symbol => None,
    }
}

fn static_value_to_property_name(value: &StaticValue) -> Option<String> {
    match value {
        StaticValue::String(s) => Some(s.to_string()),
        StaticValue::Number(n) => Some(number_to_string(*n)),
        StaticValue::Boolean(b) => Some(if *b { "true" } else { "false" }.to_string()),
        StaticValue::Null => Some("null".to_string()),
        StaticValue::Undefined => Some("undefined".to_string()),
        StaticValue::BigInt(b) => Some(b.to_decimal_string()),
        StaticValue::Symbol => None,
    }
}

fn is_treated_as_zero_by_array_at(value: &StaticValue) -> bool {
    if matches!(value, StaticValue::Symbol) {
        return false;
    }
    let Some(as_number) = to_number(value) else {
        return false;
    };
    if as_number.is_nan() {
        return true;
    }
    as_number.trunc() == 0.0
}

fn is_treated_as_zero_by_member_access(value: &StaticValue) -> bool {
    match value {
        StaticValue::String(s) => *s == "0",
        StaticValue::Number(n) => *n == 0.0,
        StaticValue::BigInt(b) => b.sign() == 0,
        _ => false,
    }
}

fn parse_big_int_literal(text: &str) -> Option<StaticValue> {
    let trimmed = text.trim();
    let trimmed = trimmed.strip_suffix('n').unwrap_or(trimmed);
    let trimmed = trimmed.replace('_', "");
    if trimmed.is_empty() {
        return None;
    }
    BigInt::set_string(&trimmed, 0).map(StaticValue::BigInt)
}

fn get_const_initializer(symbol: Option<P<Symbol>>) -> Option<P<Node>> {
    let decl = symbol?.value_declaration()?;
    if !ast::is_variable_declaration(decl) {
        return None;
    }
    let parent = decl.parent()?;
    if !ast::is_variable_declaration_list(parent) || !parent.flags().intersects(NodeFlags::Const) {
        return None;
    }
    decl.initializer()
}

fn get_static_value(
    ctx: &mut Ctx,
    node: P<Node>,
    visited: &mut FxHashSet<P<Symbol>>,
) -> Option<StaticValue> {
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => {
            return Some(StaticValue::String(node.text()));
        }
        Kind::NumericLiteral => {
            return go_parse_float(&node.text().replace('_', "")).map(StaticValue::Number);
        }
        Kind::TrueKeyword => return Some(StaticValue::Boolean(true)),
        Kind::FalseKeyword => return Some(StaticValue::Boolean(false)),
        Kind::NullKeyword => return Some(StaticValue::Null),
        Kind::BigIntLiteral => return parse_big_int_literal(node.text()),
        Kind::Identifier => {
            match node.text() {
                "undefined" => return Some(StaticValue::Undefined),
                "NaN" => return Some(StaticValue::Number(f64::NAN)),
                "Infinity" => return Some(StaticValue::Number(f64::INFINITY)),
                _ => {}
            }
            let symbol = ctx.checker.get_symbol_at_location_exported(node)?;
            if visited.contains(&symbol) {
                return None;
            }
            let initializer = get_const_initializer(Some(symbol))?;
            visited.insert(symbol);
            let result = get_static_value(ctx, initializer, visited);
            visited.remove(&symbol);
            return result;
        }
        Kind::PrefixUnaryExpression => {
            let prefix = node.as_prefix_unary_expression();
            let operand_value = get_static_value(ctx, prefix.operand, visited)?;
            match prefix.operator {
                Kind::MinusToken => {
                    if let StaticValue::BigInt(b) = &operand_value {
                        return Some(StaticValue::BigInt(b.neg()));
                    }
                    let as_number = to_number(&operand_value)?;
                    return Some(StaticValue::Number(-as_number));
                }
                Kind::PlusToken => {
                    let as_number = to_number(&operand_value)?;
                    return Some(StaticValue::Number(as_number));
                }
                _ => {}
            }
        }
        Kind::AsExpression | Kind::TypeAssertionExpression | Kind::NonNullExpression => {
            return get_static_value(ctx, node.expression().unwrap(), visited);
        }
        _ => {}
    }
    if ast::is_call_expression(node) {
        if node.question_dot_token().is_some() {
            return None;
        }
        let callee = ast::skip_parentheses(node.expression().unwrap());
        if ast::is_identifier(callee) && callee.text() == "Symbol" {
            return Some(StaticValue::Symbol);
        }
        if ast::is_property_access_expression(callee) {
            let object = ast::skip_parentheses(callee.expression().unwrap());
            if ast::is_identifier(object)
                && object.text() == "Symbol"
                && callee.name().unwrap().text() == "for"
            {
                return Some(StaticValue::Symbol);
            }
        }
    }
    None
}

fn is_static_member_access_of_value(ctx: &mut Ctx, node: P<Node>, value: &str) -> bool {
    let node = ast::skip_parentheses(node);
    if ast::is_property_access_expression(node) {
        return node.name().is_some_and(|n| n.text() == value);
    }
    if ast::is_element_access_expression(node) {
        let argument = node.as_element_access_expression().argument_expression;
        let Some(property_value) = get_static_value(ctx, argument, &mut FxHashSet::default())
        else {
            return false;
        };
        return static_value_to_property_name(&property_value).is_some_and(|n| n == value);
    }
    false
}

fn is_arrayish(ctx: &mut Ctx, t: P<Type>) -> bool {
    let mut is_at_least_one_arrayish_component = false;
    for union_part in utils::union_type_parts(t) {
        if utils::is_type_flag_set(union_part, tsrs_checker::TypeFlags::Null)
            || utils::is_type_undefined_type(union_part)
        {
            continue;
        }
        let is_array_or_intersection_thereof = utils::intersection_type_parts(union_part)
            .into_iter()
            .all(|p| ctx.checker.is_array_type(p) || tsrs_checker::is_tuple_type_exported(p));
        if !is_array_or_intersection_thereof {
            return false;
        }
        is_at_least_one_arrayish_component = true;
    }
    is_at_least_one_arrayish_component
}

fn parse_array_filter_expressions(ctx: &mut Ctx, expression: P<Node>) -> Vec<FilterExpressionData> {
    let node = ast::skip_parentheses(expression);
    if ast::is_comma_expression(node) {
        let last_expression = node.as_binary_expression().right.get();
        return parse_array_filter_expressions(ctx, last_expression);
    }
    if node.kind() == Kind::ConditionalExpression {
        let conditional = node.as_conditional_expression();
        let mut consequent_result = parse_array_filter_expressions(ctx, conditional.when_true);
        if consequent_result.is_empty() {
            return Vec::new();
        }
        let alternate_result = parse_array_filter_expressions(ctx, conditional.when_false);
        if alternate_result.is_empty() {
            return Vec::new();
        }
        consequent_result.extend(alternate_result);
        return consequent_result;
    }
    if ast::is_call_expression(node) {
        if node.question_dot_token().is_some() {
            return Vec::new();
        }
        let callee = ast::skip_parentheses(node.expression().unwrap());
        if !ast::is_property_access_expression(callee) && !ast::is_element_access_expression(callee)
        {
            return Vec::new();
        }
        if !is_static_member_access_of_value(ctx, callee, "filter") {
            return Vec::new();
        }
        let filtered_object_type =
            utils::get_constrained_type_at_location(ctx.checker, callee.expression().unwrap());
        if !is_arrayish(ctx, filtered_object_type) {
            return Vec::new();
        }
        if ast::is_property_access_expression(callee) {
            return vec![FilterExpressionData {
                filter_node: callee.name().unwrap(),
                is_bracket_syntax_for_filter: false,
            }];
        }
        return vec![FilterExpressionData {
            filter_node: callee.as_element_access_expression().argument_expression,
            is_bracket_syntax_for_filter: true,
        }];
    }
    Vec::new()
}

fn get_object_if_array_at_zero_expression(ctx: &mut Ctx, node: P<Node>) -> Option<P<Node>> {
    let args = node.arguments();
    if args.len() != 1 {
        return None;
    }
    if node.question_dot_token().is_some() {
        return None;
    }
    let callee = ast::skip_parentheses(node.expression().unwrap());
    if !ast::is_property_access_expression(callee) && !ast::is_element_access_expression(callee) {
        return None;
    }
    if !is_static_member_access_of_value(ctx, callee, "at") {
        return None;
    }
    let at_argument_value = get_static_value(ctx, args[0], &mut FxHashSet::default());
    if at_argument_value.is_some_and(|v| is_treated_as_zero_by_array_at(&v)) {
        return callee.expression();
    }
    None
}

fn is_member_access_of_zero(ctx: &mut Ctx, node: P<Node>) -> bool {
    if !ast::is_element_access_expression(node) {
        return false;
    }
    if node.question_dot_token().is_some() {
        return false;
    }
    let argument = node.as_element_access_expression().argument_expression;
    get_static_value(ctx, argument, &mut FxHashSet::default())
        .is_some_and(|v| is_treated_as_zero_by_member_access(&v))
}

fn find_array_element_access_start(
    ctx: &Ctx,
    array_node: P<Node>,
    whole_expression: P<Node>,
) -> Option<i32> {
    let (_, array_end) = ctx.trim(ast::skip_parentheses(array_node));
    let search_start = array_end as usize;
    let search_end = whole_expression.end() as usize;
    let text = ctx.text();
    let bytes = text.as_bytes();
    let mut i = search_start;
    while i < search_end {
        let r = text[i..].chars().next().unwrap();
        let size = r.len_utf8();
        if r.is_whitespace() {
            i += size;
            continue;
        }
        if bytes[i] == b'/' && i + 1 < search_end {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < search_end && bytes[i] != b'\n' && bytes[i] != b'\r' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < search_end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < search_end {
                    i += 2;
                }
                continue;
            }
        }
        if bytes[i] == b'.' || bytes[i] == b'[' {
            return Some(i as i32);
        }
        i += size;
    }
    None
}

fn report_prefer_find(
    ctx: &mut Ctx,
    node: P<Node>,
    array_node: P<Node>,
    filter_expressions: &[FilterExpressionData],
) {
    let Some(start) = find_array_element_access_start(ctx, array_node, node) else {
        return;
    };
    let remove_fix = ctx.fix_remove_range(start, node.end());
    ctx.report_node_with_suggestions(node, build_prefer_find_message(), |ctx| {
        let mut fixes: Vec<RuleFix> = Vec::with_capacity(filter_expressions.len() + 1);
        for filter_expression in filter_expressions {
            let replacement =
                if filter_expression.is_bracket_syntax_for_filter { "\"find\"" } else { "find" };
            fixes.push(ctx.fix_replace(filter_expression.filter_node, replacement));
        }
        fixes.push(remove_fix);
        vec![RuleSuggestion { message: build_prefer_find_suggestion_message(), fixes }]
    });
}

pub struct PreferFind;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(PreferFind))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::CallExpression), Listener::Enter(Kind::ElementAccessExpression)];

impl Rule for PreferFind {
    fn name(&self) -> &'static str {
        "prefer-find"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::CallExpression) => {
                let Some(object) = get_object_if_array_at_zero_expression(ctx, node) else {
                    return;
                };
                let filter_expressions = parse_array_filter_expressions(ctx, object);
                if filter_expressions.is_empty() {
                    return;
                }
                report_prefer_find(ctx, node, object, &filter_expressions);
            }
            Listener::Enter(Kind::ElementAccessExpression) => {
                if !is_member_access_of_zero(ctx, node) {
                    return;
                }
                let object = node.expression().unwrap();
                let filter_expressions = parse_array_filter_expressions(ctx, object);
                if filter_expressions.is_empty() {
                    return;
                }
                report_prefer_find(ctx, node, object, &filter_expressions);
            }
            _ => {}
        }
    }
}
