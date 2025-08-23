use std::{borrow::Cow, fmt::Write, hash::Hash};

use itertools::Itertools;
use lazy_regex::Regex;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind, AstType,
    ast::{
        Argument, ArrayExpressionElement, ArrowFunctionExpression, BindingPattern,
        BindingPatternKind, CallExpression, ChainElement, ChainExpression, Expression,
        FormalParameters, Function, FunctionBody, IdentifierReference, ReturnStatement,
        StaticMemberExpression, TSTypeAnnotation, TSTypeParameterInstantiation, TSTypeReference,
        VariableDeclarationKind, VariableDeclarator,
    },
    match_expression,
};
use oxc_ast_visit::{Visit, walk::walk_function_body};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
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

// fn missing_dependency_diagnostic(
//     hook_name: &str,
//     deps: &[Name<'_>],
//     _dependency_array_span: Span,
// ) -> OxcDiagnostic {
//     // Sort dependencies by name for deterministic output
//     let mut deps_sorted = deps.to_vec();
//     deps_sorted.sort_by(|a, b| a.name.cmp(&b.name));
//     let single = deps_sorted.len() == 1;
//     let deps_pretty = if single {
//         format!("'{}'", deps_sorted[0])
//     } else {
//         let mut iter = deps_sorted.iter();
//         let all_but_last = iter
//             .by_ref()
//             .take(deps_sorted.len() - 1)
//             .map(|s| format!("'{}'", s))
//             .collect::<Vec<_>>()
//             .join(", ");
//         let last = iter.next().unwrap();
//         format!("{all_but_last}, and '{last}'")
//     };

//     let main_label_span = deps_sorted[0].span;
//     let labels: Vec<_> = if single {
//         vec![]
//     } else {
//         deps_sorted.iter().skip(1).map(|dep| dep.span.label("")).collect()
//     };

//     OxcDiagnostic::warn(format!(
//         "React Hook {hook_name} has {}missing dependenc{}: {deps_pretty}",
//         if single { "a " } else { "" },
//         if single { "y" } else { "ies" }
//     ))
//     .with_help("Either include it or remove the dependency array.")
//     .with_labels(labels)
//     .with_error_code_scope(SCOPE)
// }

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

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveDeps(Box<ExhaustiveDepsConfig>);

#[derive(Debug, Clone, Default)]
pub struct ExhaustiveDepsConfig {
    additional_hooks: Option<Regex>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExhaustiveDepsConfigJson {
    #[serde(rename = "additionalHooks")]
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
    ///
    /// ### Options
    ///
    /// #### additionalHooks
    ///
    /// `{ type: string }`
    ///
    /// Optionally provide a regex of additional hooks to check.
    ///
    /// Example:
    ///
    /// ```json
    /// { "react/exhaustive-deps": ["error", { "additionalHooks": "useSpecialEffect" }] }
    /// ```
    ExhaustiveDeps,
    react,
    correctness,
    safe_fixes_and_dangerous_suggestions
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

        let is_hook = self.get_reactive_hook_callback_index(hook_name).is_some();

        if dependencies_node.is_none() {
            if HOOKS_USELESS_WITHOUT_DEPENDENCIES.contains(&hook_name.as_str()) {
                ctx.diagnostic_with_fix(
                    dependency_array_required_diagnostic(hook_name.as_str(), call_expr.span()),
                    |fixer| fixer.insert_text_after(callback_node, ", []"),
                );
                return;
            } else if !is_hook {
                return;
            }
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

        if callback_node.is_async() && is_hook {
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
            let mut found_dependencies = ExhaustiveDepsVisitor::new(ctx.semantic(), is_hook);
            found_dependencies.current_hook_name = Some(hook_name.to_string());

            found_dependencies.visit_formal_parameters(callback_node.parameters());

            if let Some(function_body) = callback_node.body() {
                found_dependencies.visit_function_body_root(function_body);
            }

            (found_dependencies.found_dependencies, found_dependencies.refs_inside_cleanups)
        };

        if is_hook {
            for (span, reference_id) in refs_inside_cleanups {
                let reference = ctx.scoping().get_reference(reference_id);
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

                // Skip if this ref has write references (it's being mutated)
                if has_write_reference {
                    continue;
                }

                // Check if this is a reactive value (useRef, prop, or parameter) that has a .current property
                let Some(declaration) = get_declaration_from_reference_id(reference_id, ctx) else {
                    continue;
                };

                let is_reactive = match declaration.kind() {
                    AstKind::VariableDeclarator(declarator) => {
                        // Check if the initializer is a useRef call
                        match &declarator.init {
                            Some(init) => match init.get_inner_expression() {
                                Expression::CallExpression(call_expr) => {
                                    match func_call_without_react_namespace(call_expr) {
                                        Some(init_name) => init_name == "useRef",
                                        None => false,
                                    }
                                }
                                _ => false,
                            },
                            None => false,
                        }
                    }
                    AstKind::FormalParameter(_) => {
                        // Parameters can be refs if they're passed as props
                        // Since we're already in the context of accessing .current,
                        // this parameter is being used as a ref
                        true
                    }
                    _ => false,
                };

                if !is_reactive {
                    continue;
                }

                ctx.diagnostic(ref_accessed_directly_in_effect_cleanup_diagnostic(span));
            }
        }

        let Some(dependencies_node) = dependencies_node else {
            if is_hook {
                let contains_set_state_call = {
                    let mut finder = ExhaustiveDepsVisitor::new(ctx.semantic(), true);
                    // Visit the function node itself, not just its body
                    match callback_node {
                        CallbackNode::Function(func) => {
                            finder.enter_node(AstKind::Function(func));
                            if let Some(function_body) = &func.body {
                                finder.visit_function_body_root(function_body);
                            }
                            finder.leave_node(AstKind::Function(func));
                        }
                        CallbackNode::ArrowFunction(arrow_func) => {
                            finder.enter_node(AstKind::ArrowFunctionExpression(arrow_func));
                            finder.visit_function_body_root(&arrow_func.body);
                            finder.leave_node(AstKind::ArrowFunctionExpression(arrow_func));
                        }
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

                    if let Ok(dep) = analyze_property_chain(elem, ctx, true) {
                        dep
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
                    ctx.diagnostic(literal_in_dependency_array_diagnostic(span));
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

            ctx.diagnostic(unnecessary_outer_scope_dependency_diagnostic(
                hook_name,
                &dependency.name,
                dependency.span,
            ));
        }

        // Instead of using .difference(&declared_dependencies),
        // iterate over all found_dependencies and only consider a dependency missing if no declared dependency contains it.
        let undeclared_deps = found_dependencies.iter().filter(|dep| {
            // Check if any declared dependency contains/satisfies this found dependency
            if declared_dependencies.iter().any(|decl_dep| {
                if hook_name == "useEffect" {
                    // useEffect allows overspecification, so use the lenient contains logic
                    decl_dep.contains(dep)
                } else {
                    // Other hooks require exact matches or broader dependencies only
                    // They don't allow overspecification (more specific declared deps)
                    if decl_dep.name != dep.name {
                        return false;
                    }
                    // For same-name dependencies, only allow exact matches or broader declared deps
                    if decl_dep.chain.is_empty() {
                        // Broader declared dependency (e.g., `props` satisfies `props.foo`)
                        true
                    } else if dep.chain.is_empty() {
                        // Found dependency is broader than declared (e.g., declared `props.foo` but found `props`)
                        false
                    } else {
                        // Both have chains, check if declared is prefix of found (broader or equal)
                        dep.chain.starts_with(&decl_dep.chain)
                    }
                }
            }) {
                return false;
            }

            let is_dep = is_identifier_a_dependency(
                dep.name,
                dep.reference_id,
                dep.span,
                ctx,
                component_scope_id,
            );

            if !is_dep {
                return false;
            }

            true
        });

        let undeclared_deps_vec = undeclared_deps.collect::<Vec<_>>();
        let undeclared_count = undeclared_deps_vec.len();
        if undeclared_count > 0 {
            // Deduplicate undeclared dependencies: remove any dependency that is contained by another
            let mut filtered_deps = Vec::new();
            for dep in &undeclared_deps_vec {
                let is_contained = undeclared_deps_vec
                    .iter()
                    .any(|other| other != dep && other.contains_for_dedup(dep));
                if !is_contained {
                    filtered_deps.push(dep);
                }
            }
            let undeclared =
                filtered_deps.into_iter().map(|dep| Name::from(*dep)).collect::<Vec<_>>();
            if !undeclared.is_empty() {
                ctx.diagnostic_with_dangerous_suggestion(
                    missing_dependency_diagnostic(hook_name, &undeclared, dependencies_node.span()),
                    |fixer| fix::append_dependencies(fixer, &undeclared, dependencies_node),
                );
            }
        }

        // Check for unnecessary dependencies for all hooks
        {
            // FIRST: Check for redundant dependencies within the same array
            // All hooks should flag redundant dependencies (when one declared dependency contains another)

            if hook_name != "useEffect" {
                declared_dependencies.iter().tuple_combinations().for_each(|(a, b)| {
                    if a.is_redundant_with(b) {
                        ctx.diagnostic(unnecessary_dependency_diagnostic(
                            hook_name,
                            &b.to_string(), // Report the more specific dependency as unnecessary
                            dependencies_node.span,
                        ));
                    } else if b.is_redundant_with(a) {
                        ctx.diagnostic(unnecessary_dependency_diagnostic(
                            hook_name,
                            &a.to_string(), // Report the more specific dependency as unnecessary
                            dependencies_node.span,
                        ));
                    }
                });
            }

            // SECOND: Check for truly unnecessary dependencies (declared but never used)
            // But exclude dependencies that are made redundant by other declared dependencies
            let mut unnecessary_deps: Vec<_> = declared_dependencies
                .difference(&found_dependencies)
                .filter(|dep| {
                    // Don't report a dependency as unnecessary if it contains other declared dependencies
                    // (i.e., it's the broader dependency in a redundant pair)
                    !declared_dependencies
                        .iter()
                        .any(|other_dep| other_dep != *dep && dep.contains(other_dep))
                })
                .collect();
            unnecessary_deps.sort_by(|a, b| a.name.cmp(&b.name));

            // Check for overly specific dependencies (declared dependency is more specific than what's actually used)
            // useEffect allows overspecification, but other hooks should flag overly specific dependencies
            if hook_name != "useEffect" {
                for dep in &declared_dependencies {
                    // Skip if this dependency is actually used (exact match)
                    if found_dependencies.contains(dep) {
                        continue;
                    }

                    // Check if this declared dependency is overly specific:
                    // A declared dependency is overly specific if any found dependency is LESS specific
                    // but refers to the same base (e.g., declared `props.foo.bar.baz` but only used `props.foo.bar`)
                    if found_dependencies.iter().any(|found_dep| {
                        found_dep.name == dep.name
                            && found_dep.chain.len() < dep.chain.len()
                            && found_dep.chain.iter().zip(dep.chain.iter()).all(|(a, b)| a == b)
                    }) {
                        ctx.diagnostic(unnecessary_dependency_diagnostic(
                            hook_name,
                            &dep.to_string(),
                            dependencies_node.span,
                        ));
                    }
                }
            }

            // Only flag unnecessary dependencies when the callback is completely empty
            // and the dependency is a simple local variable (not a complex prop chain)
            let is_empty_callback = match callback_node {
                CallbackNode::Function(func) => {
                    func.body.as_ref().is_none_or(|body| body.statements.is_empty())
                }
                CallbackNode::ArrowFunction(arrow) => arrow.body.statements.is_empty(),
            };

            // Check for unnecessary dependencies
            // For useEffect, only check in empty callbacks to allow overspecification
            // For other hooks, check in all callbacks
            if hook_name == "useEffect" {
                // For useEffect, only flag unnecessary dependencies in empty callbacks
                if is_empty_callback {
                    for dep in &unnecessary_deps {
                        // For useEffect, allow simple local variable overspecification
                        // but still flag complex unused dependencies
                        let is_simple_local = dep.chain.is_empty() && {
                            if let Some(symbol_id) = dep.symbol_id {
                                // Check if it's a simple local variable (not a prop or complex expression)
                                let decl = ctx.semantic().symbol_declaration(symbol_id);
                                matches!(decl.kind(), AstKind::VariableDeclarator(_))
                            } else {
                                false
                            }
                        };

                        if is_simple_local {
                            continue; // Allow simple local variable overspecification for useEffect
                        }

                        // For complex dependencies, still flag them as unnecessary
                        ctx.diagnostic(unnecessary_dependency_diagnostic(
                            hook_name,
                            &dep.to_string(),
                            dependencies_node.span,
                        ));
                    }
                }
            } else {
                // For other hooks (useMemo, useCallback, etc.), flag unnecessary dependencies in all callbacks
                for dep in &unnecessary_deps {
                    if found_dependencies.iter().any(|found_dep| dep.contains(found_dep)) {
                        continue;
                    }
                    ctx.diagnostic(unnecessary_dependency_diagnostic(
                        hook_name,
                        &dep.to_string(),
                        dependencies_node.span,
                    ));
                }
            }
        }

        for dep in declared_dependencies {
            let Some(symbol_id) = dep.symbol_id else { continue };

            if dep.chain.is_empty() && is_symbol_declaration_referentially_unique(symbol_id, ctx) {
                let name = ctx.scoping().symbol_name(symbol_id);
                let decl_span = ctx.scoping().symbol_span(symbol_id);
                ctx.diagnostic(dependency_changes_on_every_render_diagnostic(
                    hook_name, dep.span, name, decl_span,
                ));
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
        let mut result = self.name.to_string();
        for prop in &self.chain {
            write!(result, ".{prop}").unwrap();
        }
        result
    }

    fn contains(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        chain_contains(&self.chain, &other.chain)
    }

    /// Check if this dependency (broader) contains another dependency (narrower) for deduplication
    fn contains_for_dedup(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        // For deduplication: self contains other if self's chain is a prefix of other's chain
        // (self is broader than or equal to other)
        other.chain.starts_with(&self.chain)
    }

    fn is_redundant_with(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        // For redundant dependency checking, self is redundant with other if:
        // self (broader) contains other (narrower) in a unidirectional way
        redundant_chain_contains(&self.chain, &other.chain)
    }
}

fn redundant_chain_contains(self_chain: &[Atom<'_>], other_chain: &[Atom<'_>]) -> bool {
    // For redundant dependency checking, self should contain other if:
    // 1. self_chain is empty (e.g., `props` contains `props.foo`)
    // 2. other_chain starts with self_chain and is longer or equal (e.g., `props.foo` contains `props.foo.bar`)
    if self_chain.is_empty() {
        return true;
    }
    // Check if self_chain is a prefix of other_chain
    // This means self (broader) contains other (narrower)
    other_chain.starts_with(self_chain) && other_chain.len() >= self_chain.len()
}

fn chain_contains(declared_chain: &[Atom<'_>], found_chain: &[Atom<'_>]) -> bool {
    // If declared chain is empty, it contains everything (e.g., `props` contains `props.foo`)
    if declared_chain.is_empty() {
        return true;
    }

    // If found chain is empty but declared chain is not, the declared dependency
    // is more specific than needed (e.g., declaring `local.id` when using `local`)
    if found_chain.is_empty() {
        return false;
    }

    // Check if declared_chain is a prefix of found_chain
    // This means the declared dependency covers the found usage (e.g., `props.foo` covers `props.foo.bar`)
    if found_chain.starts_with(declared_chain) {
        // Allow exact matches and broader dependencies to satisfy the requirement
        return found_chain.len() >= declared_chain.len();
    }

    // Only allow a more specific declared dependency to satisfy a broader found usage
    // if the found usage is not the base identifier itself
    // (e.g., declaring `props.foo.bar` satisfies usage of `props.foo`, but NOT usage of just `props`)
    if declared_chain.starts_with(found_chain) && !found_chain.is_empty() {
        return true;
    }

    false
}

fn analyze_property_chain<'a, 'b>(
    expr: &'b Expression<'a>,
    semantic: &'b Semantic<'a>,
    strict_mode: bool,
) -> Result<Option<Dependency<'a>>, ()> {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) => {
            let dep = Dependency {
                span: ident.span(),
                name: ident.name,
                reference_id: ident.reference_id(),
                chain: vec![],
                symbol_id: semantic.scoping().get_reference(ident.reference_id()).symbol_id(),
            };
            Ok(Some(dep))
        }
        // TODO; is this correct?
        Expression::JSXElement(_) => Ok(None),
        Expression::StaticMemberExpression(expr) => concat_members(expr, semantic, strict_mode),
        Expression::ChainExpression(chain_expr) => {
            match &chain_expr.expression {
                ChainElement::StaticMemberExpression(expr) => {
                    // For ChainExpression containing StaticMemberExpression,
                    // we need special handling for optional chaining
                    analyze_optional_static_member(expr, semantic, strict_mode)
                }
                ChainElement::CallExpression(call_expr) => {
                    // For optional calls like props.foo?.toString()
                    // We need to analyze the callee, but stop at optional boundaries
                    analyze_optional_call_chain(&call_expr.callee, semantic, strict_mode)
                }
                ChainElement::ComputedMemberExpression(expr) => {
                    // For computed member access like props.foo?.[bar]
                    // Analyze the object part
                    analyze_property_chain(&expr.object, semantic, strict_mode)
                }
                _ => Err(()),
            }
        }
        Expression::CallExpression(call_expr) => {
            if strict_mode {
                // Function calls like props.method() should not be allowed in dependency arrays
                // They are complex expressions that should be rejected
                Err(())
            } else {
                // In lenient mode (useEffect body), we want to extract dependencies from function calls
                // For call expressions like history.foo.bar().something,
                // we want to extract the root dependency (history) from the callee
                analyze_property_chain(&call_expr.callee, semantic, strict_mode)
            }
        }
        Expression::ComputedMemberExpression(_computed_expr) => {
            // For computed member expressions like history.foo[bar],
            // we want to extract the root dependency (history) from the object
            // Computed member expressions like history.foo[bar] are complex expressions
            // that should not be allowed in dependency arrays
            Err(())
        }
        _ => Err(()),
    }
}

fn concat_members<'a, 'b>(
    member_expr: &'b StaticMemberExpression<'a>,
    semantic: &'b Semantic<'a>,
    strict_mode: bool,
) -> Result<Option<Dependency<'a>>, ()> {
    let Some(source) = analyze_property_chain(&member_expr.object, semantic, strict_mode)? else {
        return Ok(None);
    };

    // Always build the full chain, regardless of optional chaining
    // For dependency arrays, we want the full chain even when optional chaining is present
    // e.g., props?.foo?.bar should become props.foo.bar
    let new_chain = Vec::from([member_expr.property.name]);
    let result = Dependency {
        span: member_expr.span, // Use the full property chain span to match original snapshot
        name: source.name,
        reference_id: source.reference_id,
        chain: [source.chain, new_chain].concat(),
        symbol_id: semantic.scoping().get_reference(source.reference_id).symbol_id(),
    };
    Ok(Some(result))
}

fn analyze_optional_static_member<'a, 'b>(
    member_expr: &'b StaticMemberExpression<'a>,
    semantic: &'b Semantic<'a>,
    strict_mode: bool,
) -> Result<Option<Dependency<'a>>, ()> {
    // This function is called when we're inside a ChainExpression,
    // and we want to build the full chain even when optional chaining is present
    // For props?.foo?.bar, we want to build the full chain: props.foo.bar

    // Get the object dependency first
    let Some(source) = analyze_property_chain(&member_expr.object, semantic, strict_mode)? else {
        return Ok(None);
    };

    // Always build the full chain, regardless of optional chaining
    let new_chain = Vec::from([member_expr.property.name]);
    let result = Dependency {
        span: member_expr.span, // Use the full property chain span to match original snapshot
        name: source.name,
        reference_id: source.reference_id,
        chain: [source.chain, new_chain].concat(),
        symbol_id: semantic.scoping().get_reference(source.reference_id).symbol_id(),
    };
    Ok(Some(result))
}

fn analyze_optional_call_chain<'a, 'b>(
    expr: &'b Expression<'a>,
    semantic: &'b Semantic<'a>,
    strict_mode: bool,
) -> Result<Option<Dependency<'a>>, ()> {
    if strict_mode {
        // In strict mode (dependency arrays), optional method calls should be rejected as complex expressions
        return Err(());
    }

    // For optional method calls like props.foo?.toString(), we want to extract
    // the dependency from the object being called on (props.foo), not the full chain
    if let Expression::StaticMemberExpression(member_expr) = expr {
        // Extract dependency from the object part: props.foo?.toString() -> props.foo
        analyze_property_chain(&member_expr.object, semantic, strict_mode)
    } else {
        // For other call expressions, analyze the callee
        analyze_property_chain(expr, semantic, strict_mode)
    }
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

    // If the declaration is a function parameter, only treat as dependency if it's from the component scope
    if matches!(declaration.kind(), AstKind::FormalParameter(_)) {
        // Check if the parameter belongs to the component scope or a parent scope
        if declaration.scope_id() == component_scope_id {
            return true;
        }
        return false;
    }

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

    // Variable was declared inside a child scope (but not directly in the component scope)
    // ```tsx
    // function MyComponent() {
    //   useEffect(() => {
    //     const id = crypto.randomUUID();  // <- This should not be a dependency
    //     console.log(id);
    //   }, []);
    //  return <div />;
    // }
    // But functions declared directly in the component scope should be dependencies if they reference reactive values:
    // function MyComponent() {
    //   function increment(x) { return x + step; }  // <- This should be a dependency
    //   useEffect(() => { increment(1); }, []);
    // }
    if scopes.iter_all_scope_child_ids(component_scope_id).any(|id| id == declaration.scope_id())
        && declaration.scope_id() != component_scope_id
    {
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
    if let Some(symbol_id) = ctx.scoping().get_reference(ident_reference_id).symbol_id() {
        if !visited.insert(symbol_id) {
            return true;
        }
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

            if init_name == "useRef" {
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
                        matches!(
                            ctx.nodes().parent_kind(reference.node_id()),
                            AstKind::IdentifierReference(_) | AstKind::AssignmentExpression(_)
                        )
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
        let mut collector = ExhaustiveDepsVisitor::new(ctx.semantic(), true);
        collector.visit_function_body(function_body);
        collector.found_dependencies
    };

    deps.iter().all(|dep| {
        // Skip function parameters of the current function being analyzed - they don't make a function unstable
        // But parameters from outer scopes (like component props) should be considered reactive
        if let Some(symbol_id) = dep.symbol_id {
            let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
            if matches!(declaration.kind(), AstKind::FormalParameter(_)) {
                // If this parameter is from the component scope, it's a reactive prop
                // If it's from a different scope, it's likely a parameter of the current function being analyzed
                if declaration.scope_id() == component_scope_id {
                } else {
                    return true; // Function parameters are stable
                }
            }
        }

        let recursive_check = dep.symbol_id.zip(function_symbol_id).is_none_or(|(l, r)| l != r);
        let dep_check = !is_identifier_a_dependency_impl(
            dep.name,
            dep.reference_id,
            dep.span,
            ctx,
            component_scope_id,
            visited,
        );
        recursive_check && dep_check
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

fn walk_props<'a, F: FnMut(Vec<Atom<'a>>)>(
    props: &'a [oxc_ast::ast::BindingProperty<'a>],
    prefix: &[Atom<'a>],
    cb: &mut F,
    needs_full_identifier: &mut bool,
) {
    for prop in props {
        if prop.computed {
            *needs_full_identifier = true;
            continue;
        }
        match &prop.value.kind {
            BindingPatternKind::BindingIdentifier(_id) => {
                let mut chain = prefix.to_owned();
                if let Some(key) = prop.key.name() {
                    chain.push(Atom::from(Box::leak(key.to_string().into_boxed_str()) as &str));
                    cb(chain);
                } else {
                    *needs_full_identifier = true;
                }
            }
            BindingPatternKind::AssignmentPattern(pat) => {
                if let Some(id) = pat.left.get_binding_identifier() {
                    let mut chain = prefix.to_owned();
                    if let Some(key) = prop.key.name() {
                        chain.push(Atom::from(Box::leak(key.to_string().into_boxed_str()) as &str));
                        chain.push(id.name);
                        cb(chain);
                    } else {
                        *needs_full_identifier = true;
                    }
                } else {
                    *needs_full_identifier = true;
                }
            }
            BindingPatternKind::ArrayPattern(_) => {
                if let Some(key) = prop.key.name() {
                    let mut new_prefix = prefix.to_owned();
                    new_prefix
                        .push(Atom::from(Box::leak(key.to_string().into_boxed_str()) as &str));
                    cb(new_prefix);
                } else {
                    *needs_full_identifier = true;
                }
            }
            BindingPatternKind::ObjectPattern(obj_pat) => {
                if let Some(key) = prop.key.name() {
                    let mut new_prefix = prefix.to_owned();
                    new_prefix
                        .push(Atom::from(Box::leak(key.to_string().into_boxed_str()) as &str));
                    walk_props(&obj_pat.properties, &new_prefix, cb, needs_full_identifier);
                } else {
                    *needs_full_identifier = true;
                }
            }
        }
    }
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
    found_dependencies: FxHashSet<Dependency<'a>>,
    refs_inside_cleanups: Vec<(Span, ReferenceId)>,
    in_top_level_cleanup: bool,
    in_nested_function_in_cleanup: bool,
    in_nested_function: bool,
    in_react_hook_context: bool,
    current_hook_name: Option<String>,
}

impl<'a, 'b> ExhaustiveDepsVisitor<'a, 'b> {
    fn new(semantic: &'b Semantic<'a>, in_react_hook_context: bool) -> Self {
        Self {
            semantic,
            stack: vec![],
            decl_stack: vec![],
            skip_reporting_dependency: false,
            set_state_call: false,
            found_dependencies: FxHashSet::default(),
            refs_inside_cleanups: vec![],
            in_top_level_cleanup: false,
            in_nested_function_in_cleanup: false,
            in_nested_function: false,
            in_react_hook_context,
            current_hook_name: None,
        }
    }

    fn visit_function_body_root(&mut self, function_body: &FunctionBody<'a>) {
        walk_function_body(self, function_body);
    }

    fn is_react_hook_call(callee: &Expression<'a>) -> bool {
        if let Some(name) = get_node_name_without_react_namespace(callee) {
            // Check if it's a built-in React hook
            matches!(
                name.as_str(),
                "useState"
                    | "useEffect"
                    | "useContext"
                    | "useReducer"
                    | "useCallback"
                    | "useMemo"
                    | "useRef"
                    | "useImperativeHandle"
                    | "useLayoutEffect"
                    | "useDebugValue"
                    | "useDeferredValue"
                    | "useTransition"
                    | "useId"
                    | "useSyncExternalStore"
                    | "useInsertionEffect"
            )
        } else {
            false
        }
    }

    fn is_inside_nested_function(&self) -> bool {
        // Check if we're inside a nested function by looking at the stack
        // We're inside a nested function if we have ArrowFunctionExpression or Function in the stack
        // and we're not at the top level of the effect
        let mut function_count = 0;

        for node_type in &self.stack {
            match node_type {
                AstType::ArrowFunctionExpression | AstType::Function => {
                    function_count += 1;
                }
                _ => {}
            }
        }

        // We're inside a nested function if we have more than one function in the stack
        // (the main effect function + at least one nested function)
        function_count > 1
    }

    fn iter_destructure_bindings<F>(&self, mut cb: F) -> Option<bool>
    where
        F: FnMut(Vec<Atom<'a>>),
    {
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

        walk_props(&obj.properties, &[], &mut cb, &mut needs_full_identifier);
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

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        if let Some(argument) = &it.argument {
            let was_in_cleanup = self.in_top_level_cleanup;
            // Only detect cleanup functions when we're in a React hook context AND it's an effect hook
            let is_effect_hook = self
                .current_hook_name
                .as_ref()
                .is_some_and(|name| name == "useEffect" || name == "useLayoutEffect");
            if !self.in_top_level_cleanup && self.in_react_hook_context && is_effect_hook {
                self.in_top_level_cleanup = true;
                match argument.get_inner_expression() {
                    Expression::FunctionExpression(func) => {
                        // The returned function IS the cleanup function, not a nested function
                        // We call the base visitor directly to avoid our custom function visitor logic
                        <Self as oxc_ast_visit::Visit>::visit_function(
                            self,
                            func,
                            ScopeFlags::empty(),
                        );
                    }
                    Expression::ArrowFunctionExpression(arrow) => {
                        // The returned arrow function IS the cleanup function, not a nested function
                        // We call the base visitor directly to avoid our custom arrow function visitor logic
                        <Self as oxc_ast_visit::Visit>::visit_arrow_function_expression(
                            self, arrow,
                        );
                    }
                    _ => {
                        self.visit_expression(argument);
                    }
                }
                self.in_top_level_cleanup = was_in_cleanup;
            } else {
                // We're already in a cleanup function, so this is a nested return
                // Or we're not in a React hook context, so just visit normally
                self.visit_expression(argument);
            }
        }
        // Do not call the base trait method, as we've already walked the children
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        self.stack.push(AstType::VariableDeclarator);
        // NOTE: decl_stack is only appended when visiting initializer
        // expression.
        self.visit_binding_pattern(&decl.id);
        if let Some(init) = &decl.init {
            // Check if this is destructuring from a function call
            // If so, don't add it to decl_stack to prevent incorrect chain creation
            let is_object_destructure =
                matches!(decl.id.kind, BindingPatternKind::ObjectPattern(_));
            let is_function_call =
                matches!(init.get_inner_expression(), Expression::CallExpression(_));

            if is_object_destructure && is_function_call {
                // For destructuring from function calls like `const { bar } = foo()`,
                // we need to extract the destructured properties and add them to the dependency chain
                if let BindingPatternKind::ObjectPattern(obj_pattern) = &decl.id.kind {
                    // First visit the call expression to get the base dependency (e.g., `foo`)
                    self.visit_expression(init);

                    // Then for each destructured property, add it as a dependency with the property chain
                    for prop in &obj_pattern.properties {
                        if let oxc_ast::ast::PropertyKey::StaticIdentifier(key) = &prop.key {
                            // If we can extract dependencies from the call expression, add the property to the chain
                            if let Expression::CallExpression(call_expr) =
                                init.get_inner_expression()
                            {
                                if let Some(dep) =
                                    analyze_property_chain(&call_expr.callee, self.semantic, false)
                                        .ok()
                                        .flatten()
                                {
                                    let mut extended_dep = dep;
                                    extended_dep.chain.push(key.name);
                                    extended_dep.span = prop.span;

                                    self.found_dependencies.insert(extended_dep);
                                }
                            }
                        }
                    }
                } else {
                    // For non-object destructuring from function calls, visit normally
                    self.visit_expression(init);
                }
            } else {
                // For other cases, use the normal logic that tracks destructuring chains
                self.perform_variable_declarator_visit(decl, init);
            }
        }
        self.stack.pop();
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Check if this is a setState call at the top level (not in nested functions)
        if let Expression::Identifier(ident) = &it.callee {
            // Detect setState calls: either "setState" or "setXxx" pattern
            if ident.name == "setState" || (ident.name.starts_with("set") && ident.name.len() > 3) {
                let is_nested = self.is_inside_nested_function();
                if !is_nested {
                    self.set_state_call = true;
                }
            }
        }

        // For method calls like props.foo.bar.toString(), we only want to depend on
        // the object being called on (props.foo.bar), not the full chain including
        // the method name (props.foo.bar.toString)
        match &it.callee {
            Expression::StaticMemberExpression(member_expr) => {
                // Check if this is a ref access in a cleanup function
                if self.in_top_level_cleanup && !self.in_nested_function_in_cleanup {
                    // Check if the object chain contains "current"
                    let chain_dep = concat_members(member_expr, self.semantic, false);
                    if let Ok(Some(source)) = chain_dep {
                        if source.chain.contains(&Atom::from("current")) {
                            self.refs_inside_cleanups.push((member_expr.span, source.reference_id));
                        }
                    } else {
                        // Fallback: check if the object itself is a StaticMemberExpression with "current"
                        if let Expression::StaticMemberExpression(obj_member) = &member_expr.object
                        {
                            if obj_member.property.name == "current" {
                                if let Expression::Identifier(ident) =
                                    obj_member.object.get_inner_expression()
                                {
                                    self.refs_inside_cleanups
                                        .push((member_expr.span, ident.reference_id()));
                                }
                            }
                        }
                    }
                }

                // Visit only the object part, not the full member expression
                // We need to ensure the object is visited outside the static member context
                // so that identifiers can be properly detected as dependencies
                let chain_dep = concat_members(member_expr, self.semantic, false);
                if let Ok(Some(source)) = chain_dep {
                    if self.skip_reporting_dependency {
                    } else {
                        // For method calls, we want to depend on the object, not the method
                        // So we create a dependency with just the object chain
                        let obj_dep = Dependency {
                            span: member_expr.object.span(),
                            name: source.name,
                            reference_id: source.reference_id,
                            chain: source.chain[..source.chain.len().saturating_sub(1)].to_vec(),
                            symbol_id: source.symbol_id,
                        };
                        self.found_dependencies.insert(obj_dep);
                    }
                } else {
                    // Fallback to visiting the object directly
                    self.visit_expression(&member_expr.object);
                }
            }
            _ => {
                // For other types of callees, visit normally
                self.visit_expression(&it.callee);
            }
        }

        // Always visit arguments
        for arg in &it.arguments {
            match arg {
                Argument::SpreadElement(spread) => {
                    self.visit_expression(&spread.argument);
                }
                _ => {
                    if let Some(expr) = arg.as_expression() {
                        match expr.get_inner_expression() {
                            Expression::ArrowFunctionExpression(arrow) => {
                                let was_nested = self.in_nested_function_in_cleanup;
                                let was_nested_any = self.in_nested_function;
                                // Only suppress warnings if we're NOT in a cleanup function context
                                // In cleanup functions, we want to flag all ref accesses regardless of setTimeout
                                if !self.in_top_level_cleanup
                                    && !Self::is_react_hook_call(&it.callee)
                                {
                                    self.in_nested_function_in_cleanup = true;
                                }
                                // Always track nested functions for setState detection
                                self.in_nested_function = true;
                                self.visit_arrow_function_expression(arrow);
                                self.in_nested_function_in_cleanup = was_nested;
                                self.in_nested_function = was_nested_any;
                            }
                            Expression::FunctionExpression(func) => {
                                let was_nested = self.in_nested_function_in_cleanup;
                                let was_nested_any = self.in_nested_function;
                                // Only suppress warnings if we're NOT in a cleanup function context
                                // In cleanup functions, we want to flag all ref accesses regardless of setTimeout
                                if !self.in_top_level_cleanup
                                    && !Self::is_react_hook_call(&it.callee)
                                {
                                    self.in_nested_function_in_cleanup = true;
                                }
                                // Always track nested functions for setState detection
                                self.in_nested_function = true;
                                self.visit_function_expression(func);
                                self.in_nested_function_in_cleanup = was_nested;
                                self.in_nested_function = was_nested_any;
                            }
                            _ => {
                                self.visit_expression(expr);
                            }
                        }
                    }
                }
            }
        }
    }

    fn visit_chain_expression(&mut self, it: &ChainExpression<'a>) {
        use oxc_ast_visit::walk::walk_chain_expression;
        walk_chain_expression(self, it);
    }

    fn visit_chain_element(&mut self, it: &ChainElement<'a>) {
        match it {
            ChainElement::StaticMemberExpression(member_expr) => {
                self.visit_static_member_expression(member_expr);
            }
            ChainElement::ComputedMemberExpression(member_expr) => {
                self.visit_expression(&member_expr.object);
                self.visit_expression(&member_expr.expression);
            }
            ChainElement::CallExpression(call_expr) => {
                if let Expression::StaticMemberExpression(member_expr) = &call_expr.callee {
                    // For method calls like props.foo?.toString(), collect the dependency for the object (props.foo)
                    // not the full chain including the method (props.foo.toString)
                    self.visit_expression(&member_expr.object);
                } else {
                    self.visit_expression(&call_expr.callee);
                }
                for arg in &call_expr.arguments {
                    if let Some(expr) = arg.as_expression() {
                        self.visit_expression(expr);
                    }
                }
            }
            ChainElement::PrivateFieldExpression(private_expr) => {
                self.visit_expression(&private_expr.object);
            }
            ChainElement::TSNonNullExpression(non_null_expr) => {
                self.visit_expression(&non_null_expr.expression);
            }
        }
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if it.property.name == "current"
            && self.in_top_level_cleanup
            && !self.in_nested_function_in_cleanup
        {
            if let Expression::Identifier(ident) = it.object.get_inner_expression() {
                self.refs_inside_cleanups.push((it.span, ident.reference_id()));
            }
        }
        let full_chain_dep = concat_members(it, self.semantic, false);
        if let Ok(Some(source)) = full_chain_dep {
            if self.skip_reporting_dependency {
            } else {
                self.found_dependencies.insert(source);
            }
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        // Check if this is a setState function being passed as a callback
        if (ident.name == "setState" || (ident.name.starts_with("set") && ident.name.len() > 3))
            && !self.is_inside_nested_function()
        {
            self.set_state_call = true;
        }

        if self.skip_reporting_dependency {
            return;
        }
        if matches!(self.stack.last(), Some(AstType::StaticMemberExpression)) {
            return;
        }
        let reference_id = ident.reference_id();
        let symbol_id = self.semantic.scoping().get_reference(reference_id).symbol_id();
        let mut destructured_props: Vec<Vec<Atom<'a>>> = vec![];
        let mut did_see_ref = false;
        self.iter_destructure_bindings(|chain| {
            if chain.len() == 1 && chain[0] == Atom::from("current") {
                did_see_ref = true;
            } else {
                destructured_props.push(chain);
            }
        });

        // Check if this is a ref access in a cleanup function
        if self.in_top_level_cleanup && !self.in_nested_function_in_cleanup && did_see_ref {
            // This is a ref access in a cleanup function
            self.refs_inside_cleanups.push((ident.span, reference_id));
        }

        if destructured_props.is_empty() && !did_see_ref {
            // Function parameters are now handledv 1 in is_identifier_a_dependency
            self.found_dependencies.insert(Dependency {
                name: ident.name,
                reference_id,
                span: ident.span,
                chain: vec![],
                symbol_id,
            });
        } else if !destructured_props.is_empty() {
            for prop_chain in destructured_props {
                self.found_dependencies.insert(Dependency {
                    name: ident.name,
                    reference_id,
                    span: ident.span,
                    chain: prop_chain,
                    symbol_id,
                });
            }
        }
    }
}

impl<'a> ExhaustiveDepsVisitor<'a, '_> {
    fn perform_variable_declarator_visit(
        &mut self,
        decl: &VariableDeclarator<'a>,
        init: &Expression<'a>,
    ) {
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

    fn visit_function_expression(&mut self, it: &oxc_ast::ast::Function<'a>) {
        let was_nested = self.in_nested_function_in_cleanup;
        let was_nested_any = self.in_nested_function;
        // If we're in a cleanup context, this is a nested function
        if self.in_top_level_cleanup {
            self.in_nested_function_in_cleanup = true;
        }
        // Always track nested functions for setState detection
        self.in_nested_function = true;
        <Self as oxc_ast_visit::Visit>::visit_function(self, it, ScopeFlags::empty());
        self.in_nested_function_in_cleanup = was_nested;
        self.in_nested_function = was_nested_any;
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let was_nested = self.in_nested_function_in_cleanup;
        let was_nested_any = self.in_nested_function;
        // Only set in_nested_function_in_cleanup if we're not already in a top-level cleanup
        // (i.e., this is a truly nested function, not the cleanup function itself)
        if self.in_top_level_cleanup && !self.in_nested_function_in_cleanup {
            // This is the cleanup function itself, not a nested function
            // Don't set in_nested_function_in_cleanup = true
        } else if self.in_top_level_cleanup {
            // We're already in a cleanup and this is a nested function
            self.in_nested_function_in_cleanup = true;
        }
        // Always track nested functions for setState detection
        self.in_nested_function = true;
        <Self as oxc_ast_visit::Visit>::visit_arrow_function_expression(self, it);
        self.in_nested_function_in_cleanup = was_nested;
        self.in_nested_function = was_nested_any;
    }
}

mod fix {
    use super::Name;
    use oxc_allocator::{Allocator, CloneIn};
    use oxc_ast::{AstBuilder, ast::ArrayExpression};
    use oxc_span::{Atom, SPAN};

    use crate::fixer::{RuleFix, RuleFixer};

    pub fn append_dependencies<'c, 'a: 'c>(
        fixer: RuleFixer<'c, 'a>,
        names: &[Name<'a>],
        deps: &ArrayExpression<'a>,
    ) -> RuleFix<'a> {
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
        // TODO: this test case seems suspect. props.foo is not being used in the useEffect body, so it IS an unnecessary dependency.
        // it also does not appear to exist in the eslint plugin react hooks test cases.
        // https://github.com/facebook/react/blob/main/packages/eslint-plugin-react-hooks/__tests__/ESLintRuleExhaustiveDeps-test.js
        // r"function MyComponent(props) {
        //   const bar = props.bar;
        //   useEffect(() => {
        //     const { bar } = foo();
        //     console.log(bar);
        //   }, [props.foo]);
        // }",
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
                  // note: this was previously listed in the fail test case array with a fix me comment, but it should pass.
                  r"function MyComponent(props) {
          useCallback(() => {
            const { foo: { bar } } = props;
            console.log(bar);
          }, [props.foo.bar]);
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
        // TODO: this test also seems suspect. it's not included in the eslint-plugin-react-hooks test suite for this rule.
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
    ];

    let pass_additional_hooks = vec![(
        "function MyComponent(props) {
          useSpecialEffect(() => {
            console.log(props.foo);
          });
        }",
        Some(serde_json::json!([{ "additionalHooks": "useSpecialEffect" }])),
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
    ];

    Tester::new(
        ExhaustiveDeps::NAME,
        ExhaustiveDeps::PLUGIN,
        pass.iter().map(|&code| (code, None)).chain(pass_additional_hooks).collect::<Vec<_>>(),
        fail.iter().map(|&code| (code, None)).chain(fail_additional_hooks).collect::<Vec<_>>(),
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
