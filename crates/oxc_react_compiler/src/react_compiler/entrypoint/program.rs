// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Main entrypoint for the React Compiler.
//!
//! This module is a port of Program.ts from the TypeScript compiler. It orchestrates
//! the compilation of a program by:
//! 1. Finding program-level suppressions
//! 2. Discovering functions to compile (components, hooks)
//! 3. Processing each function through the compilation pipeline
//! 4. Applying compiled functions back to the AST

use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_ast::ast::*;
use oxc_ast::builder::AstBuilder;
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_span::{GetSpan, SPAN, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::diagnostics::{ErrorCategory, has_critical_errors, with_fallback_label};
use crate::react_compiler_hir::ReactFunctionType;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
use crate::react_compiler_lowering::FunctionNode;
use crate::scope::ScopeResolver;
use oxc_allocator::{Allocator, ArenaBox, ArenaVec, CloneIn, GetAddress, GetAllocator};
use oxc_semantic::{AstNodes, NodeId, Scoping, Semantic};
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::keyword::is_reserved_keyword;
use oxc_syntax::scope::ScopeId;
use oxc_syntax::symbol::SymbolId;

use super::compile_result::CodegenFunction;
use super::compile_result::CompileResult;
use super::imports::ProgramContext;
use super::pipeline;
use super::suppression::filter_suppressions_that_affect_function;
use super::suppression::find_program_suppressions;
use super::suppression::suppressions_to_diagnostics;
use crate::options::{
    CompilationMode, CompilerOutputMode, GatingConfig, PanicThreshold, PluginOptions,
};
use oxc_str::{Ident, Str};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const DEFAULT_ESLINT_SUPPRESSIONS: &[&str] =
    &["react-hooks/exhaustive-deps", "react-hooks/rules-of-hooks"];

/// Directives that opt a function into memoization
const OPT_IN_DIRECTIVES: &[&str] = &["use forget", "use memo"];

/// Directives that opt a function out of memoization
const OPT_OUT_DIRECTIVES: &[&str] = &["use no forget", "use no memo"];

// -----------------------------------------------------------------------
// Internal types
// -----------------------------------------------------------------------

/// A function found in the program that should be compiled.
///
/// `'a` is the arena lifetime of the discovered oxc function node.
struct CompileSource<'b, 'a> {
    kind: CompileSourceKind,
    original_kind: OriginalFnKind,
    /// Byte span of the discovered function, used as the fallback labeled span in
    /// compile-error diagnostics.
    fn_ast_span: Option<Span>,
    fn_start: Option<u32>,
    fn_end: Option<u32>,
    fn_scope_id: ScopeId,
    fn_type: ReactFunctionType,
    /// The discovered oxc function node, handed straight to lowering.
    fn_node: FunctionNode<'b, 'a>,
    /// Directive values from the function body (for opt-in/opt-out checks)
    body_directives: Vec<Str<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompileSourceKind {
    Original,
}

// -----------------------------------------------------------------------
// Directive helpers
// -----------------------------------------------------------------------

/// Check if any opt-in directive is present in the given directives.
/// Returns the first matching directive value, or None.
///
/// Also checks for dynamic gating directives (`use memo if(...)`)
fn try_find_directive_enabling_memoization<'a>(
    directives: &[Str<'a>],
    opts: &PluginOptions,
) -> Result<Option<&'a str>, Diagnostics> {
    // Check standard opt-in directives
    let opt_in = directives.iter().find(|d| OPT_IN_DIRECTIVES.contains(&d.as_str()));
    if let Some(directive) = opt_in {
        return Ok(Some(directive.as_str()));
    }

    // Check dynamic gating directives
    match find_directives_dynamic_gating(directives, opts) {
        Ok(Some(result)) => Ok(Some(result.directive)),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Check if any opt-out directive is present in the given directives.
fn find_directive_disabling_memoization<'a>(
    mut directives: impl Iterator<Item = &'a str>,
    custom_opt_out_directives: Option<&[String]>,
) -> Option<&'a str> {
    if let Some(custom_directives) = custom_opt_out_directives {
        directives.find(|d| custom_directives.iter().any(|c| c == d))
    } else {
        directives.find(|d| OPT_OUT_DIRECTIVES.contains(d))
    }
}

/// Result of a dynamic gating directive parse.
struct DynamicGatingResult<'a> {
    directive: &'a str,
    gating: GatingConfig,
}

/// Check for dynamic gating directives like `use memo if(identifier)`.
/// Returns the directive and gating config if found, or an error if malformed.
fn find_directives_dynamic_gating<'a>(
    directives: &[Str<'a>],
    opts: &PluginOptions,
) -> Result<Option<DynamicGatingResult<'a>>, Diagnostics> {
    let dynamic_gating = match &opts.dynamic_gating {
        Some(dg) => dg,
        None => return Ok(None),
    };

    let mut errors = Diagnostics::new();
    let mut matches: Vec<(&'a str, String)> = Vec::new();

    for directive in directives {
        if let Some(ident) = parse_dynamic_gating_directive(directive) {
            if is_valid_identifier(ident) {
                matches.push((directive.as_str(), ident.to_string()));
            } else {
                errors.push(
                    ErrorCategory::Gating
                        .diagnostic("Dynamic gating directive is not a valid JavaScript identifier")
                        .with_help(format!("Found '{directive}'")),
                );
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    if matches.len() > 1 {
        let names: Vec<&str> = matches.iter().map(|(d, _)| *d).collect();
        return Err(Diagnostics::from(
            ErrorCategory::Gating
                .diagnostic("Multiple dynamic gating directives found")
                .with_help(format!("Expected a single directive but found [{}]", names.join(", "))),
        ));
    }

    if matches.len() == 1 {
        Ok(Some(DynamicGatingResult {
            directive: matches[0].0,
            gating: GatingConfig {
                source: dynamic_gating.source.clone(),
                import_specifier_name: matches[0].1.clone(),
            },
        }))
    } else {
        Ok(None)
    }
}

/// Parse a `use memo if(<condition>)` directive, returning the condition.
/// Exact equivalent of the TS DYNAMIC_GATING_DIRECTIVE regex
/// `^use memo if\(([^\)]*)\)$`: the condition may not contain `)` and the
/// directive must end at the closing paren.
fn parse_dynamic_gating_directive(value: &str) -> Option<&str> {
    let condition = value.strip_prefix("use memo if(")?.strip_suffix(')')?;
    if condition.contains(')') {
        return None;
    }
    Some(condition)
}

/// Check if a string is a valid JavaScript identifier that is not a reserved
/// word (matching Babel's `isValidIdentifier`).
fn is_valid_identifier(s: &str) -> bool {
    is_identifier_name(s) && !is_reserved_keyword(s)
}

// -----------------------------------------------------------------------
// Name helpers
// -----------------------------------------------------------------------

/// Check if a string follows the React hook naming convention (use[A-Z0-9]...).
fn is_hook_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 4
        && bytes[0] == b'u'
        && bytes[1] == b's'
        && bytes[2] == b'e'
        && bytes.get(3).is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Check if a name looks like a React component (starts with uppercase letter).
fn is_component_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(|c| c.is_ascii_uppercase())
}

/// Check if an expression is a hook call (identifier with hook name, or
/// member expression `PascalCase.useHook`).
fn expr_is_hook(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(id) => is_hook_name(&id.name),
        Expression::StaticMemberExpression(member) => {
            // Property must be a hook name
            if !is_hook_name(&member.property.name) {
                return false;
            }
            // Object must be a PascalCase identifier
            if let Expression::Identifier(obj) = &member.object {
                obj.name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Whether an expression's sub-tree contains any optional chaining link. Mirrors
/// `convert_ast::ConvertCtx::expr_contains_optional`, used to decide whether a
/// `CallExpression` inside a `ChainExpression` was a regular call (before the
/// first `?.`, hook-checkable) or an optional call (not a hook).
fn expr_contains_optional(expr: &Expression) -> bool {
    match expr {
        Expression::CallExpression(c) => c.optional || expr_contains_optional(&c.callee),
        Expression::StaticMemberExpression(m) => m.optional || expr_contains_optional(&m.object),
        Expression::ComputedMemberExpression(m) => m.optional || expr_contains_optional(&m.object),
        Expression::PrivateFieldExpression(p) => p.optional || expr_contains_optional(&p.object),
        _ => false,
    }
}

/// Whether a `CallExpression` is a "regular" (non-optional) call. In Babel such a
/// call was a `CallExpression` (hook-checkable); optional / post-`?.` calls were
/// `OptionalCallExpression` (never treated as hooks). Matches the Babel-bridge
/// chain flattening exactly.
fn is_regular_call(call: &CallExpression) -> bool {
    !call.optional && !expr_contains_optional(&call.callee)
}

/// Get the inferred function name from a function's context.
///
/// For FunctionDeclaration: uses the `id` field.
/// For FunctionExpression/ArrowFunctionExpression: infers from parent context
/// (VariableDeclarator, etc.) which is passed explicitly since we don't have Babel paths.
fn get_function_name_from_id<'ast>(id: Option<&BindingIdentifier<'ast>>) -> Option<&'ast str> {
    id.map(|id| id.name.as_str())
}

// -----------------------------------------------------------------------
// AST traversal helpers
// -----------------------------------------------------------------------

/// Check if an expression is a "non-node" return value (indicating the function
/// is not a React component). This matches the TS `isNonNode` function.
fn is_non_node(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::BigIntLiteral(_)
            | Expression::ClassExpression(_)
            | Expression::NewExpression(_)
    )
}

/// Recursively check if a function body returns a non-React-node value.
/// Walks all return statements in the function (not in nested functions).
/// The last return statement visited (in DFS order) determines the result,
/// rather than short-circuiting on the first non-node return.
fn returns_non_node_in_stmts(stmts: &[Statement]) -> bool {
    let mut result = false;
    for stmt in stmts {
        returns_non_node_in_stmt(stmt, &mut result);
    }
    result
}

fn returns_non_node_in_stmt(stmt: &Statement, result: &mut bool) {
    match stmt {
        Statement::ReturnStatement(ret) => {
            *result = match &ret.argument {
                Some(arg) => is_non_node(arg),
                None => true, // bare `return;` with no argument is a non-node value
            };
        }
        Statement::BlockStatement(block) => {
            for s in &block.body {
                returns_non_node_in_stmt(s, result);
            }
        }
        Statement::IfStatement(if_stmt) => {
            returns_non_node_in_stmt(&if_stmt.consequent, result);
            if let Some(ref alt) = if_stmt.alternate {
                returns_non_node_in_stmt(alt, result);
            }
        }
        Statement::ForStatement(for_stmt) => returns_non_node_in_stmt(&for_stmt.body, result),
        Statement::WhileStatement(while_stmt) => returns_non_node_in_stmt(&while_stmt.body, result),
        Statement::DoWhileStatement(do_while) => returns_non_node_in_stmt(&do_while.body, result),
        Statement::ForInStatement(for_in) => returns_non_node_in_stmt(&for_in.body, result),
        Statement::ForOfStatement(for_of) => returns_non_node_in_stmt(&for_of.body, result),
        Statement::SwitchStatement(switch) => {
            for case in &switch.cases {
                for s in &case.consequent {
                    returns_non_node_in_stmt(s, result);
                }
            }
        }
        Statement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                returns_non_node_in_stmt(s, result);
            }
            if let Some(ref handler) = try_stmt.handler {
                for s in &handler.body.body {
                    returns_non_node_in_stmt(s, result);
                }
            }
            if let Some(ref finalizer) = try_stmt.finalizer {
                for s in &finalizer.body {
                    returns_non_node_in_stmt(s, result);
                }
            }
        }
        Statement::LabeledStatement(labeled) => returns_non_node_in_stmt(&labeled.body, result),
        Statement::WithStatement(with) => returns_non_node_in_stmt(&with.body, result),
        // Skip nested function/class declarations -- they have their own returns.
        // All other statements (incl. TS-only declarations) are opaque here.
        _ => {}
    }
}

/// Check if a function returns non-node values.
/// For arrow functions with expression body, checks the expression directly.
/// For block bodies, walks the statements.
fn returns_non_node_fn(body: &FnBody) -> bool {
    match body {
        FnBody::Block(block) => returns_non_node_in_stmts(&block.statements),
        FnBody::Expression(expr) => is_non_node(expr),
    }
}

/// Check if a function body calls hooks or creates JSX.
/// Traverses the function body (not nested functions) looking for:
/// - CallExpression where callee is a hook
/// - JSXElement or JSXFragment
fn calls_hooks_or_creates_jsx_in_stmts(stmts: &[Statement]) -> bool {
    for stmt in stmts {
        if calls_hooks_or_creates_jsx_in_stmt(stmt) {
            return true;
        }
    }
    false
}

fn calls_hooks_or_creates_jsx_in_stmt(stmt: &Statement) -> bool {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&expr_stmt.expression)
        }
        Statement::ReturnStatement(ret) => {
            if let Some(ref arg) = ret.argument {
                calls_hooks_or_creates_jsx_in_expr(arg)
            } else {
                false
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(ref init) = decl.init
                    && calls_hooks_or_creates_jsx_in_expr(init)
                {
                    return true;
                }
            }
            false
        }
        Statement::BlockStatement(block) => calls_hooks_or_creates_jsx_in_stmts(&block.body),
        Statement::IfStatement(if_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&if_stmt.test)
                || calls_hooks_or_creates_jsx_in_stmt(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| calls_hooks_or_creates_jsx_in_stmt(alt))
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                match init {
                    ForStatementInit::VariableDeclaration(var_decl) => {
                        for decl in &var_decl.declarations {
                            if let Some(ref init) = decl.init
                                && calls_hooks_or_creates_jsx_in_expr(init)
                            {
                                return true;
                            }
                        }
                    }
                    // An expression `ForStatementInit` is an `Expression` (the
                    // enum inherits the Expression variants).
                    expr => {
                        if let Some(expr) = expr.as_expression()
                            && calls_hooks_or_creates_jsx_in_expr(expr)
                        {
                            return true;
                        }
                    }
                }
            }
            if let Some(ref test) = for_stmt.test
                && calls_hooks_or_creates_jsx_in_expr(test)
            {
                return true;
            }
            if let Some(ref update) = for_stmt.update
                && calls_hooks_or_creates_jsx_in_expr(update)
            {
                return true;
            }
            calls_hooks_or_creates_jsx_in_stmt(&for_stmt.body)
        }
        Statement::WhileStatement(while_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&while_stmt.test)
                || calls_hooks_or_creates_jsx_in_stmt(&while_stmt.body)
        }
        Statement::DoWhileStatement(do_while) => {
            calls_hooks_or_creates_jsx_in_stmt(&do_while.body)
                || calls_hooks_or_creates_jsx_in_expr(&do_while.test)
        }
        Statement::ForInStatement(for_in) => {
            calls_hooks_or_creates_jsx_in_expr(&for_in.right)
                || calls_hooks_or_creates_jsx_in_stmt(&for_in.body)
        }
        Statement::ForOfStatement(for_of) => {
            calls_hooks_or_creates_jsx_in_expr(&for_of.right)
                || calls_hooks_or_creates_jsx_in_stmt(&for_of.body)
        }
        Statement::SwitchStatement(switch) => {
            if calls_hooks_or_creates_jsx_in_expr(&switch.discriminant) {
                return true;
            }
            for case in &switch.cases {
                if let Some(ref test) = case.test
                    && calls_hooks_or_creates_jsx_in_expr(test)
                {
                    return true;
                }
                if calls_hooks_or_creates_jsx_in_stmts(&case.consequent) {
                    return true;
                }
            }
            false
        }
        Statement::ThrowStatement(throw) => calls_hooks_or_creates_jsx_in_expr(&throw.argument),
        Statement::TryStatement(try_stmt) => {
            if calls_hooks_or_creates_jsx_in_stmts(&try_stmt.block.body) {
                return true;
            }
            if let Some(ref handler) = try_stmt.handler
                && calls_hooks_or_creates_jsx_in_stmts(&handler.body.body)
            {
                return true;
            }
            if let Some(ref finalizer) = try_stmt.finalizer
                && calls_hooks_or_creates_jsx_in_stmts(&finalizer.body)
            {
                return true;
            }
            false
        }
        Statement::LabeledStatement(labeled) => calls_hooks_or_creates_jsx_in_stmt(&labeled.body),
        Statement::WithStatement(with) => {
            calls_hooks_or_creates_jsx_in_expr(&with.object)
                || calls_hooks_or_creates_jsx_in_stmt(&with.body)
        }
        // Nested function declarations have their own returns; class bodies in the
        // Babel bridge carried no extracted hook/JSX metadata (always `false`), so
        // class declarations never contributed here. All other statements (incl.
        // TS-only declarations) are opaque.
        _ => false,
    }
}

fn calls_hooks_or_creates_jsx_in_expr(expr: &Expression) -> bool {
    /// Whether an argument expression is a nested function (skipped by the walk).
    fn is_nested_fn(arg: &Expression) -> bool {
        matches!(arg, Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_))
    }

    match expr {
        // JSX creates
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,

        // Hook calls. Only a "regular" (non-optional) call's callee is hook-checked;
        // optional calls (Babel `OptionalCallExpression`) never count as hooks, but
        // their callee/args are still searched.
        Expression::CallExpression(call) => {
            if is_regular_call(call) && expr_is_hook(&call.callee) {
                return true;
            }
            // Also check arguments for JSX/hooks (but not nested functions)
            if calls_hooks_or_creates_jsx_in_expr(&call.callee) {
                return true;
            }
            for arg in &call.arguments {
                // Skip function arguments -- they are nested functions
                if let Some(arg) = arg.as_expression() {
                    if is_nested_fn(arg) {
                        continue;
                    }
                    if calls_hooks_or_creates_jsx_in_expr(arg) {
                        return true;
                    }
                } else if let Argument::SpreadElement(s) = arg
                    && calls_hooks_or_creates_jsx_in_expr(&s.argument)
                {
                    return true;
                }
            }
            false
        }
        // Optional chaining (`a?.b`, `a?.()`): Babel modeled these as
        // Optional{Member,Call}Expression. Recurse into the inner element, where
        // per-call hook checks honor the optional flag via `is_regular_call`.
        Expression::ChainExpression(chain) => {
            calls_hooks_or_creates_jsx_in_chain_element(&chain.expression)
        }

        // Binary/logical
        Expression::BinaryExpression(bin) => {
            calls_hooks_or_creates_jsx_in_expr(&bin.left)
                || calls_hooks_or_creates_jsx_in_expr(&bin.right)
        }
        Expression::LogicalExpression(log) => {
            calls_hooks_or_creates_jsx_in_expr(&log.left)
                || calls_hooks_or_creates_jsx_in_expr(&log.right)
        }
        Expression::ConditionalExpression(cond) => {
            calls_hooks_or_creates_jsx_in_expr(&cond.test)
                || calls_hooks_or_creates_jsx_in_expr(&cond.consequent)
                || calls_hooks_or_creates_jsx_in_expr(&cond.alternate)
        }
        Expression::AssignmentExpression(assign) => {
            calls_hooks_or_creates_jsx_in_expr(&assign.right)
        }
        Expression::SequenceExpression(seq) => {
            seq.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        Expression::UnaryExpression(unary) => calls_hooks_or_creates_jsx_in_expr(&unary.argument),
        Expression::UpdateExpression(update) => match &update.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
            target => {
                target.as_member_expression().is_some_and(calls_hooks_or_creates_jsx_in_member)
            }
        },
        Expression::StaticMemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
                || calls_hooks_or_creates_jsx_in_expr(&member.expression)
        }
        Expression::PrivateFieldExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
        }
        Expression::AwaitExpression(await_expr) => {
            calls_hooks_or_creates_jsx_in_expr(&await_expr.argument)
        }
        Expression::YieldExpression(yield_expr) => {
            yield_expr.argument.as_ref().is_some_and(|arg| calls_hooks_or_creates_jsx_in_expr(arg))
        }
        Expression::TaggedTemplateExpression(tagged) => {
            calls_hooks_or_creates_jsx_in_expr(&tagged.tag)
                || tagged.quasi.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        Expression::TemplateLiteral(tl) => {
            tl.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        Expression::ArrayExpression(arr) => arr.elements.iter().any(|e| match e {
            ArrayExpressionElement::SpreadElement(s) => {
                calls_hooks_or_creates_jsx_in_expr(&s.argument)
            }
            ArrayExpressionElement::Elision(_) => false,
            other => other.as_expression().is_some_and(calls_hooks_or_creates_jsx_in_expr),
        }),
        Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| match prop {
            ObjectPropertyKind::SpreadProperty(s) => {
                calls_hooks_or_creates_jsx_in_expr(&s.argument)
            }
            // Object methods (`{ foo() {} }`, getters/setters): Babel modeled these
            // as `ObjectMethod` and traversed their body statements. Regular
            // properties traverse their value (nested functions are skipped).
            ObjectPropertyKind::ObjectProperty(p) => {
                if p.method || matches!(p.kind, PropertyKind::Get | PropertyKind::Set) {
                    if let Expression::FunctionExpression(func) = &p.value
                        && let Some(body) = &func.body
                    {
                        return calls_hooks_or_creates_jsx_in_stmts(&body.statements);
                    }
                    false
                } else {
                    calls_hooks_or_creates_jsx_in_expr(&p.value)
                }
            }
        }),
        Expression::ParenthesizedExpression(paren) => {
            calls_hooks_or_creates_jsx_in_expr(&paren.expression)
        }
        Expression::TSAsExpression(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        Expression::TSNonNullExpression(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        Expression::TSTypeAssertion(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        Expression::TSInstantiationExpression(ts) => {
            calls_hooks_or_creates_jsx_in_expr(&ts.expression)
        }
        Expression::NewExpression(new) => {
            if calls_hooks_or_creates_jsx_in_expr(&new.callee) {
                return true;
            }
            new.arguments.iter().any(|a| {
                if let Some(a) = a.as_expression() {
                    if is_nested_fn(a) {
                        return false;
                    }
                    calls_hooks_or_creates_jsx_in_expr(a)
                } else if let Argument::SpreadElement(s) = a {
                    calls_hooks_or_creates_jsx_in_expr(&s.argument)
                } else {
                    false
                }
            })
        }

        // Nested functions are skipped; class expressions carried no extracted
        // hook/JSX metadata in the Babel bridge (always `false`). Leaf expressions
        // fall through to `false`.
        _ => false,
    }
}

/// Search a `ChainElement` (`a?.b`, `a?.()`, ...) for hook calls / JSX, mirroring
/// the Babel `Optional{Member,Call}Expression` traversal. Optional calls never
/// count as hooks, but their callee/args are searched.
fn calls_hooks_or_creates_jsx_in_chain_element(element: &ChainElement) -> bool {
    match element {
        ChainElement::CallExpression(call) => {
            if is_regular_call(call) && expr_is_hook(&call.callee) {
                return true;
            }
            if calls_hooks_or_creates_jsx_in_expr(&call.callee) {
                return true;
            }
            call.arguments.iter().any(|arg| {
                if let Some(arg) = arg.as_expression() {
                    !matches!(
                        arg,
                        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                    ) && calls_hooks_or_creates_jsx_in_expr(arg)
                } else if let Argument::SpreadElement(s) = arg {
                    calls_hooks_or_creates_jsx_in_expr(&s.argument)
                } else {
                    false
                }
            })
        }
        ChainElement::StaticMemberExpression(m) => calls_hooks_or_creates_jsx_in_expr(&m.object),
        ChainElement::ComputedMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
                || calls_hooks_or_creates_jsx_in_expr(&m.expression)
        }
        ChainElement::PrivateFieldExpression(m) => calls_hooks_or_creates_jsx_in_expr(&m.object),
        ChainElement::TSNonNullExpression(t) => calls_hooks_or_creates_jsx_in_expr(&t.expression),
    }
}

/// Search a member expression (object side) for hook calls / JSX.
fn calls_hooks_or_creates_jsx_in_member(member: &MemberExpression) -> bool {
    match member {
        MemberExpression::StaticMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
        MemberExpression::ComputedMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
                || calls_hooks_or_creates_jsx_in_expr(&m.expression)
        }
        MemberExpression::PrivateFieldExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
    }
}

/// Check if a function body calls hooks or creates JSX.
fn calls_hooks_or_creates_jsx(params: &FormalParameters, body: &FnBody) -> bool {
    // Check default param values (TS traverses the whole function node including params)
    if calls_hooks_or_creates_jsx_in_params(params) {
        return true;
    }
    match body {
        FnBody::Block(block) => calls_hooks_or_creates_jsx_in_stmts(&block.statements),
        FnBody::Expression(expr) => calls_hooks_or_creates_jsx_in_expr(expr),
    }
}

/// Check if any parameter default values contain hooks or JSX.
///
/// Babel traversed the whole function node including params; default values live
/// on `FormalParameter.initializer`, and the binding pattern may itself contain
/// nested defaults.
fn calls_hooks_or_creates_jsx_in_params(params: &FormalParameters) -> bool {
    for param in &params.items {
        if let Some(init) = &param.initializer
            && calls_hooks_or_creates_jsx_in_expr(init)
        {
            return true;
        }
        if calls_hooks_or_creates_jsx_in_binding(&param.pattern) {
            return true;
        }
    }
    if let Some(rest) = &params.rest
        && calls_hooks_or_creates_jsx_in_binding(&rest.rest.argument)
    {
        return true;
    }
    false
}

fn calls_hooks_or_creates_jsx_in_binding(pattern: &BindingPattern) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => false,
        BindingPattern::ObjectPattern(obj) => {
            obj.properties.iter().any(|p| calls_hooks_or_creates_jsx_in_binding(&p.value))
                || obj
                    .rest
                    .as_ref()
                    .is_some_and(|r| calls_hooks_or_creates_jsx_in_binding(&r.argument))
        }
        BindingPattern::ArrayPattern(arr) => {
            arr.elements
                .iter()
                .any(|e| e.as_ref().is_some_and(calls_hooks_or_creates_jsx_in_binding))
                || arr
                    .rest
                    .as_ref()
                    .is_some_and(|r| calls_hooks_or_creates_jsx_in_binding(&r.argument))
        }
        BindingPattern::AssignmentPattern(assign) => {
            calls_hooks_or_creates_jsx_in_expr(&assign.right)
                || calls_hooks_or_creates_jsx_in_binding(&assign.left)
        }
    }
}

/// Check if a parameter's type annotation is valid for a React component prop.
/// Returns false for primitive type annotations that indicate this is NOT a component.
fn is_valid_props_annotation(type_annotation: Option<&TSTypeAnnotation>) -> bool {
    let Some(annotation) = type_annotation else {
        return true; // No annotation = valid
    };
    // Mirrors the Babel-bridge `babel_ts_type_name` of the param's annotation; the
    // disallowed TS keyword/type set. (Flow types never appear in the oxc AST.)
    !matches!(
        &annotation.type_annotation,
        TSType::TSArrayType(_)
            | TSType::TSBigIntKeyword(_)
            | TSType::TSBooleanKeyword(_)
            | TSType::TSConstructorType(_)
            | TSType::TSFunctionType(_)
            | TSType::TSLiteralType(_)
            | TSType::TSNeverKeyword(_)
            | TSType::TSNumberKeyword(_)
            | TSType::TSStringKeyword(_)
            | TSType::TSSymbolKeyword(_)
            | TSType::TSTupleType(_)
    )
}

/// Check if the function parameters are valid for a React component.
/// Components can have 0 params, 1 param (props), or 2 params (props + ref).
///
/// The Babel reference treated the rest parameter as a trailing `RestElement` in
/// the flat params list; in oxc the rest lives in `params.rest`. So the logical
/// param count is `items.len() + rest.is_some()`.
fn is_valid_component_params(params: &FormalParameters) -> bool {
    let logical_len = params.items.len() + usize::from(params.rest.is_some());
    if logical_len == 0 {
        return true;
    }
    if logical_len > 2 {
        return false;
    }
    // First param cannot be a rest element. If there are no `items`, the only param
    // is the rest -> invalid.
    let Some(first) = params.items.first() else {
        return false;
    };
    // Check type annotation on first param.
    if !is_valid_props_annotation(first.type_annotation.as_deref()) {
        return false;
    }
    if logical_len == 1 {
        return true;
    }
    // Second param: either a second `item` or the rest. The rest is not an
    // identifier, so it never looks like a ref.
    match params.items.get(1) {
        Some(second) => match &second.pattern {
            BindingPattern::BindingIdentifier(id) => {
                id.name.contains("ref") || id.name.contains("Ref")
            }
            _ => false,
        },
        None => false,
    }
}

// -----------------------------------------------------------------------
// Unified function body type for traversal
// -----------------------------------------------------------------------

/// Abstraction over function body types to simplify traversal code
enum FnBody<'a> {
    Block(&'a FunctionBody<'a>),
    Expression(&'a Expression<'a>),
}

// -----------------------------------------------------------------------
// Function type detection
// -----------------------------------------------------------------------

/// Determine the React function type for a function, given the compilation mode
/// and the function's name and context.
///
/// This is the Rust equivalent of `getReactFunctionType` in Program.ts.
#[allow(clippy::too_many_arguments)]
fn get_react_function_type(
    name: Option<&str>,
    params: &FormalParameters,
    body: &FnBody,
    body_directives: &[Str<'_>],
    is_declaration: bool,
    parent_callee_name: Option<&str>,
    opts: &PluginOptions,
    is_component_declaration: bool,
    is_hook_declaration: bool,
) -> Option<ReactFunctionType> {
    // Check for opt-in directives in the function body
    if let FnBody::Block(_) = body {
        let opt_in = try_find_directive_enabling_memoization(body_directives, opts);
        if let Ok(Some(_)) = opt_in {
            // If there's an opt-in directive, use name heuristics but fall back to Other
            return Some(
                get_component_or_hook_like(name, params, body, parent_callee_name)
                    .unwrap_or(ReactFunctionType::Other),
            );
        }
    }

    // Component and hook declarations are known components/hooks
    // (Flow `component Foo() { ... }` and `hook useFoo() { ... }` syntax,
    //  detected via __componentDeclaration / __hookDeclaration from the Hermes parser)
    let component_syntax_type = if is_declaration {
        if is_component_declaration {
            Some(ReactFunctionType::Component)
        } else if is_hook_declaration {
            Some(ReactFunctionType::Hook)
        } else {
            None
        }
    } else {
        None
    };

    match opts.compilation_mode {
        CompilationMode::Annotation => {
            // opt-ins were checked above
            None
        }
        CompilationMode::Infer => {
            // Check if this is a component or hook-like function
            component_syntax_type
                .or_else(|| get_component_or_hook_like(name, params, body, parent_callee_name))
        }
        CompilationMode::Syntax => {
            // In syntax mode, only compile declared components/hooks
            component_syntax_type
        }
        CompilationMode::All => Some(
            get_component_or_hook_like(name, params, body, parent_callee_name)
                .unwrap_or(ReactFunctionType::Other),
        ),
    }
}

/// Determine if a function looks like a React component or hook based on
/// naming conventions and code patterns.
///
/// Adapted from the ESLint rule at
/// https://github.com/facebook/react/blob/main/packages/eslint-plugin-react-hooks/src/RulesOfHooks.js
fn get_component_or_hook_like(
    name: Option<&str>,
    params: &FormalParameters,
    body: &FnBody,
    parent_callee_name: Option<&str>,
) -> Option<ReactFunctionType> {
    if let Some(fn_name) = name {
        if is_component_name(fn_name) {
            // Check if it actually looks like a component
            let is_component = calls_hooks_or_creates_jsx(params, body)
                && is_valid_component_params(params)
                && !returns_non_node_fn(body);
            return if is_component { Some(ReactFunctionType::Component) } else { None };
        } else if is_hook_name(fn_name) {
            // Hooks have hook invocations or JSX, but can take any # of arguments
            return if calls_hooks_or_creates_jsx(params, body) {
                Some(ReactFunctionType::Hook)
            } else {
                None
            };
        }
    }

    // For unnamed functions, check if they are forwardRef/memo callbacks
    if let Some(callee_name) = parent_callee_name
        && (callee_name == "forwardRef" || callee_name == "memo")
    {
        return if calls_hooks_or_creates_jsx(params, body) {
            Some(ReactFunctionType::Component)
        } else {
            None
        };
    }

    None
}

/// Extract the callee name from a CallExpression if it's a React API call
/// (forwardRef, memo, React.forwardRef, React.memo).
fn get_callee_name_if_react_api<'ast>(callee: &Expression<'ast>) -> Option<&'ast str> {
    match callee {
        Expression::Identifier(id) => {
            if id.name == "forwardRef" || id.name == "memo" {
                Some(id.name.as_str())
            } else {
                None
            }
        }
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object
                && obj.name == "React"
                && (member.property.name == "forwardRef" || member.property.name == "memo")
            {
                return Some(member.property.name.as_str());
            }
            None
        }
        _ => None,
    }
}

// -----------------------------------------------------------------------
// Error handling
// -----------------------------------------------------------------------

/// Push a failed compilation attempt's diagnostics onto the accumulator.
fn log_error(err: &Diagnostics, fn_span: Option<Span>, diagnostics: &mut Diagnostics) {
    // Detect simulated unknown exception (throwUnknownException__testonly). In TS,
    // exceptions that are not compiler errors surface as a pipeline error carrying
    // the error message rather than a per-detail compiler error.
    let is_simulated_unknown = err.len() == 1
        && err.iter().all(|d| d.message == "[ReactCompiler] Invariant: unexpected error");
    if is_simulated_unknown {
        let mut diagnostic =
            OxcDiagnostic::error("[ReactCompiler] Pipeline error: Error: unexpected error");
        if let Some(span) = fn_span {
            diagnostic = diagnostic.with_label(span);
        }
        diagnostics.push(diagnostic);
        return;
    }

    for diagnostic in err {
        diagnostics.push(with_fallback_label(diagnostic, fn_span));
    }
}

/// Handle an error according to the panicThreshold setting.
/// Returns Some(CompileResult::Error) if the error should be surfaced as fatal,
/// otherwise returns None (error was logged only).
fn handle_error<'a>(
    err: &Diagnostics,
    fn_span: Option<Span>,
    panic_threshold: PanicThreshold,
    diagnostics: &mut Diagnostics,
) -> Option<CompileResult<'a>> {
    // Log the error
    log_error(err, fn_span, diagnostics);

    let should_panic = match panic_threshold {
        PanicThreshold::AllErrors => true,
        PanicThreshold::CriticalErrors => has_critical_errors(err),
        PanicThreshold::None => false,
    };

    // Config errors always cause a panic
    let is_config_error = err.iter().any(|d| ErrorCategory::Config.matches(d));

    if should_panic || is_config_error {
        // The per-detail diagnostics were already pushed by `log_error`; the fatal
        // result just carries them. (The old JS-shim summary is dropped.)
        Some(CompileResult::Error { diagnostics: std::mem::take(diagnostics) })
    } else {
        None
    }
}

// -----------------------------------------------------------------------
// Compilation pipeline stubs
// -----------------------------------------------------------------------

/// Attempt to compile a single function.
///
/// Returns `CodegenFunction` on success or the failed attempt's diagnostics.
/// Debug log entries are accumulated on `context.debug_logs`.
fn try_compile_function<'a>(
    ast: &AstBuilder<'a>,
    source: &CompileSource<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext<'a>,
) -> Result<Option<CodegenFunction<'a>>, Diagnostics> {
    // Check for suppressions that affect this function. Suppression errors are
    // returned (not thrown), so they do NOT trigger CompileUnexpectedThrow.
    if let (Some(start), Some(end)) = (source.fn_start, source.fn_end) {
        let affecting = filter_suppressions_that_affect_function(&context.suppressions, start, end);
        if !affecting.is_empty() {
            return Err(suppressions_to_diagnostics(&affecting, context.source_text));
        }
    }

    // Run the compilation pipeline directly on the oxc function node discovered
    // during the program walk.
    pipeline::compile_fn(
        ast,
        &source.fn_node,
        scope,
        source.fn_type,
        output_mode,
        env_config,
        context,
        source.fn_ast_span,
    )
}

/// Process a single function: check directives, attempt compilation, handle results.
///
/// Returns `Ok(Some(codegen_fn))` when the function was compiled and should be applied,
/// `Ok(None)` when the function was skipped or lint-only,
/// or `Err(CompileResult)` if a fatal error should short-circuit the program.
#[allow(clippy::result_large_err)]
fn process_fn<'a>(
    ast: &AstBuilder<'a>,
    source: &CompileSource<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext<'a>,
) -> Result<Option<CodegenFunction<'a>>, CompileResult<'a>> {
    // Parse directives from the function body
    let opt_in_result =
        try_find_directive_enabling_memoization(&source.body_directives, &context.opts);
    let opt_out = find_directive_disabling_memoization(
        source.body_directives.iter().map(Str::as_str),
        context.opts.custom_opt_out_directives.as_deref(),
    );

    // If parsing opt-in directive fails, handle the error and skip
    let opt_in = match opt_in_result {
        Ok(d) => d,
        Err(err) => {
            // Apply panic threshold logic (same as compilation errors)
            if let Some(result) = handle_error(
                &err,
                source.fn_ast_span,
                context.opts.panic_threshold,
                &mut context.diagnostics,
            ) {
                return Err(result);
            }
            return Ok(None);
        }
    };

    // Attempt compilation
    let compile_result = try_compile_function(ast, source, scope, output_mode, env_config, context);

    match compile_result {
        Err(err) => {
            if opt_out.is_some() {
                // If there's an opt-out, just log the error (don't escalate)
                log_error(&err, source.fn_ast_span, &mut context.diagnostics);
            } else {
                // Apply panic threshold logic
                if let Some(result) = handle_error(
                    &err,
                    source.fn_ast_span,
                    context.opts.panic_threshold,
                    &mut context.diagnostics,
                ) {
                    return Err(result);
                }
            }
            Ok(None)
        }
        // Lowering silently declined to compile this function (e.g. it uses
        // `using`/`await using`); nothing to emit, and no diagnostic.
        Ok(None) => Ok(None),
        Ok(Some(codegen_fn)) => {
            // Functions opted out via directive are skipped; nothing to emit.
            if !context.opts.ignore_use_no_forget && opt_out.is_some() {
                // Do NOT register the memo cache import here — it is registered in
                // apply_compiled_functions() only for functions actually applied.
                return Ok(None);
            }

            // Check module scope opt-out
            if context.has_module_scope_opt_out {
                return Ok(None);
            }

            // Check output mode — lint mode doesn't apply compiled functions
            if output_mode == CompilerOutputMode::Lint {
                return Ok(None);
            }

            // Check annotation mode
            if context.opts.compilation_mode == CompilationMode::Annotation && opt_in.is_none() {
                return Ok(None);
            }

            Ok(Some(codegen_fn))
        }
    }
}

// -----------------------------------------------------------------------
// Function discovery
// -----------------------------------------------------------------------

/// Collect the directive value strings of a function body (for opt-in/opt-out
/// checks). Matches the Babel-bridge directive values (`d.expression.value`).
fn body_directive_values<'a>(body: &FunctionBody<'a>) -> Vec<Str<'a>> {
    body.directives.iter().map(|d| d.expression.value).collect()
}

/// Try to create a `CompileSource` from a discovered oxc function node.
///
/// `fn_node` is the oxc function node (its scope is the function's identity),
/// `name` the inferred function name, `original_kind` the syntactic kind,
/// and `parent_callee_name` the enclosing forwardRef/memo callee (if any).
fn try_make_compile_source<'b, 'a>(
    fn_node: FunctionNode<'b, 'a>,
    name: Option<&str>,
    original_kind: OriginalFnKind,
    parent_callee_name: Option<&str>,
    opts: &PluginOptions,
    already_compiled: &mut FxHashSet<ScopeId>,
) -> Option<CompileSource<'b, 'a>> {
    let (params, body, span, body_directives) = match fn_node {
        FunctionNode::Function(f) => {
            // Bodyless functions (`declare function`, overload signatures,
            // `TSDeclareFunction`) are never compiled; Babel modeled these as
            // separate, non-traversed statement kinds.
            let block = f.body.as_deref()?;
            (&f.params, FnBody::Block(block), f.span, body_directive_values(block))
        }
        FunctionNode::Arrow(a) => {
            let (body, directives) = if a.expression {
                // Expression-bodied arrow: the single body statement is an
                // `ExpressionStatement` wrapping the expression.
                let expr = a.get_expression().expect("expression-bodied arrow has an expression");
                (FnBody::Expression(expr), Vec::new())
            } else {
                (FnBody::Block(&a.body), body_directive_values(&a.body))
            };
            (&a.params, body, a.span, directives)
        }
    };

    let fn_scope_id = fn_node.scope_id()?;

    // Skip if already compiled (identified by the function's scope). This is a
    // workaround for Babel not consistently respecting skip().
    if already_compiled.contains(&fn_scope_id) {
        return None;
    }

    let fn_type = get_react_function_type(
        name,
        params,
        &body,
        &body_directives,
        // Flow `component`/`hook` declaration syntax never appears in the oxc AST.
        false,
        parent_callee_name,
        opts,
        false,
        false,
    )?;

    already_compiled.insert(fn_scope_id);

    Some(CompileSource {
        kind: CompileSourceKind::Original,
        original_kind,
        // The function source location flows into compile-error diagnostics as the
        // fallback labeled span (offset/length). Only the byte `index` is
        // load-bearing; line/column/filename never reach the example's output.
        fn_ast_span: Some(span),
        fn_start: Some(span.start),
        fn_end: Some(span.end),
        fn_scope_id,
        fn_type,
        fn_node,
        body_directives,
    })
}

/// Get the variable declarator name (for inferring function names from
/// `const Foo = () => {}`).
fn get_declarator_name<'ast>(decl: &VariableDeclarator<'ast>) -> Option<&'ast str> {
    match &decl.id {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

// -----------------------------------------------------------------------
// Discovery walker
// -----------------------------------------------------------------------

/// Walks the oxc `Program` to find compilable functions, mirroring the
/// TypeScript compiler's Babel `program.traverse` behavior one-for-one.
///
/// Replicates the former Babel `AstWalker` + `FunctionDiscoveryVisitor`:
/// - scope tracking via `ScopeInfo` (`node_id_to_scope`, keyed by `span.start`);
/// - `loop_expression_depth` for loop test/right positions (while.test,
///   do-while.test, for-in.right, for-of.right), which Babel treats as
///   non-program-scope in 'all' mode;
/// - parent context: `current_declarator_name` for `const Foo = () => {}` and
///   `parent_callee_stack` for forwardRef/memo wrappers;
/// - in 'all' mode, rejects functions whose scope depth `> 2` (program +
///   function) or that sit in a loop expression position.
///
/// Compiled functions have their bodies skipped (Babel's `fn.skip()`); other
/// functions are descended to find nested component/hook declarations. Classes
/// are entered only structurally (their bodies carry no compilable functions for
/// discovery — matching the Babel bridge, which extracted no metadata for them).
struct DiscoveryWalker<'a, 'b, 'ast> {
    opts: &'a PluginOptions,
    already_compiled: FxHashSet<ScopeId>,
    queue: Vec<CompileSource<'b, 'ast>>,
    scope_stack: Vec<ScopeId>,
    loop_expression_depth: usize,
    current_declarator_name: Option<&'ast str>,
    parent_callee_stack: Vec<Option<&'ast str>>,
}

impl<'a, 'b, 'ast> DiscoveryWalker<'a, 'b, 'ast> {
    fn new(opts: &'a PluginOptions) -> Self {
        Self {
            opts,
            already_compiled: FxHashSet::default(),
            queue: Vec::new(),
            scope_stack: Vec::new(),
            loop_expression_depth: 0,
            current_declarator_name: None,
            parent_callee_stack: Vec::new(),
        }
    }

    /// Try to push the scope a node creates (its semantic `scope_id` cell).
    /// Returns whether one was pushed. The stack is only consulted by the
    /// 'all'-mode scope check, so skip maintaining it in other modes.
    fn try_push_scope(&mut self, scope_id: Option<ScopeId>) -> bool {
        if self.opts.compilation_mode != CompilationMode::All {
            return false;
        }
        if let Some(scope_id) = scope_id {
            self.scope_stack.push(scope_id);
            true
        } else {
            false
        }
    }

    /// In 'all' mode, reject functions that are not at program scope. The
    /// function's own scope is on the stack, so a top-level function has
    /// `len == 2` (program + function); deeper means a nested scope.
    fn is_rejected_by_scope_check(&self) -> bool {
        self.opts.compilation_mode == CompilationMode::All
            && (self.scope_stack.len() > 2 || self.loop_expression_depth > 0)
    }

    fn current_parent_callee(&self) -> Option<&'ast str> {
        self.parent_callee_stack.last().copied().flatten()
    }

    fn walk_program(&mut self, program: &'b Program<'ast>) {
        let pushed = self.try_push_scope(program.scope_id.get());
        for stmt in &program.body {
            self.walk_statement(stmt);
        }
        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_block(&mut self, block: &'b BlockStatement<'ast>) {
        let pushed = self.try_push_scope(block.scope_id.get());
        for stmt in &block.body {
            self.walk_statement(stmt);
        }
        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_function_body_block(&mut self, body: &'b FunctionBody<'ast>) {
        // A function body BlockStatement shares the function's scope in the Babel
        // model and never gets its own scope entry, so do not push a scope here.
        for stmt in &body.statements {
            self.walk_statement(stmt);
        }
    }

    fn walk_statement(&mut self, stmt: &'b Statement<'ast>) {
        match stmt {
            Statement::BlockStatement(node) => self.walk_block(node),
            Statement::ReturnStatement(node) => {
                if let Some(arg) = &node.argument {
                    self.walk_expression(arg);
                }
            }
            Statement::ExpressionStatement(node) => self.walk_expression(&node.expression),
            Statement::IfStatement(node) => {
                self.walk_expression(&node.test);
                self.walk_statement(&node.consequent);
                if let Some(alt) = &node.alternate {
                    self.walk_statement(alt);
                }
            }
            Statement::ForStatement(node) => {
                let pushed = self.try_push_scope(node.scope_id.get());
                if let Some(init) = &node.init {
                    match init {
                        ForStatementInit::VariableDeclaration(decl) => {
                            self.walk_variable_declaration(decl);
                        }
                        expr => {
                            if let Some(expr) = expr.as_expression() {
                                self.walk_expression(expr);
                            }
                        }
                    }
                }
                if let Some(test) = &node.test {
                    self.walk_expression(test);
                }
                if let Some(update) = &node.update {
                    self.walk_expression(update);
                }
                self.walk_statement(&node.body);
                if pushed {
                    self.scope_stack.pop();
                }
            }
            Statement::WhileStatement(node) => {
                self.loop_expression_depth += 1;
                self.walk_expression(&node.test);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
            }
            Statement::DoWhileStatement(node) => {
                self.walk_statement(&node.body);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.test);
                self.loop_expression_depth -= 1;
            }
            Statement::ForInStatement(node) => {
                let pushed = self.try_push_scope(node.scope_id.get());
                self.walk_for_left(&node.left);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.right);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
                if pushed {
                    self.scope_stack.pop();
                }
            }
            Statement::ForOfStatement(node) => {
                let pushed = self.try_push_scope(node.scope_id.get());
                self.walk_for_left(&node.left);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.right);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
                if pushed {
                    self.scope_stack.pop();
                }
            }
            Statement::SwitchStatement(node) => {
                let pushed = self.try_push_scope(node.scope_id.get());
                self.walk_expression(&node.discriminant);
                for case in &node.cases {
                    if let Some(test) = &case.test {
                        self.walk_expression(test);
                    }
                    for consequent in &case.consequent {
                        self.walk_statement(consequent);
                    }
                }
                if pushed {
                    self.scope_stack.pop();
                }
            }
            Statement::ThrowStatement(node) => self.walk_expression(&node.argument),
            Statement::TryStatement(node) => {
                self.walk_block(&node.block);
                if let Some(handler) = &node.handler {
                    let pushed = self.try_push_scope(handler.scope_id.get());
                    self.walk_block(&handler.body);
                    if pushed {
                        self.scope_stack.pop();
                    }
                }
                if let Some(finalizer) = &node.finalizer {
                    self.walk_block(finalizer);
                }
            }
            Statement::LabeledStatement(node) => self.walk_statement(&node.body),
            Statement::VariableDeclaration(node) => self.walk_variable_declaration(node),
            Statement::FunctionDeclaration(node) => self.walk_function(node, None),
            Statement::WithStatement(node) => {
                self.walk_expression(&node.object);
                self.walk_statement(&node.body);
            }
            Statement::ExportNamedDeclaration(node) => {
                if let Some(decl) = &node.declaration {
                    self.walk_declaration(decl);
                }
            }
            Statement::ExportDefaultDeclaration(node) => {
                self.walk_export_default(&node.declaration);
            }
            // Classes are not descended into (their bodies hold no functions for
            // discovery); all remaining statements (TS-only declarations, imports,
            // break/continue, etc.) carry no compilable functions.
            _ => {}
        }
    }

    fn walk_for_left(&mut self, left: &'b ForStatementLeft<'ast>) {
        if let ForStatementLeft::VariableDeclaration(decl) = left {
            self.walk_variable_declaration(decl);
        }
        // Assignment-target lefts contain no functions to discover.
    }

    fn walk_variable_declaration(&mut self, decl: &'b VariableDeclaration<'ast>) {
        for declarator in &decl.declarations {
            // Only infer the declarator name when the init is a direct function
            // expression, arrow, or call expression (for forwardRef/memo wrappers).
            if let Some(init) = &declarator.init {
                if matches!(
                    init,
                    Expression::FunctionExpression(_)
                        | Expression::ArrowFunctionExpression(_)
                        | Expression::CallExpression(_)
                ) {
                    self.current_declarator_name = get_declarator_name(declarator);
                }
                self.walk_expression(init);
            }
            self.current_declarator_name = None;
        }
    }

    fn walk_declaration(&mut self, decl: &'b Declaration<'ast>) {
        match decl {
            Declaration::FunctionDeclaration(node) => self.walk_function(node, None),
            Declaration::VariableDeclaration(node) => self.walk_variable_declaration(node),
            // TS-only declarations have no runtime expressions / functions.
            _ => {}
        }
    }

    fn walk_export_default(&mut self, decl: &'b ExportDefaultDeclarationKind<'ast>) {
        match decl {
            ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
                self.walk_function(node, None)
            }
            other => {
                if let Some(expr) = other.as_expression() {
                    self.walk_expression(expr);
                }
            }
        }
    }

    /// Walk an oxc `Function` node (declaration or expression). `inferred_name`,
    /// when `Some`, supplies the name from the enclosing variable declarator (for
    /// function expressions); `None` means use the function's own id.
    fn walk_function(&mut self, func: &'b Function<'ast>, inferred_name: Option<&'ast str>) {
        let pushed = self.try_push_scope(func.scope_id.get());

        let original_kind = match func.r#type {
            FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction => {
                OriginalFnKind::FunctionDeclaration
            }
            _ => OriginalFnKind::FunctionExpression,
        };

        let skip_body = if self.is_rejected_by_scope_check() {
            false
        } else {
            // TS `getFunctionName` for FunctionDeclaration uses the node's own id;
            // for FunctionExpression it uses only the parent context (declarator).
            let name = match original_kind {
                OriginalFnKind::FunctionDeclaration => get_function_name_from_id(func.id.as_ref()),
                _ => inferred_name,
            };
            let parent_callee = self.current_parent_callee();
            if let Some(source) = try_make_compile_source(
                FunctionNode::Function(func),
                name,
                original_kind,
                parent_callee,
                self.opts,
                &mut self.already_compiled,
            ) {
                self.queue.push(source);
                true
            } else {
                false
            }
        };

        if !skip_body {
            // Babel `fn.skip()` is only called for compiled functions; other
            // functions are descended to find nested declarations.
            if let Some(body) = &func.body {
                self.walk_function_body_block(body);
            }
        }

        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_arrow(
        &mut self,
        arrow: &'b ArrowFunctionExpression<'ast>,
        inferred_name: Option<&'ast str>,
    ) {
        let pushed = self.try_push_scope(arrow.scope_id.get());

        let skip_body = if self.is_rejected_by_scope_check() {
            false
        } else {
            let parent_callee = self.current_parent_callee();
            if let Some(source) = try_make_compile_source(
                FunctionNode::Arrow(arrow),
                inferred_name,
                OriginalFnKind::ArrowFunctionExpression,
                parent_callee,
                self.opts,
                &mut self.already_compiled,
            ) {
                self.queue.push(source);
                true
            } else {
                false
            }
        };

        if !skip_body {
            for stmt in &arrow.body.statements {
                self.walk_statement(stmt);
            }
        }

        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_expression(&mut self, expr: &'b Expression<'ast>) {
        match expr {
            Expression::FunctionExpression(node) => {
                // The declarator name flows into the function expression and is
                // consumed here (so siblings don't inherit it).
                let name = self.current_declarator_name.take();
                self.walk_function(node, name);
            }
            Expression::ArrowFunctionExpression(node) => {
                let name = self.current_declarator_name.take();
                self.walk_arrow(node, name);
            }
            Expression::CallExpression(node) => {
                let callee_name = get_callee_name_if_react_api(&node.callee);
                // The declarator name only flows through forwardRef/memo calls; for
                // any other call, clear it so nested functions don't inherit it.
                if callee_name.is_none() {
                    self.current_declarator_name = None;
                }
                self.parent_callee_stack.push(callee_name);
                self.walk_expression(&node.callee);
                for arg in &node.arguments {
                    self.walk_argument(arg);
                }
                let was_react_api = self.parent_callee_stack.pop().flatten().is_some();
                if was_react_api {
                    self.current_declarator_name = None;
                }
            }
            Expression::ChainExpression(node) => self.walk_chain_element(&node.expression),
            Expression::StaticMemberExpression(node) => self.walk_expression(&node.object),
            Expression::ComputedMemberExpression(node) => {
                self.walk_expression(&node.object);
                self.walk_expression(&node.expression);
            }
            Expression::PrivateFieldExpression(node) => self.walk_expression(&node.object),
            Expression::BinaryExpression(node) => {
                self.walk_expression(&node.left);
                self.walk_expression(&node.right);
            }
            Expression::LogicalExpression(node) => {
                self.walk_expression(&node.left);
                self.walk_expression(&node.right);
            }
            Expression::UnaryExpression(node) => self.walk_expression(&node.argument),
            Expression::UpdateExpression(node) => {
                if let Some(member) = node.argument.as_member_expression() {
                    self.walk_member_expression(member);
                }
            }
            Expression::ConditionalExpression(node) => {
                self.walk_expression(&node.test);
                self.walk_expression(&node.consequent);
                self.walk_expression(&node.alternate);
            }
            Expression::AssignmentExpression(node) => self.walk_expression(&node.right),
            Expression::SequenceExpression(node) => {
                for e in &node.expressions {
                    self.walk_expression(e);
                }
            }
            Expression::ObjectExpression(node) => {
                for prop in &node.properties {
                    self.walk_object_property(prop);
                }
            }
            Expression::ArrayExpression(node) => {
                for el in &node.elements {
                    match el {
                        ArrayExpressionElement::SpreadElement(s) => {
                            self.walk_expression(&s.argument)
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        other => {
                            if let Some(e) = other.as_expression() {
                                self.walk_expression(e);
                            }
                        }
                    }
                }
            }
            Expression::NewExpression(node) => {
                self.walk_expression(&node.callee);
                for arg in &node.arguments {
                    self.walk_argument(arg);
                }
            }
            Expression::TemplateLiteral(node) => {
                for e in &node.expressions {
                    self.walk_expression(e);
                }
            }
            Expression::TaggedTemplateExpression(node) => {
                self.walk_expression(&node.tag);
                for e in &node.quasi.expressions {
                    self.walk_expression(e);
                }
            }
            Expression::AwaitExpression(node) => self.walk_expression(&node.argument),
            Expression::YieldExpression(node) => {
                if let Some(arg) = &node.argument {
                    self.walk_expression(arg);
                }
            }
            Expression::ParenthesizedExpression(node) => self.walk_expression(&node.expression),
            Expression::JSXElement(node) => self.walk_jsx_element(node),
            Expression::JSXFragment(node) => self.walk_jsx_children(&node.children),
            Expression::TSAsExpression(node) => self.walk_expression(&node.expression),
            Expression::TSSatisfiesExpression(node) => self.walk_expression(&node.expression),
            Expression::TSNonNullExpression(node) => self.walk_expression(&node.expression),
            Expression::TSTypeAssertion(node) => self.walk_expression(&node.expression),
            Expression::TSInstantiationExpression(node) => self.walk_expression(&node.expression),
            // ClassExpression bodies and leaf expressions are not descended.
            _ => {}
        }
    }

    fn walk_argument(&mut self, arg: &'b Argument<'ast>) {
        if let Some(expr) = arg.as_expression() {
            self.walk_expression(expr);
        } else if let Argument::SpreadElement(s) = arg {
            self.walk_expression(&s.argument);
        }
    }

    fn walk_member_expression(&mut self, member: &'b MemberExpression<'ast>) {
        match member {
            MemberExpression::StaticMemberExpression(m) => self.walk_expression(&m.object),
            MemberExpression::ComputedMemberExpression(m) => {
                self.walk_expression(&m.object);
                self.walk_expression(&m.expression);
            }
            MemberExpression::PrivateFieldExpression(m) => self.walk_expression(&m.object),
        }
    }

    fn walk_chain_element(&mut self, element: &'b ChainElement<'ast>) {
        match element {
            ChainElement::CallExpression(node) => {
                self.walk_expression(&node.callee);
                for arg in &node.arguments {
                    self.walk_argument(arg);
                }
            }
            ChainElement::StaticMemberExpression(m) => self.walk_expression(&m.object),
            ChainElement::ComputedMemberExpression(m) => {
                self.walk_expression(&m.object);
                self.walk_expression(&m.expression);
            }
            ChainElement::PrivateFieldExpression(m) => self.walk_expression(&m.object),
            ChainElement::TSNonNullExpression(t) => self.walk_expression(&t.expression),
        }
    }

    fn walk_object_property(&mut self, prop: &'b ObjectPropertyKind<'ast>) {
        match prop {
            ObjectPropertyKind::SpreadProperty(s) => self.walk_expression(&s.argument),
            ObjectPropertyKind::ObjectProperty(p) => {
                if p.computed {
                    self.walk_property_key(&p.key);
                }
                // Object methods (`{ foo() {} }`, getters/setters): Babel modeled
                // these as `ObjectMethod` and walked the body without queuing the
                // method itself; the body is the value FunctionExpression's body.
                let is_method = p.method || matches!(p.kind, PropertyKind::Get | PropertyKind::Set);
                if is_method {
                    if let Expression::FunctionExpression(func) = &p.value {
                        let pushed = self.try_push_scope(func.scope_id.get());
                        if let Some(body) = &func.body {
                            self.walk_function_body_block(body);
                        }
                        if pushed {
                            self.scope_stack.pop();
                        }
                    }
                } else {
                    self.walk_expression(&p.value);
                }
            }
        }
    }

    fn walk_property_key(&mut self, key: &'b PropertyKey<'ast>) {
        if let Some(expr) = key.as_expression() {
            self.walk_expression(expr);
        }
    }

    fn walk_jsx_element(&mut self, node: &'b JSXElement<'ast>) {
        for attr in &node.opening_element.attributes {
            match attr {
                JSXAttributeItem::Attribute(a) => {
                    if let Some(value) = &a.value {
                        match value {
                            JSXAttributeValue::ExpressionContainer(c) => self.walk_jsx_container(c),
                            JSXAttributeValue::Element(el) => self.walk_jsx_element(el),
                            JSXAttributeValue::Fragment(f) => self.walk_jsx_children(&f.children),
                            JSXAttributeValue::StringLiteral(_) => {}
                        }
                    }
                }
                JSXAttributeItem::SpreadAttribute(a) => self.walk_expression(&a.argument),
            }
        }
        self.walk_jsx_children(&node.children);
    }

    fn walk_jsx_children(&mut self, children: &'b ArenaVec<'ast, JSXChild<'ast>>) {
        for child in children {
            match child {
                JSXChild::Element(el) => self.walk_jsx_element(el),
                JSXChild::Fragment(f) => self.walk_jsx_children(&f.children),
                JSXChild::ExpressionContainer(c) => self.walk_jsx_container(c),
                JSXChild::Spread(s) => self.walk_expression(&s.expression),
                JSXChild::Text(_) => {}
            }
        }
    }

    fn walk_jsx_container(&mut self, container: &'b JSXExpressionContainer<'ast>) {
        if let Some(expr) = container.expression.as_expression() {
            self.walk_expression(expr);
        }
    }
}

/// Find all functions in the program that should be compiled by walking the oxc
/// `Program` directly. See [`DiscoveryWalker`] for the traversal semantics.
fn find_functions_to_compile<'b, 'ast>(
    program: &'b Program<'ast>,
    opts: &PluginOptions,
) -> Vec<CompileSource<'b, 'ast>> {
    let mut walker = DiscoveryWalker::new(opts);
    walker.walk_program(program);
    walker.queue
}

// -----------------------------------------------------------------------
// Discovery pre-check
// -----------------------------------------------------------------------

/// Cheap, sound pre-check for [`find_functions_to_compile`]: `false` means the
/// discovery walk cannot queue anything, so the compile is a no-op. Built from
/// data `Semantic` already computed instead of walking the AST, and delegating
/// all judgment to discovery's own helpers:
///
/// - every discoverable function creates a function scope, so the node behind
///   each function scope is run through [`try_make_compile_source`] with the
///   name discovery would infer — own id for declarations, the directly
///   enclosing `const Foo = ...` declarator for expressions/arrows, the only
///   name sources discovery uses outside forwardRef/memo. This covers the
///   named and directive-opt-in selection paths, nested functions included;
/// - the forwardRef/memo path needs an identifier named `memo`, `forwardRef`,
///   or `React` in callee position of a call [`get_callee_name_if_react_api`]
///   recognizes, so checking the reference shapes of those three names —
///   bindings and unresolved globals — covers wrapped anonymous functions.
///
/// Over-approximation is fine (the walk then finds an empty queue); a missed
/// witness is not, since a skipped file is never compiled.
fn may_have_functions_to_compile(semantic: &Semantic, opts: &PluginOptions) -> bool {
    // 'all' mode compiles every top-level function; always walk.
    if opts.compilation_mode == CompilationMode::All {
        return true;
    }

    let scoping = semantic.scoping();
    let nodes = semantic.nodes();

    // forwardRef/memo wrappers used as globals: O(1) lookups. Discovery matches
    // callee *names*, not bindings, so unresolved references count too.
    const WRAPPER_NAMES: [&str; 3] = ["memo", "forwardRef", "React"];
    for name in WRAPPER_NAMES {
        if let Some(reference_ids) = scoping.root_unresolved_references().get(name)
            && reference_ids.iter().any(|reference_id| {
                is_wrapper_callee(nodes, scoping.get_reference(*reference_id).node_id())
            })
        {
            return true;
        }
    }

    // Named components/hooks and directive opt-ins: run the node behind every
    // function scope through discovery's candidate constructor. parent_callee
    // is None — wrapper-selected functions are witnessed by reference shapes.
    let mut discarded = FxHashSet::default();
    for scope_id in scoping.scope_descendants_from_root() {
        if !scoping.scope_flags(scope_id).is_function() {
            continue;
        }
        let node = nodes.get_node(scoping.get_node_id(scope_id));
        let (fn_node, name, original_kind) = match node.kind() {
            AstKind::Function(func) => {
                let (name, original_kind) = match func.r#type {
                    FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction => (
                        get_function_name_from_id(func.id.as_ref()),
                        OriginalFnKind::FunctionDeclaration,
                    ),
                    _ => {
                        (declarator_name_for(nodes, node.id()), OriginalFnKind::FunctionExpression)
                    }
                };
                // A nameless function without directives can never classify.
                if name.is_none()
                    && func.body.as_ref().is_none_or(|body| body.directives.is_empty())
                {
                    continue;
                }
                (FunctionNode::Function(func), name, original_kind)
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                let name = declarator_name_for(nodes, node.id());
                if name.is_none() && arrow.body.directives.is_empty() {
                    continue;
                }
                (FunctionNode::Arrow(arrow), name, OriginalFnKind::ArrowFunctionExpression)
            }
            _ => continue,
        };
        if try_make_compile_source(fn_node, name, original_kind, None, opts, &mut discarded)
            .is_some()
        {
            return true;
        }
    }

    // Bindings named like a wrapper (`import {memo} from 'react'`, or even a
    // local `const memo = ...` — discovery treats any `memo(...)` call as a
    // wrapper regardless of what the name resolves to). Scanned last so
    // component files exit in the function-scope pass above.
    for symbol_id in scoping.symbol_ids() {
        if matches!(scoping.symbol_name(symbol_id), "memo" | "forwardRef" | "React")
            && has_wrapper_callee_reference(scoping, nodes, symbol_id)
        {
            return true;
        }
    }

    false
}

/// Whether any resolved reference of `symbol_id` is a wrapper-call callee.
fn has_wrapper_callee_reference(scoping: &Scoping, nodes: &AstNodes, symbol_id: SymbolId) -> bool {
    scoping.get_resolved_reference_ids(symbol_id).iter().any(|reference_id| {
        is_wrapper_callee(nodes, scoping.get_reference(*reference_id).node_id())
    })
}

/// The `const Foo = <fn>` name for a function/arrow node, iff the declarator's
/// init is directly this node — the same direct-init rule the discovery walker
/// applies. A function whose direct parent is the declarator can only be its
/// init (wrappers like parens or TS casts introduce an intermediate parent and
/// break the inference there too).
fn declarator_name_for<'a>(nodes: &AstNodes<'a>, node_id: NodeId) -> Option<&'a str> {
    match nodes.parent_kind(node_id) {
        AstKind::VariableDeclarator(decl) => get_declarator_name(decl),
        _ => None,
    }
}

/// Whether a reference sits in callee position of a call that
/// [`get_callee_name_if_react_api`] recognizes — directly (`memo(...)`) or as
/// the object of a called member (`React.memo(...)`).
fn is_wrapper_callee(nodes: &AstNodes, node_id: NodeId) -> bool {
    let node_address = nodes.get_node(node_id).address();
    let parent = nodes.parent_node(node_id);
    let call = match parent.kind() {
        AstKind::CallExpression(call) if call.callee.address() == node_address => call,
        AstKind::StaticMemberExpression(member) if member.object.address() == node_address => {
            match nodes.parent_kind(parent.id()) {
                AstKind::CallExpression(call) if call.callee.address() == parent.address() => call,
                _ => return false,
            }
        }
        _ => return false,
    };
    get_callee_name_if_react_api(&call.callee).is_some()
}

struct CompiledFunction<'a, 'b, 'p, 's> {
    kind: CompileSourceKind,
    source: &'s CompileSource<'b, 'p>,
    codegen_fn: CodegenFunction<'a>,
}

/// The type of the original function node, used to determine what kind of
/// replacement node to create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OriginalFnKind {
    FunctionDeclaration,
    FunctionExpression,
    ArrowFunctionExpression,
}

// =============================================================================
// oxc transform
//
// Rewrites the oxc `Program` in place by substituting each compiled oxc function
// (from codegen) for its original — matched by the original function's scope —
// applying gating, inserting outlined functions, and adding the memo-cache /
// gating imports.
// =============================================================================

/// An owned, oxc-shaped compiled function ready to substitute into the program.
struct OxcReplacement<'a> {
    fn_scope_id: ScopeId,
    original_kind: OriginalFnKind,
    codegen_fn: CodegenFunction<'a>,
    gating: Option<GatingConfig>,
}

/// The owned output of a [`compile`](crate::compile) that produced changes:
/// everything the transform phase needs, carrying only the arena lifetime — no
/// borrows of the input `Program` or `Semantic` — so the caller can drop its
/// `Semantic` (and with it the shared borrow of the program) before mutating.
pub struct CompileOutput<'a> {
    replacements: Vec<OxcReplacement<'a>>,
    context: ProgramContext<'a>,
}

impl<'a> CompileOutput<'a> {
    /// Rewrite `program` in place: substitute each original function with its
    /// memoized version (matched by the `scope_id` cells populated when the
    /// caller built `Semantic`), insert outlined declarations, add the required
    /// imports, and drop comments left dangling inside replaced functions.
    ///
    /// `program` must be the same, unmodified program [`compile`](crate::compile)
    /// saw. Afterwards any previously computed `Semantic`/`Scoping` is stale.
    pub fn transform(self, program: &mut Program<'a>) {
        let CompileOutput { replacements, mut context } = self;
        let ast = AstBuilder::new(context.allocator());
        ox_transform_program(&ast, program, &replacements, &mut context);
        // Diagnostics were extracted at the end of compilation; nothing in the
        // transform phase may add more.
        debug_assert!(context.diagnostics.is_empty());
        prune_inner_comments(program);
    }
}

/// Drop comments left dangling by compilation.
///
/// The compiled functions were rebuilt with fresh spans, so a comment that
/// pointed inside one no longer lines up with any statement and codegen would
/// re-emit it at a stale position. Keep only the comments still anchored to a
/// top-level statement.
fn prune_inner_comments(program: &mut Program<'_>) {
    if program.comments.is_empty() {
        return;
    }
    let mut top_level_starts = FxHashSet::default();
    top_level_starts.insert(0u32);
    for stmt in &program.body {
        let start = stmt.span().start;
        if start > 0 {
            top_level_starts.insert(start);
        }
    }
    program.comments.retain(|comment| top_level_starts.contains(&comment.attached_to));
}

/// Copy the TS metadata (type annotation, decorators, optional/modifier flags)
/// from a function's original parameters onto the compiled replacement parameters,
/// matched positionally. Mirrors the Babel reference's signature restoration for
/// functions that are not memoized: the parameter bindings are unchanged, so their
/// types carry through. The compiled params (from codegen) never carry types.
fn copy_param_ts_metadata<'a>(
    allocator: &'a oxc_allocator::Allocator,
    new_params: &mut FormalParameters<'a>,
    source_params: &FormalParameters<'a>,
) {
    for (param, source) in new_params.items.iter_mut().zip(source_params.items.iter()) {
        param.decorators = source.decorators.clone_in_with_semantic_ids(allocator);
        param.type_annotation = source.type_annotation.clone_in_with_semantic_ids(allocator);
        param.optional = source.optional;
        param.accessibility = source.accessibility;
        param.readonly = source.readonly;
        param.r#override = source.r#override;
    }
    if let (Some(rest), Some(source_rest)) = (&mut new_params.rest, &source_params.rest) {
        rest.decorators = source_rest.decorators.clone_in_with_semantic_ids(allocator);
        rest.type_annotation = source_rest.type_annotation.clone_in_with_semantic_ids(allocator);
    }
}

/// Build an oxc `Function` from a compiled codegen function. `r#type` selects
/// declaration vs expression. Mirrors the Babel `ReplaceFnVisitor` field copy.
fn ox_build_function<'a>(
    ast: &AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    fn_type: FunctionType,
) -> ArenaBox<'a, Function<'a>> {
    Function::boxed(
        SPAN,
        fn_type,
        codegen.id.clone_in_with_semantic_ids(ast.allocator()),
        codegen.generator,
        codegen.is_async,
        false,
        None::<ArenaBox<TSTypeParameterDeclaration>>,
        None::<ArenaBox<TSThisParameter>>,
        codegen.params.clone_in_with_semantic_ids(ast.allocator()),
        None::<ArenaBox<TSTypeAnnotation>>,
        Some(codegen.body.clone_in_with_semantic_ids(ast.allocator())),
        ast,
    )
}

/// Build the compiled replacement as an `Expression`, matching the original node
/// kind (arrow vs function expression). Mirrors `build_compiled_expression_matching_kind`.
fn ox_build_compiled_expression<'a>(
    ast: &AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    original_kind: OriginalFnKind,
) -> Expression<'a> {
    match original_kind {
        OriginalFnKind::ArrowFunctionExpression => {
            Expression::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
                SPAN,
                false,
                codegen.is_async,
                None::<ArenaBox<TSTypeParameterDeclaration>>,
                codegen.params.clone_in_with_semantic_ids(ast.allocator()),
                None::<ArenaBox<TSTypeAnnotation>>,
                codegen.body.clone_in_with_semantic_ids(ast.allocator()),
                ast,
            ))
        }
        _ => Expression::FunctionExpression(ox_build_function(
            ast,
            codegen,
            FunctionType::FunctionExpression,
        )),
    }
}

/// Substitute a compiled codegen function into the original `Function` node in
/// place. Mirrors the Babel `ReplaceFnVisitor` replacement of a function
/// declaration or expression.
fn ox_replace_function<'a>(
    ast: &AstBuilder<'a>,
    func: &mut Function<'a>,
    codegen: &CodegenFunction<'a>,
) {
    // When the compiled function does not initialize a memo cache, the body is
    // left essentially intact, so the original TS signature (type parameters,
    // `this` parameter, return type, and per-parameter type annotations) is
    // preserved. Functions that memoize drop these types, mirroring Babel.
    let keep_types = codegen.memo_slots_used == 0;
    let mut params = codegen.params.clone_in_with_semantic_ids(ast.allocator());
    if keep_types {
        copy_param_ts_metadata(ast.allocator(), &mut params, &func.params);
    } else {
        func.type_parameters = None;
        func.return_type = None;
        func.this_param = None;
    }
    func.id = codegen.id.clone_in_with_semantic_ids(ast.allocator());
    func.params = params;
    func.body = Some(codegen.body.clone_in_with_semantic_ids(ast.allocator()));
    func.generator = codegen.generator;
    func.r#async = codegen.is_async;
    func.declare = false;
}

/// Substitute a compiled codegen function into the original
/// `ArrowFunctionExpression` node in place. Mirrors the Babel
/// `ReplaceFnVisitor` replacement of an arrow function.
fn ox_replace_arrow<'a>(
    ast: &AstBuilder<'a>,
    arrow: &mut ArrowFunctionExpression<'a>,
    codegen: &CodegenFunction<'a>,
) {
    let keep_types = codegen.memo_slots_used == 0;
    let mut params = codegen.params.clone_in_with_semantic_ids(ast.allocator());
    if keep_types {
        copy_param_ts_metadata(ast.allocator(), &mut params, &arrow.params);
    } else {
        arrow.type_parameters = None;
        arrow.return_type = None;
    }
    arrow.params = params;
    arrow.body = codegen.body.clone_in_with_semantic_ids(ast.allocator());
    arrow.r#async = codegen.is_async;
    arrow.expression = false;
}

/// Build `const <name> = <gating_expression>;`
fn ox_build_gated_const_decl<'a>(
    ast: &AstBuilder<'a>,
    gating_expression: &Expression<'a>,
    name: &str,
) -> Statement<'a> {
    let declarator = VariableDeclarator::new(
        SPAN,
        VariableDeclarationKind::Const,
        BindingPattern::new_binding_identifier(SPAN, ox_atom(ast, name), ast),
        None::<ArenaBox<TSTypeAnnotation>>,
        Some(gating_expression.clone_in_with_semantic_ids(ast.allocator())),
        false,
        ast,
    );
    Statement::VariableDeclaration(VariableDeclaration::boxed(
        SPAN,
        VariableDeclarationKind::Const,
        ArenaVec::from_value_in(declarator, ast),
        false,
        ast,
    ))
}

/// The one visitor type behind every oxc-AST traversal of the transform phase,
/// selected by [`OxcVisitMode`].
///
/// Every distinct `Visit`/`VisitMut` implementor monomorphizes the entire
/// generated AST walk, so the three traversals here (batched function
/// replacement, gated replacement, original-function lookup) share this single
/// type — and thereby a single copy of the walk in the binary. Each overridden
/// method must behave exactly like the default walk in the modes that do not
/// use it.
struct OxcVisitor<'a, 'b> {
    ast: &'b AstBuilder<'a>,
    mode: OxcVisitMode<'a, 'b>,
}

/// Which traversal an [`OxcVisitor`] performs.
enum OxcVisitMode<'a, 'b> {
    /// Replace every compiled function in the oxc AST in a single traversal,
    /// each matched by its original function's scope. Mirrors the Babel
    /// `ReplaceFnVisitor`, batched so the program is walked once regardless of
    /// how many functions were compiled.
    ReplaceFns {
        /// Compiled replacement for each original function, keyed by the function's
        /// `scope_id` cell. Codegen-built nodes carry no cells, so they are never
        /// matched.
        replacements: FxHashMap<ScopeId, &'b CodegenFunction<'a>>,
        /// Replacements not yet applied; the walk stops once it reaches zero.
        remaining: usize,
    },
    /// Replace one function (matched by its scope) with a gated conditional
    /// expression. Mirrors the Babel `ReplaceWithGatedVisitor`.
    ReplaceWithGated {
        scope_id: ScopeId,
        gating_expression: &'b Expression<'a>,
        /// Pending `export default Name;` to insert after a named export-default fn.
        export_default_name: Option<Ident<'a>>,
        done: bool,
    },
    /// Find the function with scope `scope_id` and clone it as an expression
    /// (a declaration becomes a `FunctionExpression`), for
    /// [`ox_clone_original_fn_as_expression`]. Mutates nothing.
    FindOriginalFn { scope_id: ScopeId, found: Option<Expression<'a>> },
}

impl<'a> OxcVisitor<'a, '_> {
    /// In [`OxcVisitMode::ReplaceWithGated`]: if `stmt` is the target function
    /// declaration (bare, `export`-wrapped, or `export default`), substitute
    /// the gated replacement, mark the mode done, and return `true`.
    fn try_replace_gated_statement(&mut self, stmt: &mut Statement<'a>) -> bool {
        let ast = self.ast;
        let OxcVisitMode::ReplaceWithGated {
            scope_id,
            gating_expression,
            export_default_name,
            done,
        } = &mut self.mode
        else {
            return false;
        };
        let scope_id = *scope_id;
        let gating_expression = *gating_expression;
        // FunctionDeclaration → `const Foo = gating() ? ... : ...;`
        let replace_name: Option<Option<Ident<'a>>> = match &*stmt {
            Statement::FunctionDeclaration(f) if f.scope_id.get() == Some(scope_id) => {
                Some(f.id.as_ref().map(|id| id.name))
            }
            Statement::ExportNamedDeclaration(e) => match &e.declaration {
                Some(Declaration::FunctionDeclaration(f)) if f.scope_id.get() == Some(scope_id) => {
                    Some(f.id.as_ref().map(|id| id.name))
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(name) = replace_name {
            let name = name.as_deref().unwrap_or("anonymous");
            let is_export = matches!(stmt, Statement::ExportNamedDeclaration(_));
            let const_decl = ox_build_gated_const_decl(ast, gating_expression, name);
            if is_export {
                let decl = match const_decl {
                    Statement::VariableDeclaration(d) => Declaration::VariableDeclaration(d),
                    _ => unreachable!(),
                };
                *stmt = Statement::ExportNamedDeclaration(ExportNamedDeclaration::boxed(
                    SPAN,
                    Some(decl),
                    [],
                    None,
                    ImportOrExportKind::Value,
                    None::<ArenaBox<WithClause>>,
                    ast,
                ));
            } else {
                *stmt = const_decl;
            }
            *done = true;
            return true;
        }
        // ExportDefaultDeclaration with FunctionDeclaration
        if let Statement::ExportDefaultDeclaration(e) = &*stmt
            && let ExportDefaultDeclarationKind::FunctionDeclaration(f) = &e.declaration
            && f.scope_id.get() == Some(scope_id)
        {
            if let Some(id) = f.id.as_ref().map(|id| id.name) {
                *stmt = ox_build_gated_const_decl(ast, gating_expression, id.as_str());
                *export_default_name = Some(id);
            } else {
                *stmt = Statement::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(
                    SPAN,
                    ExportDefaultDeclarationKind::from(
                        gating_expression.clone_in_with_semantic_ids(ast.allocator()),
                    ),
                    ast,
                ));
            }
            *done = true;
            return true;
        }
        false
    }
}

impl<'a> oxc_ast_visit::VisitMut<'a> for OxcVisitor<'a, '_> {
    fn visit_function(&mut self, func: &mut Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let ast = self.ast;
        match &mut self.mode {
            OxcVisitMode::ReplaceFns { replacements, remaining } => {
                if *remaining == 0 {
                    return;
                }
                if let Some(scope_id) = func.scope_id.get()
                    && let Some(&codegen) = replacements.get(&scope_id)
                {
                    ox_replace_function(ast, func, codegen);
                    *remaining -= 1;
                    return;
                }
            }
            OxcVisitMode::ReplaceWithGated { .. } => {}
            OxcVisitMode::FindOriginalFn { scope_id, found } => {
                if found.is_some() {
                    return;
                }
                if func.scope_id.get() == Some(*scope_id) {
                    let f = Function::boxed(
                        SPAN,
                        FunctionType::FunctionExpression,
                        func.id.clone_in_with_semantic_ids(ast.allocator()),
                        func.generator,
                        func.r#async,
                        false,
                        None::<ArenaBox<TSTypeParameterDeclaration>>,
                        None::<ArenaBox<TSThisParameter>>,
                        func.params.clone_in_with_semantic_ids(ast.allocator()),
                        None::<ArenaBox<TSTypeAnnotation>>,
                        func.body.clone_in_with_semantic_ids(ast.allocator()),
                        ast,
                    );
                    *found = Some(Expression::FunctionExpression(f));
                    return;
                }
            }
        }
        oxc_ast_visit::walk_mut::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &mut ArrowFunctionExpression<'a>) {
        let ast = self.ast;
        match &mut self.mode {
            OxcVisitMode::ReplaceFns { replacements, remaining } => {
                if *remaining == 0 {
                    return;
                }
                if let Some(scope_id) = arrow.scope_id.get()
                    && let Some(&codegen) = replacements.get(&scope_id)
                {
                    ox_replace_arrow(ast, arrow, codegen);
                    *remaining -= 1;
                    return;
                }
            }
            OxcVisitMode::ReplaceWithGated { .. } => {}
            OxcVisitMode::FindOriginalFn { scope_id, found } => {
                if found.is_some() {
                    return;
                }
                if arrow.scope_id.get() == Some(*scope_id) {
                    *found = Some(Expression::ArrowFunctionExpression(ArenaBox::new_in(
                        arrow.clone_in_with_semantic_ids(ast.allocator()),
                        ast,
                    )));
                    return;
                }
            }
        }
        oxc_ast_visit::walk_mut::walk_arrow_function_expression(self, arrow);
    }

    fn visit_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>) {
        if !matches!(self.mode, OxcVisitMode::ReplaceWithGated { .. }) {
            oxc_ast_visit::walk_mut::walk_statements(self, stmts);
            return;
        }
        let mut i = 0;
        while i < stmts.len() {
            if matches!(self.mode, OxcVisitMode::ReplaceWithGated { done: true, .. }) {
                break;
            }
            if self.try_replace_gated_statement(&mut stmts[i]) {
                break;
            }
            self.visit_statement(&mut stmts[i]);
            i += 1;
        }

        // Insert `export default Name;` right after the replaced declaration.
        if let OxcVisitMode::ReplaceWithGated { export_default_name, .. } = &mut self.mode
            && let Some(name) = export_default_name.take()
        {
            let ident = Expression::new_identifier(SPAN, name, self.ast);
            let export = Statement::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(
                SPAN,
                ExportDefaultDeclarationKind::from(ident),
                self.ast,
            ));
            // Find the const decl we just inserted (it has name `name`); insert after.
            let pos = stmts.iter().position(|s| {
                matches!(s, Statement::VariableDeclaration(d)
                    if d.declarations.first().is_some_and(|decl| matches!(&decl.id,
                        BindingPattern::BindingIdentifier(b) if b.name == name)))
            });
            if let Some(pos) = pos {
                stmts.insert(pos + 1, export);
            } else {
                stmts.push(export);
            }
        }
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if let OxcVisitMode::ReplaceWithGated { scope_id, gating_expression, done, .. } =
            &mut self.mode
        {
            if *done {
                return;
            }
            let matched = match expr {
                Expression::FunctionExpression(f) => f.scope_id.get() == Some(*scope_id),
                Expression::ArrowFunctionExpression(f) => f.scope_id.get() == Some(*scope_id),
                _ => false,
            };
            if matched {
                *expr = gating_expression.clone_in_with_semantic_ids(self.ast.allocator());
                *done = true;
                return;
            }
        }
        oxc_ast_visit::walk_mut::walk_expression(self, expr);
    }
}

/// Allocate a `&'a str` in the arena (satisfies the builders' `Into<Ident>` /
/// `IntoIn` slots).
fn ox_atom<'a>(ast: &AstBuilder<'a>, s: &str) -> &'a str {
    oxc_allocator::StringBuilder::from_str_in(s, ast.allocator()).into_str()
}

/// Build `<callee_name>()` as an oxc call expression.
fn ox_gating_call<'a>(ast: &AstBuilder<'a>, callee_name: &str) -> Expression<'a> {
    Expression::new_call_expression(
        SPAN,
        Expression::new_identifier(SPAN, ox_atom(ast, callee_name), ast),
        None::<ArenaBox<TSTypeParameterInstantiation>>,
        [],
        false,
        ast,
    )
}

/// Apply the conditional gating pattern to the oxc program. Mirrors
/// `apply_gated_function_conditional`.
fn ox_apply_gated_conditional<'a>(
    ast: &AstBuilder<'a>,
    program: &mut Program<'a>,
    replacement: &OxcReplacement<'a>,
    gating_config: &GatingConfig,
    context: &mut ProgramContext,
) {
    let scope_id = replacement.fn_scope_id;

    let gating_import = context.add_import_specifier(
        &gating_config.source,
        &gating_config.import_specifier_name,
        None,
    );

    // Clone the original function (matched by its scope) as the fallback
    // expression BEFORE replacing it.
    let original_expr = ox_clone_original_fn_as_expression(ast, program, scope_id);
    let original_expr = match original_expr {
        Some(e) => e,
        None => return,
    };

    let compiled_expr =
        ox_build_compiled_expression(ast, &replacement.codegen_fn, replacement.original_kind);

    // gating() ? compiled : original
    let gating_expression = Expression::new_conditional_expression(
        SPAN,
        ox_gating_call(ast, &gating_import.name),
        compiled_expr,
        original_expr,
        ast,
    );

    let mut visitor = OxcVisitor {
        ast,
        mode: OxcVisitMode::ReplaceWithGated {
            scope_id,
            gating_expression: &gating_expression,
            export_default_name: None,
            done: false,
        },
    };
    oxc_ast_visit::VisitMut::visit_program(&mut visitor, program);
}

/// Clone the original function with scope `scope_id` as an `Expression`
/// (FunctionDeclaration becomes a FunctionExpression). Mirrors
/// `clone_original_fn_as_expression`.
///
/// Takes `&mut Program` only because the walk is shared with the mutating
/// [`OxcVisitor`] modes; [`OxcVisitMode::FindOriginalFn`] mutates nothing.
fn ox_clone_original_fn_as_expression<'a>(
    ast: &AstBuilder<'a>,
    program: &mut Program<'a>,
    scope_id: ScopeId,
) -> Option<Expression<'a>> {
    let mut finder =
        OxcVisitor { ast, mode: OxcVisitMode::FindOriginalFn { scope_id, found: None } };
    oxc_ast_visit::VisitMut::visit_program(&mut finder, program);
    let OxcVisitMode::FindOriginalFn { found, .. } = finder.mode else { unreachable!() };
    found.map(|e| e.clone_in_with_semantic_ids(ast.allocator()))
}

/// Substitute every compiled oxc function into the program in place and add the
/// required imports.
///
/// The replacement walks match original functions by their `scope_id` cells
/// (populated by the caller's semantic build); codegen-built nodes carry no
/// cells, so they can never be spuriously matched.
fn ox_transform_program<'a>(
    ast: &AstBuilder<'a>,
    program: &mut Program<'a>,
    replacements: &[OxcReplacement<'a>],
    context: &mut ProgramContext,
) {
    // Outlined function declarations are placed differently depending on the
    // original function's syntactic kind, mirroring `insertNewOutlinedFunctionNode`
    // in TS `Program.ts`:
    //   - FunctionDeclaration originals: inserted as a sibling immediately after the
    //     original function (Babel `insertAfter`).
    //   - (Arrow)FunctionExpression originals: appended at the end of the program
    //     body (Babel `pushContainer('body', ...)`), since inserting as a sibling
    //     would corrupt the parent expression.
    let mut appended_outlined_decls: Vec<Statement<'a>> = Vec::new();

    // Substitute every non-gated compiled function into its original in a single
    // program walk, each matched by its `scope_id` cell. In-place edits do not
    // change `program.body`, so the outlined-decl insertion and gating below
    // (which do restructure it) are unaffected by running first. Gated functions
    // are replaced by a conditional in the loop below, not edited in place.
    let replace_map: FxHashMap<ScopeId, &CodegenFunction<'a>> = replacements
        .iter()
        .filter(|r| r.gating.is_none())
        .map(|r| (r.fn_scope_id, &r.codegen_fn))
        .collect();
    if !replace_map.is_empty() {
        let mode =
            OxcVisitMode::ReplaceFns { remaining: replace_map.len(), replacements: replace_map };
        let mut visitor = OxcVisitor { ast, mode };
        oxc_ast_visit::VisitMut::visit_program(&mut visitor, program);
    }

    for replacement in replacements {
        let mut sibling_outlined_decls: Vec<Statement<'a>> = Vec::new();
        let insert_as_sibling = replacement.original_kind == OriginalFnKind::FunctionDeclaration;
        for outlined in &replacement.codegen_fn.outlined {
            let func = ox_build_function(ast, &outlined.func, FunctionType::FunctionDeclaration);
            let stmt = Statement::FunctionDeclaration(func);
            if insert_as_sibling {
                sibling_outlined_decls.push(stmt);
            } else {
                appended_outlined_decls.push(stmt);
            }
        }

        if let Some(ref gating_config) = replacement.gating {
            ox_apply_gated_conditional(ast, program, replacement, gating_config, context);
        }

        if !sibling_outlined_decls.is_empty() {
            ox_insert_outlined_after(program, replacement.fn_scope_id, sibling_outlined_decls);
        }
    }

    // Append outlined function declarations (from expression-parented originals) at
    // the top level.
    program.body.extend(appended_outlined_decls);

    // Register the memo cache import; codegen emitted its pre-reserved local name.
    if replacements.iter().any(|r| r.codegen_fn.memo_slots_used > 0) {
        context.add_memo_cache_import();
    }

    ox_add_imports_to_program(ast, program, context);
}

/// Insert outlined function declarations immediately after the top-level statement
/// that declares the function with scope `scope_id`. Mirrors Babel's
/// `originalFn.insertAfter(...)` for `FunctionDeclaration` originals. The statement
/// may be a bare `FunctionDeclaration` or one wrapped in an `export`.
fn ox_insert_outlined_after<'a>(
    program: &mut Program<'a>,
    scope_id: ScopeId,
    outlined_decls: Vec<Statement<'a>>,
) {
    let matches = |stmt: &Statement<'a>| -> bool {
        let is_target = |f: &Function<'a>| -> bool { f.scope_id.get() == Some(scope_id) };
        match stmt {
            Statement::FunctionDeclaration(f) => is_target(f),
            Statement::ExportNamedDeclaration(e) => {
                matches!(&e.declaration, Some(Declaration::FunctionDeclaration(f)) if is_target(f))
            }
            Statement::ExportDefaultDeclaration(e) => {
                matches!(&e.declaration, ExportDefaultDeclarationKind::FunctionDeclaration(f) if is_target(f))
            }
            _ => false,
        }
    };

    let index = program.body.iter().position(matches);
    match index {
        Some(idx) => {
            // Babel inserts each outlined function via `originalFn.insertAfter(...)`,
            // anchored at the same original node, so repeated insertions reverse the
            // emitted order. Insert each at `idx + 1` to reproduce that.
            for stmt in outlined_decls {
                program.body.insert(idx + 1, stmt);
            }
        }
        None => {
            // Function is nested (not a direct program-body statement); fall back to
            // appending at the top level.
            program.body.extend(outlined_decls);
        }
    }
}

/// Insert import declarations into the oxc program. Mirrors `add_imports_to_program`
/// in imports.rs but builds oxc nodes. Handles ESM imports, CommonJS require, and
/// merging into an existing non-namespaced import of the same module.
fn ox_add_imports_to_program<'a>(
    ast: &AstBuilder<'a>,
    program: &mut Program<'a>,
    context: &ProgramContext,
) {
    if !context.has_pending_imports() {
        return;
    }
    let imports = context.imports();

    // Existing non-namespaced value imports, by module name.
    let mut existing_import_indices: FxHashMap<&str, usize> = FxHashMap::default();
    for (idx, stmt) in program.body.iter().enumerate() {
        if let Statement::ImportDeclaration(import) = stmt
            && ox_is_non_namespaced_import(import)
        {
            existing_import_indices.entry(import.source.value.as_str()).or_insert(idx);
        }
    }

    let mut sorted_modules: Vec<_> = imports.iter().collect();
    sorted_modules.sort_unstable_by_key(|(a, _)| a.cow_to_lowercase());

    let is_module = matches!(program.source_type.module_kind(), oxc_span::ModuleKind::Module);

    let mut new_stmts: Vec<Statement<'a>> = Vec::new();

    for (module_name, imports_map) in sorted_modules {
        let mut sorted_imports: Vec<_> = imports_map.values().collect();
        sorted_imports.sort_unstable_by(|a, b| a.imported.as_str().cmp(b.imported.as_str()));

        if let Some(&idx) = existing_import_indices.get(module_name.as_str()) {
            // Merge into the existing import declaration.
            if let Statement::ImportDeclaration(import) = &mut program.body[idx] {
                let specifiers = import.specifiers.get_or_insert_with(|| ArenaVec::new_in(ast));
                for spec in &sorted_imports {
                    specifiers.push(ox_make_import_specifier(ast, spec));
                }
            }
        } else if is_module {
            // ESM: import { imported as local, ... } from 'module'
            let mut specifiers = ArenaVec::new_in(ast);
            for spec in &sorted_imports {
                specifiers.push(ox_make_import_specifier(ast, spec));
            }
            let source = StringLiteral::new(SPAN, ox_atom(ast, module_name), None, ast);
            let import = ImportDeclaration::boxed(
                SPAN,
                Some(specifiers),
                source,
                None,
                None::<ArenaBox<WithClause>>,
                ImportOrExportKind::Value,
                ast,
            );
            new_stmts.push(Statement::ImportDeclaration(import));
        } else {
            // CommonJS: const { imported: local, ... } = require('module')
            let mut props = ArenaVec::new_in(ast);
            for spec in &sorted_imports {
                let key =
                    PropertyKey::new_static_identifier(SPAN, ox_atom(ast, &spec.imported), ast);
                let value =
                    BindingPattern::new_binding_identifier(SPAN, ox_atom(ast, &spec.name), ast);
                props.push(BindingProperty::new(SPAN, key, value, false, false, ast));
            }
            let object_pattern = BindingPattern::new_object_pattern(
                SPAN,
                props,
                None::<ArenaBox<BindingRestElement>>,
                ast,
            );
            let require_call = Expression::new_call_expression(
                SPAN,
                Expression::new_identifier(SPAN, "require", ast),
                None::<ArenaBox<TSTypeParameterInstantiation>>,
                ArenaVec::from_value_in(
                    Argument::from(Expression::new_string_literal(
                        SPAN,
                        ox_atom(ast, module_name),
                        None,
                        ast,
                    )),
                    ast,
                ),
                false,
                ast,
            );
            let declarator = VariableDeclarator::new(
                SPAN,
                VariableDeclarationKind::Const,
                object_pattern,
                None::<ArenaBox<TSTypeAnnotation>>,
                Some(require_call),
                false,
                ast,
            );
            let decl = VariableDeclaration::boxed(
                SPAN,
                VariableDeclarationKind::Const,
                ArenaVec::from_value_in(declarator, ast),
                false,
                ast,
            );
            new_stmts.push(Statement::VariableDeclaration(decl));
        }
    }

    if !new_stmts.is_empty() {
        let old_body = std::mem::replace(&mut program.body, ArenaVec::new_in(ast));
        program.body.extend(new_stmts);
        program.body.extend(old_body);
    }
}

/// Build an oxc named import specifier `imported as local`. Mirrors `make_import_specifier`.
fn ox_make_import_specifier<'a>(
    ast: &AstBuilder<'a>,
    spec: &super::imports::NonLocalImportSpecifier,
) -> ImportDeclarationSpecifier<'a> {
    let imported = ModuleExportName::IdentifierName(IdentifierName::new(
        SPAN,
        ox_atom(ast, &spec.imported),
        ast,
    ));
    let local = BindingIdentifier::new(SPAN, ox_atom(ast, &spec.name), ast);
    ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier::boxed(
        SPAN,
        imported,
        local,
        ImportOrExportKind::Value,
        ast,
    ))
}

/// Whether an import declaration is a non-namespaced value import. Mirrors
/// `is_non_namespaced_import`.
fn ox_is_non_namespaced_import(import: &ImportDeclaration) -> bool {
    if !matches!(import.import_kind, ImportOrExportKind::Value) {
        return false;
    }
    match &import.specifiers {
        None => true,
        Some(specifiers) => {
            specifiers.iter().all(|s| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)))
        }
    }
}

/// Main entry point for the React Compiler.
///
/// Receives the arena allocator, semantic model, the full program AST, and
/// resolved options. Returns a CompileResult carrying the compiled output to
/// apply — `None` when nothing changed — along with any diagnostics.
///
/// This function implements the logic from the TS entrypoint (Program.ts):
/// - findProgramSuppressions: find eslint/flow suppression comments
/// - findFunctionsToCompile: traverse program to find components and hooks
/// - processFn: per-function compilation with directive and suppression handling
/// - applyCompiledFunctions: replace original functions with compiled versions
pub fn compile_program<'a>(
    allocator: &'a Allocator,
    semantic: &Semantic<'_>,
    program: &Program<'a>,
    options: PluginOptions,
) -> CompileResult<'a> {
    // Find all functions to compile. An empty queue means no work, so return
    // before all the setup below, which only compilation needs. The pre-check
    // decides emptiness from semantic data without walking the AST.
    if !may_have_functions_to_compile(semantic, &options) {
        return CompileResult::Success { output: None, diagnostics: Diagnostics::new() };
    }
    let queue = find_functions_to_compile(program, &options);
    if queue.is_empty() {
        return CompileResult::Success { output: None, diagnostics: Diagnostics::new() };
    }

    // Compute output mode once, up front
    let output_mode = CompilerOutputMode::from_opts(&options);

    let eslint_rules: Option<Vec<String>> =
        if options.environment.validate_exhaustive_memoization_dependencies
            && options.environment.validate_hooks_usage
        {
            // Don't check for ESLint suppressions if both validations are enabled
            None
        } else {
            Some(options.eslint_suppression_rules.clone().unwrap_or_else(|| {
                DEFAULT_ESLINT_SUPPRESSIONS.iter().map(|s| s.to_string()).collect()
            }))
        };

    // Find program-level suppressions from comments
    let suppressions = find_program_suppressions(
        &program.comments,
        program.source_text,
        eslint_rules.as_deref(),
        options.flow_suppressions,
    );

    // Check for module-scope opt-out directive
    let has_module_scope_opt_out = find_directive_disabling_memoization(
        program.directives.iter().map(|d| d.expression.value.as_str()),
        options.custom_opt_out_directives.as_deref(),
    )
    .is_some();

    // Create program context
    let mut context = ProgramContext::new(
        allocator,
        program.source_text,
        options.clone(),
        suppressions,
        has_module_scope_opt_out,
    );

    // The codegen back-end builds oxc nodes directly via this `AstBuilder`; `scope`
    // is a read-through view over `Semantic` for binding/reference lookups.
    let ast = AstBuilder::new(allocator);
    let scope = ScopeResolver::new(semantic, allocator);

    // Initialize known referenced names from scope bindings for UID collision detection
    context.init_from_scope(&scope);

    // Pre-register instrumentation imports to get stable local names.
    // These are needed before compilation so codegen can use the correct names.
    let (instrument_fn_name, instrument_gating_name) =
        if let Some(ref instrument_config) = options.environment.enable_emit_instrument_forget {
            let fn_spec = context.add_import_specifier(
                &instrument_config.fn_.source,
                &instrument_config.fn_.import_specifier_name,
                None,
            );
            let gating_name = instrument_config.gating.as_ref().map(|g| {
                let spec = context.add_import_specifier(&g.source, &g.import_specifier_name, None);
                spec.name
            });
            (Some(fn_spec.name), gating_name)
        } else {
            (None, None)
        };

    let hook_guard_name =
        options.environment.enable_emit_hook_guards.as_ref().map(|hook_guard_config| {
            let spec = context.add_import_specifier(
                &hook_guard_config.source,
                &hook_guard_config.import_specifier_name,
                None,
            );
            spec.name
        });

    // Store pre-resolved names on context for pipeline access
    context.instrument_fn_name = instrument_fn_name;
    context.instrument_gating_name = instrument_gating_name;
    context.hook_guard_name = hook_guard_name;

    // Reserve the memo-cache import's local name (`_c`, `_c2`, ...) up front so codegen
    // can emit it directly; the import itself is registered in `ox_transform_program`,
    // and only when an applied function uses memo slots.
    context.reserve_memo_cache_name(&scope);

    // Process each function and collect compiled results
    let mut compiled_fns: Vec<CompiledFunction<'_, '_, '_, '_>> = Vec::new();

    for source in &queue {
        match process_fn(&ast, source, &scope, output_mode, &options.environment, &mut context) {
            Ok(Some(codegen_fn)) => {
                compiled_fns.push(CompiledFunction { kind: source.kind, source, codegen_fn });
            }
            Ok(None) => {
                // Function was skipped or lint-only
            }
            Err(fatal_result) => {
                return fatal_result;
            }
        }
    }

    // TS invariant: if there's a module scope opt-out, no functions should have been compiled
    if has_module_scope_opt_out {
        if !compiled_fns.is_empty() {
            let err =
                Diagnostics::from(ErrorCategory::Invariant.diagnostic(
                    "Unexpected compiled functions when module scope opt-out is present",
                ));
            handle_error(&err, None, context.opts.panic_threshold, &mut context.diagnostics);
        }
        return CompileResult::Success { output: None, diagnostics: context.diagnostics };
    }

    // Convert compiled functions to owned oxc replacements (dropping the borrows of
    // `file.program`), resolving per-function gating. Dynamic gating from directives
    // (`use memo if(identifier)`) takes precedence over plugin-level gating; gating
    // only applies to 'original' (not 'outlined') functions. Mirrors the Babel path.
    let function_gating_config = options.gating.clone();
    let replacements: Vec<OxcReplacement<'a>> = compiled_fns
        .into_iter()
        .map(|cf| {
            let gating = if cf.kind == CompileSourceKind::Original {
                let dynamic_gating =
                    find_directives_dynamic_gating(&cf.source.body_directives, &options)
                        .ok()
                        .flatten()
                        .map(|r| r.gating);
                dynamic_gating.or_else(|| function_gating_config.clone())
            } else {
                None
            };
            OxcReplacement {
                fn_scope_id: cf.source.fn_scope_id,
                original_kind: cf.source.original_kind,
                codegen_fn: cf.codegen_fn,
                gating,
            }
        })
        .collect();

    // Drop the discovery results (and their borrows of `file.program`).
    drop(queue);

    // `output` is `None` when nothing was compiled — always so in lint mode, which
    // applies nothing. Substituting each compiled function for its original
    // (matched by the function's scope) is also what inserts the memo-cache /
    // gating imports, so (matching TS `addImportsToProgram`) they're added only
    // when there are replacements.
    let diagnostics = std::mem::take(&mut context.diagnostics);
    let output = if replacements.is_empty() {
        None
    } else {
        Some(Box::new(CompileOutput { replacements, context }))
    };
    CompileResult::Success { output, diagnostics }
}
