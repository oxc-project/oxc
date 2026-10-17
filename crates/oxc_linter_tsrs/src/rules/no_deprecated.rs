// Port of internal/rules/no_deprecated/no_deprecated.go.

use tsrs_ast::{self as ast, Kind, Node, Symbol, SymbolFlags};
use tsrs_checker::{CheckMode, ContextFlags, LiteralValue, Type, TypeFlags};
use tsrs_core::P;

use crate::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor, options_object};
use crate::utils::{self, TypeOrValueSpecifier};

fn build_deprecated_message(name: &str) -> RuleMessage {
    RuleMessage::new("deprecated", format!("`{name}` is deprecated."))
}

fn build_deprecated_with_reason_message(name: &str, reason: &str) -> RuleMessage {
    RuleMessage::new("deprecatedWithReason", format!("`{name}` is deprecated. {reason}"))
}

fn format_property_name_for_report(name: &str) -> String {
    if name.is_empty() {
        return "\"\"".to_string();
    }
    name.to_string()
}

fn is_node_callee_of_parent(node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        Kind::NewExpression => parent.as_new_expression().expression == node,
        Kind::CallExpression => parent.as_call_expression().expression == node,
        Kind::TaggedTemplateExpression => parent.as_tagged_template_expression().tag == node,
        Kind::JsxOpeningElement => parent.as_jsx_opening_element().tag_name == node,
        _ => false,
    }
}

fn get_call_like_node(node: P<Node>) -> Option<P<Node>> {
    let mut callee = node;
    // Walk up the tree while we're the property of a PropertyAccessExpression
    while let Some(parent) = callee.parent() {
        if parent.kind() != Kind::PropertyAccessExpression {
            break;
        }
        if parent.as_property_access_expression().name != callee {
            break;
        }
        callee = parent;
    }
    if is_node_callee_of_parent(callee) { Some(callee) } else { None }
}

fn get_reported_node_name(node: P<Node>) -> String {
    if node.kind() == Kind::SuperKeyword {
        return "super".to_string();
    }
    if node.kind() == Kind::PrivateIdentifier {
        return format!("#{}", node.text());
    }
    node.text().to_string()
}

/// Go LiteralType.String() / string value of a literal type's value.
fn literal_value_name(t: P<Type>) -> Option<String> {
    match t.as_literal_type().value()? {
        LiteralValue::String(s) => Some(s.to_string()),
        v => Some(tsrs_checker::value_to_string(v)),
    }
}

pub struct NoDeprecated {
    allow: Vec<TypeOrValueSpecifier>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    let allow = utils::unmarshal_type_or_value_specifiers(m.get("allow"))
        .map_err(|e| format!("no-deprecated: failed to unmarshal options: {e}"))?;
    Ok(Box::new(NoDeprecated { allow }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::Identifier),
    Listener::Enter(Kind::ElementAccessExpression),
    Listener::Enter(Kind::PropertyAssignment),
    Listener::Enter(Kind::ShorthandPropertyAssignment),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Enter(Kind::PrivateIdentifier),
    Listener::Enter(Kind::SuperKeyword),
];

impl Rule for NoDeprecated {
    fn name(&self) -> &'static str {
        "no-deprecated"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { opts: self })
    }
}

struct Visitor {
    opts: &'static NoDeprecated,
}

fn has_deprecated_tag(jsdocs: &[P<Node>]) -> bool {
    jsdocs.iter().any(|jsdoc| {
        jsdoc
            .as_jsdoc()
            .tags
            .is_some_and(|tags| tags.nodes().iter().any(|&t| ast::is_jsdoc_deprecated_tag(t)))
    })
}

/// Extracts the deprecation reason from a JSDoc deprecated tag.
fn get_js_doc_deprecation_from_node(node: Option<P<Node>>) -> String {
    let Some(node) = node else {
        return String::new();
    };
    for jsdoc in node.jsdoc(None) {
        let Some(tags) = jsdoc.as_jsdoc().tags else {
            continue;
        };
        for &tag in tags.nodes() {
            if ast::is_jsdoc_deprecated_tag(tag) {
                if let Some(comment) = tag.as_jsdoc_deprecated_tag().comment() {
                    if !comment.nodes().is_empty() {
                        let mut text = String::new();
                        for comment_node in comment.nodes() {
                            text.push_str(comment_node.text());
                        }
                        return text;
                    }
                }
                return String::new();
            }
        }
    }
    String::new()
}

fn get_js_doc_deprecation(ctx: &mut Ctx, symbol: Option<P<Symbol>>) -> (bool, String) {
    let Some(symbol) = symbol else {
        return (false, String::new());
    };
    for &decl in symbol.declarations() {
        if ctx.checker.is_deprecated_declaration(decl) {
            return (true, get_js_doc_deprecation_from_node(Some(decl)));
        }
    }
    (false, String::new())
}

fn search_for_deprecation_in_aliases_chain(
    ctx: &mut Ctx,
    symbol: Option<P<Symbol>>,
    check_deprecations_of_aliased_symbol: bool,
) -> (bool, String) {
    let Some(mut symbol) = symbol else {
        return (false, String::new());
    };
    if !utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
        if check_deprecations_of_aliased_symbol {
            return get_js_doc_deprecation(ctx, Some(symbol));
        }
        return (false, String::new());
    }
    let target_symbol = ctx.checker.get_aliased_symbol(symbol);
    while utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
        let (is_deprecated, reason) = get_js_doc_deprecation(ctx, Some(symbol));
        if is_deprecated {
            return (true, reason);
        }
        if ctx.checker.get_declaration_of_alias_symbol(symbol).is_none() {
            break;
        }
        let Some(immediate) = ctx.checker.get_immediate_aliased_symbol(symbol) else {
            break;
        };
        symbol = immediate;
        if check_deprecations_of_aliased_symbol && symbol == target_symbol {
            return get_js_doc_deprecation(ctx, Some(symbol));
        }
    }
    (false, String::new())
}

/// Deprecation for call-like expressions (function calls, new expressions, etc.)
fn get_call_like_deprecation(ctx: &mut Ctx, node: P<Node>) -> (bool, String) {
    let Some(ts_node) = node.parent() else {
        return (false, String::new());
    };
    let signature = ctx.checker.get_resolved_signature(ts_node, None, CheckMode::Normal);
    let signature_decl = signature.declaration();
    if let Some(decl) = signature_decl {
        if ctx.checker.is_deprecated_declaration(decl) {
            return (true, get_js_doc_deprecation_from_node(Some(decl)));
        }
    }
    let Some(symbol) = ctx.checker.get_symbol_at_location_exported(node) else {
        return (false, String::new());
    };
    let mut aliased_symbol = Some(symbol);
    if utils::is_symbol_flag_set(Some(symbol), SymbolFlags::Alias) {
        aliased_symbol = Some(ctx.checker.get_aliased_symbol(symbol));
    }
    let symbol_declaration_kind = aliased_symbol
        .and_then(|s| s.declarations().first().map(|d| d.kind()))
        .unwrap_or(Kind::Unknown);
    if symbol_declaration_kind != Kind::MethodDeclaration
        && symbol_declaration_kind != Kind::FunctionDeclaration
        && symbol_declaration_kind != Kind::MethodSignature
    {
        let (is_deprecated, reason) =
            search_for_deprecation_in_aliases_chain(ctx, Some(symbol), true);
        if is_deprecated {
            return (true, reason);
        }
    } else {
        let (is_deprecated, reason) =
            search_for_deprecation_in_aliases_chain(ctx, Some(symbol), false);
        if is_deprecated {
            return (true, reason);
        }
        if signature_decl.is_none() {
            let (is_deprecated, reason) = get_js_doc_deprecation(ctx, aliased_symbol);
            if is_deprecated {
                return (true, reason);
            }
        }
    }
    (false, String::new())
}

fn get_jsx_attribute_deprecation(
    ctx: &mut Ctx,
    element_node: P<Node>,
    property_name: &str,
) -> (bool, String) {
    let tag_name = match element_node.kind() {
        Kind::JsxSelfClosingElement => element_node.as_jsx_self_closing_element().tag_name,
        Kind::JsxOpeningElement => element_node.as_jsx_opening_element().tag_name,
        _ => return (false, String::new()),
    };
    let Some(contextual_type) = ctx.checker.get_contextual_type(tag_name, ContextFlags::None)
    else {
        return (false, String::new());
    };
    let symbol = ctx.checker.get_property_of_type(contextual_type, property_name);
    get_js_doc_deprecation(ctx, symbol)
}

fn get_object_literal_property_name(ctx: &mut Ctx, name: Option<P<Node>>) -> Option<String> {
    let mut name = name?;
    if ast::is_computed_property_name(name) {
        name = name.as_computed_property_name().expression;
        match name.kind() {
            Kind::StringLiteral | Kind::NumericLiteral | Kind::BigIntLiteral => {
                return Some(name.text().to_string());
            }
            _ => {}
        }
        let t = ctx.checker.get_type_at_location(name);
        if t.is_string_literal() || t.is_number_literal() || t.is_big_int_literal() {
            if let Some(v) = literal_value_name(t) {
                return Some(v);
            }
        }
        return None;
    }
    match name.kind() {
        Kind::Identifier
        | Kind::PrivateIdentifier
        | Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral => Some(name.text().to_string()),
        _ => None,
    }
}

struct ObjectLiteralPropertyDeprecation {
    property_name: String,
    contextual_type: Option<P<Type>>,
    property: Option<P<Symbol>>,
    is_deprecated: bool,
    reason: String,
}

fn get_contextual_object_literal_property_deprecation(
    ctx: &mut Ctx,
    property_node: P<Node>,
    name: Option<P<Node>>,
) -> ObjectLiteralPropertyDeprecation {
    let mut res = ObjectLiteralPropertyDeprecation {
        property_name: String::new(),
        contextual_type: None,
        property: None,
        is_deprecated: false,
        reason: String::new(),
    };
    let Some(parent) = property_node.parent() else {
        return res;
    };
    if !ast::is_object_literal_expression(parent) {
        return res;
    }
    let Some(property_name) = get_object_literal_property_name(ctx, name) else {
        return res;
    };
    let Some(contextual_type) =
        ctx.checker.get_apparent_type_of_contextual_type(parent, ContextFlags::None)
    else {
        return res;
    };
    let property = ctx.checker.get_property_of_type(contextual_type, &property_name);
    res.property_name = property_name;
    res.contextual_type = Some(contextual_type);
    res.property = property;
    match property {
        Some(p) if ctx.checker.is_deprecated_symbol(p) => {}
        _ => return res,
    }
    let (is_deprecated, reason) = get_js_doc_deprecation(ctx, property);
    res.is_deprecated = is_deprecated;
    res.reason = reason;
    res
}

impl Visitor {
    fn check_object_literal_property_deprecation(
        &self,
        ctx: &mut Ctx,
        property_node: P<Node>,
        name: Option<P<Node>>,
    ) {
        let d = get_contextual_object_literal_property_deprecation(ctx, property_node, name);
        if !d.is_deprecated {
            return;
        }
        let Some(name) = name else { return };
        let allow = &self.opts.allow;
        if !allow.is_empty() {
            let name_type = ctx.checker.get_type_at_location(name);
            let property_name_allowed = allow.iter().any(|specifier| {
                utils::symbol_matches_specifier_name_and_source(
                    d.property,
                    &d.property_name,
                    specifier,
                    ctx.program,
                )
            });
            if d.contextual_type
                .is_some_and(|t| utils::type_matches_some_specifier(t, allow, ctx.program))
                || utils::type_matches_some_specifier(name_type, allow, ctx.program)
                || property_name_allowed
            {
                return;
            }
        }
        let reported = format_property_name_for_report(&d.property_name);
        if d.reason.is_empty() {
            ctx.report_node(name, build_deprecated_message(&reported));
        } else {
            ctx.report_node(name, build_deprecated_with_reason_message(&reported, d.reason.trim()));
        }
    }

    fn check_identifier(&self, ctx: &mut Ctx, node: P<Node>) {
        if is_declaration(node) || is_inside_import(node) {
            return;
        }
        let (is_deprecated, deprecation_reason) = get_deprecation_reason(ctx, node);
        if !is_deprecated {
            return;
        }
        let allow = &self.opts.allow;
        if !allow.is_empty() {
            let ty = ctx.checker.get_type_at_location(node);
            if utils::type_matches_some_specifier(ty, allow, ctx.program)
                || utils::value_matches_some_specifier(node, allow, ctx.program, Some(ty))
            {
                return;
            }
        }
        let name = get_reported_node_name(node);
        if deprecation_reason.is_empty() {
            ctx.report_node(node, build_deprecated_message(&name));
        } else {
            ctx.report_node(
                node,
                build_deprecated_with_reason_message(&name, deprecation_reason.trim()),
            );
        }
    }

    /// Element access expressions with literal keys (e.g., a['b'])
    fn check_element_access_expression(&self, ctx: &mut Ctx, node: P<Node>) {
        let eae = node.as_element_access_expression();
        let argument_expression = eae.argument_expression;
        let property_type = ctx.checker.get_type_at_location(argument_expression);
        let is_string_lit = property_type.is_string_literal();
        let is_number_lit = utils::is_type_flag_set(property_type, TypeFlags::NumberLiteral);
        let is_big_int_lit = utils::is_type_flag_set(property_type, TypeFlags::BigIntLiteral);
        if !is_string_lit && !is_number_lit && !is_big_int_lit {
            return;
        }
        let object_type = ctx.checker.get_type_at_location(eae.expression);
        let Some(property_name) = literal_value_name(property_type) else {
            return;
        };
        let property = ctx.checker.get_property_of_type(object_type, &property_name);
        let (is_deprecated, reason) = get_js_doc_deprecation(ctx, property);
        if !is_deprecated {
            return;
        }
        if utils::type_matches_some_specifier(object_type, &self.opts.allow, ctx.program) {
            return;
        }
        if reason.is_empty() {
            ctx.report_node(argument_expression, build_deprecated_message(&property_name));
        } else {
            ctx.report_node(
                argument_expression,
                build_deprecated_with_reason_message(&property_name, reason.trim()),
            );
        }
    }
}

/// Walks up the tree to get the source type of a binding pattern.
fn get_binding_pattern_source_type(ctx: &mut Ctx, binding_pattern: P<Node>) -> Option<P<Type>> {
    let mut current = Some(binding_pattern);
    while let Some(cur) = current {
        match cur.kind() {
            Kind::VariableDeclaration => {
                return cur
                    .as_variable_declaration()
                    .initializer()
                    .map(|init| ctx.checker.get_type_at_location(init));
            }
            Kind::Parameter => return Some(ctx.checker.get_type_at_location(cur)),
            Kind::BindingElement => {
                // For nested destructuring like { bar: { anchor } }
                let binding_elem = cur.as_binding_element();
                if let Some(parent_pattern) = cur.parent() {
                    let parent_source_type = get_binding_pattern_source_type(ctx, parent_pattern)?;
                    let mut property_name = "";
                    if let Some(pn) = binding_elem.property_name() {
                        if pn.kind() == Kind::ComputedPropertyName {
                            return None;
                        }
                        property_name = pn.text();
                    } else if let Some(name) = binding_elem.name {
                        if name.kind() == Kind::ObjectBindingPattern
                            || name.kind() == Kind::ArrayBindingPattern
                        {
                            return None;
                        }
                        property_name = name.text();
                    }
                    if property_name.is_empty() {
                        return None;
                    }
                    if let Some(property) =
                        ctx.checker.get_property_of_type(parent_source_type, property_name)
                    {
                        return ctx.checker.get_type_of_symbol_at_location(property, Some(cur));
                    }
                }
                return None;
            }
            Kind::ArrayBindingPattern => {
                let parent_source_type = get_binding_pattern_source_type(ctx, cur.parent()?)?;
                if let Some(property) = ctx.checker.get_property_of_type(parent_source_type, "0") {
                    return ctx.checker.get_type_of_symbol_at_location(property, Some(cur));
                }
                return Some(parent_source_type);
            }
            Kind::ObjectBindingPattern => {
                current = cur.parent();
                continue;
            }
            _ => {}
        }
        current = cur.parent();
    }
    None
}

fn check_property_symbols(
    ctx: &mut Ctx,
    property: Option<P<Symbol>>,
    property_symbol: Option<P<Symbol>>,
) -> Option<(bool, String)> {
    // Check alias chain first
    let (is_deprecated, reason) =
        search_for_deprecation_in_aliases_chain(ctx, property_symbol, true);
    if is_deprecated {
        return Some((true, reason));
    }
    // Check the property on the type
    let (is_deprecated, reason) = get_js_doc_deprecation(ctx, property);
    if is_deprecated {
        return Some((true, reason));
    }
    // Check the property symbol itself
    let (is_deprecated, reason) = get_js_doc_deprecation(ctx, property_symbol);
    if is_deprecated {
        return Some((true, reason));
    }
    // Check shorthand assignment value symbol
    if let Some(value_declaration) = property_symbol.and_then(|s| s.value_declaration()) {
        let value_symbol =
            ctx.checker.get_shorthand_assignment_value_symbol(Some(value_declaration));
        let (is_deprecated, reason) = get_js_doc_deprecation(ctx, value_symbol);
        if is_deprecated {
            return Some((true, reason));
        }
    }
    None
}

fn get_deprecation_reason(ctx: &mut Ctx, node: P<Node>) -> (bool, String) {
    if let Some(call_like_node) = get_call_like_node(node) {
        return get_call_like_deprecation(ctx, call_like_node);
    }
    if let Some(parent) = node.parent() {
        if parent.kind() == Kind::JsxAttribute && node.kind() != Kind::SuperKeyword {
            if let Some(element) = parent.parent().and_then(|p| p.parent()) {
                return get_jsx_attribute_deprecation(ctx, element, node.text());
            }
        }
    }
    if let Some(parent) = node.parent().filter(|_| node.kind() != Kind::SuperKeyword) {
        // BindingElement in object destructuring: const { b } = a
        if parent.kind() == Kind::BindingElement {
            let binding_elem = parent.as_binding_element();
            if let Some(binding_pattern) = parent.parent().filter(|p| {
                p.kind() == Kind::ObjectBindingPattern || p.kind() == Kind::ArrayBindingPattern
            }) {
                let source_type = get_binding_pattern_source_type(ctx, binding_pattern)
                    .unwrap_or_else(|| ctx.checker.get_type_at_location(binding_pattern));
                let mut property_name = node.text();
                if let Some(pn) = binding_elem.property_name() {
                    if pn.kind() == Kind::ComputedPropertyName {
                        return (false, String::new());
                    }
                    property_name = pn.text();
                }
                let property = ctx.checker.get_property_of_type(source_type, property_name);
                let property_symbol = ctx.checker.get_symbol_at_location_exported(node);
                if let Some(r) = check_property_symbols(ctx, property, property_symbol) {
                    return r;
                }
            }
        }
        // Shorthand property assignments in object literals
        if parent.kind() == Kind::ShorthandPropertyAssignment {
            if let Some(object) = parent.parent() {
                let parent_type = ctx.checker.get_type_at_location(object);
                let property_symbol = ctx.checker.get_symbol_at_location_exported(node);
                let property = ctx.checker.get_property_of_type(parent_type, node.text());
                if let Some(r) = check_property_symbols(ctx, property, property_symbol) {
                    return r;
                }
            }
        }
    }
    let symbol = ctx.checker.get_symbol_at_location_exported(node);
    search_for_deprecation_in_aliases_chain(ctx, symbol, true)
}

/// Whether a node is a declaration (should not report on declarations).
fn is_declaration(node: P<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        Kind::BindingElement => {
            // Array binding elements only declare locals. Object binding patterns are
            // handled separately because they also represent property reads.
            parent.parent().is_some_and(|p| p.kind() == Kind::ArrayBindingPattern)
                && parent.name() == Some(node)
        }
        Kind::ClassExpression
        | Kind::VariableDeclaration
        | Kind::EnumMember
        | Kind::ClassDeclaration
        | Kind::MethodDeclaration
        | Kind::PropertyDeclaration
        | Kind::GetAccessor
        | Kind::SetAccessor => parent.name() == Some(node),
        Kind::PropertyAssignment => {
            // Property keys in object literals are declarations, values are uses.
            if parent.as_property_assignment().initializer() == node {
                return false;
            }
            parent.parent().is_some_and(|p| p.kind() == Kind::ObjectLiteralExpression)
        }
        Kind::ArrowFunction
        | Kind::FunctionDeclaration
        | Kind::FunctionExpression
        | Kind::EnumDeclaration
        | Kind::InterfaceDeclaration
        | Kind::ModuleDeclaration
        | Kind::MethodSignature
        | Kind::PropertySignature
        | Kind::TypeAliasDeclaration
        | Kind::TypeParameter
        | Kind::Parameter => true,
        Kind::ImportEqualsDeclaration => parent.name() == Some(node),
        _ => false,
    }
}

/// Whether we're inside an import statement.
fn is_inside_import(node: P<Node>) -> bool {
    let mut current = Some(node);
    while let Some(cur) = current {
        let kind = cur.kind();
        if kind == Kind::ImportDeclaration {
            return true;
        }
        if kind == Kind::SourceFile
            || kind == Kind::Block
            || kind == Kind::FunctionDeclaration
            || kind == Kind::ClassDeclaration
        {
            return false;
        }
        current = cur.parent();
    }
    false
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
            Kind::Identifier => {
                let Some(parent) = node.parent() else {
                    return;
                };
                match parent.kind() {
                    // Skip JSX closing elements to avoid duplicate reports
                    Kind::JsxClosingElement => return,
                    // Skip identifiers directly in export declarations (not in specifiers)
                    Kind::ExportDeclaration => return,
                    // Skip namespace exports like: export * as ns from 'module'
                    Kind::NamespaceExport => return,
                    Kind::ExportSpecifier => {
                        let export_spec = parent.as_export_specifier();
                        if export_spec.property_name == Some(node) {
                            // The local binding (foo in "export { foo as bar }")
                            return;
                        }
                        // The re-export is explicitly marked as deprecated
                        if has_deprecated_tag(parent.jsdoc(None)) {
                            return;
                        }
                    }
                    _ => {}
                }
                self.check_identifier(ctx, node);
            }
            Kind::ElementAccessExpression => self.check_element_access_expression(ctx, node),
            Kind::PropertyAssignment
            | Kind::ShorthandPropertyAssignment
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor => {
                self.check_object_literal_property_deprecation(ctx, node, node.name())
            }
            Kind::PrivateIdentifier | Kind::SuperKeyword => self.check_identifier(ctx, node),
            _ => {}
        }
    }
}
