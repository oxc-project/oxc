// Port of internal/rules/no_unnecessary_type_arguments/no_unnecessary_type_arguments.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, NodeList, Symbol, SymbolFlags};
use tsrs_checker::{CheckMode, Checker, ObjectFlags, Type};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use crate::utils;

fn build_unnecessary_type_parameter_message() -> RuleMessage {
    RuleMessage::new(
        "unnecessaryTypeParameter",
        "This is the default value for this type parameter, so it can be omitted.",
    )
}

fn is_type_context_declaration(decl: P<Node>) -> bool {
    ast::is_type_alias_declaration(decl) || ast::is_interface_declaration(decl)
}

fn is_in_type_context(node: P<Node>) -> bool {
    let parent = node.parent().unwrap();
    ast::is_type_reference_node(node)
        || ast::is_interface_declaration(parent)
        || ast::is_type_reference_node(parent)
        || (ast::is_heritage_clause(parent)
            && parent.as_heritage_clause().token == Kind::ImplementsKeyword)
}

struct TypeForComparison {
    type_value: Option<P<Type>>,
    type_arguments: &'static [P<Type>],
}

fn get_type_for_comparison(c: &mut Checker, t: P<Type>) -> TypeForComparison {
    if t.object_flags().intersects(ObjectFlags::Reference) {
        return TypeForComparison {
            type_value: t.target(),
            type_arguments: c.get_type_arguments(t),
        };
    }
    TypeForComparison { type_value: Some(t), type_arguments: &[] }
}

pub struct NoUnnecessaryTypeArguments;

pub fn create(_options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    Ok(Box::new(NoUnnecessaryTypeArguments))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ExpressionWithTypeArguments),
    Listener::Enter(Kind::TypeReference),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::NewExpression),
    Listener::Enter(Kind::TaggedTemplateExpression),
    Listener::Enter(Kind::JsxOpeningElement),
    Listener::Enter(Kind::JsxSelfClosingElement),
];

impl Rule for NoUnnecessaryTypeArguments {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-arguments"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn get_type_parameters_from_type(
    c: &mut Checker,
    node: P<Node>,
    node_name: P<Node>,
) -> &'static [P<Node>] {
    let Some(mut symbol) = c.get_symbol_at_location_exported(node_name) else {
        return &[];
    };
    if symbol.flags().intersects(SymbolFlags::Alias) {
        let (resolved, found) = c.resolve_alias_exported(Some(symbol));
        match resolved {
            Some(s) if found => symbol = s,
            _ => return &[],
        }
    }

    let mut declarations: Vec<P<Node>> = symbol.declarations().to_vec();
    if declarations.is_empty() {
        return &[];
    }

    let node_in_type_context = is_in_type_context(node);
    declarations.sort_by(|&a, &b| {
        let (a, b) = if node_in_type_context { (a, b) } else { (b, a) };
        let mut res = 0;
        if is_type_context_declaration(a) {
            res -= 1;
        }
        if is_type_context_declaration(b) {
            res += 1;
        }
        res.cmp(&0)
    });

    for decl in declarations {
        if ast::is_type_alias_declaration(decl)
            || ast::is_interface_declaration(decl)
            || ast::is_class_like(decl)
        {
            return decl.type_parameters();
        }
        if ast::is_variable_declaration(decl) {
            let t = c.get_type_of_symbol(symbol);
            let signatures = utils::get_construct_signatures(c, t);
            if signatures.is_empty() {
                continue;
            }
            if let Some(decl) = signatures[0].declaration.get() {
                return decl.type_parameters();
            }
        }
    }
    &[]
}

fn get_type_parameters_from_call(c: &mut Checker, node: P<Node>) -> &'static [P<Node>] {
    let signature = c.get_resolved_signature(node, None, CheckMode::Normal);
    if let Some(declaration) = signature.declaration.get() {
        let type_parameters = declaration.type_parameters();
        if !type_parameters.is_empty() {
            return type_parameters;
        }
    }
    if ast::is_new_expression(node) {
        return get_type_parameters_from_type(c, node, node.expression().unwrap());
    }
    &[]
}

fn type_node_references_type_parameter(
    c: &mut Checker,
    type_node: P<Node>,
    type_parameter_symbols: &FxHashSet<P<Symbol>>,
) -> bool {
    fn visit(c: &mut Checker, node: P<Node>, symbols: &FxHashSet<P<Symbol>>) -> bool {
        if ast::is_identifier(node) {
            if let Some(symbol) = c.get_symbol_at_location_exported(node) {
                if symbols.contains(&symbol) {
                    return true;
                }
            }
        }
        node.for_each_child(&mut |child| visit(c, child, symbols))
    }
    visit(c, type_node, type_parameter_symbols)
}

fn constructor_arguments_can_infer_type_parameters(
    c: &mut Checker,
    node: P<Node>,
    parameters: &[P<Node>],
) -> bool {
    if !ast::is_new_expression(node) || node.arguments().is_empty() {
        return false;
    }

    let mut type_parameter_symbols: FxHashSet<P<Symbol>> = FxHashSet::default();
    for &parameter in parameters {
        let Some(name) = parameter.name() else {
            continue;
        };
        if let Some(symbol) = c.get_symbol_at_location_exported(name) {
            type_parameter_symbols.insert(symbol);
        }
    }
    if type_parameter_symbols.is_empty() {
        return false;
    }

    let signature = c.get_resolved_signature(node, None, CheckMode::Normal);
    let Some(declaration) = signature.declaration.get() else {
        return false;
    };
    if declaration.function_like_data().is_none() {
        return false;
    }

    for &parameter in declaration.parameters() {
        if let Some(type_node) = parameter.type_node() {
            if type_node_references_type_parameter(c, type_node, &type_parameter_symbols) {
                return true;
            }
        }
    }
    false
}

fn check_args_and_parameters(
    ctx: &mut Ctx,
    node: P<Node>,
    arguments: P<NodeList>,
    parameters: &[P<Node>],
) {
    let args = arguments.nodes();
    if args.is_empty() || parameters.is_empty() {
        return;
    }

    // Just check the last one. Must specify previous type parameters if the last one is specified.
    let last_param_index = args.len() - 1;
    if last_param_index >= parameters.len() {
        return;
    }

    let type_argument = args[last_param_index];
    let type_parameter = parameters[last_param_index];

    let Some(default_type_node) = type_parameter.as_type_parameter_declaration().default_type
    else {
        return;
    };

    let default_type = ctx.checker.get_type_at_location(default_type_node);
    let arg_type = ctx.checker.get_type_at_location(type_argument);

    let mut types_match = default_type == arg_type;
    if !types_match {
        // For more complex types (such as generic object types), TS won't always create a global
        // shared type object for the type, so fall back to comparing the reference type and the
        // passed type arguments.
        let default_type_resolved = get_type_for_comparison(ctx.checker, default_type);
        let arg_type_resolved = get_type_for_comparison(ctx.checker, arg_type);
        types_match = default_type_resolved.type_value == arg_type_resolved.type_value
            && default_type_resolved.type_arguments == arg_type_resolved.type_arguments;
    }
    if !types_match {
        return;
    }

    // Removing the entire type argument list from a constructor can re-enable inference for all
    // class type parameters used by constructor parameters.
    if last_param_index == 0
        && constructor_arguments_can_infer_type_parameters(ctx.checker, node, parameters)
    {
        return;
    }

    ctx.report_node_with_fixes(type_argument, build_unnecessary_type_parameter_message(), |ctx| {
        let (pos, end) = if last_param_index == 0 {
            let r = tsrs_scanner::get_range_of_token_at_position(ctx.file, arguments.end());
            (arguments.pos() - 1, r.end())
        } else {
            (args[last_param_index - 1].end(), type_argument.end())
        };
        vec![ctx.fix_remove_range(pos, end)]
    });
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _listener: Listener, node: P<Node>) {
        let Some(arguments) = node.type_argument_list() else {
            return;
        };
        if arguments.nodes().is_empty() {
            return;
        }
        let parameters = match node.kind() {
            Kind::ExpressionWithTypeArguments => {
                get_type_parameters_from_type(ctx.checker, node, node.expression().unwrap())
            }
            Kind::TypeReference => get_type_parameters_from_type(
                ctx.checker,
                node,
                node.as_type_reference_node().type_name,
            ),
            _ => get_type_parameters_from_call(ctx.checker, node),
        };
        check_args_and_parameters(ctx, node, arguments, parameters);
    }
}
