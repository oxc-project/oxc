use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, ClassElement, ExportDefaultDeclarationKind, Expression, ObjectExpression,
        ObjectPropertyKind, PropertyKey, Statement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::PropName;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, Reference, SymbolId};
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    ast_util::iter_outer_expressions,
    config::ReactVersion,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        InnermostFunction, expression_contains_jsx, find_innermost_function_with_jsx,
        function_body_contains_jsx, function_contains_jsx, is_hoc_call, is_react_component_name,
    },
};

fn component_display_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component definition is missing display name.")
        .with_help("Add a `displayName` property to the component.")
        .with_label(span)
}

fn context_display_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Context definition is missing display name.")
        .with_help("Add a `displayName` property to the context.")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that React components have a `displayName` property.
    ///
    /// ### Why is this bad?
    ///
    /// React DevTools uses `displayName` to show component names in the component tree.
    /// Without `displayName`, components will show up as "Unknown" in DevTools.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const MyComponent = () => <div>Hello</div>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const MyComponent = () => <div>Hello</div>;
    /// MyComponent.displayName = 'MyComponent';
    /// ```
    DisplayName,
    react,
    pedantic,
    config = DisplayNameConfig,
);

#[derive(Debug)]
struct ReactComponentInfo {
    span: Span,
    is_context: bool,
    name: Option<CompactStr>,
}

// Version cache to avoid repeated version checks
#[derive(Debug, Clone)]
struct VersionCache {
    memo_forwardref_compatible: Option<bool>,
    context_objects_compatible: Option<bool>,
    version: Option<ReactVersion>,
}

impl VersionCache {
    fn new() -> Self {
        Self { memo_forwardref_compatible: None, context_objects_compatible: None, version: None }
    }

    /// Ensure the cache is fresh for the current version, clearing cached values if version changed.
    fn ensure_fresh(&mut self, ctx: &LintContext) {
        let current_version = ctx.settings().react.version;
        if self.version != current_version {
            self.version = current_version;
            self.memo_forwardref_compatible = None;
            self.context_objects_compatible = None;
        }
    }

    fn get_memo_forwardref_compatible(&mut self, ctx: &LintContext) -> bool {
        self.ensure_fresh(ctx);
        if let Some(compatible) = self.memo_forwardref_compatible {
            return compatible;
        }
        let current_version = ctx.settings().react.version.as_ref();
        let compatible = test_react_version_for_memo_forwardref_internal(current_version);
        self.memo_forwardref_compatible = Some(compatible);
        compatible
    }

    fn get_context_objects_compatible(&mut self, ctx: &LintContext) -> bool {
        self.ensure_fresh(ctx);
        if let Some(compatible) = self.context_objects_compatible {
            return compatible;
        }
        let current_version = ctx.settings().react.version.as_ref();
        let compatible = check_react_version_internal(current_version, 16, 3);
        self.context_objects_compatible = Some(compatible);
        compatible
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DisplayNameConfig {
    /// When `true`, the rule will ignore the name set by the transpiler
    /// and require a `displayName` property in this case.
    ignore_transpiler_name: bool,
    /// When `true`, this rule will warn on context objects
    /// without a `displayName`.
    ///
    /// `displayName` allows you to [name your context](https://reactjs.org/docs/context.html#contextdisplayname) object.
    /// This name is used in the React DevTools for the context's `Provider` and `Consumer`.
    check_context_objects: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DisplayName(Box<DisplayNameConfig>);

impl Rule for DisplayName {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
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

        // Phase 2: Handle anonymous components (no symbol) - ES module exports and CommonJS

        // Check for ES module export default using module_record
        if ctx.module_record().export_default.is_some() {
            // Efficiently find export default by checking only top-level statements
            let program = ctx.nodes().program();
            for stmt in &program.body {
                if let oxc_ast::ast::Statement::ExportDefaultDeclaration(export) = stmt {
                    if let Some(component_info) =
                        is_anonymous_export_component(export, ignore_transpiler_name)
                    {
                        components_to_report.push((component_info.span, component_info.is_context));
                    }
                    break; // Only one export default per module
                }
            }
        }

        // Check for CommonJS module.exports by looking at references to 'module' global
        if let Some(module_reference_ids) = ctx.scoping().root_unresolved_references().get("module")
        {
            for &reference_id in module_reference_ids {
                let reference = ctx.scoping().get_reference(reference_id);
                let node = ctx.nodes().get_node(reference.node_id());

                // Walk up to find if this is part of module.exports assignment
                for ancestor_kind in ctx.nodes().ancestor_kinds(node.id()) {
                    if let AstKind::AssignmentExpression(assign) = ancestor_kind {
                        // Handle: module.exports = () => <div />
                        if let Some(component_info) = is_module_exports_component(
                            assign,
                            ignore_transpiler_name,
                            check_context_objects,
                        ) {
                            components_to_report
                                .push((component_info.span, component_info.is_context));
                        }
                        break; // Found the assignment, no need to check further ancestors
                    }
                }
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

    // Only consider IdentifierReference nodes
    if !matches!(reference_node.kind(), AstKind::IdentifierReference(_)) {
        return false;
    }

    // Use limited parent walking - we only need to check 2 levels:
    // IdentifierReference -> StaticMemberExpression -> AssignmentExpression
    let mut parents = iter_outer_expressions(ctx.nodes(), reference_node.id());

    // First parent should be StaticMemberExpression with .displayName
    let Some(AstKind::StaticMemberExpression(member)) = parents.next() else {
        return false;
    };

    if member.property.name != "displayName" {
        return false;
    }

    // Second parent should be AssignmentExpression with the member on the left side
    let Some(AstKind::AssignmentExpression(assign)) = parents.next() else {
        return false;
    };

    // Verify this is the left-hand side of the assignment
    if !assign.left.span().contains_inclusive(reference_node.span()) {
        return false;
    }

    // If component_name is provided, verify the object matches
    if let Some(name) = component_name {
        if let Expression::Identifier(ident) = &member.object {
            return ident.name == name;
        } else if let Expression::StaticMemberExpression(_) = &member.object {
            // Handle nested case like: Namespace.Component.displayName
            let path = extract_member_expression_path(&member.object);
            return path == name;
        }
        return false;
    }

    true
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

/// Check if a call expression is a createContext call
fn is_create_context_call(call: &oxc_ast::ast::CallExpression) -> bool {
    call.callee_name().is_some_and(|name| name == "createContext")
}

/// Check if a class extends React.Component or React.PureComponent
fn extends_react_component(class: &oxc_ast::ast::Class) -> bool {
    class.super_class.as_ref().is_some_and(|super_class| {
        if let Some(member_expr) = super_class.as_member_expression()
            && let Expression::Identifier(ident) = member_expr.object()
        {
            return ident.name == "React"
                && member_expr
                    .static_property_name()
                    .is_some_and(|name| name == "Component" || name == "PureComponent");
        }
        if let Some(ident_reference) = super_class.get_identifier_reference() {
            return ident_reference.name == "Component" || ident_reference.name == "PureComponent";
        }
        false
    })
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
        for ancestor_kind in ctx.nodes().ancestor_kinds(ref_node.id()) {
            if let AstKind::AssignmentExpression(assign) = ancestor_kind
                && let Expression::CallExpression(call) = &assign.right
                && is_create_context_call(call)
            {
                return Some(ReactComponentInfo { span: assign.span, is_context: true, name });
            }

            if matches!(ancestor_kind, AstKind::AssignmentExpression(_)) {
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
                && is_create_context_call(call)
            {
                if check_context_objects {
                    return Some(ReactComponentInfo {
                        span: decl.id.span(),
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
                        span: decl.id.span(),
                        is_context: false,
                        name: if inner_has_name { name } else { None },
                    });
                }

                // Check for createReactClass
                if callee_name == "createClass" || callee_name == "createReactClass" {
                    if !has_create_react_class_display_name(call, ignore_transpiler_name) {
                        return Some(ReactComponentInfo {
                            span: decl.id.span(),
                            is_context: false,
                            name,
                        });
                    }
                    return None;
                }
            }

            // Check for function/arrow function components with JSX
            if let Some(expr) = &decl.init {
                // Check if it's a direct component (has JSX directly)
                if expression_contains_jsx(expr) {
                    return Some(ReactComponentInfo {
                        span: decl.id.span(),
                        is_context: false,
                        name, // Direct component uses the variable name
                    });
                }

                // Check if it's a HOF pattern
                if let Some(innermost) = find_innermost_function_with_jsx(expr, ctx) {
                    let inner_has_name = match innermost {
                        InnermostFunction::Function(func) => func.id.is_some(),
                        InnermostFunction::ArrowFunction => false,
                    };

                    // Only treat as HOF if inner function is unnamed
                    if !inner_has_name {
                        return Some(ReactComponentInfo {
                            span: decl.id.span(),
                            is_context: false,
                            name: None, // HOF doesn't use variable name
                        });
                    }
                }

                // Not a component at all - return None
            }

            // Check for object expressions with methods
            if let Some(Expression::ObjectExpression(obj_expr)) = &decl.init
                && let Some(name) = &name
                && has_component_methods_in_object(obj_expr, ignore_transpiler_name)
            {
                return Some(ReactComponentInfo {
                    span: decl.id.span(),
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
                if class_has_static_display_name(class) {
                    return None; // Has static displayName
                }

                if class_contains_jsx(class) {
                    return Some(ReactComponentInfo {
                        span: class.span,
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
                                    is_context: false,
                                    name: None, // HOFs don't use the outer function name as displayName
                                });
                            }

                            // Check if it returns createReactClass
                            if let Expression::CallExpression(call) = expr
                                && let Some(callee_name) = call.callee_name()
                                && (callee_name == "createClass"
                                    || callee_name == "createReactClass")
                            {
                                if !has_create_react_class_display_name(
                                    call,
                                    ignore_transpiler_name,
                                ) {
                                    return Some(ReactComponentInfo {
                                        span: func.span,
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
            None
        }
        _ => None,
    }
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
            && ignore_transpiler_name
            && is_react_component_name(&ident.name)
            && matches!(&obj_prop.value, Expression::FunctionExpression(f) if function_contains_jsx(f))
        {
            return true;
        }
    }
    false
}

/// Handle anonymous export default components
fn is_anonymous_export_component(
    export: &oxc_ast::ast::ExportDefaultDeclaration,
    ignore_transpiler_name: bool,
) -> Option<ReactComponentInfo> {
    match &export.declaration {
        ExportDefaultDeclarationKind::ArrowFunctionExpression(func) => {
            // Uses visitor pattern to handle JSX in nested control flow
            if function_body_contains_jsx(&func.body) {
                return Some(ReactComponentInfo {
                    span: export.span,
                    is_context: false,
                    name: None,
                });
            }
        }
        ExportDefaultDeclarationKind::FunctionExpression(func) => {
            // Uses visitor pattern to handle JSX in nested control flow
            if function_contains_jsx(func) && (func.id.is_none() || ignore_transpiler_name) {
                return Some(ReactComponentInfo {
                    span: export.span,
                    is_context: false,
                    name: None,
                });
            }
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            if let Some(name) = &func.id
                && ignore_transpiler_name
                && is_react_component_name(&name.name)
                && function_contains_jsx(func)
            {
                return Some(ReactComponentInfo {
                    span: export.span,
                    is_context: false,
                    name: Some(CompactStr::from(name.name.as_str())),
                });
            }
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
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
    if class_has_static_display_name(class) {
        return None;
    }

    if !class_contains_jsx(class) {
        return None;
    }

    // If class has a name
    if let Some(name) = &class.id {
        if is_react_component_name(&name.name) {
            if ignore_transpiler_name {
                return Some(ReactComponentInfo {
                    span,
                    is_context: false,
                    name: Some(CompactStr::from(name.name.as_str())),
                });
            }

            if extends_react_component(class) {
                return Some(ReactComponentInfo { span, is_context: false, name: None });
            }
        }
    } else {
        // Anonymous class
        if ignore_transpiler_name {
            return Some(ReactComponentInfo { span, is_context: false, name: None });
        }

        if extends_react_component(class) {
            return Some(ReactComponentInfo { span, is_context: false, name: None });
        }
    }

    None
}

/// Handle module.exports assignments
fn is_module_exports_component(
    assign: &oxc_ast::ast::AssignmentExpression,
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
                // Uses visitor pattern to handle JSX in nested control flow
                if function_body_contains_jsx(&func.body) {
                    return Some(ReactComponentInfo {
                        span: assign.span,
                        is_context: false,
                        name: None,
                    });
                }
            }
            Expression::FunctionExpression(func) => {
                // Uses visitor pattern to handle JSX in nested control flow
                if function_contains_jsx(func) && (func.id.is_none() || ignore_transpiler_name) {
                    return Some(ReactComponentInfo {
                        span: assign.span,
                        is_context: false,
                        name: None,
                    });
                }
            }
            Expression::CallExpression(call) => {
                if let Some(callee_name) = call.callee_name() {
                    if callee_name == "createClass" || callee_name == "createReactClass" {
                        if !has_create_react_class_display_name(call, ignore_transpiler_name) {
                            return Some(ReactComponentInfo {
                                span: assign.span,
                                is_context: false,
                                name: None,
                            });
                        }
                    } else if is_create_context_call(call) && check_context_objects {
                        return Some(ReactComponentInfo {
                            span: assign.span,
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

/// Internal version checking function
fn check_react_version_internal(
    version: Option<&ReactVersion>,
    min_major: u32,
    min_minor: u32,
) -> bool {
    match version {
        Some(v) => {
            let major = v.major();
            let minor = v.minor();
            major >= min_major && (major > min_major || minor >= min_minor)
        }
        None => {
            // If no version specified, assume modern React
            true
        }
    }
}

/// Internal memo/forwardRef version checking function
fn test_react_version_for_memo_forwardref_internal(version: Option<&ReactVersion>) -> bool {
    match version {
        Some(v) => {
            let major = v.major();
            let minor = v.minor();
            let patch = v.patch();
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
        }
        None => {
            // If no version specified, assume modern React (>= 16.12.0)
            true
        }
    }
}

/// Check if a createReactClass call has a displayName property
fn has_create_react_class_display_name(
    call: &oxc_ast::ast::CallExpression,
    ignore_transpiler_name: bool,
) -> bool {
    call.arguments.iter().any(|arg| {
        if let Some(Expression::ObjectExpression(obj_expr)) = arg.as_expression() {
            obj_expr.properties.iter().any(|prop| {
                if let Some((prop_name, _)) = prop.prop_name() {
                    prop_name == "displayName" || (!ignore_transpiler_name && prop_name == "name")
                } else {
                    false
                }
            })
        } else {
            false
        }
    })
}

/// Check if a class has a static displayName property
fn class_has_static_display_name(class: &oxc_ast::ast::Class) -> bool {
    class.body.body.iter().any(|element| match element {
        ClassElement::MethodDefinition(method_def) => {
            method_def.r#static
                && method_def.key.static_name() == Some(std::borrow::Cow::Borrowed("displayName"))
        }
        ClassElement::PropertyDefinition(prop_def) => {
            prop_def.r#static
                && prop_def.key.static_name() == Some(std::borrow::Cow::Borrowed("displayName"))
        }
        _ => false,
    })
}

/// Check if a class contains JSX in any of its methods
fn class_contains_jsx(class: &oxc_ast::ast::Class) -> bool {
    class.body.body.iter().any(|element| {
        if let ClassElement::MethodDefinition(method_def) = element {
            return function_contains_jsx(&method_def.value);
        }
        false
    })
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
                serde_json::json!({ "settings": {        "react": {          "version": "16.4.0",        },      } }),
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
                serde_json::json!({ "settings": {        "react": {          "version": "16.4.0",        },      } }),
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
        // NOTE: The following pattern is not currently detected:
        // function HelloComponent() {
        //   return createReactClass({ render: function() { return <div /> } });
        // }
        // module.exports = HelloComponent();
        // This would require tracking function return values through semantic analysis,
        // which adds significant complexity. This is a rare pattern in practice.
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
