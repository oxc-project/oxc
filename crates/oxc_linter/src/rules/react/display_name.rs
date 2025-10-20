use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, ClassElement, Expression, ObjectExpression, ObjectPropertyKind,
        PropertyKey, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::PropName;
use oxc_semantic::{AstNode, Reference, SymbolId};
use oxc_span::CompactStr;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        contains_jsx, find_innermost_function_with_jsx, function_contains_jsx, is_hoc_call,
        is_react_component_name,
    },
};
use oxc_macros::declare_oxc_lint;

fn component_display_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component definition is missing display name")
        .with_help("Add a displayName property to the component")
        .with_label(span)
}

fn context_display_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Context definition is missing display name")
        .with_help("Add a displayName property to the context")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that React components have a displayName property.
    ///
    /// ### Why is this bad?
    /// React DevTools uses displayName to show component names in the component tree.
    /// Without displayName, components will show up as "Unknown" in DevTools.
    ///
    /// ### Example
    /// ```jsx
    /// // ❌ Bad
    /// const MyComponent = () => <div>Hello</div>;
    ///
    /// // ✅ Good
    /// const MyComponent = () => <div>Hello</div>;
    /// MyComponent.displayName = 'MyComponent';
    /// ```
    DisplayName,
    react,
    style
);

#[derive(Debug, Clone)]
enum ComponentType {
    Function,
    Class,
    CreateReactClass,
    ObjectProperty,
}

#[derive(Debug)]
struct ReactComponentInfo {
    span: Span,
    _component_type: ComponentType,
    is_context: bool,
    name: Option<CompactStr>,
}

// Version cache to avoid repeated parsing
#[derive(Debug, Clone)]
struct VersionCache {
    memo_forwardref_compatible: Option<bool>,
    context_objects_compatible: Option<bool>,
    version: Option<CompactStr>,
}

impl VersionCache {
    fn new() -> Self {
        Self { memo_forwardref_compatible: None, context_objects_compatible: None, version: None }
    }

    fn get_memo_forwardref_compatible(&mut self, ctx: &LintContext) -> bool {
        let current_version = ctx.settings().react.version.as_deref();

        // Check if version changed
        if self.version.as_deref() != current_version {
            self.version = current_version.map(CompactStr::from);
            self.memo_forwardref_compatible = None;
            self.context_objects_compatible = None;
        }

        if let Some(compatible) = self.memo_forwardref_compatible {
            return compatible;
        }

        let compatible = test_react_version_for_memo_forwardref_internal(current_version);
        self.memo_forwardref_compatible = Some(compatible);
        compatible
    }

    fn get_context_objects_compatible(&mut self, ctx: &LintContext) -> bool {
        let current_version = ctx.settings().react.version.as_deref();

        // Check if version changed
        if self.version.as_deref() != current_version {
            self.version = current_version.map(CompactStr::from);
            self.memo_forwardref_compatible = None;
            self.context_objects_compatible = None;
        }

        if let Some(compatible) = self.context_objects_compatible {
            return compatible;
        }

        let compatible = check_react_version_internal(current_version, 16, 3);
        self.context_objects_compatible = Some(compatible);
        compatible
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisplayNameConfig {
    ignore_transpiler_name: bool,
    check_context_objects: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DisplayName(Box<DisplayNameConfig>);

impl Rule for DisplayName {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = if let Some(config) = value.get(0) {
            DisplayNameConfig {
                ignore_transpiler_name: config
                    .get("ignoreTranspilerName")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                check_context_objects: config
                    .get("checkContextObjects")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            }
        } else {
            DisplayNameConfig::default()
        };
        Self(Box::new(config))
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut version_cache = VersionCache::new();
        let ignore_transpiler_name = self.0.ignore_transpiler_name;
        // Only check context objects if React version is >= 16.3.0
        let check_context_objects =
            self.0.check_context_objects && version_cache.get_context_objects_compatible(ctx);

        let mut components_to_report: Vec<(Span, bool)> = Vec::new();

        // Phase 1: Iterate symbols to find components with declarations
        for symbol_id in ctx.scoping().symbol_ids() {
            let declaration = ctx.scoping().symbol_declaration(symbol_id);
            let decl_node = ctx.nodes().get_node(declaration);

            // First check if the declaration itself is a component
            if let Some(component_info) = is_react_component_node(
                decl_node,
                ctx,
                &mut version_cache,
                ignore_transpiler_name,
                check_context_objects,
            ) {
                // If component has a name and ignoreTranspilerName is false and it's not a context,
                // the name itself is considered a valid displayName
                if component_info.name.is_some()
                    && !ignore_transpiler_name
                    && !component_info.is_context
                {
                    continue; // Name is valid displayName
                }

                // Check if this symbol has a displayName assignment via semantic references
                if !has_display_name_via_semantic(symbol_id, component_info.name.as_ref(), ctx) {
                    components_to_report.push((component_info.span, component_info.is_context));
                }
            } else if check_context_objects {
                // If the declaration isn't a component, check if any write references assign createContext()
                // This handles: var Hello; Hello = createContext();
                if let Some(component_info) =
                    check_context_assignment_references(symbol_id, decl_node, ctx)
                {
                    // Check if this symbol has a displayName assignment
                    if !has_display_name_via_semantic(symbol_id, component_info.name.as_ref(), ctx)
                    {
                        components_to_report.push((component_info.span, component_info.is_context));
                    }
                }
            }
        }

        // Phase 2: Handle anonymous components (no symbol) - only specific cases
        for node in ctx.nodes() {
            match node.kind() {
                AstKind::ExportDefaultDeclaration(export) => {
                    if let Some(component_info) = is_anonymous_export_component(
                        export,
                        ctx,
                        &mut version_cache,
                        ignore_transpiler_name,
                    ) {
                        components_to_report.push((component_info.span, component_info.is_context));
                    }
                }
                AstKind::AssignmentExpression(assign) => {
                    // Handle: module.exports = () => <div />
                    if let Some(component_info) = is_module_exports_component(
                        assign,
                        ctx,
                        &mut version_cache,
                        ignore_transpiler_name,
                        check_context_objects,
                    ) {
                        components_to_report.push((component_info.span, component_info.is_context));
                    }
                }
                _ => {}
            }
        }

        // Phase 3: Report diagnostics
        for (span, is_context) in components_to_report {
            if is_context {
                ctx.diagnostic(context_display_name_diagnostic(span));
            } else {
                ctx.diagnostic(component_display_name_diagnostic(span));
            }
        }
    }
}

/// Check if a reference is part of a displayName assignment for the given symbol
fn is_display_name_assignment_for_reference(
    reference: &Reference,
    component_name: Option<&str>,
    ctx: &LintContext,
) -> bool {
    if !reference.is_read() {
        return false;
    }

    let reference_node = ctx.nodes().get_node(reference.node_id());

    // Walk up to find if this is part of: Component.displayName = ...
    for ancestor_id in ctx.nodes().ancestor_ids(reference_node.id()) {
        let ancestor = ctx.nodes().get_node(ancestor_id);

        if let AstKind::AssignmentExpression(assign) = ancestor.kind()
            && let AssignmentTarget::StaticMemberExpression(member) = &assign.left
            && member.property.name == "displayName"
        {
            // Check if the object matches the component
            if let Expression::Identifier(ident) = &member.object {
                if let Some(name) = component_name {
                    return ident.name == name;
                }
            } else if let Expression::StaticMemberExpression(_) = &member.object {
                // Handle nested case like: Namespace.Component.displayName
                // We need to extract the full path and compare
                if let Some(name) = component_name {
                    let path = extract_member_expression_path(&member.object);
                    return path == name;
                }
            }
            return true;
        }
    }

    false
}

/// Extract the full path from a member expression (e.g., "Namespace.Component")
fn extract_member_expression_path(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(ident) => ident.name.to_string(),
        Expression::StaticMemberExpression(member) => {
            let base = extract_member_expression_path(&member.object);
            format!("{}.{}", base, member.property.name)
        }
        _ => String::new(),
    }
}

/// Check if a symbol has a displayName assignment via semantic references
fn has_display_name_via_semantic(
    symbol_id: SymbolId,
    component_name: Option<&CompactStr>,
    ctx: &LintContext,
) -> bool {
    let component_name_str = component_name.map(oxc_span::CompactStr::as_str);

    // Check all references to this symbol
    for reference in ctx.scoping().get_resolved_references(symbol_id) {
        if is_display_name_assignment_for_reference(reference, component_name_str, ctx) {
            return true;
        }
    }

    false
}

/// Check if any write reference to a symbol assigns createContext()
/// This handles patterns like: var Hello; Hello = createContext();
fn check_context_assignment_references(
    symbol_id: SymbolId,
    decl_node: &AstNode,
    ctx: &LintContext,
) -> Option<ReactComponentInfo> {
    // Only check if the declaration is a VariableDeclarator with no init
    if !matches!(decl_node.kind(), AstKind::VariableDeclarator(decl) if decl.init.is_none()) {
        return None;
    }

    let name = if let AstKind::VariableDeclarator(decl) = decl_node.kind() {
        decl.id.get_identifier_name().map(|s| CompactStr::from(s.as_str()))
    } else {
        None
    };

    // Check all write references to this symbol
    for reference in ctx.scoping().get_resolved_references(symbol_id) {
        if !reference.is_write() {
            continue;
        }

        let ref_node = ctx.nodes().get_node(reference.node_id());

        // Walk up to find the assignment expression
        for ancestor_id in ctx.nodes().ancestor_ids(ref_node.id()) {
            let ancestor = ctx.nodes().get_node(ancestor_id);

            if let AstKind::AssignmentExpression(assign) = ancestor.kind()
                && let Expression::CallExpression(call) = &assign.right
                && let Some(callee_name) = call.callee_name()
                && (callee_name == "createContext" || callee_name.ends_with(".createContext"))
            {
                return Some(ReactComponentInfo {
                    span: assign.span,
                    _component_type: ComponentType::Function,
                    is_context: true,
                    name,
                });
            }

            if matches!(ancestor.kind(), AstKind::AssignmentExpression(_)) {
                break; // Found assignment, no need to check further ancestors
            }
        }
    }

    None
}

/// Consolidate component detection logic
fn is_react_component_node<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    version_cache: &mut VersionCache,
    ignore_transpiler_name: bool,
    check_context_objects: bool,
) -> Option<ReactComponentInfo> {
    match node.kind() {
        AstKind::VariableDeclarator(decl) => {
            let name = decl.id.get_identifier_name().map(|s| CompactStr::from(s.as_str()));

            // If the init is None (e.g., `var Hello;`), we can't detect anything here
            // The actual assignment might happen later via a write reference
            // That will be handled by checking write references in Phase 1
            decl.init.as_ref()?;

            // Check for createContext
            if let Some(Expression::CallExpression(call)) = &decl.init
                && let Some(callee_name) = call.callee_name()
                && (callee_name == "createContext" || callee_name.ends_with(".createContext"))
            {
                if check_context_objects {
                    return Some(ReactComponentInfo {
                        span: decl.span,
                        _component_type: ComponentType::Function,
                        is_context: true,
                        name,
                    });
                }
                return None;
            }

            if let Some(Expression::CallExpression(call)) = &decl.init
                && let Some(callee_name) = call.callee_name()
            {
                // Check for HOC patterns
                if is_hoc_call(callee_name, ctx) {
                    // Handle React.memo(React.forwardRef(...)) - skip if version compatible
                    if callee_name.ends_with("memo")
                        && let Some(first_arg) = call.arguments.first()
                        && let Some(Expression::CallExpression(inner_call)) =
                            first_arg.as_expression()
                        && let Some(inner_callee_name) = inner_call.callee_name()
                        && is_hoc_call(inner_callee_name, ctx)
                        && version_cache.get_memo_forwardref_compatible(ctx)
                    {
                        return None;
                    }

                    // For HOCs, check if the inner function/component has a name
                    // If the first argument is a named function or identifier, that counts as having a name
                    let inner_has_name = if let Some(first_arg) = call.arguments.first() {
                        match first_arg.as_expression() {
                            Some(Expression::FunctionExpression(func)) => func.id.is_some(),
                            Some(Expression::Identifier(_)) => true, // Reference to named component
                            _ => false,
                        }
                    } else {
                        false
                    };

                    // Return component info with name only if inner function has a name
                    return Some(ReactComponentInfo {
                        span: decl.span,
                        _component_type: ComponentType::Function,
                        is_context: false,
                        name: if inner_has_name { name } else { None },
                    });
                }

                // Check for createReactClass
                if callee_name == "createClass" || callee_name == "createReactClass" {
                    // Check if it has displayName in the object
                    let has_display_name = call.arguments.iter().any(|arg| {
                        if let Some(Expression::ObjectExpression(obj_expr)) = arg.as_expression() {
                            obj_expr.properties.iter().any(|prop| {
                                if let Some((prop_name, _)) = prop.prop_name() {
                                    prop_name == "displayName"
                                        || (!ignore_transpiler_name && prop_name == "name")
                                } else {
                                    false
                                }
                            })
                        } else {
                            false
                        }
                    });

                    if !has_display_name {
                        return Some(ReactComponentInfo {
                            span: decl.span,
                            _component_type: ComponentType::CreateReactClass,
                            is_context: false,
                            name,
                        });
                    }
                    return None;
                }
            }

            // Check for function/arrow function components with JSX
            if let Some(expr) = &decl.init
                && let Some((component_type, is_hof)) =
                    check_function_expression_component(expr, ctx)
            {
                return Some(ReactComponentInfo {
                    span: decl.span,
                    _component_type: component_type,
                    is_context: false,
                    name: if is_hof { None } else { name }, // HOFs don't use variable name as displayName
                });
            }

            // Check for object expressions with methods
            if let Some(Expression::ObjectExpression(obj_expr)) = &decl.init
                && let Some(name) = &name
                && has_component_methods_in_object(obj_expr, ignore_transpiler_name)
            {
                return Some(ReactComponentInfo {
                    span: decl.span,
                    _component_type: ComponentType::ObjectProperty,
                    is_context: false,
                    name: Some(name.clone()),
                });
            }

            None
        }
        AstKind::Class(class) => {
            if let Some(name) = &class.name()
                && is_react_component_name(name)
            {
                // Check if class has static displayName
                let has_static_display_name = class.body.body.iter().any(|element| match element {
                    ClassElement::MethodDefinition(method_def) => {
                        method_def.r#static
                            && method_def.key.static_name()
                                == Some(std::borrow::Cow::Borrowed("displayName"))
                    }
                    ClassElement::PropertyDefinition(prop_def) => {
                        prop_def.r#static
                            && prop_def.key.static_name()
                                == Some(std::borrow::Cow::Borrowed("displayName"))
                    }
                    _ => false,
                });

                if has_static_display_name {
                    return None; // Has static displayName
                }

                // Check if class contains JSX
                let contains_jsx_in_class = class.body.body.iter().any(|element| {
                    if let ClassElement::MethodDefinition(method_def) = element
                        && let Some(body) = &method_def.value.body
                    {
                        for stmt in &body.statements {
                            if let Statement::ReturnStatement(ret_stmt) = stmt
                                && let Some(expr) = &ret_stmt.argument
                                && contains_jsx(expr)
                            {
                                return true;
                            }
                        }
                    }
                    false
                });

                if contains_jsx_in_class {
                    return Some(ReactComponentInfo {
                        span: class.span,
                        _component_type: ComponentType::Class,
                        is_context: false,
                        name: Some(CompactStr::from(name.as_str())),
                    });
                }
            }
            None
        }
        AstKind::Function(func) => {
            if let Some(name) = &func.id
                && is_react_component_name(&name.name)
            {
                // Check if function directly contains JSX
                if function_contains_jsx(func) {
                    return Some(ReactComponentInfo {
                        span: func.span,
                        _component_type: ComponentType::Function,
                        is_context: false,
                        name: Some(CompactStr::from(name.name.as_str())),
                    });
                }

                // Check if function returns another function with JSX (HOF pattern)
                // For HOFs, we don't use the function name as displayName
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt
                            && let Some(expr) = &ret_stmt.argument
                        {
                            // Check if it returns a function with JSX
                            if find_innermost_function_with_jsx(expr, ctx).is_some() {
                                return Some(ReactComponentInfo {
                                    span: func.span,
                                    _component_type: ComponentType::Function,
                                    is_context: false,
                                    name: None, // HOFs don't use the outer function name as displayName
                                });
                            }

                            // Check if it returns createReactClass
                            if let Expression::CallExpression(call) = expr {
                                if let Some(callee_name) = call.callee_name()
                                    && (callee_name == "createClass"
                                        || callee_name == "createReactClass")
                                {
                                    // Check if it has displayName in the object
                                    let has_display_name = call.arguments.iter().any(|arg| {
                                        if let Some(Expression::ObjectExpression(obj_expr)) =
                                            arg.as_expression()
                                        {
                                            obj_expr.properties.iter().any(|prop| {
                                                if let Some((prop_name, _)) = prop.prop_name() {
                                                    prop_name == "displayName"
                                                        || (!ignore_transpiler_name
                                                            && prop_name == "name")
                                                } else {
                                                    false
                                                }
                                            })
                                        } else {
                                            false
                                        }
                                    });

                                    if !has_display_name {
                                        return Some(ReactComponentInfo {
                                            span: name.span,
                                            _component_type: ComponentType::CreateReactClass,
                                            is_context: false,
                                            name: Some(CompactStr::from(name.name.as_str())),
                                        });
                                    }
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Check if a function expression or arrow function is a component
/// Returns (ComponentType, is_hof) tuple where is_hof indicates if it's a Higher-Order Function
fn check_function_expression_component(
    expr: &Expression,
    ctx: &LintContext,
) -> Option<(ComponentType, bool)> {
    // First check if this function directly contains JSX
    let direct_jsx = match expr {
        Expression::FunctionExpression(func) => function_contains_jsx(func),
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if arrow.body.statements.len() == 1 {
                    if let Statement::ExpressionStatement(expr_stmt) = &arrow.body.statements[0] {
                        contains_jsx(&expr_stmt.expression)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                arrow.body.statements.iter().any(|stmt| {
                    if let Statement::ReturnStatement(ret_stmt) = stmt
                        && let Some(expr) = &ret_stmt.argument
                    {
                        return contains_jsx(expr);
                    }
                    false
                })
            }
        }
        _ => false,
    };

    if direct_jsx {
        return Some((ComponentType::Function, false)); // Not a HOF, directly contains JSX
    }

    // Check if this is a Higher-Order Function that returns a function with JSX
    if let Some(innermost) = find_innermost_function_with_jsx(expr, ctx) {
        // Check if the innermost function has a name
        let inner_has_name = match innermost {
            crate::utils::InnermostFunction::Function(func) => func.id.is_some(),
            crate::utils::InnermostFunction::ArrowFunction(_) => false,
        };

        // Only treat as HOF (returning name=None) if the inner function doesn't have a name
        return Some((ComponentType::Function, !inner_has_name));
    }

    None
}

/// Check if an object expression has component methods
fn has_component_methods_in_object(
    obj_expr: &ObjectExpression,
    ignore_transpiler_name: bool,
) -> bool {
    for prop in &obj_expr.properties {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
            && let PropertyKey::StaticIdentifier(ident) = &obj_prop.key
            && obj_prop.method
            && is_react_component_name(&ident.name)
            && matches!(&obj_prop.value, Expression::FunctionExpression(f) if function_contains_jsx(f))
            && ignore_transpiler_name
        {
            return true;
        }
    }
    false
}

/// Handle anonymous export default components
fn is_anonymous_export_component<'a>(
    export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    _ctx: &LintContext<'a>,
    _version_cache: &mut VersionCache,
    ignore_transpiler_name: bool,
) -> Option<ReactComponentInfo> {
    match &export.declaration {
        oxc_ast::ast::ExportDefaultDeclarationKind::ArrowFunctionExpression(func) => {
            if func.expression {
                if func.body.statements.len() == 1
                    && let Statement::ExpressionStatement(expr_stmt) = &func.body.statements[0]
                    && contains_jsx(&expr_stmt.expression)
                {
                    return Some(ReactComponentInfo {
                        span: export.span,
                        _component_type: ComponentType::Function,
                        is_context: false,
                        name: None,
                    });
                }
            } else {
                for stmt in &func.body.statements {
                    if let Statement::ReturnStatement(ret_stmt) = stmt
                        && let Some(expr) = &ret_stmt.argument
                        && contains_jsx(expr)
                    {
                        return Some(ReactComponentInfo {
                            span: export.span,
                            _component_type: ComponentType::Function,
                            is_context: false,
                            name: None,
                        });
                    }
                }
            }
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                for stmt in &body.statements {
                    if let Statement::ReturnStatement(ret_stmt) = stmt
                        && let Some(expr) = &ret_stmt.argument
                        && contains_jsx(expr)
                    {
                        // Check if function has a name
                        if func.id.is_none() || ignore_transpiler_name {
                            return Some(ReactComponentInfo {
                                span: export.span,
                                _component_type: ComponentType::Function,
                                is_context: false,
                                name: None,
                            });
                        }
                    }
                }
            }
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            if let Some(name) = &func.id
                && is_react_component_name(&name.name)
                && function_contains_jsx(func)
                && ignore_transpiler_name
            {
                return Some(ReactComponentInfo {
                    span: export.span,
                    _component_type: ComponentType::Function,
                    is_context: false,
                    name: Some(CompactStr::from(name.name.as_str())),
                });
            }
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            return check_class_component(class, export.span, ignore_transpiler_name);
        }
        _ => {}
    }
    None
}

/// Check if a class is a React component that needs displayName
fn check_class_component(
    class: &oxc_ast::ast::Class,
    span: Span,
    ignore_transpiler_name: bool,
) -> Option<ReactComponentInfo> {
    // Check if class has static displayName
    let has_static_display_name = class.body.body.iter().any(|element| match element {
        ClassElement::MethodDefinition(method_def) => {
            method_def.r#static
                && method_def.key.static_name() == Some(std::borrow::Cow::Borrowed("displayName"))
        }
        ClassElement::PropertyDefinition(prop_def) => {
            prop_def.r#static
                && prop_def.key.static_name() == Some(std::borrow::Cow::Borrowed("displayName"))
        }
        _ => false,
    });

    if has_static_display_name {
        return None;
    }

    // Check if class contains JSX
    let contains_jsx_in_class = class.body.body.iter().any(|element| {
        if let ClassElement::MethodDefinition(method_def) = element
            && let Some(body) = &method_def.value.body
        {
            for stmt in &body.statements {
                if let Statement::ReturnStatement(ret_stmt) = stmt
                    && let Some(expr) = &ret_stmt.argument
                    && contains_jsx(expr)
                {
                    return true;
                }
            }
        }
        false
    });

    if !contains_jsx_in_class {
        return None;
    }

    // If class has a name
    if let Some(name) = &class.id {
        if is_react_component_name(&name.name) {
            if ignore_transpiler_name {
                return Some(ReactComponentInfo {
                    span,
                    _component_type: ComponentType::Class,
                    is_context: false,
                    name: Some(CompactStr::from(name.name.as_str())),
                });
            }

            // Check if extends React.Component
            let extends_react_component = class.super_class.as_ref().is_some_and(|super_class| {
                if let Some(member_expr) = super_class.as_member_expression()
                    && let Expression::Identifier(ident) = member_expr.object()
                {
                    return ident.name == "React"
                        && member_expr
                            .static_property_name()
                            .is_some_and(|name| name == "Component" || name == "PureComponent");
                }
                if let Some(ident_reference) = super_class.get_identifier_reference() {
                    return ident_reference.name == "Component"
                        || ident_reference.name == "PureComponent";
                }
                false
            });

            if extends_react_component {
                return Some(ReactComponentInfo {
                    span,
                    _component_type: ComponentType::Class,
                    is_context: false,
                    name: None,
                });
            }
        }
    } else {
        // Anonymous class
        if ignore_transpiler_name {
            return Some(ReactComponentInfo {
                span,
                _component_type: ComponentType::Class,
                is_context: false,
                name: None,
            });
        }

        // Check if extends React.Component
        let extends_react_component = class.super_class.as_ref().is_some_and(|super_class| {
            if let Some(member_expr) = super_class.as_member_expression()
                && let Expression::Identifier(ident) = member_expr.object()
            {
                return ident.name == "React"
                    && member_expr
                        .static_property_name()
                        .is_some_and(|name| name == "Component" || name == "PureComponent");
            }
            if let Some(ident_reference) = super_class.get_identifier_reference() {
                return ident_reference.name == "Component"
                    || ident_reference.name == "PureComponent";
            }
            false
        });

        if extends_react_component {
            return Some(ReactComponentInfo {
                span,
                _component_type: ComponentType::Class,
                is_context: false,
                name: None,
            });
        }
    }

    None
}

/// Handle module.exports assignments
fn is_module_exports_component<'a>(
    assign: &oxc_ast::ast::AssignmentExpression<'a>,
    _ctx: &LintContext<'a>,
    _version_cache: &mut VersionCache,
    ignore_transpiler_name: bool,
    check_context_objects: bool,
) -> Option<ReactComponentInfo> {
    if let AssignmentTarget::StaticMemberExpression(member) = &assign.left
        && let Expression::Identifier(ident) = &member.object
        && ident.name == "module"
        && member.property.name == "exports"
    {
        match &assign.right {
            Expression::ArrowFunctionExpression(func) => {
                if func.expression {
                    if func.body.statements.len() == 1
                        && let Statement::ExpressionStatement(expr_stmt) = &func.body.statements[0]
                        && contains_jsx(&expr_stmt.expression)
                    {
                        return Some(ReactComponentInfo {
                            span: assign.span,
                            _component_type: ComponentType::Function,
                            is_context: false,
                            name: None,
                        });
                    }
                } else {
                    for stmt in &func.body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt
                            && let Some(expr) = &ret_stmt.argument
                            && contains_jsx(expr)
                        {
                            return Some(ReactComponentInfo {
                                span: assign.span,
                                _component_type: ComponentType::Function,
                                is_context: false,
                                name: None,
                            });
                        }
                    }
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt
                            && let Some(expr) = &ret_stmt.argument
                            && contains_jsx(expr)
                            && (func.id.is_none() || ignore_transpiler_name)
                        {
                            return Some(ReactComponentInfo {
                                span: assign.span,
                                _component_type: ComponentType::Function,
                                is_context: false,
                                name: None,
                            });
                        }
                    }
                }
            }
            Expression::CallExpression(call) => {
                if let Some(callee_name) = call.callee_name() {
                    if callee_name == "createClass" || callee_name == "createReactClass" {
                        let has_display_name = call.arguments.iter().any(|arg| {
                            if let Some(Expression::ObjectExpression(obj_expr)) =
                                arg.as_expression()
                            {
                                obj_expr.properties.iter().any(|prop| {
                                    if let Some((prop_name, _)) = prop.prop_name() {
                                        prop_name == "displayName"
                                            || (!ignore_transpiler_name && prop_name == "name")
                                    } else {
                                        false
                                    }
                                })
                            } else {
                                false
                            }
                        });

                        if !has_display_name {
                            return Some(ReactComponentInfo {
                                span: assign.span,
                                _component_type: ComponentType::CreateReactClass,
                                is_context: false,
                                name: None,
                            });
                        }
                    } else if (callee_name == "createContext"
                        || callee_name.ends_with(".createContext"))
                        && check_context_objects
                    {
                        return Some(ReactComponentInfo {
                            span: assign.span,
                            _component_type: ComponentType::Function,
                            is_context: true,
                            name: None,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse version string like "16.14.0" into (major, minor, patch)
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    // Avoid Vec allocation by using split_once and split_once again
    let (major_str, rest) = version.split_once('.')?;
    let (minor_str, patch_str) = rest.split_once('.')?;

    let major = major_str.parse::<u32>().ok()?;
    let minor = minor_str.parse::<u32>().ok()?;
    let patch = patch_str.parse::<u32>().ok()?;

    Some((major, minor, patch))
}

/// Internal version checking function to avoid repeated parsing
fn check_react_version_internal(version: Option<&str>, min_major: u32, min_minor: u32) -> bool {
    match version {
        Some(version) => {
            if let Some((major, minor, _)) = parse_version(version) {
                major >= min_major && (major > min_major || minor >= min_minor)
            } else {
                false
            }
        }
        None => {
            // If no version specified, assume modern React
            true
        }
    }
}

/// Internal memo/forwardRef version checking function
fn test_react_version_for_memo_forwardref_internal(version: Option<&str>) -> bool {
    match version {
        Some(version) => {
            if let Some((major, minor, patch)) = parse_version(version) {
                // ^0.14.10: major == 0 && minor >= 14 && patch >= 10
                if major == 0 && minor >= 14 && patch >= 10 {
                    return true;
                }
                // ^15.7.0: major == 15 && minor >= 7
                if major == 15 && minor >= 7 {
                    return true;
                }
                // >= 16.12.0: major >= 16 && (major > 16 || minor >= 12)
                if major >= 16 && (major > 16 || minor >= 12) {
                    return true;
                }
                false
            } else {
                false
            }
        }
        None => {
            // If no version specified, assume modern React (>= 16.12.0)
            true
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = React.createClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
        	        class Hello extends React.Component {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	        Hello.displayName = 'Hello'
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        class Hello {
        	          render() {
        	            return 'Hello World';
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello extends Greetings {
        	          static text = 'Hello World';
        	          render() {
        	            return Hello.text;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello {
        	          method;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          static get displayName() {
        	            return 'Hello';
        	          }
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          static displayName = 'Widget';
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Hello extends React.Component {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export default class Hello {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello;
        	        Hello = createReactClass({
        	          render: function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        module.exports = createReactClass({
        	          "displayName": "Hello",
        	          "render": function() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        });
        	      "#,
            None,
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            let { a, ...b } = obj;
        	            let c = { ...d };
        	            return <div />;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        export default class {
        	          render() {
        	            return <div>Hello {this.props.name}</div>;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export const Hello = React.memo(function Hello() {
        	          return <p />;
        	        })
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = function() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        function Hello() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = () => {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        module.exports = function Hello() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
			        function Hello() {
			          return <div>Hello {this.props.name}</div>;
			        }
			        Hello.displayName = 'Hello';
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = () => {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	        Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = function() {
        	          return <div>Hello {this.props.name}</div>;
        	        }
        	        Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Mixins = {
        	          Greetings: {
        	            Hello: function() {
        	              return <div>Hello {this.props.name}</div>;
        	            }
        	          }
        	        }
        	        Mixins.Greetings.Hello.displayName = 'Hello';
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          render: function() {
        	            return <div>{this._renderHello()}</div>;
        	          },
        	          _renderHello: function() {
        	            return <span>Hello {this.props.name}</span>;
        	          }
        	        });
        	      ",
            None,
            None,
        ),
        (
            "
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {
        	            return <div>{this._renderHello()}</div>;
        	          },
        	          _renderHello: function() {
        	            return <span>Hello {this.props.name}</span>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        const Mixin = {
        	          Button() {
        	            return (
        	              <button />
        	            );
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        var obj = {
        	          pouf: function() {
        	            return any
        	          }
        	        };
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
        	        var obj = {
        	          pouf: function() {
        	            return any
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        export default {
        	          renderHello() {
        	            let {name} = this.props;
        	            return <div>{name}</div>;
        	          }
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React, { createClass } from 'react';
        	        export default createClass({
        	          displayName: 'Foo',
        	          render() {
        	            return <h1>foo</h1>;
        	          }
        	        });
        	      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            r#"
        	        import React, {Component} from "react";
        	        function someDecorator(ComposedComponent) {
        	          return class MyDecorator extends Component {
        	            render() {return <ComposedComponent {...this.props} />;}
        	          };
        	        }
        	        module.exports = someDecorator;
        	      "#,
            None,
            None,
        ),
        (
            r#"
        	        import React, {createElement} from "react";
        	        const SomeComponent = (props) => {
        	          const {foo, bar} = props;
        	          return someComponentFactory({
        	            onClick: () => foo(bar("x"))
        	          });
        	        };
        	      "#,
            None,
            None,
        ),
        (
            "
        	        const element = (
        	          <Media query={query} render={() => {
        	            renderWasCalled = true
        	            return <div/>
        	          }}/>
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        const element = (
        	          <Media query={query} render={function() {
        	            renderWasCalled = true
        	            return <div/>
        	          }}/>
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        module.exports = {
        	          createElement: tagName => document.createElement(tagName)
        	        };
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        const { createElement } = document;
        	        createElement("a");
        	      "#,
            None,
            None,
        ),
        (
            "
        	        import React from 'react'
        	        import { string } from 'prop-types'

        	        function Component({ world }) {
        	          return <div>Hello {world}</div>
        	        }

        	        Component.propTypes = {
        	          world: string,
        	        }

        	        export default React.memo(Component)
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const ComponentWithMemo = React.memo(function Component({ world }) {
        	          return <div>Hello {world}</div>
        	        })
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const Hello = React.memo(function Hello() {
        	          return;
        	        });
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const ForwardRefComponentLike = React.forwardRef(function ComponentLike({ world }, ref) {
        	          return <div ref={ref}>Hello {world}</div>
        	        })
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        function F() {
        	          let items = [];
        	          let testData = [
        	            {a: "test1", displayName: "test2"}, {a: "test1", displayName: "test2"}];
        	          for (let item of testData) {
        	              items.push({a: item.a, b: item.displayName});
        	          }
        	          return <div>{items}</div>;
        	        }
        	      "#,
            None,
            None,
        ),
        // NOTE: this test throws an unexpected token error.
        // It seems that eslint-plugin-react parses this as Flow rather than TypeScript, so it rightly fails.
        // (
        //     r#"
        // 	        import {Component} from "react";
        // 	        type LinkProps = {...{}};
        // 	        class Link extends Component<LinkProps> {}
        // 	      "#,
        //     None,
        //     None,
        // ),
        (
            r#"
        	        const x = {
        	          title: "URL",
        	          dataIndex: "url",
        	          key: "url",
        	          render: url => (
        	            <a href={url} target="_blank" rel="noopener noreferrer">
        	              <p>lol</p>
        	            </a>
        	          )
        	        }
        	      "#,
            None,
            None,
        ),
        (
            "
        	        const renderer = a => function Component(listItem) {
        	          return <div>{a} {listItem}</div>;
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        const Comp = React.forwardRef((props, ref) => <main />);
        	        Comp.displayName = 'MyCompName';
        	      ",
            None,
            None,
        ),
        (
            r#"
        	        const Comp = React.forwardRef((props, ref) => <main data-as="yes" />) as SomeComponent;
        	        Comp.displayName = 'MyCompNameAs';
        	      "#,
            None,
            None,
        ),
        (
            "
        	        function Test() {
        	          const data = [
        	            {
        	              name: 'Bob',
        	            },
        	          ];

        	          const columns = [
        	            {
        	              Header: 'Name',
        	              accessor: 'name',
        	              Cell: ({ value }) => <div>{value}</div>,
        	            },
        	          ];

        	          return <ReactTable columns={columns} data={data} />;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        const f = (a) => () => {
        	          if (a) {
        	            return null;
        	          }
        	          return 1;
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        class Test {
        	          render() {
        	            const data = [
        	              {
        	                name: 'Bob',
        	                Cell: ({ value }) => <div>{value}</div>,
        	              },
        	            ];

        	            return <ReactTable columns={columns} data={data} />;
        	          }
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        export const demo = (a) => (b) => {
        	          if (a == null) return null;
        	          return b;
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        let demo = null;
        	        demo = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };",
            None,
            None,
        ),
        (
            "
        	        obj._property = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        _variable = (a) => {
        	          if (a == null) return null;
        	          return f(a);
        	        };
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = () => () => null;
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = {
        	          property: () => () => null
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = function() {return function() {return null;};};
        	      ",
            None,
            None,
        ),
        (
            "
        	        demo = {
        	          property: function() {return function() {return null;};}
        	        }
        	      ",
            None,
            None,
        ),
        (
            "
        	        function MyComponent(props) {
        	          return <b>{props.name}</b>;
        	        }

        	        const MemoizedMyComponent = React.memo(
        	          MyComponent,
        	          (prevProps, nextProps) => prevProps.name === nextProps.name
        	        )
        	      ",
            None,
            None,
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.14.0",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(({ world }, ref) => {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.0",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function ComponentLike({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.12.1",        },      } }),
            ),
        ),
        (
            "
        	        export const ComponentWithForwardRef = React.memo(
        	          React.forwardRef(function Component({ world }) {
        	            return <div>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.11",        },      } }),
            ),
        ),
        (
            "
        	        import React from 'react'

        	        const MemoizedForwardRefComponentLike = React.memo(
        	          React.forwardRef(function({ world }, ref) {
        	            return <div ref={ref}>Hello {world}</div>
        	          })
        	        )
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.7.1",        },      } }),
            ),
        ),
        (
            r#"
        	        import React from 'react';

        	        const Hello = React.createContext();
        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();

        	        const obj = {};
        	        obj.displayName = "False positive";

        	        Hello.displayName = "HelloContext"
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import * as React from 'react';

        	        const Hello = React.createContext();

        	        const obj = {};
        	        obj.displayName = "False positive";

        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        const obj = {};
        	        obj.displayName = "False positive";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "16.2.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        let Hello;
        	        Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
        	        import { createContext } from 'react';

        	        const Hello = createContext();
        	      ",
            Some(serde_json::json!([{ "checkContextObjects": false }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": ">16.3.0",        },      } }),
            ),
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        var Hello;
        	        Hello = createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            r#"
        	        import { createContext } from 'react';

        	        var Hello;
        	        Hello = React.createContext();
        	        Hello.displayName = "HelloContext";
        	      "#,
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    let fail = vec![
        (
            r#"
        	        var Hello = createReactClass({
        	          render: function() {
        	            return React.createElement("div", {}, "text content");
        	          }
        	        });
        	      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            r#"
        	        var Hello = React.createClass({
        	          render: function() {
        	            return React.createElement("div", {}, "text content");
        	          }
        	        });
        	      "#,
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function HelloComponent() {
			          return createReactClass({
			            render: function() {
			              return <div>Hello {this.props.name}</div>;
			            }
			          });
			        }
			        module.exports = HelloComponent();
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        module.exports = () => {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = function() {
			          return <div>Hello {props.name}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        module.exports = createReactClass({
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "pragma": "Foo",          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        /** @jsx Foo */
			        var Hello = Foo.createClass({
			          _renderHello: function() {
			            return <span>Hello {this.props.name}</span>;
			          },
			          render: function() {
			            return <div>{this._renderHello()}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            Some(
                serde_json::json!({ "settings": {        "react": {          "createClass": "createClass",        },      } }),
            ),
        ),
        (
            "
			        const Mixin = {
			          Button() {
			            return (
			              <button />
			            );
			          }
			        };
			      ",
            Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
            None,
        ),
        (
            "
			        function Hof() {
			          return function () {
			            return <div />
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            r#"
			        import React, { createElement } from "react";
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(({ world }) => {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ComponentWithMemo = React.memo(function() {
			          return <div>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(({ world }, ref) => {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const ForwardRefComponentLike = React.forwardRef(function({ world }, ref) {
			          return <div ref={ref}>Hello {world}</div>
			        })
			      ",
            None,
            None,
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(({ world }, ref) => {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.6.0",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "0.14.2",        },      } }),
            ),
        ),
        (
            "
			        import React from 'react'

			        const MemoizedForwardRefComponentLike = React.memo(
			          React.forwardRef(function ComponentLike({ world }, ref) {
			            return <div ref={ref}>Hello {world}</div>
			          })
			        )
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "react": {          "version": "15.0.1",        },      } }),
            ),
        ),
        (
            r#"
			        import React from "react";
			        const { createElement } = React;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        import React from "react";
			        const createElement = React.createElement;
			        export default (props) => {
			          return createElement("div", {}, "hello");
			        };
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = function () {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }
			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            r#"
			        module.exports = () => {
			          function a () {}
			          const b = function b () {}
			          const c = function () {}
			          const d = () => {}
			          const obj = {
			            a: function a () {},
			            b: function b () {},
			            c () {},
			            d: () => {},
			          }

			          return React.createElement("div", {}, "text content");
			        }
			      "#,
            None,
            None,
        ),
        (
            "
			        export default class extends React.Component {
			          render() {
			            function a () {}
			            const b = function b () {}
			            const c = function () {}
			            const d = () => {}
			            const obj = {
			              a: function a () {},
			              b: function b () {},
			              c () {},
			              d: () => {},
			            }
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export default class extends React.PureComponent {
			          render() {
			            return <Card />;
			          }
			        }

			        const Card = (() => {
			          return React.memo(({ }) => (
			            <div />
			          ));
			        })();
			      ",
            None,
            None,
        ),
        (
            "
			        const renderer = a => listItem => (
			          <div>{a} {listItem}</div>
			        );
			      ",
            None,
            None,
        ),
        (
            "
        	        const processData = (options?: { value: string }) => options?.value || 'no data';

        	        export const Component = observer(() => {
        	          const data = processData({ value: 'data' });
        	          return <div>{data}</div>;
        	        });

        	        export const Component2 = observer(() => {
        	          const data = processData();
        	          return <div>{data}</div>;
        	        });
        	      ",
            None,
            Some(
                serde_json::json!({ "settings": { "react": { "componentWrapperFunctions": ["observer"] } } }),
            ),
        ),
        (
            "
			        import React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import * as React from 'react';

			        const Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        const Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
        (
            "
			        import { createContext } from 'react';

			        var Hello;
			        Hello = React.createContext();
			      ",
            Some(serde_json::json!([{ "checkContextObjects": true }])),
            None,
        ),
    ];

    Tester::new(DisplayName::NAME, DisplayName::PLUGIN, pass, fail).test_and_snapshot();
}
