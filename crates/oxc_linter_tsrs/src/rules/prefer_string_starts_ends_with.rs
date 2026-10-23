// Port of internal/rules/prefer_string_starts_ends_with/prefer_string_starts_ends_with.go.

use std::fmt::Write;

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor, options_object};
use crate::utils;

fn build_prefer_starts_with_message() -> RuleMessage {
    RuleMessage::new("preferStartsWith", "Use 'String#startsWith' method instead.")
}

fn build_prefer_ends_with_message() -> RuleMessage {
    RuleMessage::new("preferEndsWith", "Use 'String#endsWith' method instead.")
}

struct ParsedRegExp {
    is_starts_with: bool,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NullishKind {
    Unknown,
    Null,
    Undefined,
}

pub struct PreferStringStartsEndsWith {
    allow_single_element_equality_always: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let always = match m.get("allowSingleElementEquality") {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::String(s)) if s == "always" => true,
        Some(serde_json::Value::String(s)) if s == "never" => false,
        Some(v) => {
            return Err(format!(
                "prefer-string-starts-ends-with: invalid value (expected one of \"always\", \"never\"): {v}"
            ));
        }
    };
    Ok(Box::new(PreferStringStartsEndsWith { allow_single_element_equality_always: always }))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::BinaryExpression), Listener::Enter(Kind::CallExpression)];

impl Rule for PreferStringStartsEndsWith {
    fn name(&self) -> &'static str {
        "prefer-string-starts-ends-with"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static PreferStringStartsEndsWith,
}

/// Go strconv.Quote. unicode.IsPrint is approximated for non-ASCII runes: whitespace/separators,
/// controls, common format characters and private-use code points are escaped; unassigned code points
/// are kept as is.
fn go_strconv_quote(s: &str) -> String {
    fn is_print(r: char) -> bool {
        let c = r as u32;
        if c < 0x80 {
            return (0x20..0x7f).contains(&c);
        }
        if r.is_whitespace() || r.is_control() {
            return false;
        }
        !matches!(c,
            0xAD | 0x600..=0x605 | 0x61C | 0x6DD | 0x70F | 0x890..=0x891 | 0x8E2 | 0x180E
            | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F | 0xFEFF
            | 0xFFF9..=0xFFFB | 0x110BD | 0x110CD | 0x13430..=0x1343F | 0x1BCA0..=0x1BCA3
            | 0x1D173..=0x1D17A | 0xE0001 | 0xE0020..=0xE007F
            | 0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD)
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for r in s.chars() {
        match r {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ if is_print(r) => out.push(r),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            _ => {
                let c = r as u32;
                if c < 0x20 || c == 0x7f {
                    let _ = write!(out, "\\x{c:02x}");
                } else if c < 0x10000 {
                    let _ = write!(out, "\\u{c:04x}");
                } else {
                    let _ = write!(out, "\\U{c:08x}");
                }
            }
        }
    }
    out.push('"');
    out
}

/// Go unicode.IsDigit (Unicode decimal digits). Non-ASCII runes are approximated with char::is_numeric.
fn go_is_digit(ch: char) -> bool {
    if ch.is_ascii() { ch.is_ascii_digit() } else { ch.is_numeric() }
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn is_equality_comparison(node: P<Node>) -> bool {
    if !ast::is_binary_expression(node) {
        return false;
    }
    matches!(
        node.as_binary_expression().operator_token.kind(),
        Kind::EqualsEqualsToken
            | Kind::EqualsEqualsEqualsToken
            | Kind::ExclamationEqualsToken
            | Kind::ExclamationEqualsEqualsToken
    )
}

fn is_negative_equality_operator(kind: Kind) -> bool {
    kind == Kind::ExclamationEqualsToken || kind == Kind::ExclamationEqualsEqualsToken
}

fn is_loose_equality_operator(kind: Kind) -> bool {
    kind == Kind::EqualsEqualsToken || kind == Kind::ExclamationEqualsToken
}

fn member_object(node: P<Node>) -> Option<P<Node>> {
    if ast::is_property_access_expression(node) || ast::is_element_access_expression(node) {
        return node.expression();
    }
    None
}

fn member_optional(node: P<Node>) -> bool {
    if ast::is_property_access_expression(node) || ast::is_element_access_expression(node) {
        return node.question_dot_token().is_some();
    }
    false
}

fn member_optional_operator(node: P<Node>) -> &'static str {
    if member_optional(node) { "?." } else { "." }
}

fn get_property_name(node: P<Node>) -> Option<&'static str> {
    if ast::is_property_access_expression(node) {
        return node.name().map(|n| n.text());
    }
    if ast::is_element_access_expression(node) {
        let arg = ast::skip_parentheses(node.as_element_access_expression().argument_expression);
        if ast::is_string_literal(arg) || arg.kind() == Kind::NoSubstitutionTemplateLiteral {
            return Some(arg.text());
        }
    }
    None
}

fn call_member_callee(call_expr: P<Node>) -> Option<P<Node>> {
    let expr = ast::skip_parentheses(call_expr.expression().unwrap());
    if ast::is_property_access_expression(expr) || ast::is_element_access_expression(expr) {
        return Some(expr);
    }
    None
}

fn static_number(node: P<Node>) -> Option<f64> {
    let node = ast::skip_parentheses(node);
    if ast::is_numeric_literal(node) {
        // strconv.ParseFloat fails (instead of returning an infinity) on overflow.
        return node.text().replace('_', "").parse::<f64>().ok().filter(|n| n.is_finite());
    }
    if ast::is_prefix_unary_expression(node) {
        let prefix = node.as_prefix_unary_expression();
        if prefix.operator == Kind::MinusToken {
            if let Some(n) = static_number(prefix.operand) {
                return Some(-n);
            }
        }
    }
    None
}

fn split_regex_literal(text: &str) -> Option<(&str, &str)> {
    let bytes = text.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'/' {
        return None;
    }
    let mut closing_slash = None;
    for i in (1..bytes.len()).rev() {
        if bytes[i] != b'/' {
            continue;
        }
        let mut backslashes = 0;
        let mut j = i as isize - 1;
        while j >= 0 && bytes[j as usize] == b'\\' {
            backslashes += 1;
            j -= 1;
        }
        if backslashes % 2 == 0 {
            closing_slash = Some(i);
            break;
        }
    }
    let closing_slash = closing_slash?;
    Some((&text[1..closing_slash], &text[closing_slash + 1..]))
}

fn parse_reg_exp_text(pattern: &str) -> Option<String> {
    let is_starts_with = pattern.starts_with('^');
    let is_ends_with = pattern.ends_with('$');
    if is_starts_with == is_ends_with {
        return None;
    }
    let content = if is_starts_with { &pattern[1..] } else { &pattern[..pattern.len() - 1] };
    let mut builder = String::new();
    let mut escaped = false;
    for ch in content.chars() {
        if escaped {
            match ch {
                'n' => builder.push('\n'),
                'r' => builder.push('\r'),
                't' => builder.push('\t'),
                'v' => builder.push('\u{b}'),
                'f' => builder.push('\u{c}'),
                '0' => builder.push('\0'),
                'd' | 'D' | 'w' | 'W' | 's' | 'S' | 'b' | 'B' | 'c' | 'x' | 'u' | 'k' | 'p'
                | 'P' => return None,
                _ => {
                    if go_is_digit(ch) {
                        return None;
                    }
                    builder.push(ch);
                }
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        match ch {
            '.' | '*' | '+' | '?' | '|' | '^' | '$' | '[' | ']' | '(' | ')' | '{' | '}' => {
                return None;
            }
            _ => builder.push(ch),
        }
    }
    if escaped {
        return None;
    }
    Some(builder)
}

impl Visitor {
    fn normalized_node_text(&self, ctx: &Ctx, node: P<Node>) -> String {
        let (pos, end) = ctx.trim(ast::skip_parentheses(node));
        ctx.text()[pos as usize..end as usize].chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn is_same_tokens(&self, ctx: &Ctx, node1: P<Node>, node2: P<Node>) -> bool {
        self.normalized_node_text(ctx, node1) == self.normalized_node_text(ctx, node2)
    }

    fn get_property_range(&self, ctx: &Ctx, node: P<Node>) -> (i32, i32) {
        let Some(obj) = member_object(node) else {
            return ctx.trim(node);
        };
        let start = tsrs_scanner::get_range_of_token_at_position(ctx.file, obj.end());
        let (_, member_end) = ctx.trim(node);
        (start.pos(), member_end)
    }

    fn is_string_type(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let t = ctx.checker.get_type_at_location(ast::skip_parentheses(node));
        utils::get_type_name(ctx.checker, t) == "string"
    }

    fn static_string(&self, ctx: &mut Ctx, node: P<Node>) -> Option<&'static str> {
        let node = ast::skip_parentheses(node);
        if ast::is_string_literal(node) || node.kind() == Kind::NoSubstitutionTemplateLiteral {
            return Some(node.text());
        }
        if ast::is_identifier(node) {
            let symbol = ctx.checker.get_symbol_at_location_exported(node)?;
            let decl = symbol.value_declaration()?;
            if !ast::is_variable_declaration(decl) {
                return None;
            }
            let init = decl.initializer()?;
            return self.static_string(ctx, init);
        }
        None
    }

    fn is_character(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        self.static_string(ctx, node).is_some_and(|v| utf16_len(v) == 1)
    }

    fn get_nullish_kind(&self, ctx: &mut Ctx, node: P<Node>) -> NullishKind {
        let node = ast::skip_parentheses(node);
        if utils::is_null_literal(Some(node)) {
            return NullishKind::Null;
        }
        if utils::is_undefined_literal(Some(node)) {
            return NullishKind::Undefined;
        }
        if ast::is_identifier(node) {
            let Some(symbol) = ctx.checker.get_symbol_at_location_exported(node) else {
                return NullishKind::Unknown;
            };
            let Some(decl) = symbol.value_declaration() else {
                return NullishKind::Unknown;
            };
            if !ast::is_variable_declaration(decl) {
                return NullishKind::Unknown;
            }
            let Some(init) = decl.initializer() else {
                return NullishKind::Unknown;
            };
            return self.get_nullish_kind(ctx, init);
        }
        NullishKind::Unknown
    }

    fn is_length_expression(&self, ctx: &mut Ctx, node: P<Node>, expected_object: P<Node>) -> bool {
        let node = ast::skip_parentheses(node);
        let expected_object = ast::skip_parentheses(expected_object);
        if (ast::is_property_access_expression(node) || ast::is_element_access_expression(node))
            && get_property_name(node) == Some("length")
        {
            if let Some(obj) = member_object(node) {
                if self.is_same_tokens(ctx, obj, expected_object) {
                    return true;
                }
            }
        }
        let evaluated_length = static_number(node);
        let evaluated_string = self.static_string(ctx, expected_object);
        match (evaluated_length, evaluated_string) {
            (Some(l), Some(s)) => l == utf16_len(s) as f64,
            _ => false,
        }
    }

    fn is_length_ahead_of_end(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        substring: P<Node>,
        parent_string: P<Node>,
    ) -> bool {
        let node = ast::skip_parentheses(node);
        if ast::is_prefix_unary_expression(node) {
            let prefix = node.as_prefix_unary_expression();
            return prefix.operator == Kind::MinusToken
                && self.is_length_expression(ctx, prefix.operand, substring);
        }
        if ast::is_binary_expression(node) {
            let bin = node.as_binary_expression();
            return bin.operator_token.kind() == Kind::MinusToken
                && self.is_length_expression(ctx, bin.left, parent_string)
                && self.is_length_expression(ctx, bin.right.get(), substring);
        }
        false
    }

    fn is_last_index_expression(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        expected_object: P<Node>,
    ) -> bool {
        let node = ast::skip_parentheses(node);
        if !ast::is_binary_expression(node) {
            return false;
        }
        let bin = node.as_binary_expression();
        bin.operator_token.kind() == Kind::MinusToken
            && self.is_length_expression(ctx, bin.left, expected_object)
            && static_number(bin.right.get()) == Some(1.0)
    }

    fn parse_reg_exp(&self, ctx: &mut Ctx, node: P<Node>) -> Option<ParsedRegExp> {
        let node = ast::skip_parentheses(node);
        let pattern: &str;
        let mut flags: &str = "";
        if node.kind() == Kind::RegularExpressionLiteral {
            let (p, f) = split_regex_literal(node.text())?;
            pattern = p;
            flags = f;
        } else if ast::is_new_expression(node) {
            let expression = node.expression().unwrap();
            if !ast::is_identifier(expression) || expression.text() != "RegExp" {
                return None;
            }
            let args = node.arguments();
            if args.is_empty() || !ast::is_string_literal(args[0]) {
                return None;
            }
            pattern = args[0].text();
            if args.len() > 1 {
                if !ast::is_string_literal(args[1]) {
                    return None;
                }
                flags = args[1].text();
            }
        } else if ast::is_identifier(node) {
            let symbol = ctx.checker.get_symbol_at_location_exported(node)?;
            let decl = symbol.value_declaration()?;
            if !ast::is_variable_declaration(decl) {
                return None;
            }
            let init = decl.initializer()?;
            return self.parse_reg_exp(ctx, init);
        } else {
            return None;
        }
        if flags.contains(['i', 'm', 'g', 'y']) {
            return None;
        }
        let is_starts_with = pattern.starts_with('^');
        let is_ends_with = pattern.ends_with('$');
        if is_starts_with == is_ends_with {
            return None;
        }
        let text = parse_reg_exp_text(pattern)?;
        Some(ParsedRegExp { is_starts_with, text })
    }

    fn fix_with_right_operand(
        &self,
        ctx: &Ctx,
        bin_node: P<Node>,
        member_node: P<Node>,
        kind: &str,
    ) -> Vec<RuleFix> {
        let bin = bin_node.as_binary_expression();
        let mut fixes = Vec::new();
        if is_negative_equality_operator(bin.operator_token.kind()) {
            fixes.push(ctx.fix_insert_before(bin_node, "!"));
        }
        let (prop_pos, _) = self.get_property_range(ctx, member_node);
        let (right_pos, right_end) = ctx.trim(ast::skip_parentheses(bin.right.get()));
        let (_, bin_end) = ctx.trim(bin_node);
        fixes.push(ctx.fix_replace_range(
            prop_pos,
            right_pos,
            format!("{}{kind}sWith(", member_optional_operator(member_node)),
        ));
        fixes.push(ctx.fix_replace_range(right_end, bin_end, ")"));
        fixes
    }

    fn fix_with_argument(
        &self,
        ctx: &Ctx,
        bin_node: P<Node>,
        call_expr: P<Node>,
        member_node: P<Node>,
        kind: &str,
    ) -> Vec<RuleFix> {
        let bin = bin_node.as_binary_expression();
        let mut fixes = Vec::new();
        if is_negative_equality_operator(bin.operator_token.kind()) {
            fixes.push(ctx.fix_insert_before(bin_node, "!"));
        }
        let (prop_pos, prop_end) = self.get_property_range(ctx, member_node);
        let (_, call_end) = ctx.trim(call_expr);
        let (_, bin_end) = ctx.trim(bin_node);
        fixes.push(ctx.fix_replace_range(
            prop_pos,
            prop_end,
            format!("{}{kind}sWith", member_optional_operator(member_node)),
        ));
        fixes.push(ctx.fix_remove_range(call_end, bin_end));
        fixes
    }

    fn report_string_index_or_char_at(
        &self,
        ctx: &mut Ctx,
        bin_node: P<Node>,
        member_node: P<Node>,
        index_node: P<Node>,
    ) {
        let Some(obj) = member_object(member_node) else {
            return;
        };
        if !self.is_string_type(ctx, obj) {
            return;
        }
        let is_ends_with = self.is_last_index_expression(ctx, index_node, obj);
        let allow_single_element_equality = self.o.allow_single_element_equality_always;
        if allow_single_element_equality && is_ends_with {
            return;
        }
        let starts_at_zero = !is_ends_with && static_number(index_node) == Some(0.0);
        if allow_single_element_equality && starts_at_zero {
            return;
        }
        if !starts_at_zero && !is_ends_with {
            return;
        }
        let (message, kind) = if starts_at_zero {
            (build_prefer_starts_with_message(), "start")
        } else {
            (build_prefer_ends_with_message(), "end")
        };
        ctx.report_node_with_fixes(bin_node, message, |ctx| {
            if !self.is_character(ctx, bin_node.as_binary_expression().right.get()) {
                return Vec::new();
            }
            self.fix_with_right_operand(ctx, bin_node, member_node, kind)
        });
    }

    fn handle_binary_call_expression(&self, ctx: &mut Ctx, bin_node: P<Node>, call_expr: P<Node>) {
        let Some(member_node) = call_member_callee(call_expr) else {
            return;
        };
        let Some(prop_name) = get_property_name(member_node) else {
            return;
        };
        let bin = bin_node.as_binary_expression();
        let bin_right = bin.right.get();
        let operator = bin.operator_token.kind();
        let args = call_expr.arguments();
        match prop_name {
            "charAt" => {
                if args.len() != 1 {
                    return;
                }
                self.report_string_index_or_char_at(ctx, bin_node, member_node, args[0]);
            }
            "indexOf" => {
                if args.len() != 1 {
                    return;
                }
                if static_number(bin_right) != Some(0.0) {
                    return;
                }
                let Some(obj) = member_object(member_node) else {
                    return;
                };
                if !self.is_string_type(ctx, obj) {
                    return;
                }
                ctx.report_node_with_fixes(bin_node, build_prefer_starts_with_message(), |ctx| {
                    self.fix_with_argument(ctx, bin_node, call_expr, member_node, "start")
                });
            }
            "lastIndexOf" => {
                if args.len() != 1 || !ast::is_binary_expression(bin_right) {
                    return;
                }
                let right_bin = bin_right.as_binary_expression();
                if right_bin.operator_token.kind() != Kind::MinusToken {
                    return;
                }
                let Some(obj) = member_object(member_node) else {
                    return;
                };
                if !self.is_string_type(ctx, obj) {
                    return;
                }
                if !self.is_length_expression(ctx, right_bin.left, obj)
                    || !self.is_length_expression(ctx, right_bin.right.get(), args[0])
                {
                    return;
                }
                ctx.report_node_with_fixes(bin_node, build_prefer_ends_with_message(), |ctx| {
                    self.fix_with_argument(ctx, bin_node, call_expr, member_node, "end")
                });
            }
            "match" => {
                if args.len() != 1 {
                    return;
                }
                let obj = member_object(member_node);
                let operand_nullish_kind = self.get_nullish_kind(ctx, bin_right);
                let Some(obj) = obj else { return };
                if !self.is_string_type(ctx, obj) || operand_nullish_kind == NullishKind::Unknown {
                    return;
                }
                if !is_loose_equality_operator(operator)
                    && operand_nullish_kind != NullishKind::Null
                {
                    return;
                }
                let Some(parsed) = self.parse_reg_exp(ctx, args[0]) else {
                    return;
                };
                let (message, method) = if parsed.is_starts_with {
                    (build_prefer_starts_with_message(), "startsWith")
                } else {
                    (build_prefer_ends_with_message(), "endsWith")
                };
                ctx.report_node_with_fixes(bin_node, message, |ctx| {
                    let mut fixes = Vec::new();
                    if !is_negative_equality_operator(operator) {
                        fixes.push(ctx.fix_insert_before(bin_node, "!"));
                    }
                    let (prop_pos, prop_end) = self.get_property_range(ctx, member_node);
                    let (_, call_end) = ctx.trim(call_expr);
                    let (arg_pos, arg_end) = ctx.trim(args[0]);
                    let (_, bin_end) = ctx.trim(bin_node);
                    fixes.push(ctx.fix_replace_range(
                        prop_pos,
                        prop_end,
                        format!("{}{method}", member_optional_operator(member_node)),
                    ));
                    fixes.push(ctx.fix_replace_range(
                        arg_pos,
                        arg_end,
                        go_strconv_quote(&parsed.text),
                    ));
                    fixes.push(ctx.fix_remove_range(call_end, bin_end));
                    fixes
                });
            }
            "slice" | "substring" => {
                let Some(obj) = member_object(member_node) else {
                    return;
                };
                if !self.is_string_type(ctx, obj) {
                    return;
                }
                let mut is_starts_with = false;
                let mut is_ends_with = false;
                if args.len() == 1 {
                    if self.is_length_ahead_of_end(ctx, args[0], bin_right, obj) {
                        is_ends_with = true;
                    }
                } else if args.len() == 2 {
                    if static_number(args[0]) == Some(0.0)
                        && self.is_length_expression(ctx, args[1], bin_right)
                    {
                        is_starts_with = true;
                    } else if (self.is_length_expression(ctx, args[1], obj)
                        || static_number(args[1]) == Some(0.0))
                        && self.is_length_ahead_of_end(ctx, args[0], bin_right, obj)
                    {
                        is_ends_with = true;
                    }
                }
                if !is_starts_with && !is_ends_with {
                    return;
                }
                let (message, kind) = if is_starts_with {
                    (build_prefer_starts_with_message(), "start")
                } else {
                    (build_prefer_ends_with_message(), "end")
                };
                ctx.report_node_with_fixes(bin_node, message, |ctx| {
                    if is_loose_equality_operator(operator) {
                        let r = ast::skip_parentheses(bin_right);
                        if !ast::is_string_literal(r) {
                            return Vec::new();
                        }
                    }
                    if is_starts_with {
                        if args.len() < 2 || !self.is_length_expression(ctx, args[1], bin_right) {
                            return Vec::new();
                        }
                    } else {
                        let pos_node = ast::skip_parentheses(args[0]);
                        let mut valid_pos = false;
                        if ast::is_binary_expression(pos_node) {
                            let pos_bin = pos_node.as_binary_expression();
                            valid_pos = pos_bin.operator_token.kind() == Kind::MinusToken
                                && self.is_length_expression(ctx, pos_bin.left, obj)
                                && self.is_length_expression(ctx, pos_bin.right.get(), bin_right);
                        } else if prop_name == "slice" && ast::is_prefix_unary_expression(pos_node)
                        {
                            let prefix = pos_node.as_prefix_unary_expression();
                            valid_pos = prefix.operator == Kind::MinusToken
                                && self.is_length_expression(ctx, prefix.operand, bin_right);
                        }
                        if !valid_pos {
                            return Vec::new();
                        }
                    }
                    self.fix_with_right_operand(ctx, bin_node, member_node, kind)
                });
            }
            _ => {}
        }
    }

    fn handle_binary(&self, ctx: &mut Ctx, node: P<Node>) {
        if !is_equality_comparison(node) {
            return;
        }
        let left = ast::skip_parentheses(node.as_binary_expression().left);
        if ast::is_element_access_expression(left) {
            let index = left.as_element_access_expression().argument_expression;
            self.report_string_index_or_char_at(ctx, node, left, index);
            return;
        }
        if ast::is_call_expression(left) {
            self.handle_binary_call_expression(ctx, node, left);
        }
    }

    fn handle_regex_test_call(&self, ctx: &mut Ctx, node: P<Node>) {
        let args = node.arguments();
        if args.len() != 1 {
            return;
        }
        let Some(member_node) = call_member_callee(node) else {
            return;
        };
        if get_property_name(member_node) != Some("test") {
            return;
        }
        let Some(obj) = member_object(member_node) else {
            return;
        };
        let Some(parsed) = self.parse_reg_exp(ctx, obj) else {
            return;
        };
        let (message, method) = if parsed.is_starts_with {
            (build_prefer_starts_with_message(), "startsWith")
        } else {
            (build_prefer_ends_with_message(), "endsWith")
        };
        let arg = args[0];
        ctx.report_node_with_fixes(node, message, |ctx| {
            let arg = ast::skip_parentheses(arg);
            let needs_paren = !ast::is_literal_expression(arg)
                && arg.kind() != Kind::NoSubstitutionTemplateLiteral
                && arg.kind() != Kind::TemplateExpression
                && !ast::is_identifier(arg)
                && !ast::is_property_access_expression(arg)
                && !ast::is_element_access_expression(arg)
                && !ast::is_call_expression(arg);
            let (call_pos, _) = ctx.trim(node);
            let (arg_pos, _) = ctx.trim(arg);
            let mut fixes = vec![ctx.fix_remove_range(call_pos, arg_pos)];
            if needs_paren {
                fixes.push(ctx.fix_insert_before(arg, "("));
                fixes.push(ctx.fix_insert_after(arg, ")"));
            }
            fixes.push(ctx.fix_insert_after(
                arg,
                format!(
                    "{}{method}({}",
                    member_optional_operator(member_node),
                    go_strconv_quote(&parsed.text)
                ),
            ));
            fixes
        });
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::BinaryExpression) => self.handle_binary(ctx, node),
            Listener::Enter(Kind::CallExpression) => self.handle_regex_test_call(ctx, node),
            _ => {}
        }
    }
}
