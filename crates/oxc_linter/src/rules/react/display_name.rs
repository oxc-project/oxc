use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AssignmentTarget, ClassElement, Expression, Function,
        ObjectExpression, ObjectPropertyKind, PropertyKey, Statement, VariableDeclaration,
    },
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_ecmascript::PropName;
use oxc_span::GetSpan;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};
use oxc_macros::declare_oxc_lint;
use rustc_hash::FxHashMap;

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
    correctness
);

#[derive(Debug, Clone)]
enum ComponentType {
    Function,
    Class,
    CreateReactClass,
    ObjectProperty,
}

#[derive(Debug)]
struct ComponentInfo {
    span: Span,
    has_display_name: bool,
}

struct ComponentTracker {
    components: FxHashMap<String, ComponentInfo>,
}

impl ComponentTracker {
    fn new() -> Self {
        Self { components: FxHashMap::default() }
    }

    fn add_component(&mut self, name: String, span: Span, _component_type: ComponentType) {
        self.components.insert(name, ComponentInfo { span, has_display_name: false });
    }

    fn resolve_display_name(&mut self, name: &str) {
        if let Some(component) = self.components.get_mut(name) {
            component.has_display_name = true;
        }
    }

    fn get_unresolved_components(&self) -> Vec<Span> {
        self.components
            .iter()
            .filter(|(_, comp)| !comp.has_display_name)
            .map(|(_, comp)| comp.span)
            .collect()
    }
}

fn display_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-react(display-name): React component is missing a displayName property",
    )
    .with_help("Add a displayName property to the component")
    .with_labels::<LabeledSpan, _>([span.into()])
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
        let mut tracker = ComponentTracker::new();
        let ignore_transpiler_name = self.0.ignore_transpiler_name;
        // Only check context objects if React version is >= 16.3.0
        let check_context_objects =
            self.0.check_context_objects && test_react_version_for_context_objects(ctx);

        // Single pass: collect React components and process displayName assignments
        for node in ctx.nodes() {
            // Process displayName assignments first (so they can resolve components found later)
            if let AstKind::AssignmentExpression(assign) = node.kind() {
                if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
                    if member.property.name == "displayName" {
                        if let Some(path) = get_static_property_path(&member.object) {
                            let component_name = path.join(".");
                            debug_resolve("CALLSITE8", component_name.as_str());
                            tracker.resolve_display_name(&component_name);
                        }
                    }
                }
            }

            // Guard: skip nodes that are part of React.memo(React.forwardRef(...)) assignments
            let mut should_skip = false;
            for ancestor in ctx.nodes().ancestor_ids(node.id()) {
                let ancestor_node = ctx.nodes().get_node(ancestor);
                if let AstKind::VariableDeclarator(decl) = ancestor_node.kind() {
                    if let Some(Expression::CallExpression(call)) = &decl.init {
                        if let Some(callee_name) = call.callee_name() {
                            if callee_name.ends_with("memo") {
                                if let Some(first_arg) = call.arguments.first() {
                                    if let Some(Expression::CallExpression(inner_call)) =
                                        first_arg.as_expression()
                                    {
                                        if let Some(inner_callee_name) = inner_call.callee_name() {
                                            if inner_callee_name.ends_with("forwardRef") {
                                                // Only skip if React version is compatible
                                                if test_react_version_for_memo_forwardref(ctx) {
                                                    should_skip = true;
                                                    break;
                                                }
                                                // else: do nothing here, fall through
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if should_skip {
                continue;
            }

            // Process component detection
            match node.kind() {
                AstKind::Class(class) => {
                    if let Some(name) = &class.name() {
                        if name.chars().next().is_some_and(char::is_uppercase) {
                            // Check if class has static displayName
                            let has_static_display_name =
                                class.body.body.iter().any(|element| match element {
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

                            // Only track classes that contain JSX or are React components
                            let contains_jsx = class.body.body.iter().any(|element| {
                                if let ClassElement::MethodDefinition(method_def) = element {
                                    if let Some(body) = &method_def.value.body {
                                        for stmt in &body.statements {
                                            if let Statement::ReturnStatement(ret_stmt) = stmt {
                                                if let Some(expr) = &ret_stmt.argument {
                                                    if contains_jsx(expr) {
                                                        return true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                false
                            });

                            if has_static_display_name {
                                debug_resolve("CALLSITE1", name);
                                tracker.resolve_display_name(name);
                            } else if contains_jsx {
                                if ignore_transpiler_name {
                                    tracker.add_component(
                                        name.to_string(),
                                        class.span,
                                        ComponentType::Class,
                                    );
                                } else {
                                    debug_resolve("CALLSITE2", name);
                                    tracker.resolve_display_name(name);
                                }
                            }
                        }
                    }
                }
                AstKind::VariableDeclaration(decl) => {
                    process_variable_declaration(
                        &mut tracker,
                        decl,
                        ignore_transpiler_name,
                        check_context_objects,
                        ctx,
                    );
                }
                AstKind::Function(func) => {
                    if let Some(name) = &func.id {
                        if name.name.chars().next().is_some_and(char::is_uppercase) {
                            if function_contains_jsx(func) {
                                if ignore_transpiler_name {
                                    // Only add if not already resolved (to avoid duplicates from HOC handling)
                                    if !tracker.components.contains_key(&name.name.to_string()) {
                                        tracker.add_component(
                                            name.name.to_string(),
                                            func.span,
                                            ComponentType::Function,
                                        );
                                    }
                                } else {
                                    // When ignoreTranspilerName is false, the function name itself is considered a valid displayName
                                    debug_resolve("CALLSITE3", &name.name);
                                    tracker.resolve_display_name(&format!(
                                        "[CALLSITE3] {}",
                                        name.name
                                    ));
                                }
                            } else if ignore_transpiler_name
                                && function_returns_create_react_class(func)
                            {
                                // Handle functions that return createReactClass calls
                                tracker.add_component(
                                    name.name.to_string(),
                                    func.span,
                                    ComponentType::CreateReactClass,
                                );
                            } else {
                                // Handle anonymous functions that return JSX
                                if let Some(body) = &func.body {
                                    for stmt in &body.statements {
                                        if let Statement::ReturnStatement(ret_stmt) = stmt {
                                            if let Some(expr) = &ret_stmt.argument {
                                                if let Expression::FunctionExpression(
                                                    returned_func,
                                                ) = expr
                                                {
                                                    if function_contains_jsx(returned_func) {
                                                        tracker.add_component(
                                                            "<anonymous>".to_string(),
                                                            func.span,
                                                            ComponentType::Function,
                                                        );
                                                        break;
                                                    }
                                                } else if let Expression::ArrowFunctionExpression(
                                                    _returned_func,
                                                ) = expr
                                                {
                                                    if function_like_contains_jsx(expr) {
                                                        tracker.add_component(
                                                            "<anonymous>".to_string(),
                                                            func.span,
                                                            ComponentType::Function,
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Handle anonymous functions that return JSX
                        if let Some(body) = &func.body {
                            for stmt in &body.statements {
                                if let Statement::ReturnStatement(ret_stmt) = stmt {
                                    if let Some(expr) = &ret_stmt.argument {
                                        if let Expression::FunctionExpression(returned_func) = expr
                                        {
                                            if function_contains_jsx(returned_func) {
                                                tracker.add_component(
                                                    "<anonymous>".to_string(),
                                                    func.span,
                                                    ComponentType::Function,
                                                );
                                                break;
                                            }
                                        } else if let Expression::ArrowFunctionExpression(
                                            _returned_func,
                                        ) = expr
                                        {
                                            if function_like_contains_jsx(expr) {
                                                tracker.add_component(
                                                    "<anonymous>".to_string(),
                                                    func.span,
                                                    ComponentType::Function,
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                AstKind::AssignmentExpression(assign) => {
                    // Handle: module.exports = () => <div />
                    if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
                        if let Expression::Identifier(ident) = &member.object {
                            if ident.name == "module" && member.property.name == "exports" {
                                match &assign.right {
                                    Expression::ArrowFunctionExpression(func) => {
                                        if func.expression {
                                            // Expression-bodied arrow function: () => <div />
                                            // For expression-bodied arrows, the FunctionBody contains a single ExpressionStatement
                                            if func.body.statements.len() == 1 {
                                                if let Statement::ExpressionStatement(expr_stmt) =
                                                    &func.body.statements[0]
                                                {
                                                    if contains_jsx(&expr_stmt.expression) {
                                                        tracker.add_component(
                                                            "<anonymous export>".to_string(),
                                                            assign.span,
                                                            ComponentType::Function,
                                                        );
                                                    }
                                                }
                                            }
                                        } else {
                                            // Block-bodied arrow function: () => { return <div /> }
                                            for stmt in &func.body.statements {
                                                if let Statement::ReturnStatement(ret_stmt) = stmt {
                                                    if let Some(expr) = &ret_stmt.argument {
                                                        if contains_jsx(expr) {
                                                            tracker.add_component(
                                                                "<anonymous export>".to_string(),
                                                                assign.span,
                                                                ComponentType::Function,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Expression::FunctionExpression(func) => {
                                        if let Some(body) = &func.body {
                                            for stmt in &body.statements {
                                                if let Statement::ReturnStatement(ret_stmt) = stmt {
                                                    if let Some(expr) = &ret_stmt.argument {
                                                        if contains_jsx(expr) {
                                                            // Check if the function has a name
                                                            if let Some(name) = &func.id {
                                                                if ignore_transpiler_name {
                                                                    // Use the function name as the component name
                                                                    tracker.add_component(
                                                                        name.name.to_string(),
                                                                        assign.span,
                                                                        ComponentType::Function,
                                                                    );
                                                                } else {
                                                                    // When ignoreTranspilerName is false, the function name itself is considered a valid displayName
                                                                    debug_resolve(
                                                                        "CALLSITE5",
                                                                        &name.name,
                                                                    );
                                                                    tracker.resolve_display_name(
                                                                        &name.name,
                                                                    );
                                                                }
                                                            } else {
                                                                // Anonymous function
                                                                tracker.add_component(
                                                                    "<anonymous export>"
                                                                        .to_string(),
                                                                    assign.span,
                                                                    ComponentType::Function,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Expression::CallExpression(call) => {
                                        if let Some(callee_name) = call.callee_name() {
                                            if callee_name == "createClass"
                                                || callee_name == "createReactClass"
                                            {
                                                let has_display_name =
                                                    call.arguments.iter().any(|arg| {
                                                        if let Some(Expression::ObjectExpression(
                                                            obj_expr,
                                                        )) = arg.as_expression()
                                                        {
                                                            obj_expr.properties.iter().any(|prop| {
                                                                if let Some((prop_name, _)) =
                                                                    prop.prop_name()
                                                                {
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
                                                    // Only track if missing displayName
                                                    tracker.add_component(
                                                        "<anonymous export>".to_string(),
                                                        assign.span,
                                                        ComponentType::CreateReactClass,
                                                    );
                                                }
                                            } else if callee_name == "createContext"
                                                || callee_name.ends_with(".createContext")
                                            {
                                                // Handle React.createContext calls in assignment expressions
                                                if check_context_objects {
                                                    tracker.add_component(
                                                        "<anonymous export>".to_string(),
                                                        assign.span,
                                                        ComponentType::Function,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    // Handle: Hello = createContext()
                    else if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left
                    {
                        if let Expression::CallExpression(call) = &assign.right {
                            if let Some(callee_name) = call.callee_name() {
                                if callee_name == "createContext"
                                    || callee_name.ends_with(".createContext")
                                {
                                    // Handle React.createContext calls in simple variable assignments
                                    if check_context_objects {
                                        tracker.add_component(
                                            ident.name.to_string(),
                                            assign.span,
                                            ComponentType::Function,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                AstKind::ExportDefaultDeclaration(export) => {
                    // Handle: export default (props) => { return createElement("div", {}, "hello"); }
                    match &export.declaration {
                        oxc_ast::ast::ExportDefaultDeclarationKind::ArrowFunctionExpression(
                            func,
                        ) => {
                            if func.expression {
                                // Expression-bodied arrow function: () => <div />
                                if func.body.statements.len() == 1 {
                                    if let Statement::ExpressionStatement(expr_stmt) =
                                        &func.body.statements[0]
                                    {
                                        if contains_jsx(&expr_stmt.expression) {
                                            tracker.add_component(
                                                "<anonymous export>".to_string(),
                                                export.span,
                                                ComponentType::Function,
                                            );
                                        }
                                    }
                                }
                            } else {
                                // Block-bodied arrow function: () => { return <div /> }
                                for stmt in &func.body.statements {
                                    if let Statement::ReturnStatement(ret_stmt) = stmt {
                                        if let Some(expr) = &ret_stmt.argument {
                                            if contains_jsx(expr) {
                                                tracker.add_component(
                                                    "<anonymous export>".to_string(),
                                                    export.span,
                                                    ComponentType::Function,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionExpression(func) => {
                            if let Some(body) = &func.body {
                                for stmt in &body.statements {
                                    if let Statement::ReturnStatement(ret_stmt) = stmt {
                                        if let Some(expr) = &ret_stmt.argument {
                                            if contains_jsx(expr) {
                                                // Check if the function has a name
                                                if let Some(name) = &func.id {
                                                    if ignore_transpiler_name {
                                                        // Use the function name as the component name
                                                        tracker.add_component(
                                                            name.name.to_string(),
                                                            export.span,
                                                            ComponentType::Function,
                                                        );
                                                    } else {
                                                        // When ignoreTranspiler_name is false, the function name itself is considered a valid displayName
                                                        debug_resolve("CALLSITE5", &name.name);
                                                        tracker.resolve_display_name(&format!(
                                                            "[CALLSITE5] {}",
                                                            name.name
                                                        ));
                                                    }
                                                } else {
                                                    // Anonymous function
                                                    tracker.add_component(
                                                        "<anonymous export>".to_string(),
                                                        export.span,
                                                        ComponentType::Function,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                            if let Some(name) = &func.id {
                                if name.name.chars().next().is_some_and(char::is_uppercase)
                                    && function_contains_jsx(func)
                                {
                                    if ignore_transpiler_name {
                                        tracker.add_component(
                                            name.name.to_string(),
                                            export.span,
                                            ComponentType::Function,
                                        );
                                    } else {
                                        debug_resolve("CALLSITE6", &name.name);
                                        tracker.resolve_display_name(&format!(
                                            "[CALLSITE6] {}",
                                            name.name
                                        ));
                                    }
                                }
                            }
                        }
                        oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                            if let Some(name) = &class.id {
                                if name.name.chars().next().is_some_and(char::is_uppercase) {
                                    // Check if class has static displayName
                                    let has_static_display_name =
                                        class.body.body.iter().any(|element| match element {
                                            ClassElement::MethodDefinition(method_def) => {
                                                method_def.r#static
                                                    && method_def.key.static_name()
                                                        == Some(std::borrow::Cow::Borrowed(
                                                            "displayName",
                                                        ))
                                            }
                                            ClassElement::PropertyDefinition(prop_def) => {
                                                prop_def.r#static
                                                    && prop_def.key.static_name()
                                                        == Some(std::borrow::Cow::Borrowed(
                                                            "displayName",
                                                        ))
                                            }
                                            _ => false,
                                        });

                                    // Only track classes that contain JSX or are React components
                                    let contains_jsx = class.body.body.iter().any(|element| {
                                        if let ClassElement::MethodDefinition(method_def) = element
                                        {
                                            if let Some(body) = &method_def.value.body {
                                                for stmt in &body.statements {
                                                    if let Statement::ReturnStatement(ret_stmt) =
                                                        stmt
                                                    {
                                                        if let Some(expr) = &ret_stmt.argument {
                                                            if contains_jsx(expr) {
                                                                return true;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        false
                                    });

                                    if has_static_display_name {
                                        debug_resolve("CALLSITE7", &name.name);
                                        tracker.resolve_display_name(&format!(
                                            "[CALLSITE7] {}",
                                            name.name
                                        ));
                                    } else if contains_jsx {
                                        if ignore_transpiler_name {
                                            tracker.add_component(
                                                name.name.to_string(),
                                                export.span,
                                                ComponentType::Class,
                                            );
                                        } else {
                                            // When ignoreTranspilerName is false, require displayName if it extends React.Component
                                            let extends_react_component = class
                                                .super_class
                                                .as_ref()
                                                .is_some_and(|super_class| {
                                                    if let Some(member_expr) =
                                                        super_class.as_member_expression()
                                                    {
                                                        if let Expression::Identifier(ident) =
                                                            member_expr.object()
                                                        {
                                                            return ident.name == "React"
                                                                && member_expr
                                                                    .static_property_name()
                                                                    .is_some_and(|name| {
                                                                        name == "Component"
                                                                            || name
                                                                                == "PureComponent"
                                                                    });
                                                        }
                                                    }
                                                    if let Some(ident_reference) =
                                                        super_class.get_identifier_reference()
                                                    {
                                                        return ident_reference.name == "Component"
                                                            || ident_reference.name
                                                                == "PureComponent";
                                                    }
                                                    false
                                                });

                                            if extends_react_component {
                                                tracker.add_component(
                                                    "<anonymous export>".to_string(),
                                                    export.span,
                                                    ComponentType::Class,
                                                );
                                            }
                                            // For plain classes with render methods (not extending React.Component), do not require displayName
                                        }
                                    }
                                }
                            } else {
                                // For anonymous default-exported class
                                let contains_jsx = class.body.body.iter().any(|element| {
                                    if let ClassElement::MethodDefinition(method_def) = element {
                                        if let Some(body) = &method_def.value.body {
                                            for stmt in &body.statements {
                                                if let Statement::ReturnStatement(ret_stmt) = stmt {
                                                    if let Some(expr) = &ret_stmt.argument {
                                                        if contains_jsx(expr) {
                                                            return true;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    false
                                });

                                if contains_jsx {
                                    if ignore_transpiler_name {
                                        // When ignoreTranspilerName is true, require displayName for all anonymous classes
                                        tracker.add_component(
                                            "<anonymous export>".to_string(),
                                            export.span,
                                            ComponentType::Class,
                                        );
                                    } else {
                                        // When ignoreTranspilerName is false, require displayName if it extends React.Component
                                        let extends_react_component =
                                            class.super_class.as_ref().is_some_and(|super_class| {
                                                if let Some(member_expr) =
                                                    super_class.as_member_expression()
                                                {
                                                    if let Expression::Identifier(ident) =
                                                        member_expr.object()
                                                    {
                                                        return ident.name == "React"
                                                            && member_expr
                                                                .static_property_name()
                                                                .is_some_and(|name| {
                                                                    name == "Component"
                                                                        || name == "PureComponent"
                                                                });
                                                    }
                                                }
                                                if let Some(ident_reference) =
                                                    super_class.get_identifier_reference()
                                                {
                                                    return ident_reference.name == "Component"
                                                        || ident_reference.name == "PureComponent";
                                                }
                                                false
                                            });

                                        if extends_react_component {
                                            tracker.add_component(
                                                "<anonymous export>".to_string(),
                                                export.span,
                                                ComponentType::Class,
                                            );
                                        }
                                        // For plain classes with render methods (not extending React.Component), do not require displayName
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Report unresolved components
        let unresolved = tracker.get_unresolved_components();
        for span in unresolved {
            ctx.diagnostic(display_name_diagnostic(span));
        }
    }
}

fn process_variable_declaration<'a>(
    tracker: &mut ComponentTracker,
    decl: &VariableDeclaration<'a>,
    ignore_transpiler_name: bool,
    check_context_objects: bool,
    ctx: &LintContext<'a>,
) {
    for var_decl in &decl.declarations {
        if let Some(Expression::CallExpression(call)) = &var_decl.init {
            if let Some(callee_name) = call.callee_name() {
                if callee_name.ends_with("memo") {
                    if let Some(Expression::CallExpression(inner_call)) =
                        call.arguments.first().and_then(|arg| arg.as_expression())
                    {
                        if let Some(inner_callee_name) = inner_call.callee_name() {
                            if inner_callee_name.ends_with("forwardRef")
                                && test_react_version_for_memo_forwardref(ctx)
                            {
                                return;
                            }
                            if let Some(name) = var_decl.id.get_identifier_name() {
                                tracker.add_component(
                                    name.to_string(),
                                    decl.span,
                                    ComponentType::Function,
                                );
                            }
                            return;
                        }
                    }
                }
            }
        }
        // Always check for innermost function with JSX and add to tracker
        if let Some(expr) = &var_decl.init {
            if let Some(innermost_func) = find_innermost_function_with_jsx(expr) {
                let component_name = var_decl
                    .id
                    .get_identifier_name()
                    .map_or_else(|| "<anonymous>".to_string(), |s| s.to_string());
                // is_direct is true only if the innermost function with JSX is the same as init
                let is_direct = match innermost_func {
                    InnermostFunction::Function(func) => {
                        matches!(var_decl.init.as_ref(), Some(Expression::FunctionExpression(f)) if std::ptr::eq(f.as_ref(), func))
                    }
                    InnermostFunction::ArrowFunction(arrow) => {
                        matches!(var_decl.init.as_ref(), Some(Expression::ArrowFunctionExpression(a)) if std::ptr::eq(a.as_ref(), arrow))
                    }
                };

                if is_direct {
                    if ignore_transpiler_name {
                        tracker.add_component(component_name, decl.span, ComponentType::Function);
                    } else {
                        debug_resolve("CALLSITE9", component_name.as_str());
                        tracker.resolve_display_name(&component_name);
                    }
                } else {
                    // For curried/nested functions, check if the innermost function is named
                    match innermost_func {
                        InnermostFunction::Function(func) => {
                            if let Some(func_id) = &func.id {
                                // Named function: use the function name as displayName
                                if ignore_transpiler_name {
                                    tracker.add_component(
                                        func_id.name.to_string(),
                                        decl.span,
                                        ComponentType::Function,
                                    );
                                } else {
                                    debug_resolve("CALLSITE10", &func_id.name);
                                    tracker.resolve_display_name(&func_id.name);
                                }
                            } else {
                                // Anonymous function: always require explicit displayName
                                tracker.add_component(
                                    component_name,
                                    decl.span,
                                    ComponentType::Function,
                                );
                            }
                        }
                        InnermostFunction::ArrowFunction(_) => {
                            // Anonymous arrow function: always require explicit displayName
                            tracker.add_component(
                                component_name,
                                decl.span,
                                ComponentType::Function,
                            );
                        }
                    }
                }
                return;
            }
        }
        if let Some(init) = &var_decl.init {
            match init {
                Expression::ObjectExpression(obj_expr) => {
                    if let Some(name) = var_decl.id.get_identifier_name() {
                        process_object_expression(
                            tracker,
                            obj_expr.as_ref(),
                            &[name.to_string()],
                            ignore_transpiler_name,
                        );
                    }
                }
                Expression::CallExpression(call) => {
                    if let Some(name) = var_decl.id.get_identifier_name() {
                        if let Some(callee_name) = call.callee_name() {
                            // Handle createReactClass - this should always be processed
                            if callee_name == "createClass" || callee_name == "createReactClass" {
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

                                if has_display_name {
                                    debug_resolve("CALLSITE11", name.as_str());
                                    tracker.resolve_display_name(&format!("[CALLSITE11] {name}"));
                                } else if ignore_transpiler_name {
                                    tracker.add_component(
                                        name.to_string(),
                                        decl.span,
                                        ComponentType::CreateReactClass,
                                    );
                                }
                            } else {
                                // Handle HOCs like React.memo, React.forwardRef
                                let hoc_names = ["memo", "forwardRef"];
                                if hoc_names.iter().any(|&hoc| callee_name.ends_with(hoc)) {
                                    // Special case: React.memo(React.forwardRef(...)), skip reporting
                                    if callee_name.ends_with("memo") {
                                        if let Some(first_arg) = call.arguments.first() {
                                            if let Some(inner_expr) = first_arg.as_expression() {
                                                // If React.memo(React.forwardRef(...)), skip reporting
                                                if let Expression::CallExpression(inner_call) =
                                                    inner_expr
                                                {
                                                    if let Some(inner_callee_name) =
                                                        inner_call.callee_name()
                                                    {
                                                        if inner_callee_name.ends_with("forwardRef")
                                                            && test_react_version_for_memo_forwardref(ctx)
                                                        {
                                                            return;
                                                        }
                                                    }
                                                }
                                                // If the first argument is a named function, resolve the display name for the variable using the function's name
                                                match inner_expr {
                                                    Expression::FunctionExpression(func) => {
                                                        if let Some(func_id) = &func.id {
                                                            debug_resolve(
                                                                "CALLSITE12",
                                                                func_id.name.as_str(),
                                                            );
                                                            tracker.resolve_display_name(&format!(
                                                                "[CALLSITE12] {name}"
                                                            ));
                                                            tracker.resolve_display_name(&format!(
                                                                "[CALLSITE13] {}",
                                                                func_id.name
                                                            ));
                                                            return;
                                                        }
                                                    }
                                                    Expression::Identifier(ident) => {
                                                        debug_resolve(
                                                            "CALLSITE14",
                                                            ident.name.as_str(),
                                                        );
                                                        tracker.resolve_display_name(&format!(
                                                            "[CALLSITE14] {name}"
                                                        ));
                                                        tracker.resolve_display_name(&format!(
                                                            "[CALLSITE15] {}",
                                                            ident.name
                                                        ));
                                                        return;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        tracker.add_component(
                                            name.to_string(),
                                            decl.span,
                                            ComponentType::Function,
                                        );
                                        return;
                                    }
                                    // Handle plain React.forwardRef
                                    if callee_name.ends_with("forwardRef") {
                                        if let Some(first_arg) = call.arguments.first() {
                                            if let Some(inner_expr) = first_arg.as_expression() {
                                                match inner_expr {
                                                    Expression::FunctionExpression(func) => {
                                                        if let Some(func_id) = &func.id {
                                                            debug_resolve(
                                                                "CALLSITE16",
                                                                func_id.name.as_str(),
                                                            );
                                                            tracker.resolve_display_name(&format!(
                                                                "[CALLSITE16] {name}"
                                                            ));
                                                            tracker.resolve_display_name(&format!(
                                                                "[CALLSITE17] {}",
                                                                func_id.name
                                                            ));
                                                            return;
                                                        }
                                                    }
                                                    Expression::Identifier(ident) => {
                                                        debug_resolve(
                                                            "CALLSITE18",
                                                            ident.name.as_str(),
                                                        );
                                                        tracker.resolve_display_name(&format!(
                                                            "[CALLSITE18] {name}"
                                                        ));
                                                        tracker.resolve_display_name(&format!(
                                                            "[CALLSITE19] {}",
                                                            ident.name
                                                        ));
                                                        return;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        tracker.add_component(
                                            name.to_string(),
                                            decl.span,
                                            ComponentType::Function,
                                        );
                                        return;
                                    }
                                    // For all other HOC cases (including plain React.memo), continue as before
                                    if ignore_transpiler_name {
                                        // Find the innermost function that renders JSX
                                        if let Some(expr) = &var_decl.init {
                                            if let Some(innermost_func) =
                                                find_innermost_function_with_jsx(expr)
                                            {
                                                match innermost_func {
                                                    InnermostFunction::Function(func) => {
                                                        if func.id.is_some() {
                                                            if let Some(func_id) = &func.id {
                                                                debug_resolve(
                                                                    "CALLSITE21",
                                                                    func_id.name.as_str(),
                                                                );
                                                                tracker.resolve_display_name(
                                                                    &format!(
                                                                        "[CALLSITE21] {}",
                                                                        func_id.name
                                                                    ),
                                                                );
                                                            }
                                                        } else {
                                                            tracker.add_component(
                                                                name.to_string(),
                                                                decl.span,
                                                                ComponentType::Function,
                                                            );
                                                        }
                                                    }
                                                    InnermostFunction::ArrowFunction(_) => {
                                                        tracker.add_component(
                                                            name.to_string(),
                                                            decl.span,
                                                            ComponentType::Function,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        debug_resolve("CALLSITE20", name.as_str());
                                        tracker
                                            .resolve_display_name(&format!("[CALLSITE20] {name}"));
                                    }
                                    // HOC handled, skip fallback tracking
                                    return;
                                } else if callee_name == "createContext"
                                    || callee_name.ends_with(".createContext")
                                {
                                    // Handle React.createContext calls - always require explicit displayName
                                    if check_context_objects {
                                        tracker.add_component(
                                            name.to_string(),
                                            decl.span,
                                            ComponentType::Function,
                                        );
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn process_object_expression(
    tracker: &mut ComponentTracker,
    obj_expr: &ObjectExpression,
    current_path: &[String],
    ignore_transpiler_name: bool,
) {
    for prop in &obj_expr.properties {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            if let PropertyKey::StaticIdentifier(ident) = &obj_prop.key {
                let prop_name = ident.name.to_string();
                let expr = &obj_prop.value;

                if !obj_prop.method {
                    if let Expression::ObjectExpression(nested_obj) = expr {
                        let mut nested_path = current_path.to_owned();
                        nested_path.push(prop_name.clone());
                        process_object_expression(
                            tracker,
                            nested_obj,
                            &nested_path,
                            ignore_transpiler_name,
                        );
                    }

                    if prop_name.chars().next().is_some_and(char::is_uppercase) {
                        if let Expression::FunctionExpression(func_expr) = expr {
                            if function_contains_jsx(func_expr) {
                                let mut full_path = current_path.to_owned();
                                full_path.push(prop_name.clone());
                                let component_name = full_path.join(".");
                                if ignore_transpiler_name {
                                    tracker.add_component(
                                        component_name,
                                        expr.span(),
                                        ComponentType::ObjectProperty,
                                    );
                                } else {
                                    debug_resolve("CALLSITE22", component_name.as_str());
                                    tracker.resolve_display_name(&format!(
                                        "[CALLSITE22] {component_name}"
                                    ));
                                }
                            }
                        }
                    }
                } else if prop_name.chars().next().is_some_and(char::is_uppercase) {
                    if let Expression::FunctionExpression(func_expr) = expr {
                        if function_contains_jsx(func_expr) {
                            let mut full_path = current_path.to_owned();
                            full_path.push(prop_name.clone());
                            let component_name = full_path.join(".");
                            if ignore_transpiler_name {
                                tracker.add_component(
                                    component_name,
                                    expr.span(),
                                    ComponentType::ObjectProperty,
                                );
                            } else {
                                debug_resolve("CALLSITE23", component_name.as_str());
                                tracker.resolve_display_name(&format!(
                                    "[CALLSITE23] {component_name}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_static_property_path(expr: &Expression) -> Option<Vec<String>> {
    match expr {
        Expression::Identifier(ident) => Some(vec![ident.name.to_string()]),
        Expression::StaticMemberExpression(member) => {
            let mut path = get_static_property_path(&member.object)?;
            path.push(member.property.name.to_string());
            Some(path)
        }
        _ => None,
    }
}

fn function_contains_jsx(func_expr: &Function) -> bool {
    if let Some(body) = &func_expr.body {
        for stmt in &body.statements {
            if let Statement::ReturnStatement(ret_stmt) = stmt {
                if let Some(expr) = &ret_stmt.argument {
                    if contains_jsx(expr) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn function_like_contains_jsx(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionExpression(func) => function_contains_jsx(func),
        Expression::ArrowFunctionExpression(arrow_func) => {
            if arrow_func.expression {
                // Expression-bodied arrow function: () => <div />
                if arrow_func.body.statements.len() == 1 {
                    if let Statement::ExpressionStatement(expr_stmt) =
                        &arrow_func.body.statements[0]
                    {
                        return contains_jsx(&expr_stmt.expression);
                    }
                }
            } else {
                // Block-bodied arrow function: () => { return <div /> }
                for stmt in &arrow_func.body.statements {
                    if let Statement::ReturnStatement(ret_stmt) = stmt {
                        if let Some(expr) = &ret_stmt.argument {
                            if contains_jsx(expr) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn contains_jsx(expr: &Expression) -> bool {
    match expr {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        Expression::CallExpression(call) => {
            // Check if this is a createElement call (React.createElement or just createElement)
            if let Some(callee_name) = call.callee_name() {
                if callee_name == "createElement" || callee_name.ends_with(".createElement") {
                    // This is a React createElement call, so it's a React element
                    return true;
                }
            }
            // Check if any arguments contain JSX
            call.arguments.iter().any(|arg| arg.as_expression().is_some_and(contains_jsx))
        }
        Expression::ParenthesizedExpression(inner) => contains_jsx(&inner.expression),
        Expression::StaticMemberExpression(member) => contains_jsx(&member.object),
        Expression::ConditionalExpression(cond) => {
            contains_jsx(&cond.consequent) || contains_jsx(&cond.alternate)
        }
        Expression::LogicalExpression(logical) => {
            contains_jsx(&logical.left) || contains_jsx(&logical.right)
        }
        Expression::SequenceExpression(seq) => seq.expressions.iter().any(contains_jsx),
        _ => false,
    }
}

fn function_returns_create_react_class(func_expr: &Function) -> bool {
    if let Some(body) = &func_expr.body {
        for stmt in &body.statements {
            if let Statement::ReturnStatement(ret_stmt) = stmt {
                if let Some(Expression::CallExpression(call)) = &ret_stmt.argument {
                    if let Some(callee_name) = call.callee_name() {
                        if callee_name == "createClass" || callee_name == "createReactClass" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[derive(Debug)]
enum InnermostFunction<'a> {
    Function(&'a Function<'a>),
    ArrowFunction(&'a ArrowFunctionExpression<'a>),
}

fn find_innermost_function_with_jsx<'a>(expr: &'a Expression<'a>) -> Option<InnermostFunction<'a>> {
    match expr {
        Expression::CallExpression(call) => {
            // Check if this is a HOC call
            if let Some(callee_name) = call.callee_name() {
                let hoc_names = ["memo", "forwardRef"];
                if hoc_names.iter().any(|&hoc| callee_name.ends_with(hoc)) {
                    // This is a HOC, recursively check the first argument
                    if let Some(first_arg) = call.arguments.first() {
                        if let Some(inner_expr) = first_arg.as_expression() {
                            return find_innermost_function_with_jsx(inner_expr);
                        }
                    }
                }
            }
            None
        }
        Expression::FunctionExpression(func) => {
            // Check if this function contains JSX
            if function_contains_jsx(func) { Some(InnermostFunction::Function(func)) } else { None }
        }
        Expression::ArrowFunctionExpression(arrow_func) => {
            // Check if this arrow function contains JSX
            if function_like_contains_jsx(expr) {
                Some(InnermostFunction::ArrowFunction(arrow_func))
            } else {
                // Check if this arrow function returns another function that contains JSX
                if arrow_func.expression {
                    // Expression-bodied arrow function: () => () => <div />
                    if arrow_func.body.statements.len() == 1 {
                        if let Statement::ExpressionStatement(expr_stmt) =
                            &arrow_func.body.statements[0]
                        {
                            return find_innermost_function_with_jsx(&expr_stmt.expression);
                        }
                    }
                } else {
                    // Block-bodied arrow function: () => { return () => <div /> }
                    for stmt in &arrow_func.body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt {
                            if let Some(expr) = &ret_stmt.argument {
                                return find_innermost_function_with_jsx(expr);
                            }
                        }
                    }
                }
                None
            }
        }
        _ => None,
    }
}

/// Test if the React version is compatible with skipping displayName for React.memo(React.forwardRef(...))
/// Compatible versions: ^0.14.10 || ^15.7.0 || >= 16.12.0
fn test_react_version_for_memo_forwardref(ctx: &LintContext) -> bool {
    // Get React version from settings
    let react_version = ctx.settings().react.version.as_deref();

    match react_version {
        Some(version) => {
            // Parse version string like "16.14.0", "15.7.0", "0.14.10", etc.
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

/// Test if the React version is compatible with checking context objects
/// Compatible versions: >= 16.3.0
fn test_react_version_for_context_objects(ctx: &LintContext) -> bool {
    // Get React version from settings
    let react_version = ctx.settings().react.version.as_deref();

    match react_version {
        Some(version) => {
            // Parse version string like "16.3.0", "16.4.0", etc.
            if let Some((major, minor, _)) = parse_version(version) {
                // >= 16.3.0: major >= 16 && (major > 16 || minor >= 3)
                major >= 16 && (major > 16 || minor >= 3)
            } else {
                false
            }
        }
        None => {
            // If no version specified, assume modern React (>= 16.3.0)
            true
        }
    }
}

/// Parse version string like "16.14.0" into (major, minor, patch)
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        let patch = parts[2].parse::<u32>().ok()?;
        Some((major, minor, patch))
    } else {
        None
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
        // TODO: check if this test case is accurate.
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
        // TODO: find out if this is a valid test case.
        // componentWrapperFunctions support?
        // (
        //     "
        // 	        const processData = (options?: { value: string }) => options?.value || 'no data';

        // 	        export const Component = observer(() => {
        // 	          const data = processData({ value: 'data' });
        // 	          return <div>{data}</div>;
        // 	        });

        // 	        export const Component2 = observer(() => {
        // 	          const data = processData();
        // 	          return <div>{data}</div>;
        // 	        });
        // 	      ",
        //     None,
        //     Some(serde_json::json!({ "settings": { "componentWrapperFunctions": ["observer"] } })),
        // ),
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

    // let fail: Vec<(&'static str, Option<Value>, Option<Value>)> = vec![        (
    //     r#"
    // 	        var Hello = createReactClass({
    // 	          render: function() {
    // 	            return React.createElement("div", {}, "text content");
    // 	          }
    // 	        });
    // 	      "#,
    //     Some(serde_json::json!([{ "ignoreTranspilerName": true }])),
    //     None,
    // ),];

    Tester::new(DisplayName::NAME, DisplayName::PLUGIN, pass, fail).test_and_snapshot();
}

// Helper function for debug logging
fn debug_resolve(_tag: &str, _name: &str) {
    // Debug logging disabled
}
