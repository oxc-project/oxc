/// Validate source locations in the compiled output.
///
/// Port of `Validation/ValidateSourceLocations.ts` from the React Compiler.
///
/// Validates that the compiled output preserves source locations from the
/// original code, which is important for source maps and debugging.
///
/// IMPORTANT: This validation is only intended for use in unit tests.
/// It is not intended for use in production.
use oxc_ast::ast;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::{
    CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
};
use crate::hir::build_hir::LowerableFunction;
use crate::reactive_scopes::codegen_reactive_function::CodegenOutput;

/// Strict node types where both source location AND node type must match.
const STRICT_NODE_TYPES: &[&str] = &["VariableDeclaration", "VariableDeclarator", "Identifier"];

fn is_strict(name: &str) -> bool {
    STRICT_NODE_TYPES.contains(&name)
}

fn span_key(span: Span) -> u64 {
    (u64::from(span.start) << 32) | u64::from(span.end)
}

fn is_manual_memoization(expr: &ast::CallExpression<'_>) -> bool {
    match &expr.callee {
        ast::Expression::Identifier(ident) => {
            ident.name == "useMemo" || ident.name == "useCallback"
        }
        ast::Expression::StaticMemberExpression(member) => {
            if let ast::Expression::Identifier(obj) = &member.object {
                obj.name == "React"
                    && (member.property.name == "useMemo" || member.property.name == "useCallback")
            } else {
                false
            }
        }
        _ => false,
    }
}

type LocationMap = FxHashMap<u64, (Span, FxHashSet<String>)>;

fn record_location(map: &mut LocationMap, span: Span, node_type: &str) {
    if span == Span::default() {
        return;
    }
    let key = span_key(span);
    let entry = map.entry(key).or_insert_with(|| (span, FxHashSet::default()));
    entry.1.insert(node_type.to_string());
}

fn collect_original_locations(func: &LowerableFunction<'_>, map: &mut LocationMap) {
    match func {
        LowerableFunction::Function(f) => {
            record_location(map, f.span, "FunctionDeclaration");
            for param in &f.params.items {
                collect_orig_formal_param(param, map);
            }
            if let Some(body) = &f.body {
                for stmt in &body.statements {
                    collect_orig_stmt(stmt, map);
                }
            }
        }
        LowerableFunction::ArrowFunction(f) => {
            record_location(map, f.span, "ArrowFunctionExpression");
            for param in &f.params.items {
                collect_orig_formal_param(param, map);
            }
            for stmt in &f.body.statements {
                collect_orig_stmt(stmt, map);
            }
        }
    }
}

fn collect_orig_formal_param(param: &ast::FormalParameter<'_>, map: &mut LocationMap) {
    collect_orig_binding_pattern(&param.pattern, map);
}

fn collect_orig_binding_pattern(pat: &ast::BindingPattern<'_>, map: &mut LocationMap) {
    match pat {
        ast::BindingPattern::BindingIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_orig_binding_pattern(&prop.value, map);
            }
            if let Some(rest) = &obj.rest {
                collect_orig_binding_pattern(&rest.argument, map);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_orig_binding_pattern(elem, map);
            }
            if let Some(rest) = &arr.rest {
                collect_orig_binding_pattern(&rest.argument, map);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            record_location(map, assign.span, "AssignmentPattern");
            collect_orig_binding_pattern(&assign.left, map);
            collect_orig_expr(&assign.right, map);
        }
    }
}

fn collect_orig_stmt(stmt: &ast::Statement<'_>, map: &mut LocationMap) {
    match stmt {
        ast::Statement::ExpressionStatement(s) => {
            if let ast::Expression::CallExpression(call) = &s.expression
                && is_manual_memoization(call)
            {
                return;
            }
            record_location(map, s.span, "ExpressionStatement");
            collect_orig_expr(&s.expression, map);
        }
        ast::Statement::BlockStatement(s) => {
            for child in &s.body {
                collect_orig_stmt(child, map);
            }
        }
        ast::Statement::BreakStatement(s) => {
            record_location(map, s.span, "BreakStatement");
        }
        ast::Statement::ContinueStatement(s) => {
            record_location(map, s.span, "ContinueStatement");
        }
        ast::Statement::ReturnStatement(s) => {
            record_location(map, s.span, "ReturnStatement");
            if let Some(arg) = &s.argument {
                collect_orig_expr(arg, map);
            }
        }
        ast::Statement::ThrowStatement(s) => {
            record_location(map, s.span, "ThrowStatement");
            collect_orig_expr(&s.argument, map);
        }
        ast::Statement::TryStatement(s) => {
            record_location(map, s.span, "TryStatement");
            for child in &s.block.body {
                collect_orig_stmt(child, map);
            }
            if let Some(handler) = &s.handler {
                for child in &handler.body.body {
                    collect_orig_stmt(child, map);
                }
            }
            if let Some(finalizer) = &s.finalizer {
                for child in &finalizer.body {
                    collect_orig_stmt(child, map);
                }
            }
        }
        ast::Statement::IfStatement(s) => {
            record_location(map, s.span, "IfStatement");
            collect_orig_expr(&s.test, map);
            collect_orig_stmt(&s.consequent, map);
            if let Some(alt) = &s.alternate {
                collect_orig_stmt(alt, map);
            }
        }
        ast::Statement::ForStatement(s) => {
            record_location(map, s.span, "ForStatement");
            if let Some(init) = &s.init {
                collect_orig_for_init(init, map);
            }
            if let Some(test) = &s.test {
                collect_orig_expr(test, map);
            }
            if let Some(update) = &s.update {
                collect_orig_expr(update, map);
            }
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::ForInStatement(s) => {
            record_location(map, s.span, "ForInStatement");
            collect_orig_expr(&s.right, map);
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::ForOfStatement(s) => {
            record_location(map, s.span, "ForOfStatement");
            collect_orig_expr(&s.right, map);
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::WhileStatement(s) => {
            record_location(map, s.span, "WhileStatement");
            collect_orig_expr(&s.test, map);
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::DoWhileStatement(s) => {
            record_location(map, s.span, "DoWhileStatement");
            collect_orig_expr(&s.test, map);
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::SwitchStatement(s) => {
            record_location(map, s.span, "SwitchStatement");
            collect_orig_expr(&s.discriminant, map);
            for case in &s.cases {
                record_location(map, case.span, "SwitchCase");
                if let Some(test) = &case.test {
                    collect_orig_expr(test, map);
                }
                for child in &case.consequent {
                    collect_orig_stmt(child, map);
                }
            }
        }
        ast::Statement::WithStatement(s) => {
            record_location(map, s.span, "WithStatement");
            collect_orig_expr(&s.object, map);
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::LabeledStatement(s) => {
            record_location(map, s.span, "LabeledStatement");
            collect_orig_stmt(&s.body, map);
        }
        ast::Statement::VariableDeclaration(decl) => {
            collect_orig_var_decl(decl, map);
        }
        ast::Statement::FunctionDeclaration(f) => {
            record_location(map, f.span, "FunctionDeclaration");
            if let Some(id) = &f.id {
                record_location(map, id.span, "Identifier");
            }
            for param in &f.params.items {
                collect_orig_formal_param(param, map);
            }
            if let Some(body) = &f.body {
                for child in &body.statements {
                    collect_orig_stmt(child, map);
                }
            }
        }
        _ => {}
    }
}

fn collect_orig_var_decl(decl: &ast::VariableDeclaration<'_>, map: &mut LocationMap) {
    record_location(map, decl.span, "VariableDeclaration");
    for d in &decl.declarations {
        record_location(map, d.span, "VariableDeclarator");
        collect_orig_binding_pattern(&d.id, map);
        if let Some(init) = &d.init {
            collect_orig_expr(init, map);
        }
    }
}

fn collect_orig_for_init(init: &ast::ForStatementInit<'_>, map: &mut LocationMap) {
    match init {
        ast::ForStatementInit::VariableDeclaration(decl) => {
            collect_orig_var_decl(decl, map);
        }
        _ => {
            if let Some(expr) = init.as_expression() {
                collect_orig_expr(expr, map);
            }
        }
    }
}

fn collect_orig_expr(expr: &ast::Expression<'_>, map: &mut LocationMap) {
    match expr {
        ast::Expression::Identifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::Expression::ArrowFunctionExpression(f) => {
            record_location(map, f.span, "ArrowFunctionExpression");
            for param in &f.params.items {
                collect_orig_formal_param(param, map);
            }
            collect_orig_arrow_body(f, map);
        }
        ast::Expression::FunctionExpression(f) => {
            record_location(map, f.span, "FunctionExpression");
            if let Some(id) = &f.id {
                record_location(map, id.span, "Identifier");
            }
            for param in &f.params.items {
                collect_orig_formal_param(param, map);
            }
            if let Some(body) = &f.body {
                for child in &body.statements {
                    collect_orig_stmt(child, map);
                }
            }
        }
        ast::Expression::ConditionalExpression(c) => {
            record_location(map, c.span, "ConditionalExpression");
            collect_orig_expr(&c.test, map);
            collect_orig_expr(&c.consequent, map);
            collect_orig_expr(&c.alternate, map);
        }
        ast::Expression::LogicalExpression(l) => {
            record_location(map, l.span, "LogicalExpression");
            collect_orig_expr(&l.left, map);
            collect_orig_expr(&l.right, map);
        }
        ast::Expression::CallExpression(call) => {
            if is_manual_memoization(call) {
                return;
            }
            collect_orig_expr(&call.callee, map);
            for arg in &call.arguments {
                collect_orig_argument(arg, map);
            }
        }
        ast::Expression::AssignmentExpression(assign) => {
            collect_orig_assign_target(&assign.left, map);
            collect_orig_expr(&assign.right, map);
        }
        ast::Expression::BinaryExpression(bin) => {
            collect_orig_expr(&bin.left, map);
            collect_orig_expr(&bin.right, map);
        }
        ast::Expression::UnaryExpression(un) => {
            collect_orig_expr(&un.argument, map);
        }
        ast::Expression::UpdateExpression(up) => {
            collect_orig_simple_assign_target(&up.argument, map);
        }
        ast::Expression::ComputedMemberExpression(_)
        | ast::Expression::StaticMemberExpression(_)
        | ast::Expression::PrivateFieldExpression(_) => {
            if let Some(member) = expr.as_member_expression() {
                collect_orig_member_expr(member, map);
            }
        }
        ast::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::ObjectPropertyKind::ObjectProperty(p) => {
                        collect_orig_expr(&p.value, map);
                    }
                    ast::ObjectPropertyKind::SpreadProperty(s) => {
                        collect_orig_expr(&s.argument, map);
                    }
                }
            }
        }
        ast::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    ast::ArrayExpressionElement::SpreadElement(s) => {
                        collect_orig_expr(&s.argument, map);
                    }
                    ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(e) = elem.as_expression() {
                            collect_orig_expr(e, map);
                        }
                    }
                }
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            for e in &seq.expressions {
                collect_orig_expr(e, map);
            }
        }
        ast::Expression::NewExpression(new) => {
            collect_orig_expr(&new.callee, map);
            for arg in &new.arguments {
                collect_orig_argument(arg, map);
            }
        }
        ast::Expression::TaggedTemplateExpression(tag) => {
            collect_orig_expr(&tag.tag, map);
        }
        ast::Expression::TemplateLiteral(tpl) => {
            for e in &tpl.expressions {
                collect_orig_expr(e, map);
            }
        }
        ast::Expression::AwaitExpression(a) => {
            collect_orig_expr(&a.argument, map);
        }
        ast::Expression::YieldExpression(y) => {
            if let Some(arg) = &y.argument {
                collect_orig_expr(arg, map);
            }
        }
        ast::Expression::ParenthesizedExpression(p) => {
            collect_orig_expr(&p.expression, map);
        }
        ast::Expression::JSXElement(jsx) => {
            collect_orig_jsx_element(jsx, map);
        }
        ast::Expression::JSXFragment(jsx) => {
            collect_orig_jsx_children(&jsx.children, map);
        }
        _ => {}
    }
}

fn collect_orig_arrow_body(f: &ast::ArrowFunctionExpression<'_>, map: &mut LocationMap) {
    let body = f.body.as_ref();
    if body.directives.is_empty()
        && body.statements.len() == 1
        && let Some(ast::Statement::ReturnStatement(ret)) = body.statements.first()
        && let Some(arg) = &ret.argument
    {
        collect_orig_expr(arg, map);
        return;
    }
    for stmt in &body.statements {
        collect_orig_stmt(stmt, map);
    }
}

fn collect_orig_argument(arg: &ast::Argument<'_>, map: &mut LocationMap) {
    match arg {
        ast::Argument::SpreadElement(s) => {
            collect_orig_expr(&s.argument, map);
        }
        _ => {
            if let Some(e) = arg.as_expression() {
                collect_orig_expr(e, map);
            }
        }
    }
}

fn collect_orig_assign_target(target: &ast::AssignmentTarget<'_>, map: &mut LocationMap) {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            collect_orig_expr(&member.object, map);
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            collect_orig_expr(&member.object, map);
            collect_orig_expr(&member.expression, map);
        }
        ast::AssignmentTarget::ArrayAssignmentTarget(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_orig_assign_target_maybe_default(elem, map);
            }
        }
        ast::AssignmentTarget::ObjectAssignmentTarget(obj) => {
            for prop in &obj.properties {
                collect_orig_assign_target_property(prop, map);
            }
        }
        _ => {}
    }
}

fn collect_orig_assign_target_maybe_default(
    target: &ast::AssignmentTargetMaybeDefault<'_>,
    map: &mut LocationMap,
) {
    match target {
        ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(d) => {
            collect_orig_assign_target(&d.binding, map);
            collect_orig_expr(&d.init, map);
        }
        _ => {
            if let Some(t) = target.as_assignment_target() {
                collect_orig_assign_target(t, map);
            }
        }
    }
}

fn collect_orig_assign_target_property(
    prop: &ast::AssignmentTargetProperty<'_>,
    map: &mut LocationMap,
) {
    match prop {
        ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p) => {
            record_location(map, p.binding.span, "Identifier");
            if let Some(init) = &p.init {
                collect_orig_expr(init, map);
            }
        }
        ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(p) => {
            collect_orig_assign_target_maybe_default(&p.binding, map);
        }
    }
}

fn collect_orig_simple_assign_target(
    target: &ast::SimpleAssignmentTarget<'_>,
    map: &mut LocationMap,
) {
    match target {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            collect_orig_expr(&member.object, map);
        }
        ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            collect_orig_expr(&member.object, map);
            collect_orig_expr(&member.expression, map);
        }
        _ => {}
    }
}

fn collect_orig_member_expr(member: &ast::MemberExpression<'_>, map: &mut LocationMap) {
    match member {
        ast::MemberExpression::StaticMemberExpression(m) => {
            collect_orig_expr(&m.object, map);
            record_location(map, m.property.span, "Identifier");
        }
        ast::MemberExpression::ComputedMemberExpression(m) => {
            collect_orig_expr(&m.object, map);
            collect_orig_expr(&m.expression, map);
        }
        ast::MemberExpression::PrivateFieldExpression(m) => {
            collect_orig_expr(&m.object, map);
        }
    }
}

fn collect_orig_jsx_element(jsx: &ast::JSXElement<'_>, map: &mut LocationMap) {
    for attr in &jsx.opening_element.attributes {
        if let ast::JSXAttributeItem::Attribute(a) = attr
            && let Some(ast::JSXAttributeValue::ExpressionContainer(container)) = &a.value
            && let Some(expr) = container.expression.as_expression()
        {
            collect_orig_expr(expr, map);
        }
    }
    collect_orig_jsx_children(&jsx.children, map);
}

fn collect_orig_jsx_children(
    children: &oxc_allocator::Vec<'_, ast::JSXChild<'_>>,
    map: &mut LocationMap,
) {
    for child in children {
        match child {
            ast::JSXChild::Element(el) => {
                collect_orig_jsx_element(el, map);
            }
            ast::JSXChild::Fragment(frag) => {
                collect_orig_jsx_children(&frag.children, map);
            }
            ast::JSXChild::ExpressionContainer(container) => {
                if let Some(expr) = container.expression.as_expression() {
                    collect_orig_expr(expr, map);
                }
            }
            ast::JSXChild::Spread(spread) => {
                collect_orig_expr(&spread.expression, map);
            }
            ast::JSXChild::Text(_) => {}
        }
    }
}

// === Generated AST collection ===

fn collect_gen_locations(stmts: &[ast::Statement<'_>], map: &mut LocationMap) {
    for stmt in stmts {
        collect_gen_stmt(stmt, map);
    }
}

fn collect_gen_stmt(stmt: &ast::Statement<'_>, map: &mut LocationMap) {
    match stmt {
        ast::Statement::ExpressionStatement(s) => {
            record_location(map, s.span, "ExpressionStatement");
            collect_gen_expr(&s.expression, map);
        }
        ast::Statement::BlockStatement(s) => {
            record_location(map, s.span, "BlockStatement");
            for child in &s.body {
                collect_gen_stmt(child, map);
            }
        }
        ast::Statement::BreakStatement(s) => {
            record_location(map, s.span, "BreakStatement");
        }
        ast::Statement::ContinueStatement(s) => {
            record_location(map, s.span, "ContinueStatement");
        }
        ast::Statement::ReturnStatement(s) => {
            record_location(map, s.span, "ReturnStatement");
            if let Some(arg) = &s.argument {
                collect_gen_expr(arg, map);
            }
        }
        ast::Statement::ThrowStatement(s) => {
            record_location(map, s.span, "ThrowStatement");
            collect_gen_expr(&s.argument, map);
        }
        ast::Statement::TryStatement(s) => {
            record_location(map, s.span, "TryStatement");
            for child in &s.block.body {
                collect_gen_stmt(child, map);
            }
            if let Some(handler) = &s.handler {
                for child in &handler.body.body {
                    collect_gen_stmt(child, map);
                }
            }
            if let Some(finalizer) = &s.finalizer {
                for child in &finalizer.body {
                    collect_gen_stmt(child, map);
                }
            }
        }
        ast::Statement::IfStatement(s) => {
            record_location(map, s.span, "IfStatement");
            collect_gen_expr(&s.test, map);
            collect_gen_stmt(&s.consequent, map);
            if let Some(alt) = &s.alternate {
                collect_gen_stmt(alt, map);
            }
        }
        ast::Statement::ForStatement(s) => {
            record_location(map, s.span, "ForStatement");
            if let Some(ast::ForStatementInit::VariableDeclaration(decl)) = &s.init {
                collect_gen_var_decl(decl, map);
            }
            if let Some(test) = &s.test {
                collect_gen_expr(test, map);
            }
            if let Some(update) = &s.update {
                collect_gen_expr(update, map);
            }
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::ForInStatement(s) => {
            record_location(map, s.span, "ForInStatement");
            collect_gen_expr(&s.right, map);
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::ForOfStatement(s) => {
            record_location(map, s.span, "ForOfStatement");
            collect_gen_expr(&s.right, map);
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::WhileStatement(s) => {
            record_location(map, s.span, "WhileStatement");
            collect_gen_expr(&s.test, map);
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::DoWhileStatement(s) => {
            record_location(map, s.span, "DoWhileStatement");
            collect_gen_expr(&s.test, map);
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::SwitchStatement(s) => {
            record_location(map, s.span, "SwitchStatement");
            collect_gen_expr(&s.discriminant, map);
            for case in &s.cases {
                record_location(map, case.span, "SwitchCase");
                if let Some(test) = &case.test {
                    collect_gen_expr(test, map);
                }
                for child in &case.consequent {
                    collect_gen_stmt(child, map);
                }
            }
        }
        ast::Statement::WithStatement(s) => {
            record_location(map, s.span, "WithStatement");
            collect_gen_expr(&s.object, map);
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::LabeledStatement(s) => {
            record_location(map, s.span, "LabeledStatement");
            collect_gen_stmt(&s.body, map);
        }
        ast::Statement::VariableDeclaration(decl) => {
            collect_gen_var_decl(decl, map);
        }
        ast::Statement::FunctionDeclaration(f) => {
            record_location(map, f.span, "FunctionDeclaration");
            if let Some(id) = &f.id {
                record_location(map, id.span, "Identifier");
            }
            for param in &f.params.items {
                collect_gen_formal_param(param, map);
            }
            if let Some(body) = &f.body {
                for child in &body.statements {
                    collect_gen_stmt(child, map);
                }
            }
        }
        _ => {}
    }
}

fn collect_gen_var_decl(decl: &ast::VariableDeclaration<'_>, map: &mut LocationMap) {
    record_location(map, decl.span, "VariableDeclaration");
    for d in &decl.declarations {
        record_location(map, d.span, "VariableDeclarator");
        collect_gen_binding_pattern(&d.id, map);
        if let Some(init) = &d.init {
            collect_gen_expr(init, map);
        }
    }
}

fn collect_gen_formal_param(param: &ast::FormalParameter<'_>, map: &mut LocationMap) {
    collect_gen_binding_pattern(&param.pattern, map);
}

fn collect_gen_binding_pattern(pat: &ast::BindingPattern<'_>, map: &mut LocationMap) {
    match pat {
        ast::BindingPattern::BindingIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_gen_binding_pattern(&prop.value, map);
            }
            if let Some(rest) = &obj.rest {
                collect_gen_binding_pattern(&rest.argument, map);
            }
        }
        ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_gen_binding_pattern(elem, map);
            }
            if let Some(rest) = &arr.rest {
                collect_gen_binding_pattern(&rest.argument, map);
            }
        }
        ast::BindingPattern::AssignmentPattern(assign) => {
            record_location(map, assign.span, "AssignmentPattern");
            collect_gen_binding_pattern(&assign.left, map);
            collect_gen_expr(&assign.right, map);
        }
    }
}

fn collect_gen_expr(expr: &ast::Expression<'_>, map: &mut LocationMap) {
    match expr {
        ast::Expression::Identifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::Expression::ArrowFunctionExpression(f) => {
            record_location(map, f.span, "ArrowFunctionExpression");
            for param in &f.params.items {
                collect_gen_formal_param(param, map);
            }
            for stmt in &f.body.statements {
                collect_gen_stmt(stmt, map);
            }
        }
        ast::Expression::FunctionExpression(f) => {
            record_location(map, f.span, "FunctionExpression");
            if let Some(id) = &f.id {
                record_location(map, id.span, "Identifier");
            }
            for param in &f.params.items {
                collect_gen_formal_param(param, map);
            }
            if let Some(body) = &f.body {
                for child in &body.statements {
                    collect_gen_stmt(child, map);
                }
            }
        }
        ast::Expression::ConditionalExpression(c) => {
            record_location(map, c.span, "ConditionalExpression");
            collect_gen_expr(&c.test, map);
            collect_gen_expr(&c.consequent, map);
            collect_gen_expr(&c.alternate, map);
        }
        ast::Expression::LogicalExpression(l) => {
            record_location(map, l.span, "LogicalExpression");
            collect_gen_expr(&l.left, map);
            collect_gen_expr(&l.right, map);
        }
        ast::Expression::CallExpression(call) => {
            collect_gen_expr(&call.callee, map);
            for arg in &call.arguments {
                collect_gen_argument(arg, map);
            }
        }
        ast::Expression::AssignmentExpression(assign) => {
            collect_gen_assign_target(&assign.left, map);
            collect_gen_expr(&assign.right, map);
        }
        ast::Expression::BinaryExpression(bin) => {
            collect_gen_expr(&bin.left, map);
            collect_gen_expr(&bin.right, map);
        }
        ast::Expression::UnaryExpression(un) => {
            collect_gen_expr(&un.argument, map);
        }
        ast::Expression::UpdateExpression(up) => {
            collect_gen_simple_assign_target(&up.argument, map);
        }
        ast::Expression::ComputedMemberExpression(_)
        | ast::Expression::StaticMemberExpression(_)
        | ast::Expression::PrivateFieldExpression(_) => {
            if let Some(member) = expr.as_member_expression() {
                collect_gen_member_expr(member, map);
            }
        }
        ast::Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ast::ObjectPropertyKind::ObjectProperty(p) => {
                        collect_gen_expr(&p.value, map);
                    }
                    ast::ObjectPropertyKind::SpreadProperty(s) => {
                        collect_gen_expr(&s.argument, map);
                    }
                }
            }
        }
        ast::Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    ast::ArrayExpressionElement::SpreadElement(s) => {
                        collect_gen_expr(&s.argument, map);
                    }
                    ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(e) = elem.as_expression() {
                            collect_gen_expr(e, map);
                        }
                    }
                }
            }
        }
        ast::Expression::SequenceExpression(seq) => {
            for e in &seq.expressions {
                collect_gen_expr(e, map);
            }
        }
        ast::Expression::NewExpression(new) => {
            collect_gen_expr(&new.callee, map);
            for arg in &new.arguments {
                collect_gen_argument(arg, map);
            }
        }
        ast::Expression::TaggedTemplateExpression(tag) => {
            collect_gen_expr(&tag.tag, map);
        }
        ast::Expression::TemplateLiteral(tpl) => {
            for e in &tpl.expressions {
                collect_gen_expr(e, map);
            }
        }
        ast::Expression::AwaitExpression(a) => {
            collect_gen_expr(&a.argument, map);
        }
        ast::Expression::YieldExpression(y) => {
            if let Some(arg) = &y.argument {
                collect_gen_expr(arg, map);
            }
        }
        ast::Expression::ParenthesizedExpression(p) => {
            collect_gen_expr(&p.expression, map);
        }
        ast::Expression::JSXElement(jsx) => {
            collect_gen_jsx_element(jsx, map);
        }
        ast::Expression::JSXFragment(jsx) => {
            collect_gen_jsx_children(&jsx.children, map);
        }
        _ => {}
    }
}

fn collect_gen_argument(arg: &ast::Argument<'_>, map: &mut LocationMap) {
    match arg {
        ast::Argument::SpreadElement(s) => {
            collect_gen_expr(&s.argument, map);
        }
        _ => {
            if let Some(e) = arg.as_expression() {
                collect_gen_expr(e, map);
            }
        }
    }
}

fn collect_gen_assign_target(target: &ast::AssignmentTarget<'_>, map: &mut LocationMap) {
    match target {
        ast::AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::AssignmentTarget::StaticMemberExpression(member) => {
            collect_gen_expr(&member.object, map);
        }
        ast::AssignmentTarget::ComputedMemberExpression(member) => {
            collect_gen_expr(&member.object, map);
            collect_gen_expr(&member.expression, map);
        }
        _ => {}
    }
}

fn collect_gen_simple_assign_target(
    target: &ast::SimpleAssignmentTarget<'_>,
    map: &mut LocationMap,
) {
    match target {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
            record_location(map, ident.span, "Identifier");
        }
        ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
            collect_gen_expr(&member.object, map);
        }
        ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            collect_gen_expr(&member.object, map);
            collect_gen_expr(&member.expression, map);
        }
        _ => {}
    }
}

fn collect_gen_member_expr(member: &ast::MemberExpression<'_>, map: &mut LocationMap) {
    match member {
        ast::MemberExpression::StaticMemberExpression(m) => {
            collect_gen_expr(&m.object, map);
            record_location(map, m.property.span, "Identifier");
        }
        ast::MemberExpression::ComputedMemberExpression(m) => {
            collect_gen_expr(&m.object, map);
            collect_gen_expr(&m.expression, map);
        }
        ast::MemberExpression::PrivateFieldExpression(m) => {
            collect_gen_expr(&m.object, map);
        }
    }
}

fn collect_gen_jsx_element(jsx: &ast::JSXElement<'_>, map: &mut LocationMap) {
    for attr in &jsx.opening_element.attributes {
        if let ast::JSXAttributeItem::Attribute(a) = attr
            && let Some(ast::JSXAttributeValue::ExpressionContainer(container)) = &a.value
            && let Some(expr) = container.expression.as_expression()
        {
            collect_gen_expr(expr, map);
        }
    }
    collect_gen_jsx_children(&jsx.children, map);
}

fn collect_gen_jsx_children(
    children: &oxc_allocator::Vec<'_, ast::JSXChild<'_>>,
    map: &mut LocationMap,
) {
    for child in children {
        match child {
            ast::JSXChild::Element(el) => {
                collect_gen_jsx_element(el, map);
            }
            ast::JSXChild::Fragment(frag) => {
                collect_gen_jsx_children(&frag.children, map);
            }
            ast::JSXChild::ExpressionContainer(container) => {
                if let Some(expr) = container.expression.as_expression() {
                    collect_gen_expr(expr, map);
                }
            }
            ast::JSXChild::Spread(spread) => {
                collect_gen_expr(&spread.expression, map);
            }
            ast::JSXChild::Text(_) => {}
        }
    }
}

// === Main validation ===

/// Validate that source locations are preserved in the output.
///
/// # Errors
/// Returns a `CompilerError` if any important source locations are missing
/// or have wrong node types in the generated output.
pub fn validate_source_locations(
    codegen: &CodegenOutput<'_>,
    original_func: Option<&LowerableFunction<'_>>,
) -> Result<(), CompilerError> {
    let Some(original_func) = original_func else {
        return Ok(());
    };

    let mut important_original_locations: LocationMap = FxHashMap::default();
    collect_original_locations(original_func, &mut important_original_locations);

    let mut generated_locations: LocationMap = FxHashMap::default();
    collect_gen_locations(&codegen.body, &mut generated_locations);
    for outlined in &codegen.outlined {
        collect_gen_locations(&outlined.fn_.body, &mut generated_locations);
    }

    let mut errors = CompilerError::new();

    for (key, (span, original_node_types)) in &important_original_locations {
        let generated_node_types = generated_locations.get(key);
        match generated_node_types {
            None => {
                let node_types_str: Vec<&str> =
                    original_node_types.iter().map(String::as_str).collect();
                report_missing_location(&mut errors, *span, &node_types_str.join(", "));
            }
            Some((_, gen_types)) => {
                for node_type in original_node_types {
                    if is_strict(node_type) && !gen_types.contains(node_type) {
                        let has_valid = gen_types.iter().any(|g| original_node_types.contains(g));
                        if has_valid {
                            report_missing_location(&mut errors, *span, node_type);
                        } else {
                            report_wrong_node_type(&mut errors, *span, node_type, gen_types);
                        }
                    }
                }
            }
        }
    }

    if errors.has_any_errors() { Err(errors) } else { Ok(()) }
}

fn report_missing_location(errors: &mut CompilerError, span: Span, node_type: &str) {
    let diagnostic = CompilerDiagnostic::create(
        ErrorCategory::Todo,
        "Important source location missing in generated code".to_string(),
        Some(format!(
            "Source location for {node_type} is missing in the generated output. \
             This can cause coverage instrumentation to fail to track this code properly, \
             resulting in inaccurate coverage reports."
        )),
        None,
    )
    .with_detail(CompilerDiagnosticDetail::Error {
        loc: Some(SourceLocation::Source(span)),
        message: None,
    });
    errors.push_diagnostic(diagnostic);
}

fn report_wrong_node_type(
    errors: &mut CompilerError,
    span: Span,
    expected_type: &str,
    actual_types: &FxHashSet<String>,
) {
    let actual_str: Vec<&str> = actual_types.iter().map(String::as_str).collect();
    let diagnostic = CompilerDiagnostic::create(
        ErrorCategory::Todo,
        "Important source location has wrong node type in generated code".to_string(),
        Some(format!(
            "Source location for {expected_type} exists in the generated output \
             but with wrong node type(s): {}. This can cause coverage instrumentation \
             to fail to track this code properly, resulting in inaccurate coverage reports.",
            actual_str.join(", ")
        )),
        None,
    )
    .with_detail(CompilerDiagnosticDetail::Error {
        loc: Some(SourceLocation::Source(span)),
        message: None,
    });
    errors.push_diagnostic(diagnostic);
}
