// Port of internal/rules/no_unsafe_assignment/no_unsafe_assignment.go.

use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node, SourceFile};
use tsrs_checker::{Checker, Type};
use tsrs_core::P;

use crate::rule::{Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor};
use crate::utils;

type Range = (i32, i32);

fn format_sender_type(sender_type: P<Type>) -> &'static str {
    if utils::is_intrinsic_error_type(sender_type) {
        return "error typed";
    }
    "any"
}

fn build_any_assignment_message(sender: P<Type>) -> RuleMessage {
    RuleMessage::new(
        "anyAssignment",
        format!("Unsafe assignment of an {} value.", format_sender_type(sender)),
    )
}
fn build_any_assignment_this_message(sender: P<Type>) -> RuleMessage {
    RuleMessage::with_help(
        "anyAssignmentThis",
        format!(
            "Unsafe assignment of an {} value. `this` is typed as `any`.\n",
            format_sender_type(sender)
        ),
        "You can try to fix this by turning on the `noImplicitThis` compiler option, or adding a `this` parameter to the function.",
    )
}
fn build_unsafe_array_pattern_message(sender: P<Type>) -> RuleMessage {
    RuleMessage::new(
        "unsafeArrayPattern",
        format!("Unsafe array destructuring of an {} array value.", format_sender_type(sender)),
    )
}
fn build_unsafe_array_pattern_from_tuple_message(sender: P<Type>) -> RuleMessage {
    RuleMessage::new(
        "unsafeArrayPatternFromTuple",
        format!(
            "Unsafe array destructuring of a tuple element with an {} value.",
            format_sender_type(sender)
        ),
    )
}
fn build_unsafe_array_spread_message(sender: P<Type>) -> RuleMessage {
    RuleMessage::new(
        "unsafeArraySpread",
        format!("Unsafe spread of an {} value in an array.", format_sender_type(sender)),
    )
}
fn build_unsafe_assignment_message() -> RuleMessage {
    RuleMessage::new("unsafeAssignment", "Unsafe assignment between incompatible types.")
}

fn build_assignment_diagnostic(
    primary_range: Range,
    sender_range: Range,
    receiver_range: Range,
    sender_type: &str,
    receiver_type: &str,
    message: RuleMessage,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: primary_range.0,
        end: primary_range.1,
        message,
        labeled_ranges: vec![
            LabeledRange {
                label: format!("Assigned value has type `{sender_type}`."),
                pos: sender_range.0,
                end: sender_range.1,
            },
            LabeledRange {
                label: format!("Target expects type `{receiver_type}`."),
                pos: receiver_range.0,
                end: receiver_range.1,
            },
        ],
    }
}

fn build_this_assignment_diagnostic(
    primary_range: Range,
    this_range: Range,
    receiver_range: Range,
    this_type: &str,
    receiver_type: &str,
    message: RuleMessage,
) -> RuleDiagnostic {
    let mut diagnostic = build_assignment_diagnostic(
        primary_range,
        this_range,
        receiver_range,
        this_type,
        receiver_type,
        message,
    );
    diagnostic.labeled_ranges[0].label = format!("`this` has type `{this_type}`.");
    diagnostic
}

fn build_destructure_diagnostic(
    receiver_range: Range,
    sender_range: Range,
    sender_type: &str,
    unsafe_type: &str,
    message: RuleMessage,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: receiver_range.0,
        end: receiver_range.1,
        message,
        labeled_ranges: vec![
            LabeledRange {
                label: format!("Destructured source provides type `{sender_type}`."),
                pos: sender_range.0,
                end: sender_range.1,
            },
            LabeledRange {
                label: format!("This binding receives type `{unsafe_type}`."),
                pos: receiver_range.0,
                end: receiver_range.1,
            },
        ],
    }
}

fn build_array_spread_diagnostic(
    spread_range: Range,
    value_range: Range,
    value_type: &str,
    message: RuleMessage,
) -> RuleDiagnostic {
    RuleDiagnostic {
        pos: spread_range.0,
        end: spread_range.1,
        message,
        labeled_ranges: vec![LabeledRange {
            label: format!("Spread value has type `{value_type}`."),
            pos: value_range.0,
            end: value_range.1,
        }],
    }
}

fn diagnostic_type_text(c: &mut Checker, t: P<Type>) -> String {
    if utils::is_intrinsic_error_type(t) {
        return "error".to_string();
    }
    utils::type_to_string(c, t)
}

fn assignment_relation_range(
    source_file: P<SourceFile>,
    receiver_node: P<Node>,
    sender_node: P<Node>,
) -> Range {
    let mut s = tsrs_scanner::get_scanner_for_source_file(source_file, receiver_node.end());
    let mut colon_range: Option<Range> = None;
    while s.token() != Kind::EndOfFile && s.token_range().pos() < sender_node.end() {
        match s.token() {
            Kind::EqualsToken => {
                let r = s.token_range();
                return (r.pos(), r.end());
            }
            Kind::ColonToken => {
                let r = s.token_range();
                colon_range = Some((r.pos(), r.end()));
            }
            _ => {}
        }
        if s.token_range().pos() >= sender_node.pos() {
            break;
        }
        s.scan();
    }
    if let Some(r) = colon_range {
        if r != (0, 0) {
            return r;
        }
    }
    utils::trim_node_text_range(source_file, receiver_node)
}

fn local_target_range(
    source_file: P<SourceFile>,
    receiver_node: P<Node>,
    type_annotation_node: Option<P<Node>>,
) -> Range {
    if let Some(t) = type_annotation_node {
        if ast::get_source_file_of_node(t) == Some(source_file) {
            return utils::trim_node_text_range(source_file, t);
        }
    }
    utils::trim_node_text_range(source_file, receiver_node)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComparisonType {
    /// Do no assignment comparison
    None,
    /// Use the receiver's type for comparison
    Basic,
    /// Use the sender's contextual type for comparison
    Contextual,
}

fn can_skip_sender_type_check(node: P<Node>, comp_type: ComparisonType) -> bool {
    let node = ast::skip_parentheses(node);
    match node.kind() {
        Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::RegularExpressionLiteral
        | Kind::NoSubstitutionTemplateLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword => true,
        Kind::PrefixUnaryExpression => {
            let expr = node.as_prefix_unary_expression();
            (expr.operator == Kind::PlusToken || expr.operator == Kind::MinusToken)
                && ast::skip_parentheses(expr.operand).kind() == Kind::NumericLiteral
        }
        Kind::ArrowFunction | Kind::FunctionExpression | Kind::ClassExpression => true,
        Kind::ObjectLiteralExpression => !node
            .as_object_literal_expression()
            .properties
            .nodes()
            .iter()
            .any(|&p| ast::is_spread_assignment(p)),
        Kind::ArrayLiteralExpression => {
            comp_type == ComparisonType::None
                && !node
                    .as_array_literal_expression()
                    .elements
                    .nodes()
                    .iter()
                    .any(|&e| ast::is_spread_element(e))
        }
        _ => false,
    }
}

pub struct NoUnsafeAssignment;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnsafeAssignment))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::PropertyDeclaration),
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::BindingElement),
    Listener::Enter(Kind::Parameter),
    Listener::Enter(Kind::ShorthandPropertyAssignment),
    Listener::Enter(Kind::VariableDeclaration),
    Listener::NotAllowPattern(Kind::ObjectLiteralExpression),
    Listener::NotAllowPattern(Kind::ArrayLiteralExpression),
    Listener::Enter(Kind::JsxAttribute),
];

impl Rule for NoUnsafeAssignment {
    fn name(&self) -> &'static str {
        "no-unsafe-assignment"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let compiler_options = ctx.program.options();
        let is_no_implicit_this = utils::is_strict_compiler_option_enabled(
            &compiler_options,
            compiler_options.no_implicit_this,
        );
        Box::new(Visitor { is_no_implicit_this })
    }
}

struct Visitor {
    is_no_implicit_this: bool,
}

fn report_destructure(
    ctx: &mut Ctx,
    receiver_node: P<Node>,
    sender_node: P<Node>,
    sender_type: P<Type>,
    unsafe_type: &str,
    message: RuleMessage,
) {
    let receiver_range = ctx.trim(receiver_node);
    let sender_range = ctx.trim(sender_node);
    let sender_text = diagnostic_type_text(ctx.checker, sender_type);
    ctx.report_diagnostic(build_destructure_diagnostic(
        receiver_range,
        sender_range,
        &sender_text,
        unsafe_type,
        message,
    ));
}

/// returns true if the assignment reported
fn check_object_destructure(
    ctx: &mut Ctx,
    receiver_node: P<Node>,
    sender_type: P<Type>,
    sender_node: P<Node>,
) -> bool {
    let property_symbols = ctx.checker.get_properties_of_type(sender_type);
    let mut properties: FxHashMap<&'static str, P<Type>> =
        FxHashMap::with_capacity_and_hasher(property_symbols.len(), Default::default());
    for &property in property_symbols {
        if let Some(t) = ctx.checker.get_type_of_symbol_at_location(property, Some(sender_node)) {
            properties.insert(property.name(), t);
        }
    }

    let check_object_property =
        |ctx: &mut Ctx, property_key: P<Node>, property_value: P<Node>| -> bool {
            let key: String = if !ast::is_computed_property_name(property_key) {
                property_key.text().to_string()
            } else if ast::is_literal_expression(property_key.expression().unwrap()) {
                property_key.expression().unwrap().text().to_string()
            } else {
                // can't figure out the name, so skip it
                return false;
            };

            let Some(&sender_type) = properties.get(key.as_str()) else {
                return false;
            };

            // check for the any type first so we can handle {x: {y: z}} = {x: any}
            if utils::is_type_any_type(sender_type) {
                // TODO(port): why object reported with "array" message?
                let text = diagnostic_type_text(ctx.checker, sender_type);
                report_destructure(
                    ctx,
                    property_value,
                    sender_node,
                    sender_type,
                    &text,
                    build_unsafe_array_pattern_from_tuple_message(sender_type),
                );
                return true;
            } else if ast::is_array_binding_pattern(property_value)
                || ast::is_array_literal_expression(property_value)
            {
                return check_array_destructure(ctx, property_value, sender_type, sender_node);
            } else if ast::is_object_binding_pattern(property_value)
                || ast::is_object_literal_expression(property_value)
            {
                return check_object_destructure(ctx, property_value, sender_type, sender_node);
            }
            false
        };

    let mut did_report = false;
    if ast::is_object_literal_expression(receiver_node) {
        for &receiver_property in receiver_node.as_object_literal_expression().properties.nodes() {
            if ast::is_spread_assignment(receiver_property) {
                // don't bother checking rest
                continue;
            }
            if (ast::is_property_assignment(receiver_property)
                && check_object_property(
                    ctx,
                    receiver_property.name().unwrap(),
                    receiver_property.initializer().unwrap(),
                ))
                || (ast::is_shorthand_property_assignment(receiver_property)
                    && check_object_property(
                        ctx,
                        receiver_property.name().unwrap(),
                        receiver_property.name().unwrap(),
                    ))
            {
                did_report = true;
            }
        }
    } else if ast::is_object_binding_pattern(receiver_node) {
        for &receiver_property in receiver_node.as_binding_pattern().elements.nodes() {
            let property = receiver_property.as_binding_element();
            if property.dot_dot_dot_token().is_some() {
                // don't bother checking rest
                continue;
            }
            let name = property.name().unwrap();
            let property_key = property.property_name().unwrap_or(name);
            if check_object_property(ctx, property_key, name) {
                did_report = true;
            }
        }
    }
    did_report
}

/// returns true if the assignment reported
fn check_object_destructure_helper(
    ctx: &mut Ctx,
    receiver_node: P<Node>,
    sender_node: P<Node>,
) -> bool {
    if !ast::is_object_binding_pattern(receiver_node)
        && !ast::is_object_literal_expression(receiver_node)
    {
        return false;
    }
    let sender_type = ctx.checker.get_type_at_location(sender_node);
    check_object_destructure(ctx, receiver_node, sender_type, sender_node)
}

/// returns true if the assignment reported
fn check_array_destructure(
    ctx: &mut Ctx,
    receiver_node: P<Node>,
    sender_type: P<Type>,
    sender_node: P<Node>,
) -> bool {
    // any array
    // const [x] = ([] as any[]);
    if utils::is_type_any_array_type(sender_type, ctx.checker) {
        report_destructure(
            ctx,
            receiver_node,
            sender_node,
            sender_type,
            "any",
            build_unsafe_array_pattern_message(sender_type),
        );
        return false;
    }

    if !sender_type.is_tuple_type() {
        return true;
    }

    let tuple_elements = ctx.checker.get_type_arguments(sender_type);

    let check_array_element =
        |ctx: &mut Ctx, receiver_element: Option<P<Node>>, receiver_index: usize| -> bool {
            let Some(receiver_element) = receiver_element else {
                return false;
            };
            if receiver_index >= tuple_elements.len() {
                return false;
            }
            let sender_type = tuple_elements[receiver_index];

            // check for the any type first so we can handle [[[x]]] = [any]
            if utils::is_type_any_type(sender_type) {
                let text = diagnostic_type_text(ctx.checker, sender_type);
                report_destructure(
                    ctx,
                    receiver_element,
                    sender_node,
                    sender_type,
                    &text,
                    build_unsafe_array_pattern_from_tuple_message(sender_type),
                );
                return true;
            } else if ast::is_array_binding_pattern(receiver_element)
                || ast::is_array_literal_expression(receiver_element)
            {
                return check_array_destructure(ctx, receiver_element, sender_type, sender_node);
            } else if ast::is_object_binding_pattern(receiver_element)
                || ast::is_object_literal_expression(receiver_element)
            {
                return check_object_destructure(ctx, receiver_element, sender_type, sender_node);
            }
            false
        };

    // tuple with any
    // const [x] = [1 as any];
    let mut did_report = false;
    if ast::is_array_literal_expression(receiver_node) {
        for (receiver_index, &receiver_element) in
            receiver_node.as_array_literal_expression().elements.nodes().iter().enumerate()
        {
            if ast::is_spread_element(receiver_element) {
                // don't handle rests as they're not a 1:1 assignment
                continue;
            }
            if check_array_element(ctx, Some(receiver_element), receiver_index) {
                did_report = true;
            }
        }
    } else if ast::is_array_binding_pattern(receiver_node) {
        for (receiver_index, &receiver_element) in
            receiver_node.as_binding_pattern().elements.nodes().iter().enumerate()
        {
            if receiver_element.kind() == Kind::BindingElement
                && receiver_element.as_binding_element().dot_dot_dot_token().is_some()
            {
                // don't handle rests as they're not a 1:1 assignment
                continue;
            }
            if check_array_element(ctx, receiver_element.name(), receiver_index) {
                // TODO(port): in original rule didReport was reassigned every time. isn't it a bug?
                did_report = true;
            }
        }
    }
    did_report
}

/// returns true if the assignment reported
fn check_array_destructure_helper(
    ctx: &mut Ctx,
    receiver_node: P<Node>,
    sender_node: P<Node>,
) -> bool {
    if !ast::is_array_binding_pattern(receiver_node)
        && !ast::is_array_literal_expression(receiver_node)
    {
        return false;
    }
    let sender_type = ctx.checker.get_type_at_location(sender_node);
    check_array_destructure(ctx, receiver_node, sender_type, sender_node)
}

fn get_comparison_type(node_with_type_annotation: P<Node>) -> ComparisonType {
    if node_with_type_annotation.type_node().is_some() {
        // if there's a type annotation, we can do a comparison
        return ComparisonType::Basic;
    }
    // no type annotation means the variable's type will just be inferred, thus equal
    ComparisonType::None
}

impl Visitor {
    /// returns true if the assignment reported
    fn check_assignment(
        &self,
        ctx: &mut Ctx,
        receiver_node: P<Node>,
        sender_node: P<Node>,
        type_annotation_node: Option<P<Node>>,
        primary_range: Range,
        comp_type: ComparisonType,
    ) -> bool {
        // Fast path: return early when we know that the sender definitely cannot have an `any` type,
        // because it is syntactically impossible given the sender's node kind.
        if can_skip_sender_type_check(sender_node, comp_type) {
            return false;
        }

        let sender_type = ctx.checker.get_type_at_location(sender_node);

        let get_receiver_type = |ctx: &mut Ctx| -> P<Type> {
            if comp_type == ComparisonType::Contextual {
                if let Some(t) = utils::get_contextual_type(ctx.checker, sender_node) {
                    return t;
                }
                if let Some(t) = utils::get_contextual_type(ctx.checker, receiver_node) {
                    return t;
                }
            }
            ctx.checker.get_type_at_location(receiver_node)
        };

        if utils::is_type_any_type(sender_type) {
            let receiver_type = get_receiver_type(ctx);
            let receiver_range = local_target_range(ctx.file, receiver_node, type_annotation_node);
            let set_inferred_target_label = |ctx: &mut Ctx, diagnostic: &mut RuleDiagnostic| {
                if comp_type == ComparisonType::None {
                    diagnostic.labeled_ranges[1].label = format!(
                        "Target is inferred as `{}`.",
                        diagnostic_type_text(ctx.checker, receiver_type)
                    );
                }
            };

            // handle cases when we assign any ==> unknown.
            if utils::is_type_unknown_type(receiver_type) {
                return false;
            }

            if !self.is_no_implicit_this {
                // `var foo = this`
                if let Some(this_expression) = utils::get_this_expression(sender_node) {
                    let this_type =
                        utils::get_constrained_type_at_location(ctx.checker, this_expression);
                    if utils::is_type_any_type(this_type) {
                        let this_range = ctx.trim(this_expression);
                        let this_text = diagnostic_type_text(ctx.checker, this_type);
                        let receiver_text = diagnostic_type_text(ctx.checker, receiver_type);
                        let mut diagnostic = build_this_assignment_diagnostic(
                            primary_range,
                            this_range,
                            receiver_range,
                            &this_text,
                            &receiver_text,
                            build_any_assignment_this_message(sender_type),
                        );
                        set_inferred_target_label(ctx, &mut diagnostic);
                        ctx.report_diagnostic(diagnostic);
                        return true;
                    }
                }
            }

            let sender_range = ctx.trim(sender_node);
            let sender_text = diagnostic_type_text(ctx.checker, sender_type);
            let receiver_text = diagnostic_type_text(ctx.checker, receiver_type);
            let mut diagnostic = build_assignment_diagnostic(
                primary_range,
                sender_range,
                receiver_range,
                &sender_text,
                &receiver_text,
                build_any_assignment_message(sender_type),
            );
            set_inferred_target_label(ctx, &mut diagnostic);
            ctx.report_diagnostic(diagnostic);
            return true;
        }

        if comp_type == ComparisonType::None {
            return false;
        }
        if !tsrs_checker::is_non_deferred_type_reference(sender_type) {
            return false;
        }

        let receiver_type = get_receiver_type(ctx);

        let Some((receiver, sender)) =
            utils::is_unsafe_assignment(sender_type, receiver_type, ctx.checker, Some(sender_node))
        else {
            return false;
        };

        let sender_range = ctx.trim(sender_node);
        let receiver_range = local_target_range(ctx.file, receiver_node, type_annotation_node);
        let sender_text = diagnostic_type_text(ctx.checker, sender);
        let receiver_text = diagnostic_type_text(ctx.checker, receiver);
        ctx.report_diagnostic(build_assignment_diagnostic(
            primary_range,
            sender_range,
            receiver_range,
            &sender_text,
            &receiver_text,
            build_unsafe_assignment_message(),
        ));
        true
    }

    fn check_assignment_full(
        &self,
        ctx: &mut Ctx,
        id: Option<P<Node>>,
        init: Option<P<Node>>,
        type_annotation_node: Option<P<Node>>,
        primary_range: Range,
    ) {
        let (Some(id), Some(init)) = (id, init) else {
            return;
        };
        let mut did_report = self.check_assignment(
            ctx,
            id,
            init,
            type_annotation_node,
            primary_range,
            // the variable already has some form of a type to compare against
            ComparisonType::Basic,
        );
        if !did_report {
            did_report = check_array_destructure_helper(ctx, id, init);
        }
        if !did_report {
            check_object_destructure_helper(ctx, id, init);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            // ESTree PropertyDefinition, AccessorProperty
            Listener::Enter(Kind::PropertyDeclaration) => {
                let Some(initializer) = node.initializer() else {
                    return;
                };
                let name = node.name().unwrap();
                let range = assignment_relation_range(ctx.file, name, initializer);
                self.check_assignment(
                    ctx,
                    name,
                    initializer,
                    node.type_node(),
                    range,
                    get_comparison_type(node),
                );
            }
            // ESTree AssignmentExpression, AssignmentPattern
            Listener::Enter(Kind::BinaryExpression) => {
                if !ast::is_assignment_expression(node, true) {
                    return;
                }
                let expr = node.as_binary_expression();
                let range = ctx.trim(expr.operator_token);
                self.check_assignment_full(
                    ctx,
                    Some(expr.left),
                    Some(expr.right.get()),
                    None,
                    range,
                );
            }
            // ESTree AssignmentPattern
            Listener::Enter(Kind::BindingElement) | Listener::Enter(Kind::Parameter) => {
                if let Some(initializer) = node.initializer() {
                    let name = node.name().unwrap();
                    let range = assignment_relation_range(ctx.file, name, initializer);
                    self.check_assignment_full(
                        ctx,
                        Some(name),
                        Some(initializer),
                        node.type_node(),
                        range,
                    );
                }
            }
            // ESTree AssignmentPattern
            Listener::Enter(Kind::ShorthandPropertyAssignment) => {
                let assignment = node.as_shorthand_property_assignment();
                if let Some(initializer) = assignment.object_assignment_initializer() {
                    let name = node.name().unwrap();
                    let range = assignment_relation_range(ctx.file, name, initializer);
                    self.check_assignment_full(ctx, Some(name), Some(initializer), None, range);
                }
            }
            Listener::Enter(Kind::VariableDeclaration) => {
                let Some(init) = node.initializer() else {
                    return;
                };
                let id = node.name().unwrap();
                let range = assignment_relation_range(ctx.file, id, init);
                let mut did_report = self.check_assignment(
                    ctx,
                    id,
                    init,
                    node.type_node(),
                    range,
                    get_comparison_type(node),
                );
                if !did_report {
                    did_report = check_array_destructure_helper(ctx, id, init);
                }
                if !did_report {
                    check_object_destructure_helper(ctx, id, init);
                }
            }
            // object pattern props are checked via assignments
            Listener::NotAllowPattern(Kind::ObjectLiteralExpression) => {
                for &prop in node.as_object_literal_expression().properties.nodes() {
                    let init = if ast::is_property_assignment(prop) {
                        prop.initializer()
                    } else if ast::is_shorthand_property_assignment(prop) {
                        prop.name()
                    } else {
                        continue;
                    };
                    let Some(init) = init else {
                        return;
                    };
                    let init = ast::skip_parentheses(init);
                    if ast::is_assignment_expression(init, false) {
                        // node.value.type === AST_NODE_TYPES.TSEmptyBodyFunctionExpression
                        // handled by other selector
                        return;
                    }
                    let name = prop.name().unwrap();
                    let range = assignment_relation_range(ctx.file, name, init);
                    self.check_assignment(ctx, name, init, None, range, ComparisonType::Contextual);
                }
            }
            Listener::NotAllowPattern(Kind::ArrayLiteralExpression) => {
                for &element in node.as_array_literal_expression().elements.nodes() {
                    if !ast::is_spread_element(element) {
                        continue;
                    }
                    let expression = element.expression().unwrap();
                    let rest_type = ctx.checker.get_type_at_location(expression);
                    if utils::is_type_any_type(rest_type)
                        || utils::is_type_any_array_type(rest_type, ctx.checker)
                    {
                        let node_range = ctx.trim(element);
                        let value_range = ctx.trim(expression);
                        let text = diagnostic_type_text(ctx.checker, rest_type);
                        ctx.report_diagnostic(build_array_spread_diagnostic(
                            (node_range.0, node_range.0 + 3),
                            value_range,
                            &text,
                            build_unsafe_array_spread_message(rest_type),
                        ));
                    }
                }
            }
            Listener::Enter(Kind::JsxAttribute) => {
                let Some(init) = node.initializer() else {
                    return;
                };
                if init.kind() != Kind::JsxExpression {
                    return;
                }
                let Some(expr) = init.as_jsx_expression().expression else {
                    return;
                };
                let name = node.name().unwrap();
                let range = assignment_relation_range(ctx.file, name, expr);
                self.check_assignment(ctx, name, expr, None, range, ComparisonType::Contextual);
            }
            _ => {}
        }
    }
}
