// Port of internal/rules/unbound_method/unbound_method.go (and natively_bound_members.go).

use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol};
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

const BASE_MESSAGE: &str =
    "Avoid referencing unbound methods which may cause unintentional scoping of `this`.";

fn build_unbound_message() -> RuleMessage {
    RuleMessage::new("unbound", BASE_MESSAGE)
}
fn build_unbound_without_this_annotation_message() -> RuleMessage {
    RuleMessage::with_help(
        "unboundWithoutThisAnnotation",
        BASE_MESSAGE,
        "If your function does not access `this`, you can annotate it with `this: void`, or consider using an arrow function instead.",
    )
}

fn build_unbound_diagnostic(
    source_file: P<SourceFile>,
    node: P<Node>,
    dangerous_reference: P<Node>,
    message: RuleMessage,
) -> RuleDiagnostic {
    let (pos, end) = utils::trim_node_text_range(source_file, node);
    let (dpos, dend) = utils::trim_node_text_range(source_file, dangerous_reference);
    let mut labeled_ranges = Vec::new();
    if pos != dpos || end != dend {
        labeled_ranges.push(LabeledRange {
            label: "This reference may be unbound and lose `this` context".to_string(),
            pos: dpos,
            end: dend,
        });
    }
    RuleDiagnostic { pos, end, message, labeled_ranges }
}

fn is_node_inside_type_declaration(node: P<Node>) -> bool {
    let mut parent = node.parent();
    while let Some(p) = parent {
        match p.kind() {
            Kind::ClassDeclaration if utils::includes_modifier(p, Kind::DeclareKeyword) => {
                return true;
            }
            Kind::MethodDeclaration if utils::includes_modifier(p, Kind::AbstractKeyword) => {
                return true;
            }
            Kind::FunctionDeclaration if p.body().is_none() => {
                return true;
            }
            Kind::FunctionType | Kind::InterfaceDeclaration | Kind::TypeAliasDeclaration => {
                return true;
            }
            Kind::VariableStatement if utils::includes_modifier(p, Kind::DeclareKeyword) => {
                return true;
            }
            _ => {}
        }
        parent = p.parent();
    }
    false
}

fn is_safe_use(mut node: P<Node>) -> bool {
    let mut parent = node;
    loop {
        node = parent;
        let Some(p) = parent.parent() else {
            break;
        };
        parent = p;
        match parent.kind() {
            Kind::ParenthesizedExpression => continue,
            Kind::IfStatement
            | Kind::ForStatement
            | Kind::PropertyAccessExpression
            | Kind::ElementAccessExpression
            | Kind::SwitchStatement
            | Kind::WhileStatement => return true,
            Kind::PostfixUnaryExpression => {
                let operator = parent.as_postfix_unary_expression().operator;
                return operator == Kind::PlusPlusToken || operator == Kind::MinusMinusToken;
            }
            Kind::PrefixUnaryExpression => {
                let operator = parent.as_prefix_unary_expression().operator;
                return operator == Kind::PlusPlusToken
                    || operator == Kind::MinusMinusToken
                    || operator == Kind::ExclamationToken;
            }
            Kind::CallExpression => return parent.expression() == Some(node),
            Kind::ConditionalExpression => {
                return parent.as_conditional_expression().condition == node;
            }
            Kind::TaggedTemplateExpression => {
                return parent.as_tagged_template_expression().tag == node;
            }
            Kind::DeleteExpression | Kind::TypeOfExpression | Kind::VoidExpression => return true,
            Kind::BinaryExpression => {
                let expr = parent.as_binary_expression();
                let operator_kind = expr.operator_token.kind();
                match operator_kind {
                    Kind::AmpersandAmpersandToken => {
                        if expr.left == node {
                            // this is safe, as && will return the left if and only if it's falsy
                            return true;
                        }
                        // in all other cases, it's likely the logical expression will return the method ref
                        // so make sure the parent is a safe usage
                        continue;
                    }
                    Kind::ExclamationEqualsToken
                    | Kind::ExclamationEqualsEqualsToken
                    | Kind::EqualsEqualsToken
                    | Kind::EqualsEqualsEqualsToken
                    | Kind::InstanceOfKeyword => return true,
                    _ => {}
                }
                if ast::is_logical_binary_operator(operator_kind) {
                    continue;
                }
                if ast::is_assignment_expression(parent, true) {
                    return node == expr.left
                        || (ast::is_access_expression(node)
                            && node.expression().unwrap().kind() == Kind::SuperKeyword
                            && ast::is_access_expression(expr.left)
                            && expr.left.expression().unwrap().kind() == Kind::ThisKeyword);
                }
                return false;
            }
            Kind::NonNullExpression | Kind::AsExpression | Kind::TypeAssertionExpression => {
                continue;
            }
            _ => {}
        }
        return false;
    }
    false
}

fn is_not_imported(symbol: P<Symbol>, current_source_file: P<SourceFile>) -> bool {
    let Some(decl) = symbol.value_declaration() else {
        // working around https://github.com/microsoft/TypeScript/issues/31294
        return false;
    };
    ast::get_source_file_of_node(decl) != Some(current_source_file)
}

const SUPPORTED_GLOBAL_TYPES: &[&str] = &[
    "NumberConstructor",
    "ObjectConstructor",
    "StringConstructor",
    "SymbolConstructor",
    "ArrayConstructor",
    "Array",
    "ProxyConstructor",
    "Console",
    "DateConstructor",
    "Atomics",
    "Math",
    "JSON",
];

/// Returns (dangerous, first_param_is_this).
fn check_method(value_declaration: P<Node>, ignore_static: bool) -> (bool, bool) {
    let params = value_declaration.parameters();
    let first_param_is_this = !params.is_empty()
        && ast::is_parameter_declaration(params[0])
        && params[0].name().is_some_and(|n| ast::is_identifier(n) && n.text() == "this");
    let this_arg_is_void =
        first_param_is_this && params[0].type_node().is_some_and(|t| t.kind() == Kind::VoidKeyword);
    let dangerous = !this_arg_is_void
        && (!ignore_static || !utils::includes_modifier(value_declaration, Kind::StaticKeyword));
    (dangerous, first_param_is_this)
}

/// Returns (dangerous, first_param_is_this).
fn check_if_method(symbol: P<Symbol>, ignore_static: bool) -> (bool, bool) {
    let Some(value_declaration) = symbol.value_declaration() else {
        // working around https://github.com/microsoft/TypeScript/issues/31294
        return (false, false);
    };
    match value_declaration.kind() {
        Kind::PropertyDeclaration => {
            let init = value_declaration.initializer();
            (init.is_some_and(ast::is_function_expression), true)
        }
        Kind::PropertyAssignment => {
            let assignee = value_declaration.initializer().unwrap();
            if !ast::is_function_expression(assignee) {
                return (false, false);
            }
            check_method(assignee, ignore_static)
        }
        Kind::MethodDeclaration | Kind::MethodSignature => {
            check_method(value_declaration, ignore_static)
        }
        _ => (false, false),
    }
}

/// Generated from natively_bound_members.go (nativelyBoundMembers).
fn is_natively_bound_member(object: &str, property: &str) -> bool {
    match object {
        "Number" => matches!(
            property,
            "isFinite" | "isInteger" | "isNaN" | "isSafeInteger" | "parseFloat" | "parseInt"
        ),
        "Object" => matches!(
            property,
            "assign"
                | "getOwnPropertyDescriptor"
                | "getOwnPropertyDescriptors"
                | "getOwnPropertyNames"
                | "getOwnPropertySymbols"
                | "hasOwn"
                | "is"
                | "preventExtensions"
                | "seal"
                | "create"
                | "defineProperties"
                | "defineProperty"
                | "freeze"
                | "getPrototypeOf"
                | "setPrototypeOf"
                | "isExtensible"
                | "isFrozen"
                | "isSealed"
                | "keys"
                | "entries"
                | "fromEntries"
                | "values"
                | "groupBy"
        ),
        "String" => matches!(property, "fromCharCode" | "fromCodePoint" | "raw"),
        "Symbol" => matches!(property, "for" | "keyFor"),
        "Array" => matches!(property, "isArray" | "from" | "of" | "fromAsync"),
        "Proxy" => matches!(property, "revocable"),
        "Date" => matches!(property, "now" | "parse" | "UTC"),
        "Atomics" => matches!(
            property,
            "load"
                | "store"
                | "add"
                | "sub"
                | "and"
                | "or"
                | "xor"
                | "exchange"
                | "compareExchange"
                | "isLockFree"
                | "wait"
                | "waitAsync"
                | "notify"
        ),
        "Reflect" => matches!(
            property,
            "defineProperty"
                | "deleteProperty"
                | "apply"
                | "construct"
                | "get"
                | "getOwnPropertyDescriptor"
                | "getPrototypeOf"
                | "has"
                | "isExtensible"
                | "ownKeys"
                | "preventExtensions"
                | "set"
                | "setPrototypeOf"
        ),
        "console" => matches!(
            property,
            "log"
                | "info"
                | "debug"
                | "warn"
                | "error"
                | "dir"
                | "time"
                | "timeEnd"
                | "timeLog"
                | "trace"
                | "assert"
                | "clear"
                | "count"
                | "countReset"
                | "group"
                | "groupEnd"
                | "table"
                | "dirxml"
                | "groupCollapsed"
                | "Console"
                | "profile"
                | "profileEnd"
                | "timeStamp"
                | "context"
                | "createTask"
        ),
        "Math" => matches!(
            property,
            "abs"
                | "acos"
                | "acosh"
                | "asin"
                | "asinh"
                | "atan"
                | "atanh"
                | "atan2"
                | "ceil"
                | "cbrt"
                | "expm1"
                | "clz32"
                | "cos"
                | "cosh"
                | "exp"
                | "floor"
                | "fround"
                | "hypot"
                | "imul"
                | "log"
                | "log1p"
                | "log2"
                | "log10"
                | "max"
                | "min"
                | "pow"
                | "random"
                | "round"
                | "sign"
                | "sin"
                | "sinh"
                | "sqrt"
                | "tan"
                | "tanh"
                | "trunc"
        ),
        "JSON" => matches!(property, "parse" | "stringify" | "rawJSON" | "isRawJSON"),
        "Intl" => matches!(
            property,
            "getCanonicalLocales"
                | "supportedValuesOf"
                | "DateTimeFormat"
                | "NumberFormat"
                | "Collator"
                | "PluralRules"
                | "RelativeTimeFormat"
                | "ListFormat"
                | "Locale"
                | "DisplayNames"
                | "Segmenter"
        ),
        _ => false,
    }
}

pub struct UnboundMethod {
    ignore_static: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(UnboundMethod { ignore_static: opt_bool(&m, "ignoreStatic", false) }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::PropertyAccessExpression),
    Listener::AllowPattern(Kind::ObjectLiteralExpression),
    Listener::Enter(Kind::ObjectBindingPattern),
];

impl Rule for UnboundMethod {
    fn name(&self) -> &'static str {
        "unbound-method"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static UnboundMethod,
}

impl Visitor {
    fn is_natively_bound(&self, ctx: &mut Ctx, object: P<Node>, property: P<Node>) -> bool {
        // We can't rely entirely on the type-level checks made at the end of this
        // function, because sometimes type declarations don't come from the
        // default library, but come from, for example, "@types/node". And we can't
        // tell if a method is unbound just by looking at its signature declared in
        // the interface.
        if ast::is_identifier(object)
            && ast::is_identifier(property)
            && is_natively_bound_member(object.text(), property.text())
        {
            let object_symbol = ctx.checker.get_symbol_at_location_exported(object);
            if let Some(object_symbol) = object_symbol {
                if is_not_imported(object_symbol, ctx.file) {
                    return true;
                }
            }
        }

        // if `${object.name}.${property.name}` doesn't match any of
        // the nativelyBoundMembers, then we fallback to type-level checks
        let object_type = ctx.checker.get_type_at_location(object);
        if !utils::is_builtin_symbol_like(
            ctx.program,
            ctx.checker,
            object_type,
            SUPPORTED_GLOBAL_TYPES,
        ) {
            return false;
        }
        let property_type = ctx.checker.get_type_at_location(property);
        utils::is_any_builtin_symbol_like(ctx.program, ctx.checker, property_type)
    }

    fn check_if_method_and_report(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        dangerous_reference: P<Node>,
        symbol: Option<P<Symbol>>,
    ) -> bool {
        let Some(symbol) = symbol else {
            return false;
        };
        let (dangerous, first_param_is_this) = check_if_method(symbol, self.o.ignore_static);
        if !dangerous {
            return false;
        }
        let message = if first_param_is_this {
            build_unbound_message()
        } else {
            build_unbound_without_this_annotation_message()
        };
        let d = build_unbound_diagnostic(ctx.file, node, dangerous_reference, message);
        ctx.report_diagnostic(d);
        true
    }

    fn check_binding_property(
        &self,
        ctx: &mut Ctx,
        pattern_node: P<Node>,
        init_node: Option<P<Node>>,
        property_name: P<Node>,
        parent_is_assignment_pattern_like: bool,
    ) {
        // Skip computed property names as they cannot be statically analyzed
        if ast::is_computed_property_name(property_name) {
            return;
        }
        if let Some(init_node) = init_node {
            if !self.is_natively_bound(ctx, init_node, property_name) {
                let t = ctx.checker.get_type_at_location(init_node);
                let symbol = ctx.checker.get_property_of_type(t, property_name.text());
                if self.check_if_method_and_report(ctx, property_name, property_name, symbol) {
                    return;
                }
                // In assignment patterns, we should also check the type of
                // Foo's nativelyBound method because initNode might be used as
                // default value:
                //   function ({ nativelyBound }: Foo = NativeObject) {}
            } else if !parent_is_assignment_pattern_like {
                return;
            }
        }
        let pattern_type = ctx.checker.get_type_at_location(pattern_node);
        utils::type_recurser(pattern_type, &mut |t| {
            let symbol = ctx.checker.get_property_of_type(t, property_name.text());
            self.check_if_method_and_report(ctx, property_name, property_name, symbol)
        });
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::Enter(Kind::PropertyAccessExpression) => {
                let name = node.name().unwrap();
                if is_safe_use(node)
                    || self.is_natively_bound(ctx, node.expression().unwrap(), name)
                {
                    return;
                }
                let symbol = ctx.checker.get_symbol_at_location_exported(node);
                self.check_if_method_and_report(ctx, node, name, symbol);
            }
            Listener::AllowPattern(Kind::ObjectLiteralExpression) => {
                let parent = node.parent().unwrap();
                if !ast::is_assignment_expression(parent, true) {
                    return;
                }
                let init_node = parent.as_binary_expression().right.get();
                for &property in node.properties() {
                    if !ast::is_property_assignment(property)
                        && !ast::is_shorthand_property_assignment(property)
                    {
                        continue;
                    }
                    self.check_binding_property(
                        ctx,
                        node,
                        Some(init_node),
                        property.name().unwrap(),
                        true,
                    );
                }
            }
            Listener::Enter(Kind::ObjectBindingPattern) => {
                if is_node_inside_type_declaration(node) {
                    return;
                }
                let parent = node.parent().unwrap();
                let parent_is_assignment_pattern_like =
                    ast::is_binding_element(parent) || ast::is_parameter_declaration(parent);
                let init_node =
                    if ast::is_variable_declaration(parent) || parent_is_assignment_pattern_like {
                        parent.initializer()
                    } else {
                        None
                    };
                for &property in node.elements() {
                    if !ast::is_binding_element(property) {
                        continue;
                    }
                    let binding_elem = property.as_binding_element();
                    let property_name =
                        binding_elem.property_name().or_else(|| binding_elem.name()).unwrap();
                    if binding_elem.dot_dot_dot_token().is_some()
                        || !ast::is_identifier(property_name)
                    {
                        continue;
                    }
                    self.check_binding_property(
                        ctx,
                        node,
                        init_node,
                        property_name,
                        parent_is_assignment_pattern_like,
                    );
                }
            }
            _ => {}
        }
    }
}
