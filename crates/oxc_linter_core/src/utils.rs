use oxc_ast::ast::{CallExpression, Expression};

pub fn is_method_call<'a>(
    call_expr: &CallExpression<'a>,
    objects: Option<&[&'a str]>,
    methods: Option<&[&'a str]>,
    min_arg_count: Option<usize>,
    max_arg_count: Option<usize>,
) -> bool {
    if let Some(min_arg_count) = min_arg_count
        && call_expr.arguments.len() < min_arg_count
    {
        return false;
    }

    if let Some(max_arg_count) = max_arg_count
        && call_expr.arguments.len() > max_arg_count
    {
        return false;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    if let Some(objects) = objects {
        let Expression::Identifier(ident) = member_expr.object().get_inner_expression() else {
            return false;
        };
        if !objects.contains(&ident.name.as_str()) {
            return false;
        }
    }

    if let Some(methods) = methods {
        let Some(static_property_name) = member_expr.static_property_name() else {
            return false;
        };
        if !methods.iter().any(|method| *method == static_property_name) {
            return false;
        }
    }

    true
}

/// Checks if the node is a React component name. React component names must
/// always start with an uppercase letter.
pub fn is_react_component_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}
