// Port of internal/rules/no_unnecessary_condition/no_unnecessary_condition.go.
//
// This rule prevents unnecessary conditions in TypeScript code by detecting expressions that are always truthy,
// always falsy, or comparing values that have no overlap. Based on @typescript-eslint/no-unnecessary-condition.

use tsrs_ast::{self as ast, Kind, Node, SymbolFlags};
use tsrs_checker::{
    AccessFlags, AliasArg, LiteralValue, MappedTypeModifiers, ObjectFlags, Type, TypeFlags,
    TypePredicateKind,
};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, options_object,
};
use crate::utils;

fn build_always_truthy_message() -> RuleMessage {
    RuleMessage::new("alwaysTruthy", "Unnecessary conditional, value is always truthy.")
}
fn build_always_falsy_message() -> RuleMessage {
    RuleMessage::new("alwaysFalsy", "Unnecessary conditional, value is always falsy.")
}
fn build_never_message() -> RuleMessage {
    RuleMessage::new("never", "Unnecessary conditional, value is `never`.")
}
fn build_always_truthy_func_message() -> RuleMessage {
    RuleMessage::new(
        "alwaysTruthyFunc",
        "This callback should return a conditional, but return is always truthy.",
    )
}
fn build_always_falsy_func_message() -> RuleMessage {
    RuleMessage::new(
        "alwaysFalsyFunc",
        "This callback should return a conditional, but return is always falsy.",
    )
}
fn build_never_nullish_message() -> RuleMessage {
    RuleMessage::new(
        "neverNullish",
        "Unnecessary conditional, expected left-hand side of `??` operator to be possibly null or undefined.",
    )
}
fn build_never_optional_chain_message() -> RuleMessage {
    RuleMessage::new("neverOptionalChain", "Unnecessary optional chain on a non-nullish value.")
}
fn build_no_strict_null_check_message() -> RuleMessage {
    RuleMessage::new(
        "noStrictNullCheck",
        "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    )
}
fn build_literal_binary_expression_message() -> RuleMessage {
    RuleMessage::new(
        "comparisonBetweenLiteralTypes",
        "Unnecessary comparison between literal values.",
    )
}
fn build_always_nullish_message() -> RuleMessage {
    RuleMessage::new("alwaysNullish", "Unnecessary conditional, value is always nullish.")
}
fn build_type_guard_already_is_type_message() -> RuleMessage {
    RuleMessage::new(
        "typeGuardAlreadyIsType",
        "Type predicate is unnecessary as the parameter type already satisfies the predicate.",
    )
}

type Range = (i32, i32);

fn diagnostic(
    message: RuleMessage,
    primary: Range,
    labeled_ranges: Vec<LabeledRange>,
) -> RuleDiagnostic {
    RuleDiagnostic { pos: primary.0, end: primary.1, message, labeled_ranges }
}

fn labeled(label: String, range: Range) -> LabeledRange {
    LabeledRange { label, pos: range.0, end: range.1 }
}

fn build_typed_value_diagnostic(
    message: RuleMessage,
    primary_range: Range,
    type_range: Range,
    type_name: &str,
) -> RuleDiagnostic {
    let mut labels = Vec::new();
    if !type_name.is_empty() {
        labels.push(labeled(format!("Type: {type_name}"), type_range));
    }
    diagnostic(message, primary_range, labels)
}

fn build_typed_return_diagnostic(
    message: RuleMessage,
    primary_range: Range,
    type_range: Range,
    type_name: &str,
) -> RuleDiagnostic {
    let mut labels = Vec::new();
    if !type_name.is_empty() {
        labels.push(labeled(format!("Return type: {type_name}"), type_range));
    }
    diagnostic(message, primary_range, labels)
}

fn build_type_guard_diagnostic(
    primary_range: Range,
    type_range: Range,
    value_type: &str,
    predicate_type: &str,
) -> RuleDiagnostic {
    diagnostic(
        build_type_guard_already_is_type_message(),
        primary_range,
        vec![labeled(
            format!("Type {value_type} already satisfies predicate type {predicate_type}"),
            type_range,
        )],
    )
}

fn build_comparison_diagnostic(
    message: RuleMessage,
    primary_range: Range,
    left_type: &str,
    left_range: Range,
    right_type: &str,
    right_range: Range,
) -> RuleDiagnostic {
    diagnostic(
        message,
        primary_range,
        vec![
            labeled(format!("Type: {left_type}"), left_range),
            labeled(format!("Type: {right_type}"), right_range),
        ],
    )
}

fn build_no_overlap_diagnostic(
    primary_range: Range,
    left_type: &str,
    left_range: Range,
    right_type: &str,
    right_range: Range,
) -> RuleDiagnostic {
    build_comparison_diagnostic(
        RuleMessage::new(
            "noOverlapBooleanExpression",
            "This condition will always return the same value since the types have no overlap.",
        ),
        primary_range,
        left_type,
        left_range,
        right_type,
        right_range,
    )
}

fn truncate_type_name_for_diagnostic(type_name: &str) -> String {
    const MAX_TYPE_NAME_LENGTH: usize = 120;
    let runes: Vec<char> = type_name.chars().collect();
    if runes.len() > MAX_TYPE_NAME_LENGTH {
        let mut s: String = runes[..MAX_TYPE_NAME_LENGTH - 3].iter().collect();
        s.push_str("...");
        return s;
    }
    type_name.to_string()
}

fn type_name_for_diagnostic(ctx: &mut Ctx, t: P<Type>) -> String {
    truncate_type_name_for_diagnostic(&utils::type_to_string(ctx.checker, t))
}

fn is_self_explanatory_literal(node: P<Node>) -> bool {
    matches!(
        ast::skip_parentheses(node).kind(),
        Kind::TrueKeyword
            | Kind::FalseKeyword
            | Kind::NullKeyword
            | Kind::NumericLiteral
            | Kind::BigIntLiteral
            | Kind::StringLiteral
            | Kind::NoSubstitutionTemplateLiteral
    )
}

fn type_name_for_node_diagnostic(ctx: &mut Ctx, t: P<Type>, node: P<Node>) -> String {
    if is_self_explanatory_literal(node) {
        return String::new();
    }
    type_name_for_diagnostic(ctx, t)
}

/// Whether a type cannot be determined at compile time (any, unknown, type parameters, keyof T).
fn is_indeterminate_type(t: P<Type>) -> bool {
    t.flags().intersects(
        TypeFlags::Any | TypeFlags::Unknown | TypeFlags::TypeParameter | TypeFlags::Index,
    )
}

/// Whether a type is always null, undefined, or void.
fn is_always_nullish_type(t: P<Type>) -> bool {
    t.flags().intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void)
}

/// Go LiteralType.String().
fn literal_string(t: P<Type>) -> String {
    match t.as_literal_type().value() {
        Some(v) => tsrs_checker::value_to_string(v),
        None => panic!("unhandled value type in valueToString"),
    }
}

/// toStaticValue: only the static-ness signal is used.
fn is_static_value(t: Option<P<Type>>) -> bool {
    let Some(t) = t else { return false };
    let flags = t.flags();
    if flags.intersects(TypeFlags::BooleanLiteral) {
        return is_true_literal_type_value(Some(t)).1;
    }
    if flags.intersects(TypeFlags::Undefined) || flags.intersects(TypeFlags::Null) {
        return true;
    }
    if flags.intersects(TypeFlags::StringLiteral) && t.is_string_literal() {
        return true;
    }
    if flags.intersects(TypeFlags::NumberLiteral) && t.is_number_literal() {
        return true;
    }
    if flags.intersects(TypeFlags::BigIntLiteral) && t.is_big_int_literal() {
        return true;
    }
    false
}

/// (value, ok)
fn is_true_literal_type_value(t: Option<P<Type>>) -> (bool, bool) {
    let Some(t) = t else { return (false, false) };
    if !t.flags().intersects(TypeFlags::BooleanLiteral) {
        return (false, false);
    }
    if utils::is_intrinsic_type(t) {
        match t.as_intrinsic_type().intrinsic_name() {
            "true" => return (true, true),
            "false" => return (false, true),
            _ => {}
        }
    }
    match literal_string(t).as_str() {
        "true" => (true, true),
        "false" => (false, true),
        _ => (false, false),
    }
}

fn literal_string_value(t: P<Type>) -> Option<&'static str> {
    match t.as_literal_type().value() {
        Some(LiteralValue::String(s)) => Some(s),
        _ => None,
    }
}

/// checkTypeCondition: (is_truthy, is_falsy).
fn check_type_condition(t: P<Type>) -> (bool, bool) {
    let flags = t.flags();
    // Never type is always falsy (empty type, no values exist)
    if flags.intersects(TypeFlags::Never) {
        return (false, true);
    }
    // Indexed access types are indeterminate.
    if flags.intersects(TypeFlags::IndexedAccess) {
        return (false, false);
    }
    if utils::is_union_type(t) {
        let mut all_truthy = true;
        let mut all_falsy = true;
        for &part in t.types() {
            let (part_truthy, part_falsy) = check_type_condition(part);
            if !part_truthy {
                all_truthy = false;
            }
            if !part_falsy {
                all_falsy = false;
            }
        }
        return (all_truthy, all_falsy);
    }
    if utils::is_intersection_type(t) {
        let mut all_truthy = true;
        for &part in t.types() {
            let (part_truthy, part_falsy) = check_type_condition(part);
            if part_falsy {
                return (false, true);
            }
            if !part_truthy {
                all_truthy = false;
            }
        }
        return (all_truthy, false);
    }
    if flags.intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void) {
        return (false, true);
    }
    if flags.intersects(TypeFlags::Object | TypeFlags::NonPrimitive) {
        return (true, false);
    }
    if flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
        return (true, false);
    }
    if flags.intersects(TypeFlags::BooleanLiteral) {
        if utils::is_intrinsic_type(t) {
            match t.as_intrinsic_type().intrinsic_name() {
                "true" => return (true, false),
                "false" => return (false, true),
                _ => {}
            }
        } else {
            match literal_string(t).as_str() {
                "true" => return (true, false),
                "false" => return (false, true),
                _ => {}
            }
        }
    }
    if flags.intersects(TypeFlags::StringLiteral) && t.is_string_literal() {
        if t.as_literal_type().value() == Some(LiteralValue::String("")) {
            return (false, true);
        }
        return (true, false);
    }
    if flags.intersects(TypeFlags::NumberLiteral) && t.is_number_literal() {
        let value = literal_string(t);
        if value == "0" || value == "NaN" {
            return (false, true);
        }
        return (true, false);
    }
    if flags.intersects(TypeFlags::BigIntLiteral) && t.is_big_int_literal() {
        let value = literal_string(t);
        if value == "0" || value == "0n" {
            return (false, true);
        }
        return (true, false);
    }
    (false, false)
}

/// Whether a type can be null, undefined, or void (any union part).
fn is_nullish_type(t: P<Type>) -> bool {
    if utils::is_union_type(t) {
        return t.types().iter().any(|&p| is_nullish_type(p));
    }
    t.flags().intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void)
}

/// Removes null, undefined, and void from a union type; returns the first non-nullish part.
fn remove_nullish_from_type(t: P<Type>) -> Option<P<Type>> {
    if !utils::is_union_type(t) {
        if t.flags().intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void) {
            return None;
        }
        return Some(t);
    }
    t.types().iter().copied().find(|&p| !is_nullish_type(p))
}

/// Literals allowed in loop conditions with "only-allowed-literals": true, false, 0, 1.
fn is_allowed_constant_literal(node: P<Node>) -> bool {
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::TrueKeyword | Kind::FalseKeyword => true,
        Kind::NumericLiteral => {
            let text = node.as_numeric_literal().text();
            text == "0" || text == "1"
        }
        _ => false,
    }
}

fn normalize_allow_constant_loop_conditions(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => "never".to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Bool(true)) => "always".to_string(),
        _ => "never".to_string(),
    }
}

fn is_property_access(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| n.kind() == Kind::PropertyAccessExpression)
}
fn is_element_access(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| n.kind() == Kind::ElementAccessExpression)
}
fn is_call_expr(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| n.kind() == Kind::CallExpression)
}

fn is_indexed_access_flags(flags: TypeFlags) -> bool {
    flags.intersects(TypeFlags::IndexedAccess)
}

/// Whether an expression has optional chaining.
fn has_optional_chain(n: P<Node>) -> bool {
    let n = ast::skip_parentheses(n);
    match n.kind() {
        Kind::PropertyAccessExpression => {
            let pa = n.as_property_access_expression();
            pa.question_dot_token().is_some() || has_optional_chain(pa.expression)
        }
        Kind::ElementAccessExpression => {
            let ea = n.as_element_access_expression();
            ea.question_dot_token().is_some() || has_optional_chain(ea.expression)
        }
        Kind::CallExpression => {
            let ce = n.as_call_expression();
            ce.question_dot_token().is_some() || has_optional_chain(ce.expression)
        }
        _ => false,
    }
}

fn uses_optional_chaining(expr: P<Node>) -> bool {
    let expr = ast::skip_parentheses(expr);
    match expr.kind() {
        Kind::PropertyAccessExpression => {
            expr.as_property_access_expression().question_dot_token().is_some()
        }
        Kind::ElementAccessExpression => {
            expr.as_element_access_expression().question_dot_token().is_some()
        }
        Kind::CallExpression => expr.as_call_expression().question_dot_token().is_some(),
        _ => false,
    }
}

/// (expression, question_dot_token) of an access/call.
fn access_parts(node: P<Node>) -> Option<(P<Node>, Option<P<Node>>)> {
    match node.kind() {
        Kind::PropertyAccessExpression => {
            let pa = node.as_property_access_expression();
            Some((pa.expression, pa.question_dot_token()))
        }
        Kind::ElementAccessExpression => {
            let ea = node.as_element_access_expression();
            Some((ea.expression, ea.question_dot_token()))
        }
        Kind::CallExpression => {
            let ce = node.as_call_expression();
            Some((ce.expression, ce.question_dot_token()))
        }
        _ => None,
    }
}

pub struct NoUnnecessaryCondition {
    loop_condition_mode: String,
    check_type_predicates: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let check_type_predicates = match m.get("checkTypePredicates") {
        Some(serde_json::Value::Bool(b)) => *b,
        None | Some(serde_json::Value::Null) => false,
        Some(_) => {
            return Err(
                "no-unnecessary-condition: failed to unmarshal options: checkTypePredicates"
                    .to_string(),
            );
        }
    };
    Ok(Box::new(NoUnnecessaryCondition {
        loop_condition_mode: normalize_allow_constant_loop_conditions(
            m.get("allowConstantLoopConditions"),
        ),
        check_type_predicates,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::IfStatement),
    Listener::Enter(Kind::WhileStatement),
    Listener::Enter(Kind::DoStatement),
    Listener::Enter(Kind::ForStatement),
    Listener::Enter(Kind::ConditionalExpression),
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::PropertyAccessExpression),
    Listener::Enter(Kind::ElementAccessExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::CaseClause),
];

impl Rule for NoUnnecessaryCondition {
    fn name(&self) -> &'static str {
        "no-unnecessary-condition"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let compiler_options = ctx.program.options();
        let is_strict_null_checks = utils::is_strict_compiler_option_enabled(
            &compiler_options,
            compiler_options.strict_null_checks,
        );
        if !is_strict_null_checks {
            ctx.report_range(0, 0, build_no_strict_null_check_message());
        }
        let no_unchecked_indexed_access = compiler_options.no_unchecked_indexed_access.is_true();
        Box::new(Visitor { opts: self, is_strict_null_checks, no_unchecked_indexed_access })
    }
}

struct Visitor {
    opts: &'static NoUnnecessaryCondition,
    is_strict_null_checks: bool,
    no_unchecked_indexed_access: bool,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        let Listener::Enter(kind) = listener else {
            return;
        };
        match kind {
            Kind::IfStatement => {
                let e = node.as_if_statement().expression;
                self.check_node(ctx, Some(e), false, None);
            }
            Kind::WhileStatement => {
                let e = node.as_while_statement().expression;
                self.check_if_loop_is_necessary_conditional(ctx, Some(e));
            }
            Kind::DoStatement => {
                let e = node.as_do_statement().expression;
                self.check_if_loop_is_necessary_conditional(ctx, Some(e));
            }
            Kind::ForStatement => {
                let e = node.as_for_statement().condition;
                self.check_if_loop_is_necessary_conditional(ctx, e);
            }
            Kind::ConditionalExpression => {
                let e = node.as_conditional_expression().condition;
                self.check_node(ctx, Some(e), false, None);
            }
            Kind::BinaryExpression => {
                let b = node.as_binary_expression();
                let op = b.operator_token.kind();
                match op {
                    Kind::AmpersandAmpersandToken
                    | Kind::BarBarToken
                    | Kind::QuestionQuestionToken => {
                        self.check_logical_expression_for_unnecessary_conditionals(ctx, node)
                    }
                    Kind::AmpersandAmpersandEqualsToken
                    | Kind::BarBarEqualsToken
                    | Kind::QuestionQuestionEqualsToken => {
                        self.check_assignment_expression(ctx, node)
                    }
                    Kind::LessThanToken
                    | Kind::GreaterThanToken
                    | Kind::LessThanEqualsToken
                    | Kind::GreaterThanEqualsToken
                    | Kind::EqualsEqualsToken
                    | Kind::EqualsEqualsEqualsToken
                    | Kind::ExclamationEqualsToken
                    | Kind::ExclamationEqualsEqualsToken => {
                        let (left, right) = (b.left, b.right.get());
                        self.check_if_bool_expression_is_necessary_conditional(
                            ctx, node, left, right, op,
                        )
                    }
                    _ => {}
                }
            }
            Kind::PropertyAccessExpression | Kind::ElementAccessExpression => {
                self.check_optional_chain(ctx, node)
            }
            Kind::CallExpression => self.check_call_expression(ctx, node),
            Kind::CaseClause => {
                let Some(expression) = node.as_case_or_default_clause().expression else {
                    return;
                };
                let Some(switch) = node.parent().and_then(|p| p.parent()) else {
                    return;
                };
                if switch.kind() != Kind::SwitchStatement {
                    return;
                }
                let discriminant = switch.as_switch_statement().expression;
                self.check_if_bool_expression_is_necessary_conditional(
                    ctx,
                    expression,
                    discriminant,
                    expression,
                    Kind::EqualsEqualsEqualsToken,
                );
            }
            _ => {}
        }
    }
}

impl Visitor {
    fn resolve_indexed_access_type(&self, ctx: &mut Ctx, t: Option<P<Type>>) -> Option<P<Type>> {
        let t = t?;
        if !t.flags().intersects(TypeFlags::IndexedAccess) {
            return Some(t);
        }
        let indexed_access = t.as_indexed_access_type();
        let (Some(mut object_type), Some(mut index_type)) =
            (indexed_access.object_type.get(), indexed_access.index_type.get())
        else {
            return Some(t);
        };
        if let Some(c) = ctx.checker.get_base_constraint_of_type(object_type) {
            object_type = c;
        }
        if let Some(c) = ctx.checker.get_base_constraint_of_type(index_type) {
            index_type = c;
        }
        if index_type.flags().intersects(TypeFlags::StringLiteral) && index_type.is_string_literal()
        {
            if let Some(property_name) = literal_string_value(index_type) {
                if let Some(prop_type) =
                    ctx.checker.get_type_of_property_of_type(object_type, property_name)
                {
                    return Some(prop_type);
                }
            }
        }
        if index_type.flags().intersects(TypeFlags::NumberLiteral) && index_type.is_number_literal()
        {
            let name = literal_string(index_type);
            if let Some(prop_type) = ctx.checker.get_type_of_property_of_type(object_type, &name) {
                return Some(prop_type);
            }
        }
        let index_parts = utils::union_type_parts(index_type);
        if index_parts.iter().any(|p| p.flags().intersects(TypeFlags::StringLike)) {
            let string_type = ctx.checker.string_type;
            if let Some(s) = ctx.checker.get_index_type_of_type(object_type, string_type) {
                return Some(s);
            }
        }
        if index_parts.iter().any(|p| p.flags().intersects(TypeFlags::NumberLike)) {
            let number_type = ctx.checker.number_type;
            if let Some(n) = ctx.checker.get_index_type_of_type(object_type, number_type) {
                return Some(n);
            }
        }
        Some(t)
    }

    fn get_resolved_type(&self, ctx: &mut Ctx, node: P<Node>) -> Option<P<Type>> {
        let node_type = ctx.checker.get_type_at_location(node);
        let (constraint_type, is_type_parameter) =
            utils::get_constraint_info(ctx.checker, node_type);
        if is_type_parameter && constraint_type.is_none() {
            return None;
        }
        if is_type_parameter {
            return self.resolve_indexed_access_type(ctx, constraint_type);
        }
        self.resolve_indexed_access_type(ctx, Some(node_type))
    }

    fn node_is_array_type(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let Some(node_type) = self.get_resolved_type(ctx, node) else {
            return false;
        };
        utils::union_type_parts(node_type)
            .into_iter()
            .any(|part| ctx.checker.is_array_type_exported(part))
    }

    fn node_is_tuple_type(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let Some(node_type) = self.get_resolved_type(ctx, node) else {
            return false;
        };
        utils::union_type_parts(node_type).into_iter().any(tsrs_checker::is_tuple_type_exported)
    }

    fn is_array_index_expression(&self, ctx: &mut Ctx, node: Option<P<Node>>) -> bool {
        let Some(node) = node else { return false };
        let node = ast::skip_parentheses(node);
        if !ast::is_element_access_expression(node) {
            return false;
        }
        let elem_access = node.as_element_access_expression();
        self.node_is_array_type(ctx, elem_access.expression)
            || (self.node_is_tuple_type(ctx, elem_access.expression)
                && ast::skip_parentheses(elem_access.argument_expression).kind()
                    != Kind::NumericLiteral)
    }

    fn is_conditional_always_necessary(&self, t: P<Type>) -> bool {
        utils::union_type_parts(t).into_iter().any(|part| {
            part.flags().intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::TypeVariable)
        })
    }

    fn is_true_literal_type(&self, t: Option<P<Type>>) -> bool {
        let (value, ok) = is_true_literal_type_value(t);
        ok && value
    }

    /// Resolves an indexed access type via its base constraint. Returns (resolved, should_skip).
    fn constrain_indexed_access_type(
        &self,
        ctx: &mut Ctx,
        t: Option<P<Type>>,
    ) -> (Option<P<Type>>, bool) {
        match t {
            Some(tt) if is_indexed_access_flags(tt.flags()) => {
                let constraint_type = self.resolve_indexed_access_type(ctx, t);
                match constraint_type {
                    Some(c) if !is_indexed_access_flags(c.flags()) => (Some(c), false),
                    _ => (None, true),
                }
            }
            _ => (t, false),
        }
    }

    fn get_property_name_from_literal_type(&self, t: P<Type>) -> Option<String> {
        let flags = t.flags();
        if flags.intersects(TypeFlags::StringLiteral) && t.is_string_literal() {
            if let Some(v) = literal_string_value(t) {
                return Some(v.to_string());
            }
        }
        if flags.intersects(TypeFlags::NumberLiteral) && t.is_number_literal() {
            return Some(literal_string(t));
        }
        None
    }

    fn is_nullable_property_type(
        &self,
        ctx: &mut Ctx,
        obj_type: P<Type>,
        property_type: P<Type>,
    ) -> bool {
        if utils::is_union_type(property_type) {
            for &part in property_type.types() {
                if self.is_nullable_property_type(ctx, obj_type, part) {
                    return true;
                }
            }
            return false;
        }
        if let Some(property_name) = self.get_property_name_from_literal_type(property_type) {
            if let Some(prop_type) =
                ctx.checker.get_type_of_property_of_type(obj_type, &property_name)
            {
                return is_nullish_type(prop_type);
            }
        }
        let property_type_name = utils::get_type_name(ctx.checker, property_type);
        for &info in ctx.checker.get_index_infos_of_type(obj_type) {
            let key_type = info.key_type.get().unwrap();
            if utils::get_type_name(ctx.checker, key_type) == property_type_name {
                return true;
            }
        }
        false
    }

    fn is_nullable_element_access_expression(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let elem_access = node.as_element_access_expression();
        let object_type = ctx.checker.get_type_at_location(elem_access.expression);
        let property_type = ctx.checker.get_type_at_location(elem_access.argument_expression);
        self.is_nullable_property_type(ctx, object_type, property_type)
    }

    /// Effective type of a call expression.
    fn get_call_return_type(&self, ctx: &mut Ctx, call_expr: P<Node>) -> Option<P<Type>> {
        if call_expr.kind() != Kind::CallExpression {
            return None;
        }
        let call = call_expr.as_call_expression();
        if call.question_dot_token().is_none() && !has_optional_chain(call.expression) {
            if let Some(call_type) = self.get_resolved_type(ctx, call_expr) {
                return Some(call_type);
            }
        }
        // Go falls back to the callee's call signatures when getResolvedSignature returns nil; tsrs always
        // returns a signature (the unknown signature at worst), so that fallback is unreachable here.
        let resolved_signature =
            ctx.checker.get_resolved_signature(call_expr, None, tsrs_checker::CheckMode::Normal);
        Some(ctx.checker.get_return_type_of_signature_exported(resolved_signature))
    }

    /// Property type from a base type given a property access expression.
    fn get_property_type_from_base(
        &self,
        ctx: &mut Ctx,
        base_type: P<Type>,
        prop_access: P<Node>,
    ) -> Option<P<Type>> {
        if prop_access.kind() != Kind::PropertyAccessExpression {
            return None;
        }
        let non_nullish_base = remove_nullish_from_type(base_type)?;
        let name_node = prop_access.as_property_access_expression().name;
        let prop_name = ast::get_text_of_property_name(name_node);
        if prop_name.is_empty() {
            return Some(ctx.checker.get_type_at_location(prop_access));
        }
        // Try to get the property directly first
        if let Some(prop) = ctx.checker.get_property_of_type(non_nullish_base, &prop_name) {
            return Some(ctx.checker.get_type_of_symbol_exported(prop));
        }
        // For mapped types, try the apparent type which may have the property
        let apparent_type = ctx.checker.get_apparent_type(non_nullish_base);
        if apparent_type != non_nullish_base {
            if let Some(prop) = ctx.checker.get_property_of_type(apparent_type, &prop_name) {
                return Some(ctx.checker.get_type_of_symbol_exported(prop));
            }
        }
        let string_type = ctx.checker.string_type;
        let mut string_index_type =
            ctx.checker.get_index_type_of_type(non_nullish_base, string_type);
        if string_index_type.is_none() {
            string_index_type = ctx.checker.get_index_type_of_type(apparent_type, string_type);
        }
        if string_index_type.is_none()
            && non_nullish_base.object_flags().intersects(ObjectFlags::Mapped)
        {
            let properties = ctx.checker.get_properties_of_type(non_nullish_base);
            for &p in properties {
                if p.name() == prop_name {
                    if !p.flags.get().intersects(SymbolFlags::Optional) {
                        return Some(ctx.checker.get_type_of_symbol_exported(p));
                    }
                    return None;
                }
            }
            return None;
        }
        if let Some(string_index_type) = string_index_type {
            if ctx.program.options().no_unchecked_indexed_access.is_true() {
                return None;
            }
            return Some(string_index_type);
        }
        Some(ctx.checker.get_type_at_location(prop_access))
    }

    /// Type from property/element access on a call expression result (foo?.().bar).
    fn get_type_from_call_property(
        &self,
        ctx: &mut Ctx,
        call_expr: P<Node>,
        access_expr: P<Node>,
    ) -> Option<P<Type>> {
        let return_type = self
            .get_call_return_type(ctx, call_expr)
            .unwrap_or_else(|| ctx.checker.get_type_at_location(call_expr));
        let non_nullish_return = remove_nullish_from_type(return_type)?;
        if is_property_access(Some(access_expr)) {
            return self.get_property_type_from_base(ctx, non_nullish_return, access_expr);
        }
        Some(ctx.checker.get_type_at_location(access_expr))
    }

    fn is_call_expression_nullable_origin_from_callee(&self, ctx: &mut Ctx, call: P<Node>) -> bool {
        let callee = call.as_call_expression().expression;
        let Some(prev_type) = self.get_resolved_type(ctx, callee) else {
            return false;
        };
        if !utils::is_union_type(prev_type) {
            return false;
        }
        let mut is_own_nullable = false;
        'outer: for &part in prev_type.types() {
            for &sig in utils::get_call_signatures(ctx.checker, part) {
                let return_type = ctx.checker.get_return_type_of_signature_exported(sig);
                if is_nullish_type(return_type) {
                    is_own_nullable = true;
                    break 'outer;
                }
            }
        }
        !is_own_nullable && is_nullish_type(prev_type)
    }

    fn is_member_expression_nullable_origin_from_object(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
    ) -> bool {
        let node = ast::skip_parentheses(node);
        let object_expr;
        let mut property_type = None;
        let mut property_name = String::new();
        let mut is_computed = false;
        match node.kind() {
            Kind::PropertyAccessExpression => {
                let pa = node.as_property_access_expression();
                object_expr = pa.expression;
                property_name = ast::get_text_of_property_name(pa.name);
            }
            Kind::ElementAccessExpression => {
                let ea = node.as_element_access_expression();
                object_expr = ea.expression;
                is_computed = true;
                property_type = self.get_resolved_type(ctx, ea.argument_expression);
            }
            _ => return false,
        }
        let Some(prev_type) = self.get_resolved_type(ctx, object_expr) else {
            return false;
        };
        if !utils::is_union_type(prev_type) {
            return false;
        }
        let mut is_own_nullable = false;
        for &part in prev_type.types() {
            if is_nullish_type(part) {
                continue;
            }
            if is_computed {
                if let Some(pt) = property_type {
                    if self.is_nullable_property_type(ctx, part, pt) {
                        is_own_nullable = true;
                        break;
                    }
                }
                continue;
            }
            if property_name.is_empty() {
                continue;
            }
            if let Some(prop_type) = ctx.checker.get_type_of_property_of_type(part, &property_name)
            {
                if is_nullish_type(prop_type) {
                    is_own_nullable = true;
                    break;
                }
                continue;
            }
            for &info in ctx.checker.get_index_infos_of_type(part) {
                let key_type = info.key_type.get().unwrap();
                if utils::get_type_name(ctx.checker, key_type) != "string" {
                    continue;
                }
                if self.no_unchecked_indexed_access
                    || is_nullish_type(info.value_type.get().unwrap())
                {
                    is_own_nullable = true;
                    break;
                }
            }
            if is_own_nullable {
                break;
            }
        }
        !is_own_nullable && is_nullish_type(prev_type)
    }

    fn is_optionable_expression(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let node = ast::skip_parentheses(node);
        let Some(node_type) = self.get_resolved_type(ctx, node) else {
            return false;
        };
        if self.is_conditional_always_necessary(node_type) {
            return true;
        }
        let is_own_nullable = match node.kind() {
            Kind::PropertyAccessExpression | Kind::ElementAccessExpression => {
                !self.is_member_expression_nullable_origin_from_object(ctx, node)
            }
            Kind::CallExpression => !self.is_call_expression_nullable_origin_from_callee(ctx, node),
            _ => true,
        };
        is_own_nullable && is_nullish_type(node_type)
    }

    /// Whether element access is a safe tuple access with a literal index.
    fn is_safe_tuple_access(&self, ctx: &mut Ctx, elem_access: P<Node>) -> bool {
        let ea = elem_access.as_element_access_expression();
        let arg = ast::skip_parentheses(ea.argument_expression);
        if arg.kind() == Kind::NumericLiteral {
            if let Some(base_type) = self.get_resolved_type(ctx, ea.expression) {
                if tsrs_checker::is_tuple_type_exported(base_type) {
                    return true;
                }
            }
        }
        false
    }

    fn has_unsafe_element_access(&self, ctx: &mut Ctx, expr: P<Node>) -> bool {
        let expr = ast::skip_parentheses(expr);
        match expr.kind() {
            Kind::ElementAccessExpression => {
                let ea = expr.as_element_access_expression();
                if ea.question_dot_token().is_none()
                    && !ctx.program.options().no_unchecked_indexed_access.is_true()
                    && !self.is_safe_tuple_access(ctx, expr)
                {
                    return true;
                }
                self.has_unsafe_element_access(ctx, ea.expression)
            }
            Kind::PropertyAccessExpression => {
                self.has_unsafe_element_access(ctx, expr.as_property_access_expression().expression)
            }
            Kind::CallExpression => {
                self.has_unsafe_element_access(ctx, expr.as_call_expression().expression)
            }
            _ => false,
        }
    }

    /// Validates optional chaining (?.) to detect unnecessary usage.
    fn check_optional_chain(&self, ctx: &mut Ctx, node: P<Node>) {
        let Some((expression, question_dot_token)) = access_parts(node) else {
            return;
        };
        let Some(question_dot_token) = question_dot_token else {
            return;
        };
        let mut effective_type_name = String::new();
        let no_unchecked = ctx.program.options().no_unchecked_indexed_access.is_true();

        let expression_skipped = ast::skip_parentheses(expression);
        // Rule 1: Expression itself is unguarded element access (but not safe tuple access)
        if is_element_access(Some(expression_skipped))
            && !no_unchecked
            && expression_skipped.as_element_access_expression().question_dot_token().is_none()
            && !self.is_safe_tuple_access(ctx, expression_skipped)
        {
            return;
        }
        // Rule 2: Expression uses optional chaining AND contains unguarded element access
        if uses_optional_chaining(expression) && self.has_unsafe_element_access(ctx, expression) {
            return;
        }
        if self.is_optionable_expression(ctx, expression) {
            return;
        }

        // Chained access: for foo?.bar?.baz, when checking the second ?., expression is foo?.bar
        // and base_expression is foo.
        let mut base_expression = None;
        let mut is_chained_access = false;
        if let Some((base, qdt)) = access_parts(expression) {
            if qdt.is_some() {
                is_chained_access = true;
                base_expression = Some(base);
            }
        }

        let mut expr_type: Option<P<Type>>;
        if is_chained_access {
            let Some(base_type) = self.get_resolved_type(ctx, base_expression.unwrap()) else {
                return;
            };
            let Some(non_nullish_base) = remove_nullish_from_type(base_type) else {
                return;
            };
            if is_property_access(Some(expression)) {
                expr_type = self.get_property_type_from_base(ctx, non_nullish_base, expression);
                if expr_type.is_none() {
                    // For chained access with optional chains, GetTypeAtLocation includes the
                    // short-circuit undefined; remove it for non-optional mapped types.
                    expr_type = Some(ctx.checker.get_type_at_location(expression));
                    if !ctx.program.options().no_unchecked_indexed_access.is_true()
                        && non_nullish_base.object_flags().intersects(ObjectFlags::Mapped)
                    {
                        let modifiers = tsrs_checker::get_mapped_type_modifiers(non_nullish_base);
                        if !modifiers.intersects(MappedTypeModifiers::IncludeOptional) {
                            expr_type = expr_type.and_then(remove_nullish_from_type);
                        }
                    }
                }
            } else if is_element_access(Some(expression)) {
                let arg_expr = expression.as_element_access_expression().argument_expression;
                let key_type = ctx.checker.get_type_at_location(arg_expr);
                let key_flags = key_type.flags();
                let mut is_literal_key = false;
                let mut literal_keys: Vec<&'static str> = Vec::new();
                if key_flags.intersects(TypeFlags::StringLiteral) {
                    is_literal_key = true;
                    if key_type.is_string_literal() {
                        literal_keys.push(literal_string_value(key_type).unwrap());
                    }
                } else if utils::is_union_type(key_type) {
                    let mut all_literals = true;
                    for &part in key_type.types() {
                        if part.flags().intersects(TypeFlags::StringLiteral) {
                            if part.is_string_literal() {
                                literal_keys.push(literal_string_value(part).unwrap());
                            }
                        } else {
                            all_literals = false;
                            break;
                        }
                    }
                    is_literal_key = all_literals;
                }
                if is_literal_key && !literal_keys.is_empty() {
                    let mut all_non_nullish = true;
                    let mut representative_type = None;
                    let mut property_type_names: Vec<String> = Vec::new();
                    for key in literal_keys {
                        let Some(prop) = ctx.checker.get_property_of_type(non_nullish_base, key)
                        else {
                            all_non_nullish = false;
                            break;
                        };
                        let prop_type = ctx.checker.get_type_of_symbol_exported(prop);
                        if is_nullish_type(prop_type) {
                            all_non_nullish = false;
                            break;
                        }
                        if representative_type.is_none() {
                            representative_type = Some(prop_type);
                        }
                        let property_type_name = utils::type_to_string(ctx.checker, prop_type);
                        if !property_type_names.contains(&property_type_name) {
                            property_type_names.push(property_type_name);
                        }
                    }
                    if all_non_nullish {
                        expr_type = representative_type;
                        effective_type_name =
                            truncate_type_name_for_diagnostic(&property_type_names.join(" | "));
                    } else {
                        expr_type = Some(ctx.checker.get_type_at_location(expression));
                    }
                } else {
                    expr_type = Some(ctx.checker.get_type_at_location(expression));
                }
            } else if is_call_expr(Some(expression)) {
                expr_type = self.get_call_return_type(ctx, expression);
                if expr_type.is_none() {
                    expr_type = self.get_resolved_type(ctx, expression);
                    if expr_type.is_none() {
                        return;
                    }
                }
            } else {
                expr_type = Some(ctx.checker.get_type_at_location(expression));
            }
        } else if is_property_access(Some(expression)) || is_element_access(Some(expression)) {
            // Property/element access on call expression result, e.g. foo?.().bar?.baz
            let inner_expr = if is_property_access(Some(expression)) {
                expression.as_property_access_expression().expression
            } else {
                expression.as_element_access_expression().expression
            };
            if is_call_expr(Some(inner_expr)) {
                expr_type = self.get_type_from_call_property(ctx, inner_expr, expression);
                if expr_type.is_none() {
                    return;
                }
            } else {
                expr_type = self.get_resolved_type(ctx, expression);
            }
        } else if is_call_expr(Some(expression)) {
            expr_type = self.get_call_return_type(ctx, expression);
            if expr_type.is_none() {
                expr_type = self.get_resolved_type(ctx, expression);
                if expr_type.is_none() {
                    return;
                }
            }
        } else {
            expr_type = self.get_resolved_type(ctx, expression);
        }

        let Some(mut expr_type) = expr_type else {
            return;
        };

        if is_call_expr(Some(expression))
            && self.is_call_expression_nullable_origin_from_callee(ctx, expression)
        {
            match remove_nullish_from_type(expr_type) {
                Some(t) => expr_type = t,
                None => return,
            }
        }

        // If expression is a call to a union of functions and any function returns nullish, allow
        // the optional chain.
        if is_call_expr(Some(expression)) {
            let callee = expression.as_call_expression().expression;
            if let Some(func_type) = self.get_resolved_type(ctx, callee) {
                for part in utils::union_type_parts(func_type) {
                    if part
                        .flags()
                        .intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void)
                    {
                        continue;
                    }
                    let sigs = utils::get_call_signatures(ctx.checker, part);
                    if let Some(&sig) = sigs.first() {
                        let ret_type = ctx.checker.get_return_type_of_signature_exported(sig);
                        if is_nullish_type(ret_type) {
                            return;
                        }
                    }
                }
            }
        }

        // Allow optional chain on indeterminate types.
        if is_indeterminate_type(expr_type) {
            return;
        }
        if utils::is_union_type(expr_type)
            && expr_type.types().iter().any(|&t| is_indeterminate_type(t))
        {
            return;
        }

        if !is_nullish_type(expr_type) {
            let mut type_name = effective_type_name;
            if type_name.is_empty() {
                type_name = type_name_for_node_diagnostic(ctx, expr_type, expression);
            }
            let primary = ctx.trim(question_dot_token);
            let type_range = ctx.trim(expression);
            ctx.report_diagnostic(build_typed_value_diagnostic(
                build_never_optional_chain_message(),
                primary,
                type_range,
                &type_name,
            ));
        }
    }

    fn is_nullable_member_expression(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let node = ast::skip_parentheses(node);
        match node.kind() {
            Kind::PropertyAccessExpression => {
                let pa = node.as_property_access_expression();
                let name_node = pa.name;
                let base_type = ctx.checker.get_type_at_location(pa.expression);
                let prop_name = ast::get_text_of_property_name(name_node);
                if prop_name.is_empty() {
                    return false;
                }
                let mut prop_symbol = ctx.checker.get_symbol_at_location_exported(name_node);
                if prop_symbol.is_none() {
                    prop_symbol = ctx.checker.get_property_of_type(base_type, &prop_name);
                }
                if prop_symbol.is_none() {
                    prop_symbol = ctx
                        .checker
                        .get_properties_of_type(base_type)
                        .iter()
                        .copied()
                        .find(|p| p.name() == prop_name);
                }
                prop_symbol.is_some_and(|s| s.flags.get().intersects(SymbolFlags::Optional))
            }
            Kind::ElementAccessExpression => self.is_nullable_element_access_expression(ctx, node),
            _ => false,
        }
    }

    fn option_chain_contains_option_array_index(&self, ctx: &mut Ctx, node: P<Node>) -> bool {
        let node = ast::skip_parentheses(node);
        let Some((lhs_node, qdt)) = access_parts(node) else {
            return false;
        };
        if qdt.is_some() && self.is_array_index_expression(ctx, Some(lhs_node)) {
            return true;
        }
        match lhs_node.kind() {
            Kind::CallExpression
            | Kind::PropertyAccessExpression
            | Kind::ElementAccessExpression => {
                self.option_chain_contains_option_array_index(ctx, lhs_node)
            }
            _ => false,
        }
    }

    fn check_node(
        &self,
        ctx: &mut Ctx,
        expression: Option<P<Node>>,
        is_unary_not_argument: bool,
        report_node: Option<P<Node>>,
    ) {
        let Some(expression) = expression else { return };
        let report_node = report_node.unwrap_or(expression);
        let expression = ast::skip_parentheses(expression);

        if expression.kind() == Kind::PrefixUnaryExpression {
            let unary = expression.as_prefix_unary_expression();
            if unary.operator == Kind::ExclamationToken {
                self.check_node(
                    ctx,
                    Some(unary.operand),
                    !is_unary_not_argument,
                    Some(report_node),
                );
                return;
            }
        }

        if !self.no_unchecked_indexed_access
            && self.is_array_index_expression(ctx, Some(expression))
        {
            return;
        }

        if expression.kind() == Kind::BinaryExpression {
            let b = expression.as_binary_expression();
            let op = b.operator_token.kind();
            if op != Kind::QuestionQuestionToken
                && (op == Kind::AmpersandAmpersandToken || op == Kind::BarBarToken)
            {
                self.check_node(ctx, Some(b.right.get()), false, None);
                return;
            }
        }

        let Some(mut node_type) = self.get_resolved_type(ctx, expression) else {
            return;
        };
        if self.is_conditional_always_necessary(node_type) {
            return;
        }

        if is_element_access(Some(expression)) && is_indexed_access_flags(node_type.flags()) {
            let base = expression.as_element_access_expression().expression;
            if let Some(base_type) = self.get_resolved_type(ctx, base) {
                let string_type = ctx.checker.string_type;
                if let Some(s) = ctx.checker.get_index_type_of_type(base_type, string_type) {
                    node_type = s;
                }
            }
        }

        let flags = node_type.flags();
        if flags.intersects(TypeFlags::Never) {
            let name = type_name_for_node_diagnostic(ctx, node_type, expression);
            let primary = ctx.trim(report_node);
            let type_range = ctx.trim(expression);
            ctx.report_diagnostic(build_typed_value_diagnostic(
                build_never_message(),
                primary,
                type_range,
                &name,
            ));
            return;
        }

        let (is_truthy, is_falsy) = check_type_condition(node_type);
        if is_falsy || is_truthy {
            let message = if is_falsy != is_unary_not_argument {
                build_always_falsy_message()
            } else {
                build_always_truthy_message()
            };
            let name = type_name_for_node_diagnostic(ctx, node_type, expression);
            let primary = ctx.trim(report_node);
            let type_range = ctx.trim(expression);
            ctx.report_diagnostic(build_typed_value_diagnostic(
                message, primary, type_range, &name,
            ));
        }
    }

    fn get_nullish_coalescing_type(&self, ctx: &mut Ctx, node: P<Node>) -> (Option<P<Type>>, bool) {
        let node_type = self.get_resolved_type(ctx, node);
        let node = ast::skip_parentheses(node);
        if !self.no_unchecked_indexed_access || !ast::is_assignment_target(node) {
            return (node_type, false);
        }
        let read_type = match node.kind() {
            Kind::PropertyAccessExpression => {
                let access = node.as_property_access_expression();
                let name = access.name;
                if name.kind() == Kind::PrivateIdentifier {
                    return (node_type, false);
                }
                let base_type = self.get_resolved_type(ctx, access.expression);
                let text = ast::get_text_of_property_name(name);
                let Some(base_type) = base_type else {
                    return (node_type, false);
                };
                if ctx.checker.get_property_of_type(base_type, &text).is_some() {
                    return (node_type, false);
                }
                ctx.checker.get_type_of_property_or_index_signature_of_type(base_type, &text)
            }
            Kind::ElementAccessExpression => {
                let access = node.as_element_access_expression();
                let base_type = self.get_resolved_type(ctx, access.expression);
                let key_type = self.get_resolved_type(ctx, access.argument_expression);
                let (Some(base_type), Some(key_type)) = (base_type, key_type) else {
                    return (node_type, false);
                };
                ctx.checker.get_indexed_access_type_or_undefined(
                    base_type,
                    key_type,
                    AccessFlags::ExpressionPosition,
                    None,
                    AliasArg::None,
                )
            }
            _ => None,
        };
        let Some(read_type) = read_type else {
            return (node_type, false);
        };
        // Assignment-target types omit unchecked-index undefined and flow narrowing.
        (Some(ctx.checker.get_flow_type_of_reference(node, read_type)), true)
    }

    fn check_node_for_nullish(&self, ctx: &mut Ctx, node: P<Node>) {
        let (node_type, recovered_read_type) = self.get_nullish_coalescing_type(ctx, node);
        let Some(mut node_type) = node_type else {
            return;
        };
        if self.is_conditional_always_necessary(node_type) {
            return;
        }
        let (constrained_type, should_skip) =
            self.constrain_indexed_access_type(ctx, Some(node_type));
        if should_skip {
            return;
        } else if let Some(c) = constrained_type {
            node_type = c;
        }

        let flags = node_type.flags();
        if flags.intersects(TypeFlags::Never) {
            let name = type_name_for_node_diagnostic(ctx, node_type, node);
            let r = ctx.trim(node);
            ctx.report_diagnostic(build_typed_value_diagnostic(build_never_message(), r, r, &name));
            return;
        }
        if is_always_nullish_type(node_type) {
            let name = type_name_for_node_diagnostic(ctx, node_type, node);
            let r = ctx.trim(node);
            ctx.report_diagnostic(build_typed_value_diagnostic(
                build_always_nullish_message(),
                r,
                r,
                &name,
            ));
            return;
        }

        // A recovered read type already accounts for optionality and control-flow narrowing.
        if !is_nullish_type(node_type)
            && (recovered_read_type || !self.is_nullable_member_expression(ctx, node))
        {
            let node = ast::skip_parentheses(node);
            if self.no_unchecked_indexed_access
                || !(self.is_array_index_expression(ctx, Some(node))
                    || (has_optional_chain(node)
                        && self.option_chain_contains_option_array_index(ctx, node)))
            {
                let name = type_name_for_node_diagnostic(ctx, node_type, node);
                let r = ctx.trim(node);
                ctx.report_diagnostic(build_typed_value_diagnostic(
                    build_never_nullish_message(),
                    r,
                    r,
                    &name,
                ));
            }
        }
    }

    fn check_if_bool_expression_is_necessary_conditional(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        left: P<Node>,
        right: P<Node>,
        op_kind: Kind,
    ) {
        let left_type = self.get_resolved_type(ctx, left);
        let right_type = self.get_resolved_type(ctx, right);
        let (Some(left_type), Some(right_type)) = (left_type, right_type) else {
            return;
        };

        let primary_range = |ctx: &Ctx| {
            if ast::is_binary_expression(node) {
                ctx.trim(node.as_binary_expression().operator_token)
            } else {
                ctx.trim(node)
            }
        };

        if is_static_value(Some(left_type)) && is_static_value(Some(right_type)) {
            let primary = primary_range(ctx);
            let left_name = type_name_for_diagnostic(ctx, left_type);
            let left_range = ctx.trim(left);
            let right_name = type_name_for_diagnostic(ctx, right_type);
            let right_range = ctx.trim(right);
            ctx.report_diagnostic(build_comparison_diagnostic(
                build_literal_binary_expression_message(),
                primary,
                &left_name,
                left_range,
                &right_name,
                right_range,
            ));
            return;
        }

        if !self.is_strict_null_checks {
            return;
        }

        let left_flags = left_type.flags();
        let right_flags = right_type.flags();

        fn is_comparable(t: P<Type>, mut flags: TypeFlags, op_kind: Kind) -> bool {
            flags |= TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::TypeParameter
                | TypeFlags::TypeVariable;
            if op_kind == Kind::EqualsEqualsToken || op_kind == Kind::ExclamationEqualsToken {
                flags |= TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void;
            }
            let type_flags = t.flags();
            if type_flags.intersects(flags) {
                return true;
            }
            if type_flags.intersects(
                TypeFlags::Conditional
                    | TypeFlags::TypeVariable
                    | TypeFlags::Substitution
                    | TypeFlags::IncludesConstrainedTypeVariable,
            ) {
                return true;
            }
            if utils::is_union_type(t) {
                return t.types().iter().any(|&p| is_comparable(p, flags, op_kind));
            }
            if utils::is_intersection_type(t) {
                let parts = t.types();
                if parts.is_empty() {
                    return false;
                }
                return parts.iter().all(|&p| is_comparable(p, flags, op_kind));
            }
            false
        }

        let undefined_or_void = TypeFlags::Undefined | TypeFlags::Void;
        if (left_flags == TypeFlags::Undefined
            && !is_comparable(right_type, undefined_or_void, op_kind))
            || (right_flags == TypeFlags::Undefined
                && !is_comparable(left_type, undefined_or_void, op_kind))
            || (left_flags == TypeFlags::Null
                && !is_comparable(right_type, TypeFlags::Null, op_kind))
            || (right_flags == TypeFlags::Null
                && !is_comparable(left_type, TypeFlags::Null, op_kind))
        {
            let primary = primary_range(ctx);
            let left_name = type_name_for_diagnostic(ctx, left_type);
            let right_name = type_name_for_diagnostic(ctx, right_type);
            ctx.report_diagnostic(build_no_overlap_diagnostic(
                primary,
                &left_name,
                (left.pos(), left.end()),
                &right_name,
                (right.pos(), right.end()),
            ));
        }
    }

    fn check_logical_expression_for_unnecessary_conditionals(&self, ctx: &mut Ctx, node: P<Node>) {
        let b = node.as_binary_expression();
        if b.operator_token.kind() == Kind::QuestionQuestionToken {
            self.check_node_for_nullish(ctx, b.left);
            return;
        }
        self.check_node(ctx, Some(b.left), false, None);
    }

    fn check_if_loop_is_necessary_conditional(&self, ctx: &mut Ctx, test: Option<P<Node>>) {
        let Some(test) = test else { return };
        let mode = self.opts.loop_condition_mode.as_str();
        if mode == "only-allowed-literals" && is_allowed_constant_literal(test) {
            return;
        }
        if mode == "always" {
            if ast::skip_parentheses(test).kind() == Kind::TrueKeyword {
                return;
            }
            let test_type = self.get_resolved_type(ctx, test);
            if self.is_true_literal_type(test_type) {
                return;
            }
        }
        self.check_node(ctx, Some(test), false, None);
    }

    fn check_assignment_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let b = node.as_binary_expression();
        match b.operator_token.kind() {
            Kind::AmpersandAmpersandEqualsToken | Kind::BarBarEqualsToken => {
                self.check_node(ctx, Some(b.left), false, None)
            }
            Kind::QuestionQuestionEqualsToken => self.check_node_for_nullish(ctx, b.left),
            _ => {}
        }
    }

    fn check_type_predicate_call_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        if !self.opts.check_type_predicates {
            return;
        }
        let mut checkable_arguments = Vec::new();
        for &arg in node.as_call_expression().arguments.nodes() {
            if arg.kind() == Kind::SpreadElement {
                break;
            }
            checkable_arguments.push(arg);
        }
        if checkable_arguments.is_empty() {
            return;
        }
        let call_signature =
            ctx.checker.get_resolved_signature(node, None, tsrs_checker::CheckMode::Normal);
        let Some(type_predicate) =
            ctx.checker.get_type_predicate_of_signature_exported(call_signature)
        else {
            return;
        };
        let param_index = type_predicate.parameter_index.get();
        if param_index < 0 || param_index as usize >= checkable_arguments.len() {
            return;
        }
        let arg = checkable_arguments[param_index as usize];
        let predicate_type = type_predicate.t.get();
        match type_predicate.kind.get() {
            TypePredicateKind::AssertsIdentifier | TypePredicateKind::Identifier => {
                if type_predicate.kind.get() == TypePredicateKind::AssertsIdentifier
                    && predicate_type.is_none()
                {
                    self.check_node(ctx, Some(arg), false, None);
                    return;
                }
                let arg_type = utils::get_constrained_type_at_location(ctx.checker, arg);
                if Some(arg_type) == predicate_type {
                    let arg_range = ctx.trim(arg);
                    let value_name = type_name_for_diagnostic(ctx, arg_type);
                    let predicate_name = type_name_for_diagnostic(ctx, predicate_type.unwrap());
                    ctx.report_diagnostic(build_type_guard_diagnostic(
                        arg_range,
                        arg_range,
                        &value_name,
                        &predicate_name,
                    ));
                }
            }
            _ => {}
        }
    }

    fn check_call_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        self.check_optional_chain(ctx, node);
        if utils::is_array_method_call_with_predicate(ctx.checker, node) {
            if let Some(&arg) = node.as_call_expression().arguments.nodes().first() {
                check_predicate_function(ctx, arg, self.opts.check_type_predicates);
            }
        }
        self.check_type_predicate_call_expression(ctx, node);
    }
}

/// Analyzes predicate functions used in array methods like filter/find.
fn check_predicate_function(ctx: &mut Ctx, func_node: P<Node>, check_type_guards: bool) {
    let func_type = ctx.checker.get_type_at_location(func_node);
    let signatures = utils::get_call_signatures(ctx.checker, func_type);
    for &signature in signatures {
        let type_predicate = ctx.checker.get_type_predicate_of_signature_exported(signature);
        if let (true, Some(type_predicate)) = (check_type_guards, type_predicate) {
            let params = signature.parameters.get();
            if !params.is_empty() {
                let param_index = type_predicate.parameter_index.get();
                if param_index >= 0 && (param_index as usize) < params.len() {
                    let param = params[param_index as usize];
                    let param_type = ctx.checker.get_type_of_symbol_exported(param);
                    let predicate_kind = type_predicate.kind.get();
                    if predicate_kind == TypePredicateKind::Identifier
                        || predicate_kind == TypePredicateKind::This
                    {
                        if let Some(predicate_type) = type_predicate.t.get() {
                            if ctx.checker.is_type_assignable_to(param_type, predicate_type) {
                                let mut primary_range = ctx.trim(func_node);
                                let mut type_range = primary_range;
                                if let Some(declaration) = param.value_declaration() {
                                    if ast::get_source_file_of_node(declaration) == Some(ctx.file) {
                                        type_range = ctx.trim(declaration);
                                        primary_range = type_range;
                                    }
                                }
                                let value_name = type_name_for_diagnostic(ctx, param_type);
                                let predicate_name = type_name_for_diagnostic(ctx, predicate_type);
                                ctx.report_diagnostic(build_type_guard_diagnostic(
                                    primary_range,
                                    type_range,
                                    &value_name,
                                    &predicate_name,
                                ));
                                return;
                            }
                        }
                    }
                }
            }
        }

        let mut return_type = ctx.checker.get_return_type_of_signature_exported(signature);
        if return_type.flags().intersects(TypeFlags::TypeParameter) {
            if let Some(constraint) =
                ctx.checker.get_constraint_of_type_parameter_exported(return_type)
            {
                return_type = constraint;
            }
        }

        let (is_truthy, is_falsy) = check_type_condition(return_type);
        if is_truthy || is_falsy {
            let is_literal_function = func_node.kind() == Kind::ArrowFunction
                || func_node.kind() == Kind::FunctionExpression;
            let mut report_node = func_node;
            if is_literal_function {
                if let Some(body) = func_node.body() {
                    report_node = body;
                }
            }
            let report_range = ctx.trim(report_node);
            let message = if is_truthy {
                if is_literal_function {
                    build_always_truthy_message()
                } else {
                    build_always_truthy_func_message()
                }
            } else if is_literal_function {
                build_always_falsy_message()
            } else {
                build_always_falsy_func_message()
            };
            let name = type_name_for_node_diagnostic(ctx, return_type, report_node);
            ctx.report_diagnostic(build_typed_return_diagnostic(
                message,
                report_range,
                report_range,
                &name,
            ));
        }
    }
}
