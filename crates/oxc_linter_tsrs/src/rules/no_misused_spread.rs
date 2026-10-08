// Port of internal/rules/no_misused_spread/no_misused_spread.go.

use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, ObjectFlags, Type, TypeFlags};
use tsrs_compiler::Program;
use tsrs_core::P;

use crate::rule::{
    Ctx, Listener, Rule, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor, options_object,
};
use crate::utils::{self, TypeOrValueSpecifier};

fn build_add_await_message() -> RuleMessage {
    RuleMessage::new("addAwait", "Add await operator.")
}
fn build_no_array_spread_in_object_message() -> RuleMessage {
    RuleMessage::new(
        "noArraySpreadInObject",
        "Using the spread operator on an array in an object will result in a list of indices.",
    )
}
fn build_no_class_declaration_spread_in_object_message() -> RuleMessage {
    RuleMessage::new(
        "noClassDeclarationSpreadInObject",
        "Using the spread operator on class declarations will spread only their static properties, and will lose their class prototype.",
    )
}
fn build_no_class_instance_spread_in_object_message() -> RuleMessage {
    RuleMessage::new(
        "noClassInstanceSpreadInObject",
        "Using the spread operator on class instances will lose their class prototype.",
    )
}
fn build_no_function_spread_in_object_message() -> RuleMessage {
    RuleMessage::with_help(
        "noFunctionSpreadInObject",
        "Using the spread operator on a function without additional properties can cause unexpected behavior.",
        "Did you forget to call the function?",
    )
}
fn build_no_iterable_spread_in_object_message() -> RuleMessage {
    RuleMessage::new(
        "noIterableSpreadInObject",
        "Using the spread operator on an Iterable in an object can cause unexpected behavior.",
    )
}
fn build_no_map_spread_in_object_message() -> RuleMessage {
    RuleMessage::with_help(
        "noMapSpreadInObject",
        "Using the spread operator on a Map in an object will result in an empty object.",
        "Did you mean to use `Object.fromEntries(map)` instead?",
    )
}
fn build_no_promise_spread_in_object_message() -> RuleMessage {
    RuleMessage::with_help(
        "noPromiseSpreadInObject",
        "Using the spread operator on Promise in an object can cause unexpected behavior.",
        "Did you forget to await the promise before spreading it?",
    )
}
fn build_no_string_spread_message() -> RuleMessage {
    RuleMessage::with_help(
        "noStringSpread",
        "Using the spread operator on a string can mishandle special characters, because it produces Unicode code points, which will break complex characters (like emojis) into multiple parts.",
        "Consider using `Intl.Segmenter` for locale-aware string decomposition. Otherwise, if you don't need to preserve emojis or other non-ASCII characters, disable this lint rule on this line or configure the 'allow' rule option.",
    )
}
fn build_replace_map_spread_in_object_message() -> RuleMessage {
    RuleMessage::new(
        "replaceMapSpreadInObject",
        "Replace map spread in object with `Object.fromEntries()`",
    )
}

fn is_string(t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| utils::is_type_flag_set(t, TypeFlags::StringLike))
}

fn is_promise(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| utils::is_promise_like(program, c, t))
}

fn is_function_without_props(c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        !utils::get_call_signatures(c, t).is_empty() && c.get_properties_of_type(t).is_empty()
    })
}

fn is_map(program: &Program, c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        utils::is_builtin_symbol_like(program, c, t, &["Map", "ReadonlyMap", "WeakMap"])
    })
}

fn is_array(c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| c.is_array_or_tuple_type(t))
}

fn is_iterable(c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        utils::get_well_known_symbol_property_of_type(t, "iterator", c).is_some()
    })
}

fn is_class_instance(c: &mut Checker, t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        // If the type itself has a construct signature, it's a class(-like)
        if !utils::get_construct_signatures(c, t).is_empty() {
            return false;
        }
        let Some(symbol) = t.symbol() else {
            return false;
        };
        // If the type's symbol has a construct signature, the type is an instance
        for &decl in symbol.declarations() {
            let Some(decl_type) = c.get_type_of_symbol_at_location(symbol, Some(decl)) else {
                continue;
            };
            if !utils::get_construct_signatures(c, decl_type).is_empty() {
                return true;
            }
        }
        false
    })
}

fn is_class_declaration(t: P<Type>) -> bool {
    utils::type_recurser(t, &mut |t| {
        if utils::is_object_type(t)
            && t.object_flags().intersects(ObjectFlags::InstantiationExpressionType)
        {
            return true;
        }
        t.symbol()
            .and_then(|s| s.value_declaration())
            .is_some_and(|d| ast::is_class_expression(d) || ast::is_class_declaration(d))
    })
}

pub struct NoMisusedSpread {
    allow: Vec<TypeOrValueSpecifier>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(NoMisusedSpread {
        allow: utils::unmarshal_type_or_value_specifiers(m.get("allow"))?,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::NotAllowPattern(Kind::ArrayLiteralExpression),
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::JsxSpreadAttribute),
    Listener::NotAllowPattern(Kind::ObjectLiteralExpression),
];

impl Rule for NoMisusedSpread {
    fn name(&self) -> &'static str {
        "no-misused-spread"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoMisusedSpread,
}

fn insert_await_fix(ctx: &Ctx, node: P<Node>) -> Vec<RuleFix> {
    if utils::is_higher_precedence_than_await(node) {
        return vec![ctx.fix_insert_before(node, "await ")];
    }
    vec![ctx.fix_insert_before(node, "await ("), ctx.fix_insert_after(node, ")")]
}

fn get_map_spread_suggestions(
    ctx: &mut Ctx,
    node: P<Node>,
    argument: P<Node>,
    t: P<Type>,
) -> Vec<RuleSuggestion> {
    // TODO(port): do we need this loop?
    for t in utils::union_type_parts(t) {
        if !is_map(ctx.program, ctx.checker, t) {
            return Vec::new();
        }
    }
    let parent = node.parent().unwrap();
    if ast::is_object_literal_expression(parent) {
        let properties = parent.as_object_literal_expression().properties;
        if properties.nodes().len() == 1 {
            let open = tsrs_scanner::get_range_of_token_at_position(ctx.file, parent.pos());
            let dots = tsrs_scanner::get_range_of_token_at_position(ctx.file, node.pos());
            return vec![RuleSuggestion {
                message: build_replace_map_spread_in_object_message(),
                fixes: vec![
                    ctx.fix_remove_range(open.pos(), open.end()), // {
                    ctx.fix_replace_range(dots.pos(), dots.end(), "Object.fromEntries("), // ...
                    ctx.fix_replace_range(argument.end(), properties.end(), ")"),
                    ctx.fix_remove_range(parent.end() - 1, parent.end()), // }
                ],
            }];
        }
    }
    vec![RuleSuggestion {
        message: build_replace_map_spread_in_object_message(),
        fixes: vec![
            ctx.fix_insert_before(argument, "Object.fromEntries("),
            ctx.fix_insert_after(argument, ")"),
        ],
    }]
}

impl Visitor {
    fn check_array_or_call_spread(&self, ctx: &mut Ctx, node: P<Node>) {
        let expression = node.expression().unwrap();
        let t = utils::get_constrained_type_at_location(ctx.checker, expression);
        if !utils::type_matches_some_specifier(t, &self.rule.allow, ctx.program) && is_string(t) {
            ctx.report_node(node, build_no_string_spread_message());
        }
    }

    fn check_object_spread(&self, ctx: &mut Ctx, node: P<Node>, argument: P<Node>) {
        let t = utils::get_constrained_type_at_location(ctx.checker, argument);

        if utils::type_matches_some_specifier(t, &self.rule.allow, ctx.program) {
            return;
        }

        if is_promise(ctx.program, ctx.checker, t) {
            ctx.report_node_with_suggestions(
                node,
                build_no_promise_spread_in_object_message(),
                |ctx| {
                    vec![RuleSuggestion {
                        message: build_add_await_message(),
                        fixes: insert_await_fix(ctx, ast::skip_parentheses(argument)),
                    }]
                },
            );
            return;
        }

        if is_function_without_props(ctx.checker, t) {
            ctx.report_node(node, build_no_function_spread_in_object_message());
            return;
        }

        if is_map(ctx.program, ctx.checker, t) {
            ctx.report_node_with_suggestions(
                node,
                build_no_map_spread_in_object_message(),
                |ctx| get_map_spread_suggestions(ctx, node, argument, t),
            );
            return;
        }

        if is_array(ctx.checker, t) {
            ctx.report_node(node, build_no_array_spread_in_object_message());
            return;
        }

        // Don't report when the type is string, since TS will flag it already
        if is_iterable(ctx.checker, t) && !is_string(t) {
            ctx.report_node(node, build_no_iterable_spread_in_object_message());
            return;
        }

        if is_class_instance(ctx.checker, t) {
            ctx.report_node(node, build_no_class_instance_spread_in_object_message());
            return;
        }

        if is_class_declaration(t) {
            ctx.report_node(node, build_no_class_declaration_spread_in_object_message());
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::ArrayLiteralExpression => {
                for &element in node.as_array_literal_expression().elements.nodes() {
                    if ast::is_spread_element(element) {
                        self.check_array_or_call_spread(ctx, element);
                    }
                }
            }
            Kind::CallExpression => {
                for &element in node.arguments() {
                    if ast::is_spread_element(element) {
                        self.check_array_or_call_spread(ctx, element);
                    }
                }
            }
            Kind::JsxSpreadAttribute => {
                let expression = node.expression().unwrap();
                self.check_object_spread(ctx, node, expression);
            }
            Kind::ObjectLiteralExpression => {
                for &element in node.as_object_literal_expression().properties.nodes() {
                    if ast::is_spread_assignment(element) {
                        let expression = element.expression().unwrap();
                        self.check_object_spread(ctx, element, expression);
                    }
                }
            }
            _ => {}
        }
    }
}
