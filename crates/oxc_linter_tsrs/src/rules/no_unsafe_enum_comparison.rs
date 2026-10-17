// Port of internal/rules/no_unsafe_enum_comparison/{no_unsafe_enum_comparison,static_value,suggestion}.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol, SymbolFlags};
use tsrs_checker::{Checker, LiteralValue, Type, TypeFlags};
use tsrs_core::{LanguageVariant, P, jsnum};

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};
use crate::utils;

fn build_mismatched_case_message() -> RuleMessage {
    RuleMessage::with_help(
        "mismatchedCase",
        "The case statement does not have a shared enum type with the switch predicate.",
        "Compare against a member of the same enum as the switch value, or normalize both sides to the same primitive representation first.",
    )
}
fn build_mismatched_condition_message() -> RuleMessage {
    RuleMessage::with_help(
        "mismatchedCondition",
        "The two values in this comparison do not have a shared enum type.",
        "Compare enum values to members of the same enum, or convert both sides to the same primitive type before comparing them.",
    )
}

fn build_operand_range(
    source_file: P<SourceFile>,
    label: &str,
    node: P<Node>,
    type_text: &str,
) -> LabeledRange {
    let (pos, end) = utils::trim_node_text_range(source_file, node);
    LabeledRange { label: format!("{label}: {type_text}"), pos, end }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Go signature: buildComparisonDiagnostic takes label, node and type of both operands"
)]
fn build_comparison_diagnostic(
    source_file: P<SourceFile>,
    c: &mut Checker,
    node: P<Node>,
    message: RuleMessage,
    left_label: &str,
    left_node: P<Node>,
    left_type: P<Type>,
    right_label: &str,
    right_node: P<Node>,
    right_type: P<Type>,
) -> RuleDiagnostic {
    let (pos, end) = utils::trim_node_text_range(source_file, node);
    let left_text = utils::type_to_string(c, left_type);
    let right_text = utils::type_to_string(c, right_type);
    RuleDiagnostic {
        pos,
        end,
        message,
        labeled_ranges: vec![
            build_operand_range(source_file, left_label, left_node, &left_text),
            build_operand_range(source_file, right_label, right_node, &right_text),
        ],
    }
}

/// What type a type's enum value is (number or string), if either.
fn get_enum_value_type(t: P<Type>) -> TypeFlags {
    if utils::is_type_flag_set(t, TypeFlags::EnumLike) {
        if utils::is_type_flag_set(t, TypeFlags::NumberLiteral) {
            return TypeFlags::Number;
        }
        return TypeFlags::String;
    }
    TypeFlags::empty()
}

fn type_is_primitive_like(t: P<Type>, flags: TypeFlags) -> bool {
    utils::union_type_parts(t).into_iter().all(|union_part| {
        utils::intersection_type_parts(union_part)
            .into_iter()
            .any(|intersection_part| utils::is_type_flag_set(intersection_part, flags))
    })
}

/// Whether the right type is an unsafe comparison against any left type.
fn type_violates(left_type_parts: &[P<Type>], right_type: P<Type>) -> bool {
    let right_number_like =
        type_is_primitive_like(right_type, TypeFlags::Number | TypeFlags::NumberLike);
    let right_string_like =
        type_is_primitive_like(right_type, TypeFlags::String | TypeFlags::StringLike);
    if !right_number_like && !right_string_like {
        return false;
    }
    for &type_part in left_type_parts {
        let t = get_enum_value_type(type_part);
        if (t == TypeFlags::Number && right_number_like)
            || (t == TypeFlags::String && right_string_like)
        {
            return true;
        }
    }
    false
}

fn is_mismatched_comparison(c: &mut Checker, left_type: P<Type>, right_type: P<Type>) -> bool {
    if left_type == right_type {
        return false;
    }

    // Allow comparisons that don't have anything to do with enums: `1 === 2;`
    let left_enum_types = utils::get_enum_types(c, left_type);
    let right_enum_types: FxHashSet<P<Type>> =
        utils::get_enum_types(c, right_type).into_iter().collect();
    if left_enum_types.is_empty() && right_enum_types.is_empty() {
        return false;
    }

    // Allow comparisons that share an enum type: `Fruit.Apple === Fruit.Banana;`
    if left_enum_types.iter().any(|t| right_enum_types.contains(t)) {
        return false;
    }

    // We need to split the type into the union type parts in order to find valid enum comparisons
    // like: `declare const something: Fruit | Vegetable; something === Fruit.Apple;`
    let left_type_parts = utils::union_type_parts(left_type);
    let right_type_parts = utils::union_type_parts(right_type);

    // If a type exists in both sides, we consider this comparison safe:
    // `declare const fruit: Fruit.Apple | 0; fruit === 0;`
    for left_type_part in &left_type_parts {
        if right_type_parts.contains(left_type_part) {
            return false;
        }
    }

    type_violates(&left_type_parts, right_type) || type_violates(&right_type_parts, left_type)
}

// ---- static_value.go ----

#[derive(Clone, Copy, PartialEq, Eq)]
enum StaticValueKind {
    String,
    Number,
}

#[derive(Clone)]
struct StaticValue {
    kind: StaticValueKind,
    string_value: String,
    number_value: jsnum::Number,
}

impl StaticValue {
    fn string(s: String) -> StaticValue {
        StaticValue {
            kind: StaticValueKind::String,
            string_value: s,
            number_value: jsnum::Number(0.0),
        }
    }
    fn number(n: jsnum::Number) -> StaticValue {
        StaticValue { kind: StaticValueKind::Number, string_value: String::new(), number_value: n }
    }
}

fn get_static_value(node: P<Node>) -> Option<StaticValue> {
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => {
            Some(StaticValue::string(node.text().to_string()))
        }
        Kind::NumericLiteral => Some(StaticValue::number(jsnum::from_string(node.text()))),
        Kind::PrefixUnaryExpression => {
            let prefix = node.as_prefix_unary_expression();
            let value = get_static_value(prefix.operand)?;
            if value.kind != StaticValueKind::Number {
                return None;
            }
            match prefix.operator {
                Kind::MinusToken => Some(StaticValue::number(jsnum::Number(-value.number_value.0))),
                Kind::PlusToken => Some(value),
                _ => None,
            }
        }
        Kind::AsExpression
        | Kind::TypeAssertionExpression
        | Kind::NonNullExpression
        | Kind::SatisfiesExpression => get_static_value(node.expression().unwrap()),
        Kind::BinaryExpression => {
            let expr = node.as_binary_expression();
            if expr.operator_token.kind() != Kind::PlusToken {
                return None;
            }
            let left = get_static_value(expr.left)?;
            let right = get_static_value(expr.right.get())?;
            if left.kind == StaticValueKind::String || right.kind == StaticValueKind::String {
                return Some(StaticValue::string(
                    static_value_to_string(&left) + &static_value_to_string(&right),
                ));
            }
            Some(StaticValue::number(jsnum::Number(left.number_value.0 + right.number_value.0)))
        }
        _ => None,
    }
}

fn static_value_to_string(value: &StaticValue) -> String {
    if value.kind == StaticValueKind::String {
        return value.string_value.clone();
    }
    value.number_value.to_string()
}

// ---- suggestion.go ----

fn build_replace_value_with_enum_message() -> RuleMessage {
    RuleMessage::new("replaceValueWithEnum", "Replace with an enum value comparison.")
}

fn enum_value_matches_static_value(enum_value: Option<LiteralValue>, value: &StaticValue) -> bool {
    match value.kind {
        StaticValueKind::String => {
            matches!(enum_value, Some(LiteralValue::String(s)) if s == value.string_value)
        }
        StaticValueKind::Number => {
            matches!(enum_value, Some(LiteralValue::Number(n)) if n.0 == value.number_value.0)
        }
    }
}

fn enum_member_suffix_text(source_file: P<SourceFile>, member: P<Node>) -> String {
    let Some(member_name) = member.name() else {
        return String::new();
    };
    match member_name.kind() {
        Kind::Identifier => format!(".{}", member_name.text()),
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => {
            let text = member_name.text();
            if tsrs_scanner::is_identifier_text(text, LanguageVariant::Standard) {
                return format!(".{text}");
            }
            format!("[{}]", utils::quote_single_string_literal(text))
        }
        Kind::ComputedPropertyName => {
            let (pos, end) =
                utils::trim_node_text_range(source_file, member_name.expression().unwrap());
            format!("[{}]", &source_file.text()[pos as usize..end as usize])
        }
        _ => String::new(),
    }
}

fn is_qualified_identifier_text(value: &str) -> bool {
    value.split('.').all(|part| tsrs_scanner::is_identifier_text(part, LanguageVariant::Standard))
}

fn symbol_matches_enum(
    c: &mut Checker,
    symbol: Option<P<Symbol>>,
    enum_symbol: Option<P<Symbol>>,
) -> bool {
    let (Some(symbol), Some(enum_symbol)) = (symbol, enum_symbol) else {
        return false;
    };
    if symbol == enum_symbol || c.get_export_symbol_of_symbol(symbol) == enum_symbol {
        return true;
    }
    if utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
        let aliased = c.get_aliased_symbol(symbol);
        return aliased == enum_symbol || c.get_export_symbol_of_symbol(aliased) == enum_symbol;
    }
    false
}

fn enum_qualifier_from_type(
    c: &mut Checker,
    at_node: P<Node>,
    enum_type: P<Type>,
    enum_declaration: P<Node>,
) -> String {
    let qualifier = utils::type_to_string(c, enum_type);
    if !is_qualified_identifier_text(&qualifier) {
        return String::new();
    }
    if qualifier.contains('.') {
        return qualifier;
    }
    let enum_symbol =
        c.get_symbol_at_location_exported(enum_declaration.as_enum_declaration().name());
    for scope_symbol in c.get_symbols_in_scope_exported(at_node, SymbolFlags::Value) {
        if scope_symbol.name() == qualifier
            && symbol_matches_enum(c, Some(scope_symbol), enum_symbol)
        {
            return qualifier;
        }
    }
    String::new()
}

fn enum_comparison_suggestions(
    ctx: &mut Ctx,
    node: P<Node>,
    left_type: P<Type>,
    right_type: P<Type>,
) -> Vec<RuleSuggestion> {
    let expr = node.as_binary_expression();
    let (left, right) = (expr.left, expr.right.get());
    if let Some(right_value) = get_static_value(right) {
        let enum_key = get_enum_key_for_literal(
            ctx,
            node,
            left,
            &utils::get_enum_literals(left_type),
            &right_value,
        );
        if !enum_key.is_empty() {
            return vec![RuleSuggestion {
                message: build_replace_value_with_enum_message(),
                fixes: vec![ctx.fix_replace(right, enum_key)],
            }];
        }
    }
    if let Some(left_value) = get_static_value(left) {
        let enum_key = get_enum_key_for_literal(
            ctx,
            node,
            right,
            &utils::get_enum_literals(right_type),
            &left_value,
        );
        if !enum_key.is_empty() {
            return vec![RuleSuggestion {
                message: build_replace_value_with_enum_message(),
                fixes: vec![ctx.fix_replace(left, enum_key)],
            }];
        }
    }
    Vec::new()
}

fn symbol_matches_enum_declaration(
    c: &mut Checker,
    symbol: Option<P<Symbol>>,
    enum_declaration: P<Node>,
) -> bool {
    let Some(mut symbol) = symbol else {
        return false;
    };
    if utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
        symbol = c.get_aliased_symbol(symbol);
    }
    let symbol = c.get_export_symbol_of_symbol(symbol);
    symbol
        .value_declaration()
        .is_some_and(|vd| ast::is_enum_member(vd) && vd.parent() == Some(enum_declaration))
}

fn enum_qualifier_from_member_access(
    ctx: &mut Ctx,
    node: P<Node>,
    enum_declaration: P<Node>,
) -> String {
    let node = ast::skip_parentheses(node);
    if ast::is_property_access_expression(node) {
        let symbol = ctx.checker.get_symbol_at_location_exported(node.name().unwrap());
        if !symbol_matches_enum_declaration(ctx.checker, symbol, enum_declaration) {
            return String::new();
        }
        let (pos, end) = ctx.trim(node.expression().unwrap());
        return ctx.text()[pos as usize..end as usize].to_string();
    }
    if ast::is_element_access_expression(node) {
        let symbol = ctx.checker.get_symbol_at_location_exported(node);
        if !symbol_matches_enum_declaration(ctx.checker, symbol, enum_declaration) {
            let symbol = ctx.checker.get_symbol_at_location_exported(
                node.as_element_access_expression().argument_expression,
            );
            if !symbol_matches_enum_declaration(ctx.checker, symbol, enum_declaration) {
                return String::new();
            }
        }
        let (pos, end) = ctx.trim(node.expression().unwrap());
        return ctx.text()[pos as usize..end as usize].to_string();
    }
    String::new()
}

fn get_enum_key_for_literal(
    ctx: &mut Ctx,
    at_node: P<Node>,
    enum_side_node: P<Node>,
    enum_literals: &[P<Type>],
    value: &StaticValue,
) -> String {
    for &enum_literal in enum_literals {
        let Some(symbol) = enum_literal.symbol() else {
            continue;
        };
        let Some(value_declaration) = symbol.value_declaration() else {
            continue;
        };
        if !ast::is_enum_member(value_declaration) {
            continue;
        }
        let enum_value = ctx.checker.get_constant_value(value_declaration);
        if !enum_value_matches_static_value(enum_value, value) {
            continue;
        }
        let suffix = enum_member_suffix_text(ctx.file, value_declaration);
        if suffix.is_empty() {
            continue;
        }
        let enum_declaration = value_declaration.parent().unwrap();
        let mut qualifier =
            enum_qualifier_from_member_access(ctx, enum_side_node, enum_declaration);
        if qualifier.is_empty() {
            let enum_type = ctx.checker.get_type_at_location(enum_declaration);
            qualifier = enum_qualifier_from_type(ctx.checker, at_node, enum_type, enum_declaration);
        }
        if qualifier.is_empty() {
            continue;
        }
        return qualifier + &suffix;
    }
    String::new()
}

// ---- rule ----

pub struct NoUnsafeEnumComparison;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeEnumComparison))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::BinaryExpression), Listener::Enter(Kind::CaseClause)];

impl Rule for NoUnsafeEnumComparison {
    fn name(&self) -> &'static str {
        "no-unsafe-enum-comparison"
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
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::BinaryExpression => {
                let expr = node.as_binary_expression();
                let op_kind = expr.operator_token.kind();
                if !matches!(
                    op_kind,
                    Kind::LessThanToken
                        | Kind::LessThanEqualsToken
                        | Kind::GreaterThanToken
                        | Kind::GreaterThanEqualsToken
                        | Kind::EqualsEqualsToken
                        | Kind::EqualsEqualsEqualsToken
                        | Kind::ExclamationEqualsToken
                        | Kind::ExclamationEqualsEqualsToken
                ) {
                    return;
                }
                let (left, right) = (expr.left, expr.right.get());
                let left_type = ctx.checker.get_type_at_location(left);
                let right_type = ctx.checker.get_type_at_location(right);
                if is_mismatched_comparison(ctx.checker, left_type, right_type) {
                    let diagnostic = build_comparison_diagnostic(
                        ctx.file,
                        ctx.checker,
                        node,
                        build_mismatched_condition_message(),
                        "Left operand",
                        left,
                        left_type,
                        "Right operand",
                        right,
                        right_type,
                    );
                    ctx.report_diagnostic_with_suggestions(diagnostic, |ctx| {
                        enum_comparison_suggestions(ctx, node, left_type, right_type)
                    });
                }
            }
            Kind::CaseClause => {
                let switch_expression =
                    node.parent().unwrap().parent().unwrap().expression().unwrap();
                let case_expression = node.expression().unwrap();
                let left_type = ctx.checker.get_type_at_location(switch_expression);
                let right_type = ctx.checker.get_type_at_location(case_expression);
                if is_mismatched_comparison(ctx.checker, left_type, right_type) {
                    let diagnostic = build_comparison_diagnostic(
                        ctx.file,
                        ctx.checker,
                        node,
                        build_mismatched_case_message(),
                        "Switch value",
                        switch_expression,
                        left_type,
                        "Case value",
                        case_expression,
                        right_type,
                    );
                    ctx.report_diagnostic(diagnostic);
                }
            }
            _ => {}
        }
    }
}
