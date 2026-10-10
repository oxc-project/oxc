// Port of internal/rules/strict_boolean_expressions/strict_boolean_expressions.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, FunctionFlags, Kind, Node};
use tsrs_checker::{Checker, Type, TypeFlags, TypePredicate, TypePredicateKind};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, opt_bool, options_object};
use crate::utils;

fn build_condition_error_number_message() -> RuleMessage {
    RuleMessage::new(
        "conditionErrorNumber",
        "Unexpected number value in conditional. A number can be falsy (0, NaN) or truthy.",
    )
}
fn build_condition_error_string_message() -> RuleMessage {
    RuleMessage::new(
        "conditionErrorString",
        "Unexpected string value in conditional. A string can be falsy (empty string) or truthy.",
    )
}
fn build_condition_error_object_message() -> RuleMessage {
    RuleMessage::new(
        "conditionErrorObject",
        "Unexpected object value in conditional. An object is always truthy.",
    )
}
fn build_condition_error_nullish_message() -> RuleMessage {
    RuleMessage::new(
        "conditionErrorNullish",
        "Unexpected nullish value in conditional. The expression is always falsy.",
    )
}
fn build_condition_error_other_message() -> RuleMessage {
    RuleMessage::new(
        "conditionErrorOther",
        "Unexpected value in conditional. A union of different types has inconsistent truthiness.",
    )
}
fn build_condition_error_nullable_boolean_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorNullableBoolean",
        "Unexpected nullable boolean value in conditional.",
        "Handle the nullish case explicitly.",
    )
}
fn build_condition_error_nullable_object_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorNullableObject",
        "Unexpected nullable object value in conditional.",
        "Handle the nullish case explicitly.",
    )
}
fn build_condition_error_nullable_string_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorNullableString",
        "Unexpected nullable string value in conditional.",
        "Handle the nullish and empty string cases explicitly.",
    )
}
fn build_condition_error_nullable_number_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorNullableNumber",
        "Unexpected nullable number value in conditional.",
        "Handle the nullish and zero cases explicitly.",
    )
}
fn build_condition_error_nullable_enum_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorNullableEnum",
        "Unexpected nullable enum value in conditional.",
        "Handle the nullish and falsy enum cases explicitly.",
    )
}
fn build_condition_error_any_message() -> RuleMessage {
    RuleMessage::with_help(
        "conditionErrorAny",
        "Unexpected any value in conditional.",
        "Use a more specific type to ensure type safety.",
    )
}
fn build_no_strict_null_check_message() -> RuleMessage {
    RuleMessage::new(
        "noStrictNullCheck",
        "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    )
}
fn build_predicate_cannot_be_async_message() -> RuleMessage {
    RuleMessage::new(
        "predicateCannotBeAsync",
        "Predicate function should not be 'async'; expected a boolean return type.",
    )
}

pub struct StrictBooleanExpressions {
    allow_any: bool,
    allow_nullable_boolean: bool,
    allow_nullable_enum: bool,
    allow_nullable_number: bool,
    allow_nullable_object: bool,
    allow_nullable_string: bool,
    allow_number: bool,
    allow_string: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(StrictBooleanExpressions {
        allow_any: opt_bool(&m, "allowAny", false),
        allow_nullable_boolean: opt_bool(&m, "allowNullableBoolean", false),
        allow_nullable_enum: opt_bool(&m, "allowNullableEnum", false),
        allow_nullable_number: opt_bool(&m, "allowNullableNumber", false),
        allow_nullable_object: opt_bool(&m, "allowNullableObject", true),
        allow_nullable_string: opt_bool(&m, "allowNullableString", false),
        allow_number: opt_bool(&m, "allowNumber", true),
        allow_string: opt_bool(&m, "allowString", true),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::IfStatement),
    Listener::Enter(Kind::WhileStatement),
    Listener::Enter(Kind::DoStatement),
    Listener::Enter(Kind::ForStatement),
    Listener::Enter(Kind::ConditionalExpression),
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::PrefixUnaryExpression),
];

impl Rule for StrictBooleanExpressions {
    fn name(&self) -> &'static str {
        "strict-boolean-expressions"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let options = ctx.program.options();
        if !utils::is_strict_compiler_option_enabled(&options, options.strict_null_checks) {
            ctx.report_range(0, 0, build_no_strict_null_check_message());
        }
        Box::new(Visitor { opts: self, traversed_nodes: FxHashSet::default() })
    }
}

struct Visitor {
    opts: &'static StrictBooleanExpressions,
    traversed_nodes: FxHashSet<P<Node>>,
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
                self.traverse_node(ctx, e, true);
            }
            Kind::WhileStatement => {
                let e = node.as_while_statement().expression;
                self.traverse_node(ctx, e, true);
            }
            Kind::DoStatement => {
                let e = node.as_do_statement().expression;
                self.traverse_node(ctx, e, true);
            }
            Kind::ForStatement => {
                if let Some(cond) = node.as_for_statement().condition {
                    self.traverse_node(ctx, cond, true);
                }
            }
            Kind::ConditionalExpression => {
                let c = node.as_conditional_expression().condition;
                self.traverse_node(ctx, c, true);
            }
            Kind::BinaryExpression
                if ast::is_logical_expression(node)
                    && node.as_binary_expression().operator_token.kind()
                        != Kind::QuestionQuestionToken =>
            {
                self.traverse_logical_expression(ctx, node, false);
            }
            Kind::CallExpression => self.visit_call_expression(ctx, node),
            Kind::PrefixUnaryExpression => {
                let unary = node.as_prefix_unary_expression();
                if unary.operator == Kind::ExclamationToken {
                    self.traverse_node(ctx, unary.operand, true);
                }
            }
            _ => {}
        }
    }
}

impl Visitor {
    fn visit_call_expression(&mut self, ctx: &mut Ctx, node: P<Node>) {
        if let Some(asserted_argument) = find_truthiness_asserted_argument(ctx.checker, node) {
            self.traverse_node(ctx, asserted_argument, true);
        }
        if utils::is_array_method_call_with_predicate(ctx.checker, node) {
            let args = node.as_call_expression().arguments;
            let Some(&arg) = args.nodes().first() else {
                return;
            };
            if ast::get_function_flags(Some(arg)).intersects(FunctionFlags::Async) {
                ctx.report_node(arg, build_predicate_cannot_be_async_message());
                return;
            }
            let func_type = ctx.checker.get_type_at_location(arg);
            let signatures = utils::get_call_signatures(ctx.checker, func_type);
            let mut types = Vec::new();
            for &signature in signatures {
                let mut return_type = ctx.checker.get_return_type_of_signature_exported(signature);
                if return_type.flags().intersects(TypeFlags::TypeParameter) {
                    if let Some(constraint) =
                        ctx.checker.get_constraint_of_type_parameter_exported(return_type)
                    {
                        return_type = constraint;
                    }
                }
                types.extend(utils::union_type_parts(return_type));
            }
            check_condition(ctx, node, &types, self.opts);
        }
    }

    fn traverse_logical_expression(
        &mut self,
        ctx: &mut Ctx,
        bin_expr: P<Node>,
        is_condition: bool,
    ) {
        let b = bin_expr.as_binary_expression();
        let (left, right) = (b.left, b.right.get());
        self.traverse_node(ctx, left, true);
        self.traverse_node(ctx, right, is_condition);
    }

    fn traverse_node(&mut self, ctx: &mut Ctx, node: P<Node>, is_condition: bool) {
        if !self.traversed_nodes.insert(node) {
            return;
        }
        if node.kind() == Kind::ParenthesizedExpression {
            let e = node.as_parenthesized_expression().expression.get();
            self.traverse_node(ctx, e, is_condition);
            return;
        }
        if node.kind() == Kind::BinaryExpression
            && ast::is_logical_expression(node)
            && node.as_binary_expression().operator_token.kind() != Kind::QuestionQuestionToken
        {
            self.traverse_logical_expression(ctx, node, is_condition);
            return;
        }
        if !is_condition {
            return;
        }
        check_node(ctx, node, self.opts);
    }
}

fn find_truthiness_asserted_argument(checker: &mut Checker, call: P<Node>) -> Option<P<Node>> {
    let call_expr = call.as_call_expression();
    let mut checkable_arguments = Vec::new();
    for &argument in call_expr.arguments.nodes() {
        if argument.kind() == Kind::SpreadElement {
            break;
        }
        checkable_arguments.push(argument);
    }
    if checkable_arguments.is_empty() {
        return None;
    }
    let callee_type = checker.get_type_at_location(call_expr.expression);
    let union_types = utils::union_type_parts(callee_type);
    let is_union_type = union_types.len() > 1;
    let signature = checker.get_resolved_signature_exported(call);
    let Some(first_type_predicate_result) =
        checker.get_type_predicate_of_signature_exported(signature)
    else {
        if !is_union_type {
            return None;
        }
        return find_truthiness_asserted_argument_in_union_signatures(
            checker,
            &union_types,
            &checkable_arguments,
        );
    };
    find_truthiness_asserted_argument_in_predicate(
        Some(first_type_predicate_result),
        &checkable_arguments,
    )
}

fn find_truthiness_asserted_argument_in_union_signatures(
    checker: &mut Checker,
    union_types: &[P<Type>],
    checkable_arguments: &[P<Node>],
) -> Option<P<Node>> {
    for &t in union_types {
        for &sig in utils::get_call_signatures(checker, t) {
            let type_predicate = checker.get_type_predicate_of_signature_exported(sig);
            if let Some(argument) =
                find_truthiness_asserted_argument_in_predicate(type_predicate, checkable_arguments)
            {
                return Some(argument);
            }
        }
    }
    None
}

fn find_truthiness_asserted_argument_in_predicate(
    type_predicate: Option<P<TypePredicate>>,
    checkable_arguments: &[P<Node>],
) -> Option<P<Node>> {
    let tp = type_predicate?;
    if tp.kind.get() != TypePredicateKind::AssertsIdentifier || tp.t.get().is_some() {
        return None;
    }
    let parameter_index = tp.parameter_index.get();
    if parameter_index < 0 || parameter_index as usize >= checkable_arguments.len() {
        return None;
    }
    Some(checkable_arguments[parameter_index as usize])
}

fn check_node(ctx: &mut Ctx, node: P<Node>, opts: &StrictBooleanExpressions) {
    let node_type = utils::get_constrained_type_at_location(ctx.checker, node);
    check_condition(ctx, node, &utils::union_type_parts(node_type), opts);
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum TypeVariant {
    #[default]
    Nullish,
    Boolean,
    String,
    Number,
    BigInt,
    Object,
    Any,
    Unknown,
    Never,
    Mixed,
    Generic,
}

#[derive(Default)]
struct TypeInfo {
    variant: TypeVariant,
    is_nullable: bool,
    is_truthy: bool,
    is_enum: bool,
}

fn analyze_type_parts(types: &[P<Type>]) -> TypeInfo {
    let mut info = TypeInfo::default();
    let mut has_non_nullish_variant = false;
    let mut met_not_truthy = false;
    for &part in types {
        let part_info = analyze_type_part(part);
        if part_info.variant == TypeVariant::Nullish {
            info.is_nullable = true;
        } else if !has_non_nullish_variant {
            info.variant = part_info.variant;
            has_non_nullish_variant = true;
        } else if info.variant != part_info.variant {
            info.variant = TypeVariant::Mixed;
        }
        if part_info.is_enum {
            info.is_enum = true;
        }
        if matches!(
            part_info.variant,
            TypeVariant::Boolean | TypeVariant::Number | TypeVariant::String | TypeVariant::BigInt
        ) {
            if met_not_truthy {
                continue;
            }
            info.is_truthy = part_info.is_truthy;
            met_not_truthy = !part_info.is_truthy;
        }
    }
    if types.is_empty() {
        info.variant = TypeVariant::Mixed;
    }
    info
}

/// Go LiteralType.String().
fn literal_string(t: P<Type>) -> String {
    match t.as_literal_type().value() {
        Some(v) => tsrs_checker::value_to_string(v),
        None => panic!("unhandled value type in valueToString"),
    }
}

fn analyze_type_part(t: P<Type>) -> TypeInfo {
    let mut info = TypeInfo::default();
    let flags = t.flags();
    if utils::is_intersection_type(t) {
        let is_boolean =
            t.types().iter().any(|&t2| analyze_type_part(t2).variant == TypeVariant::Boolean);
        info.variant = if is_boolean { TypeVariant::Boolean } else { TypeVariant::Object };
        return info;
    }
    if flags.intersects(TypeFlags::TypeParameter) {
        info.variant = TypeVariant::Generic;
        return info;
    }
    if flags.intersects(TypeFlags::Any) {
        info.variant = TypeVariant::Any;
        return info;
    }
    if flags.intersects(TypeFlags::Unknown) {
        info.variant = TypeVariant::Unknown;
        return info;
    }
    if flags.intersects(TypeFlags::Never) {
        info.variant = TypeVariant::Never;
        return info;
    }
    if flags.intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Void) {
        info.variant = TypeVariant::Nullish;
        return info;
    }
    if flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral | TypeFlags::BooleanLike) {
        if literal_string(t) == "true" {
            info.is_truthy = true;
        }
        info.variant = TypeVariant::Boolean;
        return info;
    }
    if flags.intersects(TypeFlags::Enum | TypeFlags::EnumLiteral | TypeFlags::EnumLike) {
        info.variant = if flags.intersects(TypeFlags::StringLiteral) {
            TypeVariant::String
        } else {
            TypeVariant::Number
        };
        info.is_enum = true;
        return info;
    }
    if flags.intersects(TypeFlags::String | TypeFlags::StringLiteral | TypeFlags::StringLike) {
        info.variant = TypeVariant::String;
        if t.is_string_literal() {
            let value = t.as_literal_type().value();
            if value != Some(tsrs_checker::LiteralValue::String("")) {
                info.is_truthy = true;
            }
        }
        return info;
    }
    if flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral | TypeFlags::NumberLike) {
        info.variant = TypeVariant::Number;
        if t.is_number_literal() && literal_string(t) != "0" {
            info.is_truthy = true;
        }
        return info;
    }
    if flags.intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral) {
        info.variant = TypeVariant::BigInt;
        if t.is_big_int_literal() && literal_string(t) != "0" {
            info.is_truthy = true;
        }
        return info;
    }
    if flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
        info.variant = TypeVariant::Object;
        return info;
    }
    if flags.intersects(TypeFlags::Object | TypeFlags::NonPrimitive) {
        info.variant = TypeVariant::Object;
        return info;
    }
    info.variant = TypeVariant::Mixed;
    info
}

fn check_condition(
    ctx: &mut Ctx,
    node: P<Node>,
    types: &[P<Type>],
    opts: &StrictBooleanExpressions,
) {
    let info = analyze_type_parts(types);
    match info.variant {
        TypeVariant::Any | TypeVariant::Unknown | TypeVariant::Generic => {
            if !opts.allow_any {
                ctx.report_node(node, build_condition_error_any_message());
            }
        }
        TypeVariant::Never => {}
        TypeVariant::Nullish => ctx.report_node(node, build_condition_error_nullish_message()),
        TypeVariant::String => {
            // Known edge case: truthy primitives and nullish values are always valid boolean expressions
            if opts.allow_string && info.is_truthy {
                return;
            }
            if info.is_nullable {
                if info.is_enum {
                    if !opts.allow_nullable_enum {
                        ctx.report_node(node, build_condition_error_nullable_enum_message());
                    }
                } else if !opts.allow_nullable_string {
                    ctx.report_node(node, build_condition_error_nullable_string_message());
                }
            } else if !opts.allow_string {
                ctx.report_node(node, build_condition_error_string_message());
            }
        }
        TypeVariant::Number => {
            if opts.allow_number && info.is_truthy {
                return;
            }
            if info.is_nullable {
                if info.is_enum {
                    if !opts.allow_nullable_enum {
                        ctx.report_node(node, build_condition_error_nullable_enum_message());
                    }
                } else if !opts.allow_nullable_number {
                    ctx.report_node(node, build_condition_error_nullable_number_message());
                }
            } else if !opts.allow_number {
                ctx.report_node(node, build_condition_error_number_message());
            }
        }
        TypeVariant::Boolean => {
            if info.is_truthy {
                return;
            }
            if info.is_nullable && !opts.allow_nullable_boolean {
                ctx.report_node(node, build_condition_error_nullable_boolean_message());
            }
        }
        TypeVariant::Object => {
            if info.is_nullable && !opts.allow_nullable_object {
                ctx.report_node(node, build_condition_error_nullable_object_message());
            } else if !info.is_nullable {
                ctx.report_node(node, build_condition_error_object_message());
            }
        }
        TypeVariant::Mixed => {
            if info.is_enum {
                if info.is_nullable && !opts.allow_nullable_enum {
                    ctx.report_node(node, build_condition_error_nullable_enum_message());
                }
                return;
            }
            ctx.report_node(node, build_condition_error_other_message());
        }
        TypeVariant::BigInt => {
            if info.is_nullable && !opts.allow_nullable_number {
                ctx.report_node(node, build_condition_error_nullable_number_message());
            } else if !info.is_nullable && !opts.allow_number {
                ctx.report_node(node, build_condition_error_number_message());
            }
        }
    }
}
