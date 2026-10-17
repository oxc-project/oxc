// Port of internal/rules/dot_notation (dot_notation.go, options.go).

use std::fmt::Write;

use tsrs_ast::{self as ast, Kind, ModifierFlags, Node, Symbol};
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor, opt_bool, options_object,
};

const ES3_KEYWORDS: &[&str] = &[
    "abstract",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "function",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "volatile",
    "while",
    "with",
];

fn build_use_dot_message(key: &str) -> RuleMessage {
    RuleMessage::new("useDot", format!("[{key}] is better written in dot notation."))
}

fn build_use_brackets_message(key: &str) -> RuleMessage {
    RuleMessage::new("useBrackets", format!(".{key} is a syntax error."))
}

fn is_keyword(name: &str) -> bool {
    ES3_KEYWORDS.contains(&name)
}

/// eslint dot-notation's validIdentifier: `/^[a-zA-Z_$][\w$]*$/u`.
fn is_dot_notation_identifier(name: &str) -> bool {
    let bytes = name.as_bytes();
    let Some(&first) = bytes.first() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_' || first == b'$') {
        return false;
    }
    bytes[1..].iter().all(|&ch| ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$')
}

/// Go strconv.Quote.
fn go_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\u{b}' => out.push_str("\\v"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// (value, formatted) of a computed property key that could be written in dot notation.
fn get_computed_property_value(node: P<Node>) -> Option<(String, String)> {
    match node.kind() {
        Kind::StringLiteral => {
            let value = node.text().to_string();
            let formatted = go_quote(&value);
            Some((value, formatted))
        }
        Kind::TrueKeyword => Some(("true".into(), "true".into())),
        Kind::FalseKeyword => Some(("false".into(), "false".into())),
        Kind::NullKeyword => Some(("null".into(), "null".into())),
        Kind::NoSubstitutionTemplateLiteral => {
            let value = node.text().to_string();
            let formatted = format!("`{value}`");
            Some((value, formatted))
        }
        _ => None,
    }
}

fn get_static_property_name(node: P<Node>) -> Option<&'static str> {
    match node.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => Some(node.text()),
        _ => None,
    }
}

fn is_decimal_integer(node: P<Node>) -> bool {
    if !ast::is_numeric_literal(node) {
        return false;
    }
    let text = node.text();
    if text.is_empty() {
        return false;
    }
    if text.contains(|c| ".eExXoObBnN".contains(c)) {
        return false;
    }
    for (i, ch) in text.char_indices() {
        if ch.is_ascii_digit() {
            continue;
        }
        if ch == '_' && i > 0 && i < text.len() - 1 {
            continue;
        }
        return false;
    }
    true
}

fn needs_space_after_identifier(source_text: &str, member_expr_end: i32) -> bool {
    let next_pos = tsrs_scanner::skip_trivia(source_text, member_expr_end);
    if next_pos != member_expr_end || next_pos as usize >= source_text.len() {
        return false;
    }
    // utf8.DecodeRuneInString yields U+FFFD for invalid input; source text is valid UTF-8.
    let next_rune = source_text[next_pos as usize..].chars().next().unwrap();
    tsrs_scanner::is_identifier_part(next_rune as i32)
}

/// The property declaration whose visibility modifiers should be consulted for the given
/// element-access expression.
fn get_relevant_declaration(symbol: Option<P<Symbol>>, access_node: P<Node>) -> Option<P<Node>> {
    let symbol = symbol?;
    let declarations = symbol.declarations();
    if declarations.is_empty() {
        return None;
    }
    let mut getter = None;
    let mut setter = None;
    for &declaration in declarations {
        if getter.is_none() && ast::is_get_accessor_declaration(declaration) {
            getter = Some(declaration);
        } else if setter.is_none() && ast::is_set_accessor_declaration(declaration) {
            setter = Some(declaration);
        }
    }
    if getter.is_none() && setter.is_none() {
        return Some(declarations[0]);
    }
    // For a get/set accessor pair, consult the accessor actually involved in the access: a write goes
    // through the setter, while a read goes through the getter.
    if ast::is_write_access(access_node) {
        return setter.or(getter);
    }
    getter.or(setter)
}

pub struct DotNotation {
    allow_keywords: bool,
    allow_pattern: Option<regex::Regex>,
    allow_private_class_property_access: bool,
    allow_protected_class_property_access: bool,
    allow_index_signature_property_access_option: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let allow_pattern = match m.get("allowPattern") {
        Some(serde_json::Value::String(p)) if !p.is_empty() => match regex::Regex::new(p) {
            Ok(re) => Some(re),
            Err(err) => {
                return Err(format!("dot-notation: invalid allowPattern {p:?}: {err}"));
            }
        },
        _ => None,
    };
    Ok(Box::new(DotNotation {
        allow_keywords: opt_bool(&m, "allowKeywords", true),
        allow_pattern,
        allow_private_class_property_access: opt_bool(&m, "allowPrivateClassPropertyAccess", false),
        allow_protected_class_property_access: opt_bool(
            &m,
            "allowProtectedClassPropertyAccess",
            false,
        ),
        allow_index_signature_property_access_option: opt_bool(
            &m,
            "allowIndexSignaturePropertyAccess",
            false,
        ),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Exit(Kind::ElementAccessExpression),
    Listener::Enter(Kind::PropertyAccessExpression),
];

impl Rule for DotNotation {
    fn name(&self) -> &'static str {
        "dot-notation"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let no_property_access_from_index_signature =
            ctx.program.options().no_property_access_from_index_signature.is_true();
        Box::new(Visitor {
            opts: self,
            allow_index_signature_property_access: self
                .allow_index_signature_property_access_option
                || no_property_access_from_index_signature,
            no_property_access_from_index_signature,
        })
    }
}

struct Visitor {
    opts: &'static DotNotation,
    allow_index_signature_property_access: bool,
    no_property_access_from_index_signature: bool,
}

impl Visitor {
    fn report_computed_property(&self, ctx: &mut Ctx, node: P<Node>) {
        let element_access = node.as_element_access_expression();
        let property = element_access.argument_expression;
        let Some((value, formatted)) = get_computed_property_value(property) else {
            return;
        };
        if !is_dot_notation_identifier(&value) {
            return;
        }
        if !self.opts.allow_keywords && is_keyword(&value) {
            return;
        }
        if self.opts.allow_pattern.as_ref().is_some_and(|re| re.is_match(&value)) {
            return;
        }

        ctx.report_node_with_fixes(property, build_use_dot_message(&formatted), |ctx| {
            let expression = node.expression().unwrap();
            let object_end = ctx.trim(expression).1;
            let property_end = ctx.trim(property).1;
            let left_bracket_range =
                tsrs_scanner::get_range_of_token_at_position(ctx.file, object_end);
            let right_bracket_range =
                tsrs_scanner::get_range_of_token_at_position(ctx.file, property_end);

            let mut replace_start = left_bracket_range.pos();
            let mut fixes: Vec<RuleFix> = Vec::with_capacity(3);
            match element_access.question_dot_token() {
                None => {
                    let dot_text = if is_decimal_integer(expression) { " ." } else { "." };
                    fixes.push(ctx.fix_replace_range(
                        left_bracket_range.pos(),
                        left_bracket_range.pos(),
                        dot_text,
                    ));
                }
                Some(question_dot) => replace_start = question_dot.end(),
            }
            fixes.push(ctx.fix_replace_range(replace_start, right_bracket_range.end(), value));
            if needs_space_after_identifier(ctx.text(), node.end()) {
                fixes.push(ctx.fix_insert_after(node, " "));
            }
            fixes
        });
    }

    fn report_dot_keyword_access(&self, ctx: &mut Ctx, node: P<Node>) {
        if self.opts.allow_keywords {
            return;
        }
        let property_access = node.as_property_access_expression();
        let property = property_access.name();
        if !ast::is_identifier(property) {
            return;
        }
        let property_name = property.text();
        if !is_keyword(property_name) {
            return;
        }
        ctx.report_node_with_fixes(property, build_use_brackets_message(property_name), |ctx| {
            let expression = node.expression().unwrap();
            if property_access.question_dot_token().is_none()
                && ast::is_identifier(expression)
                && expression.text() == "let"
            {
                return Vec::new();
            }
            let object_end = ctx.trim(expression).1;
            let dot_range = tsrs_scanner::get_range_of_token_at_position(ctx.file, object_end);
            let mut fixes = Vec::with_capacity(2);
            if property_access.question_dot_token().is_none() {
                fixes.push(ctx.fix_remove_range(dot_range.pos(), dot_range.end()));
            }
            fixes.push(ctx.fix_replace(property, format!("[\"{property_name}\"]")));
            fixes
        });
    }

    fn should_skip_for_type_aware_allowances(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        if !self.opts.allow_private_class_property_access
            && !self.opts.allow_protected_class_property_access
            && !self.allow_index_signature_property_access
        {
            return false;
        }

        let property = node.as_element_access_expression().argument_expression;
        let expression = node.expression().unwrap();
        let c = &mut *ctx.checker;

        let mut property_symbol = c.get_symbol_at_location_exported(property);
        let static_property_name = get_static_property_name(property);
        if property_symbol.is_none() {
            if let Some(property_name) = static_property_name {
                let t = c.get_type_at_location(expression);
                let object_type = c.get_non_nullable_type(t);
                property_symbol = c
                    .get_properties_of_type(object_type)
                    .iter()
                    .copied()
                    .find(|candidate| candidate.name() == property_name);
            }
        }

        if let Some(declaration) = get_relevant_declaration(property_symbol, node) {
            if self.opts.allow_private_class_property_access
                && ast::has_modifier(declaration, ModifierFlags::Private)
            {
                return true;
            }
            if self.opts.allow_protected_class_property_access
                && ast::has_modifier(declaration, ModifierFlags::Protected)
            {
                return true;
            }
            if self.allow_index_signature_property_access
                && declaration.kind() == Kind::IndexSignature
            {
                return true;
            }
        }

        if property_symbol.is_none_or(|s| s.declarations().is_empty())
            && self.allow_index_signature_property_access
        {
            if self.no_property_access_from_index_signature {
                return true;
            }
            let t = c.get_type_at_location(expression);
            let object_type = c.get_non_nullable_type(t);
            let object_type = c.get_apparent_type(object_type);
            let key_type = c.get_type_at_location(property);
            let base_key_type = c.get_base_type_of_literal_type(key_type);
            if c.get_index_type_of_type(object_type, key_type).is_some()
                || c.get_index_type_of_type(object_type, base_key_type).is_some()
            {
                return true;
            }
        }

        false
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Exit(Kind::ElementAccessExpression) => {
                if self.should_skip_for_type_aware_allowances(ctx, node) {
                    return;
                }
                self.report_computed_property(ctx, node);
            }
            _ => self.report_dot_keyword_access(ctx, node),
        }
    }
}
