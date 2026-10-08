// Port of internal/rules/no_useless_default_assignment/no_useless_default_assignment.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, ContextFlags, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion,
    RuleVisitor,
};
use crate::utils;

fn build_no_strict_null_check_message() -> RuleMessage {
    RuleMessage::new(
        "noStrictNullCheck",
        "This rule requires the `strictNullChecks` compiler option to be turned on to function correctly.",
    )
}

fn build_prefer_optional_syntax_message() -> RuleMessage {
    RuleMessage::with_help(
        "preferOptionalSyntax",
        "Using `= undefined` to make a parameter optional adds unnecessary runtime logic.",
        "Use the `?` optional syntax instead.",
    )
}

fn build_useless_default_assignment_message(assignment_type: &str) -> RuleMessage {
    RuleMessage::with_help(
        "uselessDefaultAssignment",
        format!(
            "Default value is useless because the {assignment_type} is not nullish. This default assignment will never be used."
        ),
        "Remove the default assignment",
    )
}

fn build_useless_default_assignment_with_type_message(
    assignment_type: &str,
    type_text: &str,
) -> RuleMessage {
    RuleMessage::with_help(
        "uselessDefaultAssignment",
        format!(
            "Default value is useless because the {assignment_type} has type `{type_text}` (not nullish). This default assignment will never be used."
        ),
        "Remove the default assignment",
    )
}

fn build_remove_default_assignment_suggestion_message() -> RuleMessage {
    RuleMessage::new("removeDefaultAssignment", "Remove the default assignment.")
}

fn build_useless_undefined_message(plural_assignment_type: &str) -> RuleMessage {
    RuleMessage::with_help(
        "uselessUndefined",
        format!(
            "Default value is useless because it is undefined. Optional {plural_assignment_type} are already undefined by default."
        ),
        "Remove the default assignment",
    )
}

fn can_be_undefined(t: Option<P<Type>>) -> bool {
    let Some(t) = t else { return false };
    if utils::is_type_any_type(t) || utils::is_type_unknown_type(t) {
        return true;
    }
    utils::union_type_parts(t).into_iter().any(utils::is_type_undefined_type)
}

fn get_property_name(node: Option<P<Node>>) -> Option<&'static str> {
    let mut node = node?;
    if ast::is_computed_property_name(node) {
        node = node.expression().unwrap();
    }
    if ast::is_identifier(node) {
        return Some(node.text());
    }
    if ast::is_literal_expression(node) || node.kind() == Kind::NoSubstitutionTemplateLiteral {
        return Some(node.text());
    }
    None
}

fn get_array_element_type(
    c: &mut Checker,
    array_type: P<Type>,
    element_index: i32,
) -> Option<P<Type>> {
    if tsrs_checker::is_tuple_type_exported(array_type) {
        let tuple_args = c.get_type_arguments(array_type);
        if element_index >= 0 && (element_index as usize) < tuple_args.len() {
            return Some(tuple_args[element_index as usize]);
        }
    }
    utils::get_number_index_type(c, array_type)
}

fn find_node_index(nodes: &[P<Node>], target: P<Node>) -> i32 {
    nodes.iter().position(|&n| n == target).map_or(-1, |i| i as i32)
}

fn get_default_assignment_start(node: P<Node>) -> i32 {
    if ast::is_parameter_declaration(node) {
        if let Some(t) = node.type_node() {
            return t.end();
        }
        if let Some(name) = node.name() {
            return name.end();
        }
    }
    if ast::is_binding_element(node) {
        if let Some(name) = node.as_binding_element().name() {
            return name.end();
        }
    }
    node.pos()
}

fn is_simple_type_for_message(t: Option<P<Type>>) -> bool {
    let Some(t) = t else { return false };
    if utils::is_union_type(t) || utils::is_intersection_type(t) {
        return false;
    }
    if utils::is_type_flag_set(
        t,
        TypeFlags::StringLike
            | TypeFlags::NumberLike
            | TypeFlags::BooleanLike
            | TypeFlags::BigIntLike
            | TypeFlags::ESSymbolLike,
    ) {
        return true;
    }
    t.symbol().is_some_and(|s| !s.name().is_empty())
}

fn get_plural_assignment_type(assignment_type: &str) -> String {
    match assignment_type {
        "property" => "properties".to_string(),
        "parameter" => "parameters".to_string(),
        _ => format!("{assignment_type}s"),
    }
}

fn get_assignment_target_node(node: P<Node>) -> Option<P<Node>> {
    if ast::is_parameter_declaration(node) {
        return node.type_node().or_else(|| node.name());
    }
    if ast::is_binding_element(node) {
        return node.as_binding_element().name();
    }
    None
}

fn has_property_in_all_branches(expression: P<Node>, property_name: &str) -> bool {
    let expression = ast::skip_parentheses(expression);
    if ast::is_object_literal_expression(expression) {
        for &property in expression.properties() {
            if ast::is_spread_assignment(property) {
                continue;
            }
            if get_property_name(property.name()) == Some(property_name) {
                return true;
            }
        }
        return false;
    }
    if ast::is_conditional_expression(expression) {
        let conditional = expression.as_conditional_expression();
        return has_property_in_all_branches(conditional.when_true, property_name)
            && has_property_in_all_branches(conditional.when_false, property_name);
    }
    false
}

pub struct NoUselessDefaultAssignment;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUselessDefaultAssignment))
}

const LISTENERS: &[Listener] =
    &[Listener::Enter(Kind::BindingElement), Listener::Enter(Kind::Parameter)];

impl Rule for NoUselessDefaultAssignment {
    fn name(&self) -> &'static str {
        "no-useless-default-assignment"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let options = ctx.program.options();
        if !utils::is_strict_compiler_option_enabled(&options, options.strict_null_checks) {
            ctx.report_range(0, 0, build_no_strict_null_check_message());
        }
        Box::new(Visitor)
    }
}

struct Visitor;

fn get_type_of_binding_element(ctx: &mut Ctx, binding_element: P<Node>) -> Option<P<Type>> {
    if !ast::is_binding_element(binding_element) {
        return None;
    }
    let parent_pattern = binding_element.parent()?;
    if ast::is_object_binding_pattern(parent_pattern) {
        let source_type = get_source_type_for_pattern(ctx, parent_pattern)?;
        let be = binding_element.as_binding_element();
        let property_name_node = be.property_name().or_else(|| be.name());
        let property_name = get_property_name(property_name_node)?;
        let symbol = ctx.checker.get_property_of_type(source_type, property_name)?;
        if utils::is_symbol_flag_set(Some(symbol), tsrs_ast::SymbolFlags::Optional) {
            let parent = parent_pattern.parent();
            match parent {
                Some(parent)
                    if ast::is_variable_declaration(parent) && parent.initializer().is_some() =>
                {
                    if !has_property_in_all_branches(parent.initializer().unwrap(), property_name) {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        return ctx.checker.get_type_of_symbol_at_location(symbol, Some(binding_element));
    }
    if ast::is_array_binding_pattern(parent_pattern) {
        let source_type = get_source_type_for_pattern(ctx, parent_pattern)?;
        let element_index = find_node_index(parent_pattern.elements(), binding_element);
        if element_index == -1 {
            return None;
        }
        return get_array_element_type(ctx.checker, source_type, element_index);
    }
    None
}

fn get_source_type_for_pattern(ctx: &mut Ctx, pattern: P<Node>) -> Option<P<Type>> {
    let parent = pattern.parent()?;
    if ast::is_variable_declaration(parent) {
        if let Some(init) = parent.initializer() {
            return Some(ctx.checker.get_type_at_location(init));
        }
    }
    if ast::is_parameter_declaration(parent) {
        let function_node = parent.parent()?;
        if !ast::is_function_like(function_node) {
            return None;
        }
        let signature = ctx.checker.get_signature_from_declaration_exported(function_node);
        let params = function_node.parameters();
        let mut param_index = find_node_index(params, parent);
        if param_index == -1 {
            return None;
        }
        if signature.this_parameter().is_some() && !params.is_empty() {
            let first_parameter = params[0];
            if first_parameter.name().is_some_and(|n| ast::is_identifier(n) && n.text() == "this") {
                param_index -= 1;
            }
        }
        let parameters = signature.parameters.get();
        if param_index < 0 || param_index as usize >= parameters.len() {
            return None;
        }
        return Some(ctx.checker.get_type_of_symbol(parameters[param_index as usize]));
    }
    if ast::is_binding_element(parent) {
        return get_type_of_binding_element(ctx, parent);
    }
    if ast::is_array_binding_pattern(parent) {
        let array_type = get_source_type_for_pattern(ctx, parent)?;
        let element_index = find_node_index(parent.elements(), pattern);
        if element_index == -1 {
            return None;
        }
        return get_array_element_type(ctx.checker, array_type, element_index);
    }
    None
}

fn build_remove_default_fix(ctx: &Ctx, node: P<Node>) -> RuleFix {
    ctx.fix_remove_range(get_default_assignment_start(node), node.end())
}

fn report_useless_default_assignment(
    ctx: &mut Ctx,
    node: P<Node>,
    assignment_type: &str,
    value_type: Option<P<Type>>,
) {
    let Some(initializer) = node.initializer() else {
        return;
    };
    let mut message = build_useless_default_assignment_message(assignment_type);
    let mut type_text = String::new();
    if is_simple_type_for_message(value_type) {
        type_text = utils::type_to_string(ctx.checker, value_type.unwrap());
        message = build_useless_default_assignment_with_type_message(assignment_type, &type_text);
    }
    let (pos, end) = ctx.trim(initializer);
    let fixes = vec![build_remove_default_fix(ctx, node)];
    let mut labeled_ranges = vec![LabeledRange { label: "Default value".to_string(), pos, end }];
    if !type_text.is_empty() {
        if let Some(target_node) = get_assignment_target_node(node) {
            let (tpos, tend) = ctx.trim(target_node);
            labeled_ranges.push(LabeledRange {
                label: format!("{assignment_type} type `{type_text}` is not nullish"),
                pos: tpos,
                end: tend,
            });
        }
    }
    ctx.report_diagnostic_with_suggestions(
        RuleDiagnostic { pos, end, message, labeled_ranges },
        |_| {
            vec![RuleSuggestion {
                message: build_remove_default_assignment_suggestion_message(),
                fixes,
            }]
        },
    );
}

fn report_useless_undefined(ctx: &mut Ctx, node: P<Node>, assignment_type: &str) {
    let Some(initializer) = node.initializer() else {
        return;
    };
    ctx.report_node_with_fixes(
        initializer,
        build_useless_undefined_message(&get_plural_assignment_type(assignment_type)),
        |ctx| vec![build_remove_default_fix(ctx, node)],
    );
}

fn report_prefer_optional_syntax(ctx: &mut Ctx, node: P<Node>) {
    let Some(initializer) = node.initializer() else {
        return;
    };
    ctx.report_node_with_fixes(initializer, build_prefer_optional_syntax_message(), |ctx| {
        let mut fixes = vec![build_remove_default_fix(ctx, node)];
        if ast::is_parameter_declaration(node) {
            if let Some(name) = node.name() {
                if ast::is_identifier(name) {
                    let (_, end) = ctx.trim(name);
                    fixes.push(ctx.fix_replace_range(end, end, "?"));
                }
            }
        }
        fixes
    });
}

fn check_function_expression_parameter(ctx: &mut Ctx, node: P<Node>) {
    let Some(parent) = node.parent() else { return };
    if !ast::is_arrow_function(parent) && !ast::is_function_expression(parent) {
        return;
    }
    let param_index = find_node_index(parent.parameters(), node);
    if param_index == -1 {
        return;
    }
    let Some(contextual_type) = ctx.checker.get_contextual_type(parent, ContextFlags::None) else {
        return;
    };
    let signatures = utils::get_call_signatures(ctx.checker, contextual_type);
    if signatures.is_empty() || signatures[0].declaration() == Some(parent) {
        return;
    }
    let parameters = signatures[0].parameters.get();
    if param_index as usize >= parameters.len() {
        return;
    }
    let param_symbol = parameters[param_index as usize];
    if let Some(vd) = param_symbol.value_declaration() {
        if ast::is_parameter_declaration(vd)
            && vd.as_parameter_declaration().dot_dot_dot_token().is_some()
        {
            return;
        }
    }
    if !utils::is_symbol_flag_set(Some(param_symbol), tsrs_ast::SymbolFlags::Optional) {
        let param_type = ctx.checker.get_type_of_symbol(param_symbol);
        if !utils::is_type_parameter(param_type) && !can_be_undefined(Some(param_type)) {
            report_useless_default_assignment(ctx, node, "parameter", Some(param_type));
        }
    }
}

fn check_assignment_pattern(ctx: &mut Ctx, node: P<Node>) {
    let Some(initializer) = node.initializer() else {
        return;
    };
    let initializer = ast::skip_parentheses(initializer);
    if utils::is_undefined_identifier(Some(initializer)) {
        if ast::is_parameter_declaration(node) {
            if let Some(type_node) = node.type_node() {
                let t = ctx.checker.get_type_from_type_node(type_node);
                if can_be_undefined(Some(t)) {
                    report_prefer_optional_syntax(ctx, node);
                    return;
                }
            }
            report_useless_undefined(ctx, node, "parameter");
            return;
        }
        report_useless_undefined(ctx, node, "property");
        return;
    }
    if ast::is_parameter_declaration(node) {
        check_function_expression_parameter(ctx, node);
        return;
    }
    if !ast::is_binding_element(node) {
        return;
    }
    let Some(parent) = node.parent() else { return };
    if ast::is_object_binding_pattern(parent) {
        let property_type = get_type_of_binding_element(ctx, node);
        if property_type.is_some() && !can_be_undefined(property_type) {
            report_useless_default_assignment(ctx, node, "property", property_type);
        }
        return;
    }
    if ast::is_array_binding_pattern(parent) {
        let Some(source_type) = get_source_type_for_pattern(ctx, parent) else {
            return;
        };
        if !tsrs_checker::is_tuple_type_exported(source_type) {
            return;
        }
        let tuple_args = ctx.checker.get_type_arguments(source_type);
        let element_index = find_node_index(parent.elements(), node);
        if element_index < 0 || element_index as usize >= tuple_args.len() {
            return;
        }
        let arg = tuple_args[element_index as usize];
        if !can_be_undefined(Some(arg)) {
            report_useless_default_assignment(ctx, node, "property", Some(arg));
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        check_assignment_pattern(ctx, node);
    }
}
