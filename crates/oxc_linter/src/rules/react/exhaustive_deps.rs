use std::{borrow::Cow, hash::Hash};

use itertools::Itertools;
use lazy_regex::Regex;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind, AstType,
    ast::{
        Argument, ArrayExpressionElement, ArrowFunctionExpression, BindingPattern,
        BindingPatternKind, CallExpression, ChainElement, Expression, FormalParameters, Function,
        FunctionBody, IdentifierReference, StaticMemberExpression, TSTypeAnnotation,
        TSTypeParameterInstantiation, TSTypeReference, VariableDeclarationKind, VariableDeclarator,
    },
    match_expression,
};
use oxc_ast_visit::{Visit, walk::walk_function_body};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, ScopeId, Semantic, SymbolId};
use oxc_span::{Atom, GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{
        get_declaration_from_reference_id, get_declaration_of_variable, get_enclosing_function,
    },
    context::LintContext,
    rule::Rule,
};

const SCOPE: &str = "eslint-plugin-react-hooks";

fn missing_callback_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("React hook {hook_name} requires an effect callback."))
        .with_label(span)
        .with_help("Did you forget to pass a callback to the hook?")
        .with_error_code_scope(SCOPE)
}

fn dependency_array_required_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} does nothing when called with only one argument."
    ))
    .with_label(span)
    .with_help("Did you forget to pass an array of dependencies?")
    .with_error_code_scope(SCOPE)
}

fn unknown_dependencies_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} received a function whose dependencies are unknown."
    ))
    .with_help("Pass an inline function instead.")
    .with_label(span)
    .with_error_code_scope(SCOPE)
}

fn async_effect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Effect callbacks are synchronous to prevent race conditions.")
        .with_label(span)
        .with_help("Consider putting the asynchronous code inside a function and calling it from the effect.")
        .with_error_code_scope(SCOPE)
}

fn missing_dependency_diagnostic(hook_name: &str, deps: &[Name<'_>], span: Span) -> OxcDiagnostic {
    let single = deps.len() == 1;
    let deps_pretty = if single {
        format!("'{}'", deps[0])
    } else {
        let mut iter = deps.iter();
        let all_but_last = iter
            .by_ref()
            .take(deps.len() - 1)
            .map(|s| format!("'{s}'",))
            .collect::<Vec<_>>()
            .join(", ");
        let last = iter.next().unwrap();
        format!("{all_but_last}, and '{last}'")
    };

    let labels = deps
        .iter()
        .map(|dep| {
            // when multiple dependencies are missing, labels can quickly get noisy,
            // so we only add labels when there's only one dependency
            if single {
                dep.span.label(format!("{hook_name} uses `{dep}` here"))
            } else {
                dep.span.into()
            }
        })
        .chain(std::iter::once(span.primary()));

    OxcDiagnostic::warn(if single {
        format!("React Hook {hook_name} has a missing dependency: {deps_pretty}")
    } else {
        format!("React Hook {hook_name} has missing dependencies: {deps_pretty}")
    })
    .with_labels(labels)
    .with_help("Either include it or remove the dependency array.")
    .with_error_code_scope(SCOPE)
}

fn unnecessary_dependency_diagnostic(hook_name: &str, dep_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("React Hook {hook_name} has unnecessary dependency: {dep_name}"))
        .with_label(span)
        .with_help("Either include it or remove the dependency array.")
        .with_error_code_scope(SCOPE)
}

fn dependency_array_not_array_literal_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} was passed a dependency list that is not an array literal. This means we can't statically verify whether you've passed the correct dependencies."
    ))
    .with_label(span)
    .with_help("Use an array literal as the second argument.")
    .with_error_code_scope(SCOPE)
}

fn literal_in_dependency_array_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The literal is not a valid dependency because it never changes.")
        .with_label(span)
        .with_help("Remove the literal from the array.")
        .with_error_code_scope(SCOPE)
}

fn duplicate_dependency_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This dependency is specified more than once in the dependency array.")
        .with_label(span)
        .with_help("Remove the duplicate dependency from the array.")
        .with_error_code_scope(SCOPE)
}

fn complex_expression_in_dependency_array_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} has a complex expression in the dependency array.",
    ))
    .with_label(span)
    .with_help("Extract the expression to a separate variable so it can be statically checked.")
    .with_error_code_scope(SCOPE)
}

fn dependency_changes_on_every_render_diagnostic(
    hook_name: &str,
    span: Span,
    dep_name: &str,
    dep_decl_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React hook {hook_name} depends on `{dep_name}`, which changes every render"
    ))
    .with_labels([
        span.primary_label("it will always cause this hook to re-evaluate"),
        dep_decl_span.label(format!("`{dep_name}` is declared here")),
    ])
    .with_help("Try memoizing this variable with `useRef` or `useCallback`.")
    .with_error_code_scope(SCOPE)
}

fn unnecessary_outer_scope_dependency_diagnostic(
    hook_name: &str,
    dep_name: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} has an unnecessary dependency: {dep_name}."
    ))
    .with_label(span)
    .with_help("Consider removing it from the dependency array. Outer scope values aren't valid dependencies because mutating them doesn't re-render the component.")
    .with_error_code_scope(SCOPE)
}

fn infinite_rerender_call_to_set_state_diagnostic(hook_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "React Hook {hook_name} contains a call to setState. Without a list of dependencies, this can lead to an infinite chain of updates."
    ))
    .with_label(span)
    .with_help("Consider adding an empty list of dependencies to make it clear which values are intended to be stable.")
    .with_error_code_scope(SCOPE)
}

fn ref_accessed_directly_in_effect_cleanup_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The ref's value `.current` is accessed directly in the effect cleanup function.")
        .with_label(span)
        .with_help("The ref value will likely have changed by the time this effect cleanup function runs. If this ref points to a node rendered by react, copy it to a variable inside the effect and use that variable in the cleanup function.")
        .with_error_code_scope(SCOPE)
}

fn functions_returned_from_use_effect_event_must_not_be_included_in_dependency_array(
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Functions returned from `useEffectEvent` must not be included in the dependency array.",
    )
    .with_label(span)
    .with_help("Remove the dependency from the dependency array.")
    .with_error_code_scope(SCOPE)
}

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveDeps(Box<ExhaustiveDepsConfig>);

#[derive(Debug, Clone, Default)]
pub struct ExhaustiveDepsConfig {
    additional_hooks: Option<Regex>,
}

#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct ExhaustiveDepsConfigJson {
    /// Optionally provide a regex of additional hooks to check.
    additional_hooks: Option<String>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Verifies the list of dependencies for Hooks like `useEffect` and similar.
    ///
    /// ### Why is this bad?
    ///
    /// React Hooks like `useEffect` and similar require a list of dependencies to be passed as an argument. This list is used to determine when the effect should be re-run. If the list is missing or incomplete, the effect may run more often than necessary, or not at all.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function MyComponent(props) {
    ///     useEffect(() => {
    ///         console.log(props.foo);
    ///     }, []);
    ///     // `props` is missing from the dependencies array
    ///     return <div />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function MyComponent(props) {
    ///     useEffect(() => {
    ///         console.log(props.foo);
    ///     }, [props]);
    ///     return <div />;
    /// }
    /// ```
    ExhaustiveDeps,
    react,
    correctness,
    safe_fixes_and_dangerous_suggestions,
    config = ExhaustiveDepsConfigJson,
);

const HOOKS_USELESS_WITHOUT_DEPENDENCIES: [&str; 2] = ["useCallback", "useMemo"];

impl Rule for ExhaustiveDeps {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|first| {
                serde_json::from_value::<ExhaustiveDepsConfigJson>(first.clone()).ok()
            })
            .map(|config_json| ExhaustiveDepsConfig {
                additional_hooks: config_json
                    .additional_hooks
                    .filter(|pattern| !pattern.is_empty())
                    .and_then(|pattern| Regex::new(&pattern).ok()),
            })
            .unwrap_or_default();

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(hook_name) = get_node_name_without_react_namespace(&call_expr.callee) else {
            return;
        };

        let component_scope_id = {
            match get_enclosing_function(node, ctx).map(oxc_semantic::AstNode::kind) {
                Some(AstKind::Function(func)) => func.scope_id(),
                Some(AstKind::ArrowFunctionExpression(arrow_func)) => arrow_func.scope_id(),
                // If we hit here, it means that the hook is called at the top level which isn't allowed, so lets bail out.
                // Reporting of this error should've been handled by rules-of-hooks
                _ => return,
            }
        };

        let Some(callback_index) = self.get_reactive_hook_callback_index(hook_name) else {
            return;
        };
        let callback_node = call_expr.arguments.get(callback_index);
        let dependencies_node = call_expr.arguments.get(callback_index + 1);

        let Some(callback_node) = callback_node else {
            ctx.diagnostic(missing_callback_diagnostic(hook_name.as_str(), call_expr.span()));
            return;
        };

        let is_effect = hook_name.as_str().contains("Effect");

        if dependencies_node.is_none() && !is_effect {
            if HOOKS_USELESS_WITHOUT_DEPENDENCIES.contains(&hook_name.as_str()) {
                ctx.diagnostic_with_fix(
                    dependency_array_required_diagnostic(hook_name.as_str(), call_expr.span()),
                    |fixer| fixer.insert_text_after(callback_node, ", []"),
                );
            }
            return;
        }

        let callback_node = match callback_node {
            Argument::SpreadElement(_) => {
                ctx.diagnostic(unknown_dependencies_diagnostic(
                    hook_name.as_str(),
                    call_expr.callee.span(),
                ));
                None
            }
            match_expression!(Argument) => {
                match callback_node.to_expression().get_inner_expression() {
                    Expression::ArrowFunctionExpression(arrow_function_expression) => {
                        Some(CallbackNode::ArrowFunction(arrow_function_expression))
                    }
                    Expression::FunctionExpression(function_expression) => {
                        Some(CallbackNode::Function(function_expression))
                    }
                    Expression::Identifier(ident) => {
                        if let Some(dependencies_node) = dependencies_node {
                            // The function passed as a callback is not written inline.
                            // But perhaps it's in the dependencies array?
                            if dependencies_node.as_expression().is_some_and(|v| {
                                if let Expression::ArrayExpression(array_expr) =
                                    v.get_inner_expression()
                                {
                                    array_expr.elements.iter().any(|elem| {
                                        elem.as_expression().is_some_and(|elem| {
                                            if let Expression::Identifier(array_el_ident) =
                                                elem.get_inner_expression()
                                            {
                                                array_el_ident.name == ident.name
                                            } else {
                                                false
                                            }
                                        })
                                    })
                                } else {
                                    false
                                }
                            }) {
                                return;
                            }

                            // Try to find the var in the current scope
                            if let Some(decl) = get_declaration_of_variable(ident, ctx.semantic()) {
                                match decl.kind() {
                                    AstKind::VariableDeclarator(var_decl) => {
                                        if let Some(init) = &var_decl.init {
                                            match init {
                                                Expression::FunctionExpression(function) => {
                                                    Some(CallbackNode::Function(function))
                                                }
                                                Expression::ArrowFunctionExpression(function) => {
                                                    Some(CallbackNode::ArrowFunction(function))
                                                }
                                                _ => {
                                                    ctx.diagnostic(missing_dependency_diagnostic(
                                                        hook_name,
                                                        &[Name::from(ident.as_ref())],
                                                        dependencies_node.span(),
                                                    ));
                                                    None
                                                }
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                    AstKind::Function(function) => {
                                        Some(CallbackNode::Function(function))
                                    }
                                    AstKind::FormalParameter(_) => {
                                        ctx.diagnostic(missing_dependency_diagnostic(
                                            hook_name,
                                            &[Name::from(ident.as_ref())],
                                            dependencies_node.span(),
                                        ));
                                        None
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => {
                        ctx.diagnostic(unknown_dependencies_diagnostic(
                            hook_name.as_str(),
                            call_expr.callee.span(),
                        ));
                        None
                    }
                }
            }
        };

        let Some(callback_node) = callback_node else {
            // either handled or we couldn't find the callback
            return;
        };

        if callback_node.is_async() && is_effect {
            ctx.diagnostic(async_effect_diagnostic(callback_node.span()));
        }

        let dependencies_node = dependencies_node.and_then(|node| match node {
            Argument::SpreadElement(_) => {
                ctx.diagnostic(dependency_array_not_array_literal_diagnostic(
                    hook_name.as_str(),
                    node.span(),
                ));
                None
            }
            match_expression!(Argument) => {
                let inner_expr = node.to_expression().get_inner_expression();
                match inner_expr {
                    Expression::ArrayExpression(array_expr) => Some(array_expr),
                    Expression::Identifier(ident)
                        if ident.name == "undefined"
                            && ctx.is_reference_to_global_variable(ident) =>
                    {
                        None
                    }
                    _ => {
                        ctx.diagnostic(dependency_array_not_array_literal_diagnostic(
                            hook_name.as_str(),
                            node.span(),
                        ));
                        None
                    }
                }
            }
        });

        let (found_dependencies, refs_inside_cleanups) = {
            let mut found_dependencies = ExhaustiveDepsVisitor::new(ctx.semantic());

            found_dependencies.visit_formal_parameters(callback_node.parameters());

            if let Some(function_body) = callback_node.body() {
                found_dependencies.visit_function_body(function_body);
            }

            (found_dependencies.found_dependencies, found_dependencies.refs_inside_cleanups)
        };

        if is_effect {
            for r#ref in refs_inside_cleanups {
                if let Expression::Identifier(ident) = r#ref.object.get_inner_expression() {
                    let reference = ctx.scoping().get_reference(ident.reference_id());
                    let has_write_reference = reference.symbol_id().is_some_and(|symbol_id| {
                        ctx.semantic().symbol_references(symbol_id).any(|reference| {
                            let parent = ctx.nodes().parent_node(reference.node_id());
                            let AstKind::StaticMemberExpression(member_expr) = parent.kind() else {
                                return false;
                            };
                            if member_expr.property.name != "current" {
                                return false;
                            }
                            let grand_parent = ctx.nodes().parent_node(parent.id());
                            matches!(grand_parent.kind(), AstKind::AssignmentExpression(_))
                        })
                    });

                    if has_write_reference
                        || get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
                            .is_some_and(|decl| decl.scope_id() != component_scope_id)
                    {
                        continue;
                    }
                }
                ctx.diagnostic(ref_accessed_directly_in_effect_cleanup_diagnostic(r#ref.span()));
            }
        }

        let Some(dependencies_node) = dependencies_node else {
            if is_effect {
                let contains_set_state_call = {
                    let mut finder = ExhaustiveDepsVisitor::new(ctx.semantic());
                    if let Some(function_body) = callback_node.body() {
                        finder.visit_function_body_root(function_body);
                    }
                    finder.set_state_call
                };

                if contains_set_state_call {
                    ctx.diagnostic(infinite_rerender_call_to_set_state_diagnostic(
                        hook_name.as_str(),
                        call_expr.callee.span(),
                    ));
                }
            }

            return;
        };

        let declared_dependencies_iter =
            dependencies_node.elements.iter().filter_map(|elem| match elem {
                ArrayExpressionElement::Elision(_) => None,
                ArrayExpressionElement::SpreadElement(_) => {
                    ctx.diagnostic(complex_expression_in_dependency_array_diagnostic(
                        hook_name.as_str(),
                        elem.span(),
                    ));
                    None
                }
                match_expression!(ArrayExpressionElement) => {
                    let elem = elem.to_expression().get_inner_expression();

                    if let Ok(dep) = analyze_property_chain(elem, ctx) {
                        dep
                    } else if elem.is_literal() {
                        ctx.diagnostic(literal_in_dependency_array_diagnostic(elem.span()));
                        None
                    } else {
                        ctx.diagnostic(complex_expression_in_dependency_array_diagnostic(
                            hook_name.as_str(),
                            elem.span(),
                        ));
                        None
                    }
                }
            });

        let declared_dependencies = {
            let mut declared_dependencies = FxHashSet::default();
            for item in declared_dependencies_iter {
                let span = item.span;
                if !declared_dependencies.insert(item) {
                    ctx.diagnostic(duplicate_dependency_diagnostic(span));
                }
            }

            declared_dependencies
        };

        for dependency in &declared_dependencies {
            if let Some(symbol_id) = dependency.symbol_id {
                let dependency_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
                if !(ctx
                    .semantic()
                    .scoping()
                    .scope_ancestors(component_scope_id)
                    .skip(1)
                    .contains(&dependency_scope_id)
                    || dependency.chain.len() == 1 && dependency.chain[0] == "current")
                {
                    continue;
                }
            }

            ctx.diagnostic_with_fix(
                unnecessary_outer_scope_dependency_diagnostic(
                    hook_name,
                    &dependency.name,
                    dependency.span,
                ),
                |fixer| fix::remove_dependency(fixer, dependency, dependencies_node),
            );
        }

        let undeclared_deps = found_dependencies.difference(&declared_dependencies).filter(|dep| {
            if declared_dependencies.iter().any(|decl_dep| dep.contains(decl_dep)) {
                return false;
            }

            if !is_identifier_a_dependency(
                dep.name,
                dep.reference_id,
                dep.span,
                ctx,
                component_scope_id,
            ) {
                return false;
            }
            true
        });

        if undeclared_deps.clone().count() > 0 {
            let undeclared = undeclared_deps.map(Name::from).collect::<Vec<_>>();
            ctx.diagnostic_with_dangerous_suggestion(
                missing_dependency_diagnostic(hook_name, &undeclared, dependencies_node.span()),
                |fixer| fix::append_dependencies(fixer, &undeclared, dependencies_node.as_ref()),
            );
        }

        for dep in &declared_dependencies {
            if let Some(symbol_id) = dep.symbol_id
                && let AstKind::VariableDeclarator(var_decl) =
                    ctx.semantic().symbol_declaration(symbol_id).kind()
                && let Some(Expression::CallExpression(call_expr)) = &var_decl.init
                && let Some(name) = func_call_without_react_namespace(call_expr)
                && name == "useEffectEvent"
            {
                ctx.diagnostic(functions_returned_from_use_effect_event_must_not_be_included_in_dependency_array(dep.span));
            }
        }

        // effects are allowed to have extra dependencies
        if !is_effect {
            // lastly, we need co compare for any unnecessary deps
            // for example if `props.foo`, AND `props.foo.bar.baz` was declared in the deps array
            // `props.foo.bar.baz` is unnecessary (already covered by `props.foo`)
            declared_dependencies.iter().tuple_combinations().for_each(|(a, b)| {
                if a.contains(b) {
                    ctx.diagnostic(unnecessary_dependency_diagnostic(
                        hook_name,
                        &a.to_string(),
                        dependencies_node.span,
                    ));
                } else if b.contains(a) {
                    ctx.diagnostic(unnecessary_dependency_diagnostic(
                        hook_name,
                        &b.to_string(),
                        dependencies_node.span,
                    ));
                }
            });

            for dep in declared_dependencies.difference(&found_dependencies) {
                if found_dependencies.iter().any(|found_dep| found_dep.contains(dep)) {
                    continue;
                }

                ctx.diagnostic(unnecessary_dependency_diagnostic(
                    hook_name,
                    &dep.to_string(),
                    dependencies_node.span,
                ));
            }
        }

        for dep in declared_dependencies {
            let Some(symbol_id) = dep.symbol_id else { continue };

            if dep.chain.is_empty() && is_symbol_declaration_referentially_unique(symbol_id, ctx) {
                let name = ctx.scoping().symbol_name(symbol_id);
                let decl_span = ctx.scoping().symbol_span(symbol_id);
                ctx.diagnostic_with_dangerous_suggestion(
                    dependency_changes_on_every_render_diagnostic(
                        hook_name, dep.span, name, decl_span,
                    ),
                    |fixer| fix::remove_dependency(fixer, &dep, dependencies_node),
                );
            }
        }
    }
}

fn is_symbol_declaration_referentially_unique(symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let decl = ctx.semantic().symbol_declaration(symbol_id);

    match decl.kind() {
        AstKind::Class(_) | AstKind::Function(_) => true,
        AstKind::VariableDeclarator(decl) => {
            if decl.id.kind.is_destructuring_pattern() {
                return false;
            }

            let Some(init) = &decl.init else { return false };

            if is_expression_referentially_unique(init) {
                return true;
            }

            false
        }
        _ => false,
    }
}

fn is_expression_referentially_unique(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::ArrayExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::RegExpLiteral(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_) => true,
        Expression::ConditionalExpression(conditional) => {
            is_expression_referentially_unique(&conditional.consequent)
                || is_expression_referentially_unique(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            is_expression_referentially_unique(&logical.left)
                || is_expression_referentially_unique(&logical.right)
        }
        Expression::BinaryExpression(bin_expr) => {
            is_expression_referentially_unique(&bin_expr.right)
        }
        Expression::AssignmentExpression(assignment) => {
            is_expression_referentially_unique(&assignment.right)
        }
        _ => false,
    }
}

#[derive(Debug)]
enum CallbackNode<'a> {
    Function(&'a Function<'a>),
    ArrowFunction(&'a ArrowFunctionExpression<'a>),
}

impl<'a> CallbackNode<'a> {
    fn is_async(&self) -> bool {
        match self {
            CallbackNode::Function(func) => func.r#async,
            CallbackNode::ArrowFunction(func) => func.r#async,
        }
    }

    fn parameters(&self) -> &FormalParameters<'a> {
        match self {
            CallbackNode::Function(func) => &func.params,
            CallbackNode::ArrowFunction(func) => &func.params,
        }
    }

    fn body(&self) -> Option<&FunctionBody<'a>> {
        match self {
            CallbackNode::Function(func) => func.body.as_deref(),
            CallbackNode::ArrowFunction(func) => Some(&func.body),
        }
    }
}

impl GetSpan for CallbackNode<'_> {
    fn span(&self) -> Span {
        match self {
            CallbackNode::Function(func) => func.span,
            CallbackNode::ArrowFunction(func) => func.span,
        }
    }
}

impl ExhaustiveDeps {
    // https://github.com/facebook/react/blob/1b0132c05acabae5aebd32c2cadddfb16bda70bc/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L1789
    fn get_reactive_hook_callback_index(&self, hook_name: &str) -> Option<usize> {
        match hook_name {
            "useEffect" | "useLayoutEffect" | "useCallback" | "useMemo" => Some(0),
            "useImperativeHandle" => Some(1),
            _ => self
                .0
                .additional_hooks
                .as_ref()
                .is_some_and(|regex| regex.is_match(hook_name))
                .then_some(0),
        }
    }
}

fn get_node_name_without_react_namespace<'a, 'b>(expr: &'b Expression<'a>) -> Option<&'b Atom<'a>> {
    match expr {
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(_ident) = &member.object {
                return Some(&member.property.name);
            }
            None
        }
        Expression::Identifier(ident) => Some(&ident.name),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct Name<'a> {
    pub span: Span,
    pub name: Cow<'a, str>,
}
impl std::fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> From<&Dependency<'a>> for Name<'a> {
    fn from(dep: &Dependency<'a>) -> Self {
        let name = if dep.chain.is_empty() {
            Cow::Borrowed(dep.name.as_str())
        } else {
            Cow::Owned(dep.to_string())
        };
        Self { name, span: dep.span }
    }
}
impl<'a> From<&IdentifierReference<'a>> for Name<'a> {
    fn from(id: &IdentifierReference<'a>) -> Self {
        Self { name: Cow::Borrowed(id.name.as_str()), span: id.span }
    }
}

#[derive(Debug)]
struct Dependency<'a> {
    span: Span,
    name: Atom<'a>,
    reference_id: ReferenceId,
    // the symbol id that this dependency is referring to
    symbol_id: Option<SymbolId>,
    chain: Vec<Atom<'a>>,
}

impl Hash for Dependency<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.symbol_id.hash(state);
        self.chain.hash(state);
    }
}

impl PartialEq for Dependency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.chain == other.chain
    }
}

impl Eq for Dependency<'_> {}

impl Dependency<'_> {
    #[expect(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        std::iter::once(&self.name).chain(self.chain.iter()).map(oxc_span::Atom::as_str).join(".")
    }

    fn contains(&self, other: &Self) -> bool {
        self.name == other.name && chain_contains(&self.chain, &other.chain)
    }
}

fn chain_contains(a: &[Atom<'_>], b: &[Atom<'_>]) -> bool {
    for (index, part) in b.iter().enumerate() {
        let Some(other) = a.get(index) else { return false };
        if other != part {
            return false;
        }
    }

    true
}

fn analyze_property_chain<'a, 'b>(
    expr: &'b Expression<'a>,
    semantic: &'b Semantic<'a>,
) -> Result<Option<Dependency<'a>>, ()> {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) => Ok(Some(Dependency {
            span: ident.span(),
            name: ident.name,
            reference_id: ident.reference_id(),
            chain: vec![],
            symbol_id: semantic.scoping().get_reference(ident.reference_id()).symbol_id(),
        })),
        // TODO; is this correct?
        Expression::JSXElement(_) => Ok(None),
        Expression::StaticMemberExpression(expr) => concat_members(expr, semantic),
        Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
            ChainElement::StaticMemberExpression(expr) => concat_members(expr, semantic),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn concat_members<'a, 'b>(
    member_expr: &'b StaticMemberExpression<'a>,
    semantic: &'b Semantic<'a>,
) -> Result<Option<Dependency<'a>>, ()> {
    let Some(source) = analyze_property_chain(&member_expr.object, semantic)? else {
        return Ok(None);
    };

    let new_chain = Vec::from([member_expr.property.name]);

    Ok(Some(Dependency {
        span: member_expr.span,
        name: source.name,
        reference_id: source.reference_id,
        chain: [source.chain, new_chain].concat(),
        symbol_id: semantic.scoping().get_reference(source.reference_id).symbol_id(),
    }))
}

fn is_identifier_a_dependency<'a>(
    ident_name: Atom<'a>,
    ident_reference_id: ReferenceId,
    ident_span: Span,
    ctx: &'_ LintContext<'a>,
    component_scope_id: ScopeId,
) -> bool {
    let mut visited = FxHashSet::default();
    is_identifier_a_dependency_impl(
        ident_name,
        ident_reference_id,
        ident_span,
        ctx,
        component_scope_id,
        &mut visited,
    )
}
fn is_identifier_a_dependency_impl<'a>(
    ident_name: Atom<'a>,
    ident_reference_id: ReferenceId,
    ident_span: Span,
    ctx: &'_ LintContext<'a>,
    component_scope_id: ScopeId,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    // if it is a global e.g. `console` or `window`, then it's not a dependency
    if ctx.scoping().root_unresolved_references().contains_key(ident_name.as_str()) {
        return false;
    }

    let Some(declaration) = get_declaration_from_reference_id(ident_reference_id, ctx) else {
        return false;
    };

    let semantic = ctx.semantic();
    let scopes = semantic.scoping();

    // if the variable was declared in the root scope, then it's not a dependency
    if declaration.scope_id() == scopes.root_scope_id() {
        return false;
    }

    // Variable was declared outside the component scope
    // ```tsx
    // const id = crypto.randomUUID();
    // function MyComponent() {
    //   useEffect(() => {
    //     console.log(id);
    //   }, []);
    //   return <div />;
    // }
    // ```
    if scopes
        .scope_ancestors(component_scope_id)
        .skip(1)
        .any(|parent| parent == declaration.scope_id())
    {
        return false;
    }

    // Variable was declared inside a child scope
    // ```tsx
    // function MyComponent() {
    //   useEffect(() => {
    //     const id = crypto.randomUUID();
    //     console.log(id);
    //   }, []);
    //  return <div />;
    // }
    if scopes.iter_all_scope_child_ids(component_scope_id).any(|id| id == declaration.scope_id()) {
        return false;
    }

    if is_stable_value(
        declaration,
        ident_name,
        ident_reference_id,
        ctx,
        component_scope_id,
        visited,
    ) {
        return false;
    }

    // Using a declaration recursively is ok
    // ```tsx
    // function MyComponent() {
    //     const recursive = useCallback((n: number): number => (n <= 0 ? 0 : n + recursive(n - 1)), []);
    //     return recursive
    // }
    // ```
    if declaration.span().contains_inclusive(ident_span) {
        return false;
    }

    true
}

// https://github.com/facebook/react/blob/fee786a057774ab687aff765345dd86fce534ab2/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L164
fn is_stable_value<'a, 'b>(
    node: &'b AstNode<'a>,
    ident_name: Atom<'a>,
    ident_reference_id: ReferenceId,
    ctx: &'b LintContext<'a>,
    component_scope_id: ScopeId,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    if let Some(symbol_id) = ctx.scoping().get_reference(ident_reference_id).symbol_id()
        && !visited.insert(symbol_id)
    {
        return true;
    }

    match node.kind() {
        AstKind::VariableDeclaration(declaration) => {
            if declaration.kind == VariableDeclarationKind::Const {
                return true;
            }

            false
        }
        AstKind::VariableDeclarator(declaration) => {
            // if the variable does not have an initializer, then it's not a stable value
            let Some(init) = &declaration.init else {
                return false;
            };

            {
                // if the variables is a function, check whether the function is stable
                let function_body = match init.get_inner_expression() {
                    Expression::ArrowFunctionExpression(arrow_func) => Some(&arrow_func.body),
                    Expression::FunctionExpression(func) => func.body.as_ref(),
                    _ => None,
                };
                if let Some(function_body) = function_body {
                    return is_function_stable(
                        function_body,
                        declaration
                            .id
                            .get_binding_identifier()
                            .map(oxc_ast::ast::BindingIdentifier::symbol_id),
                        ctx,
                        component_scope_id,
                        visited,
                    );
                }
            }

            // if the variables is a constant, and the initializer is a literal, then it's a stable value. (excluding regex literals)
            if declaration.kind == VariableDeclarationKind::Const
                && (matches!(
                    init,
                    Expression::BooleanLiteral(_)
                        | Expression::NullLiteral(_)
                        | Expression::NumericLiteral(_)
                        | Expression::BigIntLiteral(_)
                        | Expression::StringLiteral(_)
                ))
            {
                return true;
            }

            let Expression::CallExpression(init_expr) = &init else {
                return false;
            };

            let Some(init_name) = func_call_without_react_namespace(init_expr) else {
                return false;
            };

            if init_name == "useRef" || init_name == "useEffectEvent" {
                return true;
            }

            let BindingPatternKind::ArrayPattern(array_pat) = &declaration.id.kind else {
                return false;
            };

            let Some(Some(second_arg)) = array_pat.elements.get(1) else {
                return false;
            };

            let BindingPatternKind::BindingIdentifier(binding_ident) = &second_arg.kind else {
                return false;
            };

            if (init_name == "useState"
                || init_name == "useReducer"
                || init_name == "useTransition"
                || init_name == "useActionState")
                && binding_ident.name == ident_name
                && !ctx
                    .semantic()
                    .symbol_references(
                        ctx.scoping().get_reference(ident_reference_id).symbol_id().unwrap(),
                    )
                    .any(|reference| {
                        if let AstKind::AssignmentExpression(assignment_expression) =
                            ctx.nodes().parent_kind(reference.node_id())
                        {
                            assignment_expression.left.span().contains_inclusive(
                                ctx.nodes().get_node(reference.node_id()).span(),
                            )
                        } else {
                            false
                        }
                    })
            {
                return true;
            }

            false
        }
        AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => {
            let function_body = match node.kind() {
                AstKind::ArrowFunctionExpression(arrow_func) => Some(&arrow_func.body),
                AstKind::Function(func) => func.body.as_ref(),
                _ => unreachable!(),
            };

            let Some(function_body) = function_body else { return false };

            is_function_stable(function_body, None, ctx, component_scope_id, visited)
        }
        _ => false,
    }
}

fn is_function_stable<'a, 'b>(
    function_body: &'b FunctionBody<'a>,
    function_symbol_id: Option<SymbolId>,
    ctx: &'b LintContext<'a>,
    component_scope_id: ScopeId,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let deps = {
        let mut collector = ExhaustiveDepsVisitor::new(ctx.semantic());
        collector.visit_function_body(function_body);
        collector.found_dependencies
    };

    deps.iter().all(|dep| {
        dep.symbol_id.zip(function_symbol_id).is_none_or(|(l, r)| l != r)
            && !is_identifier_a_dependency_impl(
                dep.name,
                dep.reference_id,
                dep.span,
                ctx,
                component_scope_id,
                visited,
            )
    })
}

// https://github.com/facebook/react/blob/fee786a057774ab687aff765345dd86fce534ab2/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L1742
fn func_call_without_react_namespace<'a>(
    call_expr: &'a CallExpression<'a>,
) -> Option<&'a Atom<'a>> {
    let inner_exp = call_expr.callee.get_inner_expression();

    if let Expression::Identifier(ident) = inner_exp {
        return Some(&ident.name);
    }

    let Expression::StaticMemberExpression(member) = inner_exp else {
        return None;
    };

    let reference = member.object.get_identifier_reference()?;

    if reference.name == "React" {
        return Some(&member.property.name);
    }

    None
}

struct ExhaustiveDepsVisitor<'a, 'b> {
    semantic: &'b Semantic<'a>,
    stack: Vec<AstType>,
    /// Variable declarations above the current node. Only populated in initializers.
    ///
    /// NOTE: I don't expect this stack to ever have more than 1 element, since
    /// variable declarators cannot be nested. However, having this as a stack
    /// is definitely safer.
    decl_stack: Vec<&'a VariableDeclarator<'a>>,
    skip_reporting_dependency: bool,
    set_state_call: bool,
    is_callee_of_call_expr: bool,
    found_dependencies: FxHashSet<Dependency<'a>>,
    refs_inside_cleanups: Vec<&'a StaticMemberExpression<'a>>,
}

impl<'a, 'b> ExhaustiveDepsVisitor<'a, 'b> {
    fn new(semantic: &'b Semantic<'a>) -> Self {
        Self {
            semantic,
            stack: vec![],
            decl_stack: vec![],
            skip_reporting_dependency: false,
            set_state_call: false,
            is_callee_of_call_expr: false,
            found_dependencies: FxHashSet::default(),
            refs_inside_cleanups: vec![],
        }
    }

    fn visit_function_body_root(&mut self, function_body: &FunctionBody<'a>) {
        walk_function_body(self, function_body);
    }

    fn iter_destructure_bindings<F>(&self, mut cb: F) -> Option<bool>
    where
        F: FnMut(std::borrow::Cow<'a, str>),
    {
        // check for object destructuring
        // `const { foo } = props;`
        // allow `props.foo` to be a dependency
        let Some(VariableDeclarator {
            id: BindingPattern { kind: BindingPatternKind::ObjectPattern(obj), .. },
            ..
        }) = self.decl_stack.last()
        else {
            return None;
        };

        if obj.rest.is_some() {
            return Some(true);
        }

        let mut needs_full_identifier = false;
        for prop in &obj.properties {
            if prop.computed {
                needs_full_identifier = true;
                continue;
            }
            match &prop.value.kind {
                BindingPatternKind::BindingIdentifier(id) => {
                    cb(id.name.into());
                }
                BindingPatternKind::AssignmentPattern(pat) => {
                    if let Some(id) = pat.left.get_binding_identifier() {
                        cb(id.name.into());
                    } else {
                        // `const { idk: { thing } = { } } = props;`
                        // not sure what to do
                        needs_full_identifier = true;
                    }
                }
                BindingPatternKind::ArrayPattern(_) | BindingPatternKind::ObjectPattern(_) => {
                    // `const { foo: [bar] } = props;`
                    // `const { foo: { bar } } = props;`
                    // foo.bar is sufficient as a dependency
                    if let Some(key) = prop.key.name() {
                        cb(key);
                    } else {
                        needs_full_identifier = true;
                    }
                }
            }
        }

        Some(needs_full_identifier)
    }
}

impl<'a> Visit<'a> for ExhaustiveDepsVisitor<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.stack.push(kind.ty());
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.stack.pop();
    }

    fn visit_ts_type_annotation(&mut self, _it: &TSTypeAnnotation<'a>) {
        // noop
    }

    fn visit_ts_type_reference(&mut self, _it: &TSTypeReference<'a>) {
        // noop
    }

    fn visit_ts_type_parameter_instantiation(&mut self, _it: &TSTypeParameterInstantiation<'a>) {
        // noop
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        self.stack.push(AstType::VariableDeclarator);
        // NOTE: decl_stack is only appended when visiting initializer
        // expression.
        self.visit_binding_pattern(&decl.id);
        if let Some(init) = &decl.init {
            // SAFETY:
            // 1. All nodes live inside the arena, which has a lifetime of 'a.
            //    The arena lives longer than any Rule pass, so this visitor
            //    will drop before the node does.
            // 2. This visitor is read-only, and it drops all references after
            //    visiting the node.  Therefore, no mutable references will be
            //    created before this stack is dropped.
            let decl = unsafe {
                std::mem::transmute::<&VariableDeclarator<'_>, &'a VariableDeclarator<'a>>(decl)
            };
            self.decl_stack.push(decl);
            self.visit_expression(init);
            self.decl_stack.pop();
        }
        self.stack.pop();
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.stack.push(AstType::CallExpression);

        // Mark that we're visiting a callee
        self.is_callee_of_call_expr = true;
        self.visit_expression(&it.callee);
        self.is_callee_of_call_expr = false;

        // Visit arguments normally
        for arg in &it.arguments {
            self.visit_argument(arg);
        }

        self.stack.pop();
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if it.property.name == "current" && is_inside_effect_cleanup(&self.stack) {
            // Safety: this is safe
            let it = unsafe {
                std::mem::transmute::<&StaticMemberExpression<'_>, &'a StaticMemberExpression<'a>>(
                    it,
                )
            };
            self.refs_inside_cleanups.push(it);
        }

        // consider `useEffect(() => { console.log(props.foo().foo.bar); }, [props.foo]);`
        // we don't care about `foo.bar`, only `props.foo`
        if matches!(it.object.get_inner_expression(), Expression::CallExpression(_))
            || self.skip_reporting_dependency
        {
            self.visit_expression(&it.object);
            return;
        }

        let is_parent_call_expr = self.is_callee_of_call_expr;

        if let Ok(source) = analyze_property_chain(&it.object, self.semantic) {
            if let Some(source) = source {
                if is_parent_call_expr {
                    self.found_dependencies.insert(source);
                } else {
                    let new_chain = Vec::from([it.property.name]);

                    let mut destructured_props: Vec<Atom<'a>> = vec![];
                    let mut did_see_ref = false;
                    let needs_full_chain = self
                        .iter_destructure_bindings(|id| {
                            if let Cow::Borrowed(id) = id {
                                if id == "current" {
                                    did_see_ref = true;
                                } else {
                                    destructured_props.push(id.into());
                                }
                            } else {
                                // todo
                            }
                        })
                        .unwrap_or(true);

                    let symbol_id =
                        self.semantic.scoping().get_reference(source.reference_id).symbol_id();
                    if needs_full_chain || (destructured_props.is_empty() && !did_see_ref) {
                        self.found_dependencies.insert(Dependency {
                            name: source.name,
                            reference_id: source.reference_id,
                            span: source.span,
                            chain: [source.chain.clone(), new_chain].concat(),
                            symbol_id,
                        });
                    } else {
                        for prop in destructured_props {
                            self.found_dependencies.insert(Dependency {
                                name: source.name,
                                reference_id: source.reference_id,
                                span: source.span,
                                chain: [source.chain.clone(), new_chain.clone(), vec![prop]]
                                    .concat(),
                                symbol_id,
                            });
                        }
                    }
                }
            }

            let cur_skip_reporting_dependency = self.skip_reporting_dependency;
            self.skip_reporting_dependency = true;
            self.is_callee_of_call_expr = false;
            self.visit_expression(&it.object);
            self.skip_reporting_dependency = cur_skip_reporting_dependency;
        } else {
            // this means that some part of the chain could not be analyzed
            // for example `foo.bar.baz().abc`. `baz()` cannot be statically analyzed
            // instead, continue to go down, looking at the object to gather dependencies
            self.is_callee_of_call_expr = false;
            self.visit_expression(&it.object);
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.skip_reporting_dependency {
            return;
        }
        let reference_id = ident.reference_id();
        let symbol_id = self.semantic.scoping().get_reference(reference_id).symbol_id();

        let mut destructured_props: Vec<Atom<'a>> = vec![];
        let mut did_see_ref = false;
        let needs_full_identifier = self
            .iter_destructure_bindings(|id| {
                if let Cow::Borrowed(id) = id {
                    if id == "current" {
                        did_see_ref = true;
                    } else {
                        destructured_props.push(id.into());
                    }
                } else {
                    // todo: arena allocate
                }
            })
            .unwrap_or(true);
        if needs_full_identifier || (destructured_props.is_empty() && !did_see_ref) {
            self.found_dependencies.insert(Dependency {
                name: ident.name,
                reference_id,
                span: ident.span,
                chain: vec![],
                symbol_id,
            });
        } else {
            for prop in destructured_props {
                self.found_dependencies.insert(Dependency {
                    name: ident.name,
                    reference_id,
                    span: ident.span,
                    chain: vec![prop],
                    symbol_id,
                });
            }
        }

        if let Some(decl) = get_declaration_of_variable(ident, self.semantic) {
            let is_set_state_call = match decl.kind() {
                AstKind::VariableDeclarator(var_decl) => {
                    let Some(Expression::CallExpression(call_expr)) = &var_decl.init else {
                        return;
                    };

                    let Some(name) = func_call_without_react_namespace(call_expr) else {
                        return;
                    };

                    if name != "useState" && name != "useReducer" {
                        return;
                    }

                    let BindingPatternKind::ArrayPattern(array_pat) = &var_decl.id.kind else {
                        return;
                    };

                    let Some(Some(second_arg)) = array_pat.elements.get(1) else {
                        return;
                    };

                    let BindingPatternKind::BindingIdentifier(binding_ident) = &second_arg.kind
                    else {
                        return;
                    };

                    binding_ident.name == ident.name
                }
                _ => false,
            };

            if is_set_state_call
                && self
                    .stack
                    .iter()
                    .all(|&ty| !matches!(ty, AstType::Function | AstType::ArrowFunctionExpression))
            {
                self.set_state_call = true;
            }
        }
    }
}

fn is_inside_effect_cleanup(stack: &[AstType]) -> bool {
    let mut iter = stack.iter().rev();

    while let Some(&cur) = iter.next() {
        if matches!(cur, AstType::Function | AstType::ArrowFunctionExpression)
            && iter.next() == Some(&AstType::ReturnStatement)
        {
            return true;
        }
    }

    false
}

mod fix {
    use super::Name;
    use oxc_allocator::{Allocator, CloneIn};
    use oxc_ast::{
        AstBuilder,
        ast::{ArrayExpression, Expression},
    };
    use oxc_span::{Atom, GetSpan, SPAN};

    use crate::{
        fixer::{RuleFix, RuleFixer},
        rules::react::exhaustive_deps::Dependency,
    };

    pub fn append_dependencies<'c, 'a: 'c>(
        fixer: RuleFixer<'c, 'a>,
        names: &[Name<'a>],
        deps: &ArrayExpression<'a>,
    ) -> RuleFix {
        let mut codegen = fixer.codegen();

        let alloc = Allocator::default();
        let ast_builder = AstBuilder::new(&alloc);

        let mut vec = deps.elements.clone_in(&alloc);

        for name in names {
            vec.push(
                ast_builder
                    .expression_identifier(SPAN, Atom::from_cow_in(&name.name, &alloc))
                    .into(),
            );
        }

        codegen.print_expression(&ast_builder.expression_array(SPAN, vec));
        fixer.replace(deps.span, codegen.into_source_text())
    }

    pub fn remove_dependency<'c, 'a: 'c>(
        fixer: RuleFixer<'c, 'a>,
        dependency: &Dependency,
        deps: &ArrayExpression<'a>,
    ) -> RuleFix {
        let mut codegen = fixer.codegen();

        let alloc = Allocator::default();
        let ast_builder = AstBuilder::new(&alloc);

        let new_deps = deps
            .elements
            .iter()
            .filter(|el| (*el).span() != dependency.span)
            .map(|el| el.clone_in(&alloc));

        codegen.print_expression(&Expression::ArrayExpression(ast_builder.alloc_array_expression(
            deps.span,
            oxc_allocator::Vec::from_iter_in(new_deps, &alloc),
        )));
        fixer.replace(deps.span, codegen.into_source_text())
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          });
        }",
        r"function MyComponent() {
          useEffect(() => {
            const local = {};
            console.log(local);
          }, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [local]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent() {
          const local1 = {};
          {
            const local2 = {};
            useEffect(() => {
              console.log(local1);
              console.log(local2);
            });
          }
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          {
            const local2 = someFunc();
            useCallback(() => {
              console.log(local1);
              console.log(local2);
            }, [local1, local2]);
          }
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          function MyNestedComponent() {
            const local2 = someFunc();
            useCallback(() => {
              console.log(local1);
              console.log(local2);
            }, [local2]);
          }
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
            console.log(local);
          }, [local]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log(unresolved);
          }, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [,,,local,,,]);
        }",
        r"function MyComponent({ foo }) {
          useEffect(() => {
            console.log(foo.length);
          }, [foo]);
        }",
        r"function MyComponent({ foo }) {
          useEffect(() => {
            console.log(foo.length);
            console.log(foo.slice(0));
          }, [foo]);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return history.listen();
          }, [history]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {});
          useLayoutEffect(() => {});
          useImperativeHandle(props.innerRef, () => {});
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props.bar, props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props.foo, props.bar]);
        }",
        r"function MyComponent(props) {
          const local = someFunc();
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, [props.foo, props.bar, local]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props, props.foo]);

          let color = someFunc();
          useEffect(() => {
            console.log(props.foo.bar.baz);
            console.log(color);
          }, [props.foo, props.foo.bar.baz, color]);
        }",
        // destructuring
        r"function MyComponent(props) {
          useEffect(() => {
            const { foo, bar } = props;
            console.log(foo);
            console.log(bar);
          }, [props.foo, props.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            const { bar } = props.foo;
            console.log(bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            const { foo, bar } = props;
            console.log(foo);
            console.log(bar);
          }, [props]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            const { foo: { bar } } = props;
            console.log(bar);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            const [bar] = props.foo;
            console.log(bar);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          const foo = useRef()
          useEffect(() => {
            const { current: bar } = foo;
            console.log(bar);
          }, []);
        }",
        r"function MyComponent(props) {
          const bar = props.bar;
          useEffect(() => {
            const { bar } = foo();
            console.log(bar);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar?.baz ?? null);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, [props.foo!]);
        }",
        // FIXME(@DonIsaac): these should pass
        // r"function MyComponent(props) {
        //   useEffect(() => {
        //     console.log(props.foo!.bar);
        //   }, [props.foo!.bar]);
        // }",
        // r"function MyComponent(props) {
        //   useEffect(() => {
        //     console.log(props.foo!.bar!);
        //   }, [props.foo!.bar!]);
        // }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
            console.log(props.foo?.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
            console.log(props.foo?.bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.foo?.bar);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useMemo(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo.bar?.toString());
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.toString());
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo.bar.toString());
          }, [props?.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.baz);
          }, [props?.foo.bar?.baz]);
        }",
        r"function MyComponent() {
          const myEffect = () => {
            // Doesn't use anything
          };
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          const myEffect = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, [local]);
        }",
        r"function MyComponent() {
          function myEffect() {
            console.log(global);
          }
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          const myEffect = () => {
            otherThing()
          }
          const otherThing = () => {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent({delay}) {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);
          useEffect(myEffect, [myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [,myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [,myEffect,,]);
        }",
        r"let local = {};
        function myEffect() {
          console.log(local);
        }
        function MyComponent() {
          useEffect(myEffect, []);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          });
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useWithoutEffectSuffix(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          return renderHelperConfusedWithEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"const local = {};
        useEffect(() => {
          console.log(local);
        }, []);",
        r"const local1 = {};
        {
          const local2 = {};
          useEffect(() => {
            console.log(local1);
            console.log(local2);
          }, []);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, []);
        }",
        r"function MyComponent({ maybeRef2, foo }) {
          const definitelyRef1 = useRef();
          const definitelyRef2 = useRef();
          const maybeRef1 = useSomeOtherRefyThing();
          const [state1, setState1] = useState();
          const [state2, setState2] = React.useState();
          const [state3, dispatch1] = useReducer();
          const [state4, dispatch2] = React.useReducer();
          const [state5, maybeSetState] = useFunnyState();
          const [state6, maybeDispatch] = useFunnyReducer();
          const [isPending1] = useTransition();
          const [isPending2, startTransition2] = useTransition();
          const [isPending3] = React.useTransition();
          const [isPending4, startTransition4] = React.useTransition();
          const mySetState = useCallback(() => {}, []);
          let myDispatch = useCallback(() => {}, []);

          useEffect(() => {
            // Known to be static
            console.log(definitelyRef1.current);
            console.log(definitelyRef2.current);
            console.log(maybeRef1.current);
            console.log(maybeRef2.current);
            setState1();
            setState2();
            dispatch1();
            dispatch2();
            startTransition1();
            startTransition2();
            startTransition3();
            startTransition4();

            // Dynamic
            console.log(state1);
            console.log(state2);
            console.log(state3);
            console.log(state4);
            console.log(state5);
            console.log(state6);
            console.log(isPending2);
            console.log(isPending4);
            mySetState();
            myDispatch();

            // Not sure; assume dynamic
            maybeSetState();
            maybeDispatch();
          }, [
            // Dynamic
            state1, state2, state3, state4, state5, state6,
            maybeRef1, maybeRef2,
            isPending2, isPending4,

            // Not sure; assume dynamic
            mySetState, myDispatch,
            maybeSetState, maybeDispatch

            // In this test, we don't specify static deps.
            // That should be okay.
          ]);
        }",
        r"function MyComponent({ maybeRef2 }) {
          const definitelyRef1 = useRef();
          const definitelyRef2 = useRef();
          const maybeRef1 = useSomeOtherRefyThing();

          const [state1, setState1] = useState();
          const [state2, setState2] = React.useState();
          const [state3, dispatch1] = useReducer();
          const [state4, dispatch2] = React.useReducer();

          const [state5, maybeSetState] = useFunnyState();
          const [state6, maybeDispatch] = useFunnyReducer();

          const mySetState = useCallback(() => {}, []);
          let myDispatch = useCallback(() => {}, []);

          useEffect(() => {
            // Known to be static
            console.log(definitelyRef1.current);
            console.log(definitelyRef2.current);
            console.log(maybeRef1.current);
            console.log(maybeRef2.current);
            setState1();
            setState2();
            dispatch1();
            dispatch2();

            // Dynamic
            console.log(state1);
            console.log(state2);
            console.log(state3);
            console.log(state4);
            console.log(state5);
            console.log(state6);
            mySetState();
            myDispatch();

            // Not sure; assume dynamic
            maybeSetState();
            maybeDispatch();
          }, [
            // Dynamic
            state1, state2, state3, state4, state5, state6,
            maybeRef1, maybeRef2,

            // Not sure; assume dynamic
            mySetState, myDispatch,
            maybeSetState, maybeDispatch,

            // In this test, we specify static deps.
            // That should be okay too!
            definitelyRef1, definitelyRef2, setState1, setState2, dispatch1, dispatch2
          ]);
        }",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }))
        });",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }), [props.hello])
        });",
        r"function MyComponent(props) {
          let obj = someFunc();
          useEffect(() => {
            obj.foo = true;
          }, [obj]);
        }",
        r"function MyComponent(props) {
          let foo = {}
          useEffect(() => {
            foo.bar.baz = 43;
          }, [foo.bar]);
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef.current.toString())
            };
          }, []);
          return <div />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef?.current?.toString())
            };
          }, []);
          return <div />;
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef.current.toString())
            };
          }, [myRef]);
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            const node = myRef.current;
            node.addEventListener('mousemove', handleMove);
            return () => node.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMove = () => {};
            const node = myRef.current;
            node.addEventListener('mousemove', handleMove);
            return () => node.removeEventListener('mousemove', handleMove);
          }, [myRef]);
          return <div ref={myRef} />;
        }",
        r"function useMyThing(myRef) {
          useCallback(() => {
            const handleMouse = () => {};
            myRef.current.addEventListener('mousemove', handleMouse);
            myRef.current.addEventListener('mousein', handleMouse);
            return function() {
              setTimeout(() => {
                myRef.current.removeEventListener('mousemove', handleMouse);
                myRef.current.removeEventListener('mousein', handleMouse);
              });
            }
          }, [myRef]);
        }",
        r"function useMyThing() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {
              console.log(myRef.current)
            };
            window.addEventListener('mousemove', handleMove);
            return () => window.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function useMyThing() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {
              return () => window.removeEventListener('mousemove', handleMove);
            };
            window.addEventListener('mousemove', handleMove);
            return () => {};
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
          }, []);
        }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
          }, [local1, local2, local3]);
        }",
        r"function MyComponent(props) {
          const local = props.local;
          useEffect(() => {}, [local]);
        }",
        r"function Foo({ activeTab }) {
          useEffect(() => {
            window.scrollTo(0, 0);
          }, [activeTab]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props]);
          const fn2 = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo]);
          const fn3 = useMemo(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar]);
          const fn4 = useMemo(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          function handleNext() {
            console.log('hello');
          }
          useEffect(() => {
            return Store.subscribe(handleNext);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();

          function handleNext1(value) {
            let value2 = value * 100;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(foo(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(value);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function useInterval(callback, delay) {
          const savedCallback = useRef();
          useEffect(() => {
            savedCallback.current = callback;
          });
          useEffect(() => {
            function tick() {
              savedCallback.current();
            }
            if (delay !== null) {
              let id = setInterval(tick, delay);
              return () => clearInterval(id);
            }
          }, [delay]);
        }",
        r"function Counter() {
          const [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(c => c + 1);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter(unstableProp) {
          let [count, setCount] = useState(0);
          setCount = unstableProp
          useEffect(() => {
            let id = setInterval(() => {
              setCount(c => c + 1);
            }, 1000);
            return () => clearInterval(id);
          }, [setCount]);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          const [count, setCount] = useState(0);

          function tick() {
            setCount(c => c + 1);
          }

          useEffect(() => {
            let id = setInterval(() => {
              tick();
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          const [count, dispatch] = useReducer((state, action) => {
            if (action === 'inc') {
              return state + 1;
            }
          }, 0);

          useEffect(() => {
            let id = setInterval(() => {
              dispatch('inc');
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          const [count, dispatch] = useReducer((state, action) => {
            if (action === 'inc') {
              return state + 1;
            }
          }, 0);

          const tick = () => {
            dispatch('inc');
          };

          useEffect(() => {
            let id = setInterval(tick, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Podcasts() {
          useEffect(() => {
            setPodcasts([]);
          }, []);
          let [podcasts, setPodcasts] = useState(null);
        }",
        r"function withFetch(fetchPodcasts) {
          return function Podcasts({ id }) {
            let [podcasts, setPodcasts] = useState(null);
            useEffect(() => {
              fetchPodcasts(id).then(setPodcasts);
            }, [id]);
          }
        }",
        r"function Podcasts({ id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            function doFetch({ fetchPodcasts }) {
              fetchPodcasts(id).then(setPodcasts);
            }
            doFetch({ fetchPodcasts: API.fetchPodcasts });
          }, [id]);
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + 1;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + 1;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"import increment from './increment';
        function Counter() {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function withStuff(increment) {
          return function Counter() {
            let [count, setCount] = useState(0);

            useEffect(() => {
              let id = setInterval(() => {
                setCount(count => count + increment);
              }, 1000);
              return () => clearInterval(id);
            }, []);

            return <h1>{count}</h1>;
          }
        }",
        r"function App() {
          const [query, setQuery] = useState('react');
          const [state, setState] = useState(null);
          useEffect(() => {
            let ignore = false;
            fetchSomething();
            async function fetchSomething() {
              const result = await (await fetch('http://hn.algolia.com/api/v1/search?query=' + query)).json();
              if (!ignore) setState(result);
            }
            return () => { ignore = true; };
          }, [query]);
          return (
            <>
              <input value={query} onChange={e => setQuery(e.target.value)} />
              {JSON.stringify(state)}
            </>
          );
        }",
        // we don't support the following two cases as they would both cause an infinite loop at runtime
        //  r"function Example() {
        //    const foo = useCallback(() => {
        //      foo();
        //    }, []);
        //  }",
        //
        //  r"function Example({ prop }) {
        //    const foo = useCallback(() => {
        //      if (prop) {
        //        foo();
        //      }
        //    }, [prop]);
        //  }",
        r"function Hello() {
          const [state, setState] = useState(0);
          useEffect(() => {
            const handleResize = () => setState(window.innerWidth);
            window.addEventListener('resize', handleResize);
            return () => window.removeEventListener('resize', handleResize);
          });
        }",
        r"function Example() {
          useEffect(() => {
            arguments
          }, [])
        }",
        r"function Example() {
          useEffect(() => {
            const bar = () => {
              arguments;
            };
            bar();
          }, [])
        }",
        // check various forms of member expressions
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props.upperViewHeight;
          }, [props.upperViewHeight]);
        }",
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props?.upperViewHeight;
          }, [props?.upperViewHeight]);
        }",
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props?.upperViewHeight;
          }, [props]);
        }",
        r"function useFoo(foo){
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          const foo = 'hi!';
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          let {foo} = {foo: 1};
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          let [foo] = [1];
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo() {
          const foo = 'fine';
          if (true) {
            // Shadowed variable with constant construction in a nested scope is fine.
            const foo = {};
          }
          return useMemo(() => foo, [foo]);
        }",
        r"function MyComponent({foo}) {
          return useMemo(() => foo, [foo])
        }",
        r"function MyComponent() {
          const foo = true ? 'fine' : 'also fine';
          return useMemo(() => foo, [foo]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log('banana banana banana');
          }, undefined);
        }",
        // https://github.com/toeverything/AFFiNE/blob/1306a3be6108bfa51e7c48a5bcd667efd639421d/packages/frontend/core/src/components/page-list/virtualized-list.tsx#L90
        r"const useVirtuosoItems = <T extends ListItem>() => {
  const groups = useAtomValue();
  const groupCollapsedState = useAtomValue();

  return useMemo(() => {
    groupCollapsedState;
    groups;
    const items: VirtuosoItem<T>[] = [];
    return items;
  }, [groupCollapsedState, groups]);
};",
        r"function MyComponent() {
    const options = useOptions();
    useEffect(() => {
        if (!dropTargetRef.current) {
            return;
        }
        return dropTargetForElements({
            onDropTargetChange: (args) => {
                if (options && dropTargetRef.current) {
                  delete dropTargetRef.current.dataset['draggedOver'];
                }
            }
        });
    }, [options]);
}",
        "export function useCanvasZoomOrScroll() {
           useEffect(() => {
               let wheelStopTimeoutId: { current: number | undefined } = { current: undefined };

               wheelStopTimeoutId = requestAnimationFrameTimeout(() => {
                   setLastInteraction?.(null);
               }, 300);

               return () => {
                   if (wheelStopTimeoutId.current !== undefined) {
                       console.log('h1');
                   }
               };
           }, []);
        }",
        r#"function X() {
  const defaultParam1 = "";
  const myFunction = useCallback(
    (param1 = defaultParam1, param2) => {
    },
    [defaultParam1]
  );

  return null;
}
"#,
        r"function MyComponent() { const recursive = useCallback((n: number): number => (n <= 0 ? 0 : n + recursive(n - 1)), []); return recursive }",
        r"function Foo2() { useEffect(() => { foo() }, []); const foo = () => { bar() }; function bar () { foo() } }",
        r"function MyComponent(props) { useEffect(() => { console.log(props.foo!.bar) }, [props.foo!.bar]) }",
        r"function MyComponent(props) { useEffect(() => { console.log((props.foo).bar) }, [props.foo!.bar]) }",
        r"function MyComponent(props) { const external = {}; const y = useMemo(() => { const z = foo<typeof external>(); return z; }, []) }",
        r#"function Test() { const [state, setState] = useState(); useEffect(() => { console.log("state", state); }); }"#,
        "function MyComponent({ theme }) {
          const onStuff = useEffectEvent(() => {
            showNotification(theme);
          });
          useEffect(() => {
            onStuff();
          }, []);
          React.useEffect(() => {
            onStuff();
          }, []);
        }",
        // Issue #15796 - object property access should work correctly
        r"export const FileSize = ({ file, showSize = true }) => {
          const fileSizeInMB = useMemo(
            () => (showSize ? (file.size / (1024 * 1024)).toFixed(2) : undefined),
            [showSize, file.size],
          );
          return fileSizeInMB;
        }",
        // Additional tests for nested property access within expressions
        r"function MyComponent({ obj }) {
          useMemo(() => {
            return (obj.value * 2).toFixed(2);
          }, [obj.value]);
        }",
        r"function MyComponent({ data }) {
          useCallback(() => {
            console.log((data.count + 1).toString());
          }, [data.count]);
        }",
    ];

    let fail = vec![
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.toString());
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar.toString());
          }, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, []);
        }",
        r"function Counter(unstableProp) {
          let [count, setCount] = useState(0);
          setCount = unstableProp
          useEffect(() => {
            let id = setInterval(() => {
              setCount(c => c + 1);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function MyComponent() {
          let local = 42;
          useEffect(() => {
            console.log(local);
          }, []);
        }",
        r"function MyComponent() {
          const local = /foo/;
          useEffect(() => {
            console.log(local);
          }, []);
        }",
        r"function MyComponent(props) {
          const value = useMemo(() => { return 2*2; });
          const fn = useCallback(() => { alert('foo'); });
        }",
        r"function MyComponent({ fn1, fn2 }) {
          const value = useMemo(fn1);
          const fn = useCallback(fn2);
        }",
        r"function MyComponent() {
          useEffect()
          useLayoutEffect()
          useCallback()
          useMemo()
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            if (true) {
              console.log(local);
            }
          }, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            try {
              console.log(local);
            } finally {}
          }, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            function inner() {
              console.log(local);
            }
            inner();
          }, []);
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          {
            const local2 = someFunc();
            useEffect(() => {
              console.log(local1);
              console.log(local2);
            }, []);
          }
        }",
        r"function MyComponent() {
          const local1 = {};
          const local2 = {};
          useEffect(() => {
            console.log(local1);
            console.log(local2);
          }, [local1]);
        }",
        r"function MyComponent() {
          const local1 = {};
          const local2 = {};
          useMemo(() => {
            console.log(local1);
          }, [local1, local2]);
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          function MyNestedComponent() {
            const local2 = {};
            useCallback(() => {
              console.log(local1);
              console.log(local2);
            }, [local1]);
          }
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
            console.log(local);
          }, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
            console.log(local);
          }, [local, local]);
        }",
        r"function MyComponent() {
          useCallback(() => {}, [window]);
        }",
        r"function MyComponent(props) {
          let local = props.foo;
          useCallback(() => {}, [local]);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return history.listen();
          }, []);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return [
              history.foo.bar[2].dobedo.listen(),
              history.foo.bar().dobedo.listen[2]
            ];
          }, []);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return [
              history?.foo
            ];
          }, []);
        }",
        r"function MyComponent() {
          useEffect(() => {}, ['foo']);
        }",
        r"function MyComponent({ foo, bar, baz }) {
          useEffect(() => {
            console.log(foo, bar, baz);
          }, ['foo', 'bar']);
        }",
        r"function MyComponent({ foo, bar, baz }) {
          useEffect(() => {
            console.log(foo, bar, baz);
          }, [42, false, null]);
        }",
        r"function MyComponent() {
          const dependencies = [];
          useEffect(() => {}, dependencies);
        }",
        r"function MyComponent() {
          const local = {};
          const dependencies = [local];
          useEffect(() => {
            console.log(local);
          }, dependencies);
        }",
        r"function MyComponent() {
          const local = {};
          const dependencies = [local];
          useEffect(() => {
            console.log(local);
          }, [...dependencies]);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [local, ...dependencies]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [computeCacheKey(local)]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.items[0]);
          }, [props.items[0]]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.items[0]);
          }, [props.items, props.items[0]]);
        }",
        r"function MyComponent({ items }) {
          useEffect(() => {
            console.log(items[0]);
          }, [items[0]]);
        }",
        r"function MyComponent({ items }) {
          useEffect(() => {
            console.log(items[0]);
          }, [items, items[0]]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useCallback(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props, props.foo]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useCallback(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props.foo, props]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useCallback(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, []);
        }",
        // destructuring
        r"function MyComponent(props) {
          useCallback(() => {
            const { foo } = props;
            console.log(foo);
          }, [props.bar]);
        }",
        // FIXME: this should pass
        r"function MyComponent(props) {
          useCallback(() => {
            const { foo: { bar } } = props;
            console.log(bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          const foo = props.foo;
          useEffect(() => {
            const { bar } = foo();
            console.log(bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          useEffect(() => {
            console.log(local);
          }, [local.id]);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          const fn = useCallback(() => {
            console.log(local);
          }, [local.id]);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          const fn = useCallback(() => {
            console.log(local);
          }, [local.id, local]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          let color = {}
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
            console.log(color);
          }, [props.foo, props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz, props.foo]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
            console.log(props.foo.fizz.bizz);
          }, []);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props);
            console.log(props.hello);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [local, local]);
        }",
        r"function MyComponent() {
          const local1 = {};
          useCallback(() => {
            const local1 = {};
            console.log(local1);
          }, [local1]);
        }",
        r"function MyComponent() {
          const local1 = {};
          useCallback(() => {}, [local1]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, []);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, [c, a, g]);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, [a, c, g]);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, []);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, []);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, [props]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, []);
          useCallback(() => {
            console.log(props.foo);
          }, []);
          useMemo(() => {
            console.log(props.foo);
          }, []);
          React.useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useCallback(() => {
            console.log(props.foo);
          }, []);
          React.useMemo(() => {
            console.log(props.foo);
          }, []);
          React.notReactiveHook(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, []);
          useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useCustomEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [a ? local : b]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [a && local]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {}, [props?.attribute.method()]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {}, [props.method()]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          const [state, setState] = useState();
          useEffect(() => {
            ref.current = {};
            setState(state + 1);
          }, []);
        }",
        r"function MyComponent() {
          const ref = useRef();
          const [state, setState] = useState();
          useEffect(() => {
            ref.current = {};
            setState(state + 1);
          }, [ref]);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.focus();
            console.log(ref2.current.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, []);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.focus();
            console.log(ref2.current.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, [ref1.current, ref2.current, props.someOtherRefs, props.color]);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1?.current?.focus();
            console.log(ref2?.current?.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, [ref1?.current, ref2?.current, props.someOtherRefs, props.color]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref.current]);
        }",
        r"function MyComponent({ activeTab }) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.scrollTop = 0;
            ref2.current.scrollTop = 0;
          }, [ref1.current, ref2.current, activeTab]);
        }",
        r"function MyComponent({ activeTab, initY }) {
          const ref1 = useRef();
          const ref2 = useRef();
          const fn = useCallback(() => {
            ref1.current.scrollTop = initY;
            ref2.current.scrollTop = initY;
          }, [ref1.current, ref2.current, activeTab, initY]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref.current, ref]);
        }",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }), [])
        });",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props.onChange) {
              props.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props?.onChange) {
              props?.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            function play() {
              props.onPlay();
            }
            function pause() {
              props.onPause();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props.foo.onChange) {
              props.foo.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            props.onChange();
            if (props.foo.onChange) {
              props.foo.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          const [skillsCount] = useState();
          useEffect(() => {
            if (skillsCount === 0 && !props.isEditMode) {
              props.toggleEditMode();
            }
          }, [skillsCount, props.isEditMode, props.toggleEditMode]);
        }",
        r"function MyComponent(props) {
          const [skillsCount] = useState();
          useEffect(() => {
            if (skillsCount === 0 && !props.isEditMode) {
              props.toggleEditMode();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            externalCall(props);
            props.onChange();
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            props.onChange();
            externalCall(props);
          }, []);
        }",
        r"function MyComponent(props) {
          let value;
          let value2;
          let value3;
          let value4;
          let asyncValue;
          useEffect(() => {
            if (value4) {
              value = {};
            }
            value2 = 100;
            value = 43;
            value4 = true;
            console.log(value2);
            console.log(value3);
            setTimeout(() => {
              asyncValue = 100;
            });
          }, []);
        }",
        r"function MyComponent(props) {
          let value;
          let value2;
          let value3;
          let asyncValue;
          useEffect(() => {
            value = {};
            value2 = 100;
            value = 43;
            console.log(value2);
            console.log(value3);
            setTimeout(() => {
              asyncValue = 100;
            });
          }, [value, value2, value3]);
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef?.current?.addEventListener('mousemove', handleMove);
            return () => myRef?.current?.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          });
          return <div ref={myRef} />;
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          }, [myRef]);
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMouse = () => {};
            myRef.current.addEventListener('mousemove', handleMouse);
            myRef.current.addEventListener('mousein', handleMouse);
            return function() {
              setTimeout(() => {
                myRef.current.removeEventListener('mousemove', handleMouse);
                myRef.current.removeEventListener('mousein', handleMouse);
              });
            }
          }, [myRef]);
        }",
        r"function useMyThing(myRef, active) {
          useEffect(() => {
            const handleMove = () => {};
            if (active) {
              myRef.current.addEventListener('mousemove', handleMove);
              return function() {
                setTimeout(() => {
                  myRef.current.removeEventListener('mousemove', handleMove);
                });
              }
            }
          }, [myRef, active]);
        }",
        // TODO: enable once we support custom hooks
        // r"function MyComponent() {
        //   const myRef = useRef();
        //   useLayoutEffect_SAFE_FOR_SSR(() => {
        //     const handleMove = () => {};
        //     myRef.current.addEventListener('mousemove', handleMove);
        //     return () => myRef.current.removeEventListener('mousemove', handleMove);
        //   });
        //   return <div ref={myRef} />;
        // }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          const local4 = {};
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
            console.log(local4);
          }, [local1, local3]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            window.scrollTo(0, 0);
          }, [window]);
        }",
        r"import MutableStore from 'store';
        function MyComponent() {
          useEffect(() => {
            console.log(MutableStore.hello);
          }, [MutableStore.hello]);
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            useEffect(() => {
              console.log(MutableStore.hello.world, props.foo, x, y, z, global.stuff);
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            useEffect(() => {
              // nothing
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            const fn = useCallback(() => {
              // nothing
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            const fn = useCallback(() => {
              // nothing
            }, [MutableStore?.hello?.world, props.foo, x, y, z, global?.stuff]);
          }
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            setTimeout(() => console.log(taint));
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          // Shouldn't affect anything
          function handleChange() {}

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(taint);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          // Shouldn't affect anything
          const handleChange = () => {};

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(taint);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          function handleNext(value) {
            setState(value);
          }

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          const handleNext = (value) => {
            setState(value);
          };

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          const handleNext = (value) => {
            setState(value);
          };

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);

          return <div onClick={handleNext} />;
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, [handleNext1]);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, [handleNext2]);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, [handleNext3]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            handleNext1();
            return Store.subscribe(() => handleNext1());
          }, [handleNext1]);
          useLayoutEffect(() => {
            handleNext2();
            return Store.subscribe(() => handleNext2());
          }, [handleNext2]);
          useMemo(() => {
            handleNext3();
            return Store.subscribe(() => handleNext3());
          }, [handleNext3]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            handleNext1();
            return Store.subscribe(() => handleNext1());
          }, [handleNext1]);
          useLayoutEffect(() => {
            handleNext2();
            return Store.subscribe(() => handleNext2());
          }, [handleNext2]);
          useMemo(() => {
            handleNext3();
            return Store.subscribe(() => handleNext3());
          }, [handleNext3]);
          return (
            <div
              onClick={() => {
                handleNext1();
                setTimeout(handleNext2);
                setTimeout(() => {
                  handleNext3();
                });
              }}
            />
          );
        }",
        r"function MyComponent(props) {
          const handleNext1 = () => {
            console.log('hello');
          };
          function handleNext2() {
            console.log('hello');
          }
          useEffect(() => {
            return Store.subscribe(handleNext1);
            return Store.subscribe(handleNext2);
          }, [handleNext1, handleNext2]);
          useEffect(() => {
            return Store.subscribe(handleNext1);
            return Store.subscribe(handleNext2);
          }, [handleNext1, handleNext2]);
        }",
        r"function MyComponent(props) {
          let handleNext = () => {
            console.log('hello');
          };
          if (props.foo) {
            handleNext = () => {
              console.log('hello');
            };
          }
          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let taint = props.foo;

          function handleNext(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count + 1);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let [increment, setIncrement] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let [increment, setIncrement] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

           return <h1>{count}</h1>;
         }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let increment = useCustomHook();

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ step }) {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + step;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ step }) {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + step;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, [increment]);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ increment }) {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          const [count, setCount] = useState(0);

          function tick() {
            setCount(count + 1);
          }

          useEffect(() => {
            let id = setInterval(() => {
              tick();
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Podcasts() {
          useEffect(() => {
            alert(podcasts);
          }, []);
          let [podcasts, setPodcasts] = useState(null);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ api: { fetchPodcasts }, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, fetchPodcasts2, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            setTimeout(() => {
              console.log(id);
              fetchPodcasts(id).then(setPodcasts);
              fetchPodcasts2(id).then(setPodcasts);
            });
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            console.log(fetchPodcasts);
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            console.log(fetchPodcasts);
            fetchPodcasts?.(id).then(setPodcasts);
          }, [id]);
        }",
        // TODO: we currently report incorrectly for this case.
        // this is actually a user land bug (TS should catch this)
        // as `[fetchData]` is not in scope.
        // expected, `fetchData` = unnecessary dependency
        // actual: Outer scope values aren't considered dependencies as they don't re-render the component
        // r"function Thing() {
        //   useEffect(() => {
        //     const fetchData = async () => {};
        //     fetchData();
        //   }, [fetchData]);
        // }",
        r"function Hello() {
          const [state, setState] = useState(0);
          useEffect(() => {
            setState({});
          });
        }",
        r"function Hello() {
          const [data, setData] = useState(0);
          useEffect(() => {
            fetchData.then(setData);
          });
        }",
        r"function Hello({ country }) {
          const [data, setData] = useState(0);
          useEffect(() => {
            fetchData(country).then(setData);
          });
        }",
        r"function Hello({ prop1, prop2 }) {
          const [state, setState] = useState(0);
          useEffect(() => {
            if (prop1) {
              setState(prop2);
            }
          });
        }",
        r"function Thing() {
          useEffect(async () => {}, []);
        }",
        r"function Thing() {
          useEffect(async () => {});
        }",
        // NOTE: intentionally not supported, as `foo` would be referenced before it's declaration
        // r"function Example() {
        //   const foo = useCallback(() => {
        //     foo();
        //     }, [foo]);
        //     }",
        r"function Example({ prop }) {
          const foo = useCallback(() => {
            prop.hello(foo);
          }, [foo]);
          const bar = useCallback(() => {
            foo();
          }, [foo]);
        }",
        r"function MyComponent() {
          const local = {};
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = function() {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = () => {
            otherThing();
          };
          const otherThing = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);

          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);
          useEffect(myEffect, [local]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(debounce(() => {
            console.log(local);
          }, delay), []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, []);
        }",
        r"function MyComponent(props) {
          let foo = {}
          useEffect(() => {
            foo.bar.baz = 43;
            props.foo.bar.baz = 1;
          }, []);
        }",
        r"function Component() {
          const foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = [];
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = () => {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = function bar(){};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = class {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = true ? {} : 'fine';
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar || {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar ?? {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar && {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar ? baz ? {} : null : null;
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          let foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          var foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useCallback(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useLayoutEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useImperativeHandle(
            ref,
            () => {
               console.log(foo);
            },
            [foo]
          );
        }",
        r"function Foo(section) {
          const foo = section.section_components?.edges ?? [];
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo(section) {
          const foo = {};
          console.log(foo);
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = <>Hi!</>;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = <div>Hi!</div>;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = bar = {};
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = new String('foo'); // Note 'foo' will be boxed, and thus an object and thus compared by reference.
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = new Map([]);
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = /reg/;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          class Bar {};
          useMemo(() => {
            console.log(new Bar());
          }, [Bar]);
        }",
        r"function Foo() {
          const foo = {};
          useLayoutEffect(() => {
            console.log(foo);
          }, [foo]);
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        // https://github.com/oxc-project/oxc/issues/10319
        r"import { useEffect } from 'react'

        export const Test = () => {
          const handleFrame = () => {
            setTimeout(handleFrame)
          }

          useEffect(() => {
            setTimeout(handleFrame)
          }, [])

          return (
            <></>
          )
        }",
        // https://github.com/oxc-project/oxc/issues/9788
        r#"import { useCallback, useEffect } from "react";

        function Component({ foo }) {
          const log = useCallback(() => {
          console.log(foo);
        }, [foo]);
        useEffect(() => {
          log();
        }, []);
        }"#,
        r"function MyComponent({ theme }) {
          const onStuff = useEffectEvent(() => {
            showNotification(theme);
          });
          useEffect(() => {
            onStuff();
          }, [onStuff]);
          React.useEffect(() => {
            onStuff();
          }, [onStuff]);
        }",
    ];

    let pass_additional_hooks = vec![(
        "function MyComponent(props) {
          useSpecialEffect(() => {
            console.log(props.foo);
          });
        }",
        Some(serde_json::json!([{ "additionalHooks": "useSpecialEffect" }])),
    )];

    let pass_additional_hooks_empty_string = vec![(
        "function MyComponent(props) {
          const foo = bar.add();
        }
        ",
        Some(serde_json::json!([{ "additionalHooks": "" }])),
    )];

    let fail_additional_hooks = vec![(
        "function MyComponent() {
          const [state, setState] = React.useState<number>(0);

          useSpecialEffect(() => {
            const someNumber: typeof state = 2;
            setState(prevState => prevState + someNumber + state);
          }, [])
        }",
        Some(serde_json::json!([{ "additionalHooks": "useSpecialEffect" }])),
    )];

    let fix = vec![
        (
            "const useHook = x => useCallback(() => x)",
            "const useHook = x => useCallback(() => x, [])",
            // None,
            // FixKind::SafeFix,
        ),
        (
            "const useHook = x => useCallback(() => { return x; })",
            "const useHook = x => useCallback(() => { return x; }, [])",
            // None,
            // FixKind::SafeFix,
        ),
        (
            r"const useHook = () => {
              const [state, setState] = useState(0);
              const foo = useCallback(() => state, []);
            }",
            r"const useHook = () => {
              const [state, setState] = useState(0);
              const foo = useCallback(() => state, [state]);
            }",
            // None,
            // FixKind::DangerousSuggestion,
        ),
        (
            r"const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const foo = useCallback(() => x + y, []);
            }",
            r"const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const foo = useCallback(() => x + y, [x, y]);
            }",
            // None,
            // FixKind::DangerousSuggestion,
        ),
        (
            "const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const [z] = useState(0);
              const foo = useCallback(() => x + y + z, [x]);
            }",
            "const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const [z] = useState(0);
              const foo = useCallback(() => x + y + z, [\n\tx,\n\ty,\n\tz\n]);
            }",
            // None,
            // FixKind::DangerousSuggestion,
        ),
        (
            r"const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const [z] = useState(0);
              const foo = useCallback(() => x + y + z, [x, y]);
            }",
            "const useHook = () => {
              const [x] = useState(0);
              const [y] = useState(0);
              const [z] = useState(0);
              const foo = useCallback(() => x + y + z, [\n\tx,\n\ty,\n\tz\n]);
            }",
            // None,
            // FixKind::DangerousSuggestion,
        ),
        // (
        //     r#"const useHook = () => {
        //       const [state, setState] = useState(0);
        //       const foo = useCallback(() => state);
        //     }"#,
        //     r#"const useHook = () => {
        //       const [state, setState] = useState(0);
        //       const foo = useCallback(() => state, [state]);
        //     }"#,
        //     // None,
        //     // FixKind::DangerousSuggestion,
        // ),
        // Test missing dependency fixes
        (
            "function MyComponent() { const local = someFunc(); useEffect(() => { console.log(local); }, []); }",
            "function MyComponent() { const local = someFunc(); useEffect(() => { console.log(local); }, [local]); }",
        ),
        (
            "function MyComponent(props) { useEffect(() => { console.log(props.foo); }, []); }",
            "function MyComponent(props) { useEffect(() => { console.log(props.foo); }, [props.foo]); }",
        ),
        (
            "function MyComponent(props) { useEffect(() => { console.log(props.foo, props.bar); }, []); }",
            "function MyComponent(props) { useEffect(() => { console.log(props.foo, props.bar); }, [props.foo, props.bar]); }",
        ),
        // Test adding to existing dependencies
        (
            "function MyComponent(props) { const local = someFunc(); useEffect(() => { console.log(props.foo, local); }, [props.foo]); }",
            "function MyComponent(props) { const local = someFunc(); useEffect(() => { console.log(props.foo, local); }, [props.foo, local]); }",
        ),
        // Test dependency array creation for hooks that require it
        (
            "function MyComponent() { const fn = useCallback(() => { alert('foo'); }); }",
            "function MyComponent() { const fn = useCallback(() => { alert('foo'); }, []); }",
        ),
        (
            "function MyComponent() { const value = useMemo(() => { return 2*2; }); }",
            "function MyComponent() { const value = useMemo(() => { return 2*2; }, []); }",
        ),
        // Test unnecessary dependency removal for non-effect hooks
        (
            "function MyComponent() { const local1 = {}; useCallback(() => {}, [local1]); }",
            "function MyComponent() { const local1 = {}; useCallback(() => {}, []); }",
        ),
        // Test duplicate dependency removal
        (
            "function MyComponent() { const local = {}; useEffect(() => { console.log(local); }, [local, local]); }",
            "function MyComponent() { const local = {}; useEffect(() => { console.log(local); }, [local]); }",
        ),
        (
            "const x = {}; function Comp() { useEffect(() => {}, [x]) }",
            "const x = {}; function Comp() { useEffect(() => {}, []) }",
        ),
    ];

    Tester::new(
        ExhaustiveDeps::NAME,
        ExhaustiveDeps::PLUGIN,
        pass.iter()
            .map(|&code| (code, None))
            .chain(pass_additional_hooks)
            .chain(pass_additional_hooks_empty_string)
            .collect::<Vec<_>>(),
        fail.iter().map(|&code| (code, None)).chain(fail_additional_hooks).collect::<Vec<_>>(),
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
