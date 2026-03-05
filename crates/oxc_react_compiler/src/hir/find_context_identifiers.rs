/// Find context identifiers in a function.
///
/// Port of `HIR/FindContextIdentifiers.ts` from the React Compiler.
///
/// Context identifiers are variables that are captured by inner functions
/// (closures). These need special handling because they may be modified
/// from within the inner function.
///
/// A variable is a "context identifier" if:
/// - It is reassigned by an inner function, OR
/// - It is both reassigned (anywhere) AND referenced by an inner function.
use oxc_ast::ast;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

/// A set of declaration spans that are captured by inner functions.
///
/// Each entry is the `Span` of the `BindingIdentifier` where the variable
/// was declared. Using spans (rather than names) ensures that shadowed
/// variables with the same name at different scopes get separate entries.
pub type ContextIdentifiers = FxHashSet<Span>;

/// Tracking info for a single binding.
#[derive(Debug, Default)]
struct IdentifierInfo {
    reassigned: bool,
    reassigned_by_inner_fn: bool,
    referenced_by_inner_fn: bool,
}

/// Info stored for a binding in the scope chain.
#[derive(Debug, Clone, Copy)]
struct BindingLocation {
    /// The function nesting depth where this binding was declared.
    fn_depth: u32,
    /// The span of the `BindingIdentifier` AST node where this was declared.
    /// This serves as a unique key (like `binding.identifier` in the TS version).
    decl_span: Span,
}

/// State for the find_context_identifiers walk.
struct FindContextState {
    /// Stack of scope frames. Each frame maps binding names to their
    /// declaration location.
    scopes: Vec<FxHashMap<String, BindingLocation>>,

    /// Current function nesting depth. 0 = the outermost function being analyzed.
    fn_depth: u32,

    /// Per-binding info, keyed by the declaration span of the `BindingIdentifier`.
    /// Using the declaration span as the key ensures shadowed variables
    /// at different scopes get separate entries (matching the TS version
    /// which uses `binding.identifier` node identity).
    identifiers: FxHashMap<Span, IdentifierInfo>,
}

impl FindContextState {
    fn new() -> Self {
        Self { scopes: vec![FxHashMap::default()], fn_depth: 0, identifiers: FxHashMap::default() }
    }

    fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declare a binding in the current scope.
    fn declare_binding(&mut self, name: &str, decl_span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), BindingLocation { fn_depth: self.fn_depth, decl_span });
        }
    }

    /// Look up a binding by name, walking up the scope chain.
    /// Returns the binding location, or None if not found.
    fn find_binding(&self, name: &str) -> Option<BindingLocation> {
        for scope in self.scopes.iter().rev() {
            if let Some(loc) = scope.get(name) {
                return Some(*loc);
            }
        }
        None
    }
}

/// Find all context identifiers in a function body.
///
/// This walks the function's body statements, tracking nested function
/// scopes and identifying variables that are captured across scope
/// boundaries. Returns a set of declaration spans for context identifiers.
pub fn find_context_identifiers(func: &ast::Function<'_>) -> ContextIdentifiers {
    let mut state = FindContextState::new();

    // Declare parameters in the outermost scope
    declare_params(&mut state, &func.params);

    // Walk the function body
    if let Some(body) = &func.body {
        for stmt in &body.statements {
            visit_statement(&mut state, stmt);
        }
    }

    // Collect context identifiers based on the TS criteria
    let mut result = FxHashSet::default();
    for (decl_span, info) in &state.identifiers {
        if info.reassigned_by_inner_fn || (info.reassigned && info.referenced_by_inner_fn) {
            result.insert(*decl_span);
        }
    }
    result
}

/// Find all context identifiers in an arrow function body.
pub fn find_context_identifiers_arrow(
    func: &ast::ArrowFunctionExpression<'_>,
) -> ContextIdentifiers {
    let mut state = FindContextState::new();

    // Declare parameters in the outermost scope
    declare_params(&mut state, &func.params);

    // Walk the function body
    for stmt in &func.body.statements {
        visit_statement(&mut state, stmt);
    }

    // Collect context identifiers
    let mut result = FxHashSet::default();
    for (decl_span, info) in &state.identifiers {
        if info.reassigned_by_inner_fn || (info.reassigned && info.referenced_by_inner_fn) {
            result.insert(*decl_span);
        }
    }
    result
}

/// Declare function parameters as bindings in the current scope.
fn declare_params(state: &mut FindContextState, params: &ast::FormalParameters<'_>) {
    for param in &params.items {
        declare_pattern_bindings(state, &param.pattern);
    }
    if let Some(rest) = &params.rest {
        declare_pattern_bindings(state, &rest.rest.argument);
    }
}

/// Declare all binding names from a binding pattern.
fn declare_pattern_bindings(state: &mut FindContextState, pattern: &ast::BindingPattern<'_>) {
    match pattern {
        ast::BindingPattern::BindingIdentifier(ident) => {
            state.declare_binding(&ident.name, ident.span);
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                declare_pattern_bindings(state, &prop.value);
            }
            if let Some(rest) = &obj.rest {
                declare_pattern_bindings(state, &rest.argument);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                declare_pattern_bindings(state, elem);
            }
            if let Some(rest) = &arr.rest {
                declare_pattern_bindings(state, &rest.argument);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            declare_pattern_bindings(state, &assign.left);
        }
    }
}

/// Visit a statement.
fn visit_statement(state: &mut FindContextState, stmt: &ast::Statement<'_>) {
    match stmt {
        ast::Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                // Visit init expression first (before declaring the binding)
                if let Some(init) = &declarator.init {
                    visit_expression(state, init);
                }
                declare_pattern_bindings(state, &declarator.id);
            }
        }
        ast::Statement::ExpressionStatement(expr_stmt) => {
            visit_expression(state, &expr_stmt.expression);
        }
        ast::Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                visit_expression(state, arg);
            }
        }
        ast::Statement::IfStatement(if_stmt) => {
            visit_expression(state, &if_stmt.test);
            visit_statement(state, &if_stmt.consequent);
            if let Some(alt) = &if_stmt.alternate {
                visit_statement(state, alt);
            }
        }
        ast::Statement::WhileStatement(while_stmt) => {
            visit_expression(state, &while_stmt.test);
            visit_statement(state, &while_stmt.body);
        }
        ast::Statement::DoWhileStatement(do_while) => {
            visit_statement(state, &do_while.body);
            visit_expression(state, &do_while.test);
        }
        ast::Statement::ForStatement(for_stmt) => {
            state.push_scope();
            if let Some(init) = &for_stmt.init {
                match init {
                    ast::ForStatementInit::VariableDeclaration(decl) => {
                        for declarator in &decl.declarations {
                            if let Some(init_expr) = &declarator.init {
                                visit_expression(state, init_expr);
                            }
                            declare_pattern_bindings(state, &declarator.id);
                        }
                    }
                    _ => {
                        visit_expression(state, init.to_expression());
                    }
                }
            }
            if let Some(test) = &for_stmt.test {
                visit_expression(state, test);
            }
            if let Some(update) = &for_stmt.update {
                visit_expression(state, update);
            }
            visit_statement(state, &for_stmt.body);
            state.pop_scope();
        }
        ast::Statement::ForOfStatement(for_of) => {
            state.push_scope();
            match &for_of.left {
                ast::ForStatementLeft::VariableDeclaration(decl) => {
                    for declarator in &decl.declarations {
                        declare_pattern_bindings(state, &declarator.id);
                    }
                }
                ast::ForStatementLeft::AssignmentTargetIdentifier(ident) => {
                    handle_identifier_assignment(state, &ident.name);
                }
                _ => {}
            }
            visit_expression(state, &for_of.right);
            visit_statement(state, &for_of.body);
            state.pop_scope();
        }
        ast::Statement::ForInStatement(for_in) => {
            state.push_scope();
            match &for_in.left {
                ast::ForStatementLeft::VariableDeclaration(decl) => {
                    for declarator in &decl.declarations {
                        declare_pattern_bindings(state, &declarator.id);
                    }
                }
                ast::ForStatementLeft::AssignmentTargetIdentifier(ident) => {
                    handle_identifier_assignment(state, &ident.name);
                }
                _ => {}
            }
            visit_expression(state, &for_in.right);
            visit_statement(state, &for_in.body);
            state.pop_scope();
        }
        ast::Statement::BlockStatement(block) => {
            state.push_scope();
            for s in &block.body {
                visit_statement(state, s);
            }
            state.pop_scope();
        }
        ast::Statement::ThrowStatement(throw) => {
            visit_expression(state, &throw.argument);
        }
        ast::Statement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                visit_statement(state, s);
            }
            if let Some(handler) = &try_stmt.handler {
                state.push_scope();
                if let Some(param) = &handler.param {
                    declare_pattern_bindings(state, &param.pattern);
                }
                for s in &handler.body.body {
                    visit_statement(state, s);
                }
                state.pop_scope();
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for s in &finalizer.body {
                    visit_statement(state, s);
                }
            }
        }
        ast::Statement::SwitchStatement(switch) => {
            visit_expression(state, &switch.discriminant);
            for case in &switch.cases {
                if let Some(test) = &case.test {
                    visit_expression(state, test);
                }
                for s in &case.consequent {
                    visit_statement(state, s);
                }
            }
        }
        ast::Statement::LabeledStatement(labeled) => {
            visit_statement(state, &labeled.body);
        }
        ast::Statement::FunctionDeclaration(func) => {
            // Function declarations are hoisted bindings
            if let Some(id) = &func.id {
                state.declare_binding(&id.name, id.span);
            }
            // The function body creates a new fn scope
            visit_function_body(state, func);
        }
        _ => {}
    }
}

/// Visit an expression.
fn visit_expression(state: &mut FindContextState, expr: &ast::Expression<'_>) {
    match expr {
        ast::Expression::Identifier(ident) => {
            if ident.name != "undefined" {
                handle_identifier_reference(state, &ident.name);
            }
        }
        ast::Expression::AssignmentExpression(assign) => {
            // Visit the right side first
            visit_expression(state, &assign.right);
            // Handle the left side assignment
            visit_assignment_target(state, &assign.left);
        }
        ast::Expression::UpdateExpression(update) => {
            visit_simple_assignment_target_as_assignment(state, &update.argument);
        }
        ast::Expression::BinaryExpression(bin) => {
            visit_expression(state, &bin.left);
            visit_expression(state, &bin.right);
        }
        ast::Expression::LogicalExpression(logical) => {
            visit_expression(state, &logical.left);
            visit_expression(state, &logical.right);
        }
        ast::Expression::UnaryExpression(unary) => {
            visit_expression(state, &unary.argument);
        }
        ast::Expression::ConditionalExpression(cond) => {
            visit_expression(state, &cond.test);
            visit_expression(state, &cond.consequent);
            visit_expression(state, &cond.alternate);
        }
        ast::Expression::CallExpression(call) => {
            visit_expression(state, &call.callee);
            for arg in &call.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        visit_expression(state, &spread.argument);
                    }
                    _ => visit_expression(state, arg.to_expression()),
                }
            }
        }
        ast::Expression::NewExpression(new_expr) => {
            visit_expression(state, &new_expr.callee);
            for arg in &new_expr.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        visit_expression(state, &spread.argument);
                    }
                    _ => visit_expression(state, arg.to_expression()),
                }
            }
        }
        ast::Expression::StaticMemberExpression(member) => {
            visit_expression(state, &member.object);
        }
        ast::Expression::ComputedMemberExpression(member) => {
            visit_expression(state, &member.object);
            visit_expression(state, &member.expression);
        }
        ast::Expression::PrivateFieldExpression(pf) => {
            visit_expression(state, &pf.object);
        }
        ast::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    ast::ArrayExpressionElement::SpreadElement(spread) => {
                        visit_expression(state, &spread.argument);
                    }
                    ast::ArrayExpressionElement::Elision(_) => {}
                    _ => visit_expression(state, elem.to_expression()),
                }
            }
        }
        ast::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        visit_expression(state, &spread.argument);
                    }
                    ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        if prop.computed {
                            visit_expression(state, prop.key.to_expression());
                        }
                        visit_expression(state, &prop.value);
                    }
                }
            }
        }
        ast::Expression::TemplateLiteral(tpl) => {
            for expr in &tpl.expressions {
                visit_expression(state, expr);
            }
        }
        ast::Expression::TaggedTemplateExpression(tagged) => {
            visit_expression(state, &tagged.tag);
            for expr in &tagged.quasi.expressions {
                visit_expression(state, expr);
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            for expr in &seq.expressions {
                visit_expression(state, expr);
            }
        }
        ast::Expression::AwaitExpression(await_expr) => {
            visit_expression(state, &await_expr.argument);
        }
        ast::Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                visit_expression(state, arg);
            }
        }
        ast::Expression::ArrowFunctionExpression(arrow) => {
            // Entering an inner function scope
            visit_arrow_body(state, arrow);
        }
        ast::Expression::FunctionExpression(func) => {
            visit_function_body(state, func);
        }
        ast::Expression::ChainExpression(chain) => {
            visit_chain_element(state, &chain.expression);
        }
        ast::Expression::ParenthesizedExpression(paren) => {
            visit_expression(state, &paren.expression);
        }
        ast::Expression::TSAsExpression(ts_as) => {
            visit_expression(state, &ts_as.expression);
        }
        ast::Expression::TSSatisfiesExpression(ts_sat) => {
            visit_expression(state, &ts_sat.expression);
        }
        ast::Expression::TSNonNullExpression(ts_nn) => {
            visit_expression(state, &ts_nn.expression);
        }
        ast::Expression::TSTypeAssertion(ts_ta) => {
            visit_expression(state, &ts_ta.expression);
        }
        ast::Expression::JSXElement(jsx) => {
            visit_jsx_element(state, jsx);
        }
        ast::Expression::JSXFragment(frag) => {
            for child in &frag.children {
                visit_jsx_child(state, child);
            }
        }
        // Literals and other expressions that don't reference identifiers
        _ => {}
    }
}

/// Visit a chain element (optional chaining).
fn visit_chain_element(state: &mut FindContextState, element: &ast::ChainElement<'_>) {
    match element {
        ast::ChainElement::CallExpression(call) => {
            visit_expression(state, &call.callee);
            for arg in &call.arguments {
                match arg {
                    ast::Argument::SpreadElement(spread) => {
                        visit_expression(state, &spread.argument);
                    }
                    _ => visit_expression(state, arg.to_expression()),
                }
            }
        }
        ast::ChainElement::StaticMemberExpression(member) => {
            visit_expression(state, &member.object);
        }
        ast::ChainElement::ComputedMemberExpression(member) => {
            visit_expression(state, &member.object);
            visit_expression(state, &member.expression);
        }
        ast::ChainElement::TSNonNullExpression(ts_nn) => {
            visit_expression(state, &ts_nn.expression);
        }
        ast::ChainElement::PrivateFieldExpression(pf) => {
            visit_expression(state, &pf.object);
        }
    }
}

/// Visit a JSX element.
fn visit_jsx_element(state: &mut FindContextState, element: &ast::JSXElement<'_>) {
    visit_jsx_opening_name(state, &element.opening_element.name);
    for attr in &element.opening_element.attributes {
        match attr {
            ast::JSXAttributeItem::Attribute(attr) => {
                if let Some(value) = &attr.value {
                    visit_jsx_attribute_value(state, value);
                }
            }
            ast::JSXAttributeItem::SpreadAttribute(spread) => {
                visit_expression(state, &spread.argument);
            }
        }
    }
    for child in &element.children {
        visit_jsx_child(state, child);
    }
}

/// Visit a JSX opening element name (to handle component identifiers).
fn visit_jsx_opening_name(state: &mut FindContextState, name: &ast::JSXElementName<'_>) {
    match name {
        ast::JSXElementName::Identifier(ident) => {
            // Only reference-like if it starts with uppercase (component)
            if ident.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                handle_identifier_reference(state, &ident.name);
            }
        }
        ast::JSXElementName::IdentifierReference(ident) => {
            if ident.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                handle_identifier_reference(state, &ident.name);
            }
        }
        ast::JSXElementName::MemberExpression(member) => {
            visit_jsx_member_object(state, &member.object);
        }
        _ => {}
    }
}

/// Visit a JSX member expression object.
fn visit_jsx_member_object(state: &mut FindContextState, obj: &ast::JSXMemberExpressionObject<'_>) {
    match obj {
        ast::JSXMemberExpressionObject::IdentifierReference(ident) => {
            handle_identifier_reference(state, &ident.name);
        }
        ast::JSXMemberExpressionObject::MemberExpression(member) => {
            visit_jsx_member_object(state, &member.object);
        }
        ast::JSXMemberExpressionObject::ThisExpression(_) => {}
    }
}

/// Visit a JSX attribute value.
fn visit_jsx_attribute_value(state: &mut FindContextState, value: &ast::JSXAttributeValue<'_>) {
    match value {
        ast::JSXAttributeValue::ExpressionContainer(container) => {
            if let ast::JSXExpression::EmptyExpression(_) = &container.expression {
                // skip
            } else {
                visit_expression(state, container.expression.to_expression());
            }
        }
        ast::JSXAttributeValue::Element(element) => {
            visit_jsx_element(state, element);
        }
        ast::JSXAttributeValue::Fragment(frag) => {
            for child in &frag.children {
                visit_jsx_child(state, child);
            }
        }
        ast::JSXAttributeValue::StringLiteral(_) => {}
    }
}

/// Visit a JSX child.
fn visit_jsx_child(state: &mut FindContextState, child: &ast::JSXChild<'_>) {
    match child {
        ast::JSXChild::ExpressionContainer(container) => {
            if let ast::JSXExpression::EmptyExpression(_) = &container.expression {
                // skip
            } else {
                visit_expression(state, container.expression.to_expression());
            }
        }
        ast::JSXChild::Element(element) => {
            visit_jsx_element(state, element);
        }
        ast::JSXChild::Fragment(frag) => {
            for c in &frag.children {
                visit_jsx_child(state, c);
            }
        }
        ast::JSXChild::Spread(spread) => {
            visit_expression(state, &spread.expression);
        }
        ast::JSXChild::Text(_) => {}
    }
}

/// Visit the body of a function (entering an inner function scope).
fn visit_function_body(state: &mut FindContextState, func: &ast::Function<'_>) {
    state.fn_depth += 1;
    state.push_scope();

    // Declare function name if present (function expressions can reference their own name)
    if let Some(id) = &func.id {
        state.declare_binding(&id.name, id.span);
    }

    // Declare parameters
    declare_params(state, &func.params);

    // Walk body
    if let Some(body) = &func.body {
        for stmt in &body.statements {
            visit_statement(state, stmt);
        }
    }

    state.pop_scope();
    state.fn_depth -= 1;
}

/// Visit the body of an arrow function (entering an inner function scope).
fn visit_arrow_body(state: &mut FindContextState, arrow: &ast::ArrowFunctionExpression<'_>) {
    state.fn_depth += 1;
    state.push_scope();

    // Declare parameters
    declare_params(state, &arrow.params);

    // Walk body
    for stmt in &arrow.body.statements {
        visit_statement(state, stmt);
    }

    state.pop_scope();
    state.fn_depth -= 1;
}

/// Handle an identifier reference (read).
fn handle_identifier_reference(state: &mut FindContextState, name: &str) {
    let Some(loc) = state.find_binding(name) else {
        // Not a local binding (global) - skip
        return;
    };

    let info = state.identifiers.entry(loc.decl_span).or_default();

    // If we're inside an inner function and the binding was declared
    // in an outer scope (above the current inner function boundary),
    // it's referenced by an inner function.
    if state.fn_depth > 0 && loc.fn_depth < state.fn_depth {
        info.referenced_by_inner_fn = true;
    }
}

/// Handle an identifier assignment (write).
fn handle_identifier_assignment(state: &mut FindContextState, name: &str) {
    let Some(loc) = state.find_binding(name) else {
        // Not a local binding (global) - skip
        return;
    };

    let info = state.identifiers.entry(loc.decl_span).or_default();
    info.reassigned = true;

    if state.fn_depth > 0 && loc.fn_depth < state.fn_depth {
        info.reassigned_by_inner_fn = true;
    }
}

/// Visit an assignment target (LHS of assignment).
fn visit_assignment_target(state: &mut FindContextState, target: &ast::AssignmentTarget<'_>) {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            handle_identifier_assignment(state, &ident.name);
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            // Interior mutability, not a reassignment of the variable itself
            visit_expression(state, &member.object);
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            visit_expression(state, &member.object);
            visit_expression(state, &member.expression);
        }
        ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
            for elem in arr.elements.iter().flatten() {
                visit_assignment_target_maybe_default(state, elem);
            }
            if let Some(rest) = &arr.rest {
                visit_assignment_target(state, &rest.target);
            }
        }
        ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                        handle_identifier_assignment(state, &ident.binding.name);
                    }
                    ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                        visit_assignment_target_maybe_default(state, &prop.binding);
                    }
                }
            }
            if let Some(rest) = &obj.rest {
                visit_assignment_target(state, &rest.target);
            }
        }
        _ => {}
    }
}

/// Visit an assignment target that may have a default value.
fn visit_assignment_target_maybe_default(
    state: &mut FindContextState,
    target: &ast::AssignmentTargetMaybeDefault<'_>,
) {
    match target {
        ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            visit_expression(state, &with_default.init);
            visit_assignment_target(state, &with_default.binding);
        }
        _ => {
            visit_assignment_target(state, target.to_assignment_target());
        }
    }
}

/// Visit a simple assignment target as an assignment (for update expressions).
fn visit_simple_assignment_target_as_assignment(
    state: &mut FindContextState,
    target: &ast::SimpleAssignmentTarget<'_>,
) {
    match target {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
            handle_identifier_assignment(state, &ident.name);
        }
        ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            // Interior mutability
            visit_expression(state, &member.object);
        }
        ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            visit_expression(state, &member.object);
            visit_expression(state, &member.expression);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    /// Helper: returns the set of context identifier NAMES (for test assertions).
    fn get_context_id_names(source: &str) -> FxHashSet<String> {
        let allocator = Allocator::default();
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

        let func = parser_result.program.body.iter().find_map(|stmt| match stmt {
            ast::Statement::FunctionDeclaration(f) => Some(f),
            _ => None,
        });
        let func = func.as_ref().map(|f| f.as_ref());
        assert!(func.is_some(), "No function found");
        let spans = find_context_identifiers(func.unwrap());

        // Convert spans back to names using the source text
        let mut names = FxHashSet::default();
        for span in &spans {
            let name = &source[span.start as usize..span.end as usize];
            names.insert(name.to_string());
        }
        names
    }

    #[test]
    fn no_inner_function_no_context() {
        let ids = get_context_id_names("function foo() { let x = 1; x = 2; }");
        assert!(ids.is_empty(), "No inner function means no context identifiers, got: {ids:?}");
    }

    #[test]
    fn referenced_by_inner_and_reassigned() {
        let ids = get_context_id_names(
            "function foo() { let x = 1; x = 2; const fn = () => { return x; }; }",
        );
        assert!(ids.contains("x"), "x is reassigned and referenced by inner fn: {ids:?}");
    }

    #[test]
    fn reassigned_by_inner_fn() {
        let ids =
            get_context_id_names("function foo() { let x = 1; const fn = () => { x = 2; }; }");
        assert!(ids.contains("x"), "x is reassigned by inner fn: {ids:?}");
    }

    #[test]
    fn only_referenced_not_reassigned() {
        let ids =
            get_context_id_names("function foo() { const x = 1; const fn = () => { return x; }; }");
        assert!(ids.is_empty(), "x is only referenced (not reassigned) so not context: {ids:?}");
    }

    #[test]
    fn global_is_not_context() {
        let ids = get_context_id_names("function foo() { const fn = () => { console.log(x); }; }");
        assert!(ids.is_empty(), "Global references are not context: {ids:?}");
    }

    #[test]
    fn shadowed_variable_reassigned_by_inner_fn() {
        // The inner `x` is reassigned by the inner function, but the outer `x`
        // should NOT be a context identifier (it is a separate declaration).
        let source = "function foo() {
                const x = {};
                {
                    let x = 56;
                    const fn2 = function() { x = 42; };
                    fn2();
                }
                return x;
            }";
        let allocator = Allocator::default();
        let parser_result = Parser::new(&allocator, source, SourceType::jsx()).parse();
        assert!(parser_result.errors.is_empty());
        let func = parser_result.program.body.iter().find_map(|stmt| match stmt {
            ast::Statement::FunctionDeclaration(f) => Some(f),
            _ => None,
        });
        let func = func.as_ref().map(|f| f.as_ref()).unwrap();
        let spans = find_context_identifiers(func);

        // Should have exactly one context identifier (the inner x)
        assert_eq!(spans.len(), 1, "Expected exactly 1 context identifier, got: {spans:?}");

        // Convert the span to the source text to verify it's the inner x
        let span = spans.iter().next().unwrap();
        let name = &source[span.start as usize..span.end as usize];
        assert_eq!(name, "x", "Context identifier should be named x");

        // Verify the span points to the inner declaration (let x = 56), not the outer one
        // The inner x is at a later position in the source
        let outer_x_pos = source.find("const x = {}").unwrap();
        assert!(
            (span.start as usize) > outer_x_pos + 6,
            "Context identifier span should point to inner x, not outer x"
        );
    }

    #[test]
    fn shadowed_variable_outer_not_context() {
        // When only the inner shadowed variable is mutated by an inner fn,
        // the outer const variable should not become a context identifier.
        let ids = get_context_id_names(
            "function foo() {
                const x = {};
                {
                    const x = [];
                    const fn2 = function() { mutate(x); };
                    fn2();
                }
                return x;
            }",
        );
        // `mutate(x)` only references inner x (not an assignment), inner x is
        // const so not reassigned. Neither x should be context.
        assert!(ids.is_empty(), "Neither x should be context (no reassignment): {ids:?}");
    }
}
