// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Main entrypoint for the React Compiler.
//!
//! This module is a port of Program.ts from the TypeScript compiler. It orchestrates
//! the compilation of a program by:
//! 1. Checking if compilation should be skipped
//! 2. Validating restricted imports
//! 3. Finding program-level suppressions
//! 4. Discovering functions to compile (components, hooks)
//! 5. Processing each function through the compilation pipeline
//! 6. Applying compiled functions back to the AST

use oxc_ast::ast as oxc;
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::CompilerErrorOrDiagnostic;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_hir::ReactFunctionType;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
use crate::react_compiler_lowering::FunctionNode;
use crate::scope::ScopeId;
use crate::scope::ScopeInfo;
use oxc_allocator::GetAllocator;

use super::compile_result::BindingRenameInfo;
use super::compile_result::CodegenFunction;
use super::compile_result::CompileResult;
use super::compile_result::DebugLogEntry;
use super::compile_result::OrderedLogItem;
use super::imports::ProgramContext;
use super::imports::get_react_compiler_runtime_module;
use super::imports::validate_restricted_imports;
use super::pipeline;
use super::plugin_options::CompilerOutputMode;
use super::plugin_options::GatingConfig;
use super::plugin_options::PluginOptions;
use super::suppression::SuppressionRange;
use super::suppression::filter_suppressions_that_affect_function;
use super::suppression::find_program_suppressions;
use super::suppression::suppressions_to_compiler_error;

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
#[allow(dead_code)]
struct CompileSource<'a> {
    kind: CompileSourceKind,
    original_kind: OriginalFnKind,
    fn_name: Option<String>,
    /// Byte span of the discovered function, used as the fallback labeled span in
    /// compile-error diagnostics.
    fn_ast_loc: Option<Span>,
    fn_start: Option<u32>,
    fn_end: Option<u32>,
    fn_node_id: Option<u32>,
    fn_type: ReactFunctionType,
    /// The discovered oxc function node, handed straight to lowering.
    fn_node: FunctionNode<'a>,
    /// Directive values from the function body (for opt-in/opt-out checks)
    body_directives: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompileSourceKind {
    Original,
    #[allow(dead_code)]
    Outlined,
}

// -----------------------------------------------------------------------
// Directive helpers
// -----------------------------------------------------------------------

/// Check if any opt-in directive is present in the given directives.
/// Returns the first matching directive value, or None.
///
/// Also checks for dynamic gating directives (`use memo if(...)`)
fn try_find_directive_enabling_memoization<'a>(
    directives: &'a [String],
    opts: &PluginOptions,
) -> Result<Option<&'a String>, CompilerError> {
    // Check standard opt-in directives
    let opt_in = directives.iter().find(|d| OPT_IN_DIRECTIVES.contains(&d.as_str()));
    if let Some(directive) = opt_in {
        return Ok(Some(directive));
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
    directives: &'a [String],
    opts: &PluginOptions,
) -> Option<&'a String> {
    if let Some(ref custom_directives) = opts.custom_opt_out_directives {
        directives.iter().find(|d| custom_directives.contains(d))
    } else {
        directives.iter().find(|d| OPT_OUT_DIRECTIVES.contains(&d.as_str()))
    }
}

/// Result of a dynamic gating directive parse.
struct DynamicGatingResult<'a> {
    #[allow(dead_code)]
    directive: &'a String,
    gating: GatingConfig,
}

/// Check for dynamic gating directives like `use memo if(identifier)`.
/// Returns the directive and gating config if found, or an error if malformed.
fn find_directives_dynamic_gating<'a>(
    directives: &'a [String],
    opts: &PluginOptions,
) -> Result<Option<DynamicGatingResult<'a>>, CompilerError> {
    let dynamic_gating = match &opts.dynamic_gating {
        Some(dg) => dg,
        None => return Ok(None),
    };

    let mut errors: Vec<CompilerErrorDetail> = Vec::new();
    let mut matches: Vec<(&'a String, String)> = Vec::new();

    for directive in directives {
        if let Some(ident) = parse_dynamic_gating_directive(directive) {
            if is_valid_identifier(ident) {
                matches.push((directive, ident.to_string()));
            } else {
                let detail = CompilerErrorDetail::new(
                    ErrorCategory::Gating,
                    "Dynamic gating directive is not a valid JavaScript identifier",
                )
                .with_description(format!("Found '{directive}'"));
                errors.push(detail);
            }
        }
    }

    if !errors.is_empty() {
        let mut err = CompilerError::new();
        for e in errors {
            err.push_error_detail(e);
        }
        return Err(err);
    }

    if matches.len() > 1 {
        let names: Vec<String> = matches.iter().map(|(d, _)| (*d).clone()).collect();
        let mut err = CompilerError::new();
        let detail = CompilerErrorDetail::new(
            ErrorCategory::Gating,
            "Multiple dynamic gating directives found",
        )
        .with_description(format!("Expected a single directive but found [{}]", names.join(", ")));
        err.push_error_detail(detail);
        return Err(err);
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

/// Simple check for valid JavaScript identifier (alphanumeric + underscore + $, starting with letter/$/_ )
/// Also rejects reserved words like `true`, `false`, `null`, etc.
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    if !chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
        return false;
    }
    // Check for reserved words (matching Babel's t.isValidIdentifier)
    !matches!(
        s,
        "break"
            | "case"
            | "catch"
            | "continue"
            | "debugger"
            | "default"
            | "do"
            | "else"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "class"
            | "const"
            | "enum"
            | "export"
            | "extends"
            | "import"
            | "super"
            | "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
            | "null"
            | "true"
            | "false"
            | "delete"
    )
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
        && bytes.get(3).map_or(false, |c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Check if a name looks like a React component (starts with uppercase letter).
fn is_component_name(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_ascii_uppercase())
}

/// Check if an expression is a hook call (identifier with hook name, or
/// member expression `PascalCase.useHook`).
fn expr_is_hook(expr: &oxc::Expression) -> bool {
    match expr {
        oxc::Expression::Identifier(id) => is_hook_name(&id.name),
        oxc::Expression::StaticMemberExpression(member) => {
            // Property must be a hook name
            if !is_hook_name(&member.property.name) {
                return false;
            }
            // Object must be a PascalCase identifier
            if let oxc::Expression::Identifier(obj) = &member.object {
                obj.name.chars().next().map_or(false, |c| c.is_ascii_uppercase())
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
fn expr_contains_optional(expr: &oxc::Expression) -> bool {
    match expr {
        oxc::Expression::CallExpression(c) => c.optional || expr_contains_optional(&c.callee),
        oxc::Expression::StaticMemberExpression(m) => {
            m.optional || expr_contains_optional(&m.object)
        }
        oxc::Expression::ComputedMemberExpression(m) => {
            m.optional || expr_contains_optional(&m.object)
        }
        oxc::Expression::PrivateFieldExpression(p) => {
            p.optional || expr_contains_optional(&p.object)
        }
        _ => false,
    }
}

/// Whether a `CallExpression` is a "regular" (non-optional) call. In Babel such a
/// call was a `CallExpression` (hook-checkable); optional / post-`?.` calls were
/// `OptionalCallExpression` (never treated as hooks). Matches the Babel-bridge
/// chain flattening exactly.
fn is_regular_call(call: &oxc::CallExpression) -> bool {
    !call.optional && !expr_contains_optional(&call.callee)
}

/// Check if an expression is a React API call (e.g., `forwardRef` or `React.forwardRef`).
#[allow(dead_code)]
fn is_react_api(expr: &oxc::Expression, function_name: &str) -> bool {
    match expr {
        oxc::Expression::Identifier(id) => id.name == function_name,
        oxc::Expression::StaticMemberExpression(member) => {
            if let oxc::Expression::Identifier(obj) = &member.object {
                if obj.name == "React" {
                    return member.property.name == function_name;
                }
            }
            false
        }
        _ => false,
    }
}

/// Get the inferred function name from a function's context.
///
/// For FunctionDeclaration: uses the `id` field.
/// For FunctionExpression/ArrowFunctionExpression: infers from parent context
/// (VariableDeclarator, etc.) which is passed explicitly since we don't have Babel paths.
fn get_function_name_from_id(id: Option<&oxc::BindingIdentifier>) -> Option<String> {
    id.map(|id| id.name.to_string())
}

// -----------------------------------------------------------------------
// AST traversal helpers
// -----------------------------------------------------------------------

/// Check if an expression is a "non-node" return value (indicating the function
/// is not a React component). This matches the TS `isNonNode` function.
fn is_non_node(expr: &oxc::Expression) -> bool {
    matches!(
        expr,
        oxc::Expression::ObjectExpression(_)
            | oxc::Expression::ArrowFunctionExpression(_)
            | oxc::Expression::FunctionExpression(_)
            | oxc::Expression::BigIntLiteral(_)
            | oxc::Expression::ClassExpression(_)
            | oxc::Expression::NewExpression(_)
    )
}

/// Recursively check if a function body returns a non-React-node value.
/// Walks all return statements in the function (not in nested functions).
/// The last return statement visited (in DFS order) determines the result,
/// rather than short-circuiting on the first non-node return.
fn returns_non_node_in_stmts(stmts: &[oxc::Statement]) -> bool {
    let mut result = false;
    for stmt in stmts {
        returns_non_node_in_stmt(stmt, &mut result);
    }
    result
}

fn returns_non_node_in_stmt(stmt: &oxc::Statement, result: &mut bool) {
    match stmt {
        oxc::Statement::ReturnStatement(ret) => {
            *result = match &ret.argument {
                Some(arg) => is_non_node(arg),
                None => true, // bare `return;` with no argument is a non-node value
            };
        }
        oxc::Statement::BlockStatement(block) => {
            for s in &block.body {
                returns_non_node_in_stmt(s, result);
            }
        }
        oxc::Statement::IfStatement(if_stmt) => {
            returns_non_node_in_stmt(&if_stmt.consequent, result);
            if let Some(ref alt) = if_stmt.alternate {
                returns_non_node_in_stmt(alt, result);
            }
        }
        oxc::Statement::ForStatement(for_stmt) => returns_non_node_in_stmt(&for_stmt.body, result),
        oxc::Statement::WhileStatement(while_stmt) => {
            returns_non_node_in_stmt(&while_stmt.body, result)
        }
        oxc::Statement::DoWhileStatement(do_while) => {
            returns_non_node_in_stmt(&do_while.body, result)
        }
        oxc::Statement::ForInStatement(for_in) => returns_non_node_in_stmt(&for_in.body, result),
        oxc::Statement::ForOfStatement(for_of) => returns_non_node_in_stmt(&for_of.body, result),
        oxc::Statement::SwitchStatement(switch) => {
            for case in &switch.cases {
                for s in &case.consequent {
                    returns_non_node_in_stmt(s, result);
                }
            }
        }
        oxc::Statement::TryStatement(try_stmt) => {
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
        oxc::Statement::LabeledStatement(labeled) => {
            returns_non_node_in_stmt(&labeled.body, result)
        }
        oxc::Statement::WithStatement(with) => returns_non_node_in_stmt(&with.body, result),
        // Skip nested function/class declarations -- they have their own returns.
        // All other statements (incl. TS-only declarations) are opaque here.
        _ => {}
    }
}

/// Check if a function returns non-node values.
/// For arrow functions with expression body, checks the expression directly.
/// For block bodies, walks the statements.
fn returns_non_node_fn(params: &oxc::FormalParameters, body: &FunctionBody) -> bool {
    let _ = params;
    match body {
        FunctionBody::Block(block) => returns_non_node_in_stmts(&block.statements),
        FunctionBody::Expression(expr) => is_non_node(expr),
    }
}

/// Check if a function body calls hooks or creates JSX.
/// Traverses the function body (not nested functions) looking for:
/// - CallExpression where callee is a hook
/// - JSXElement or JSXFragment
fn calls_hooks_or_creates_jsx_in_stmts(stmts: &[oxc::Statement]) -> bool {
    for stmt in stmts {
        if calls_hooks_or_creates_jsx_in_stmt(stmt) {
            return true;
        }
    }
    false
}

fn calls_hooks_or_creates_jsx_in_stmt(stmt: &oxc::Statement) -> bool {
    match stmt {
        oxc::Statement::ExpressionStatement(expr_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&expr_stmt.expression)
        }
        oxc::Statement::ReturnStatement(ret) => {
            if let Some(ref arg) = ret.argument {
                calls_hooks_or_creates_jsx_in_expr(arg)
            } else {
                false
            }
        }
        oxc::Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(ref init) = decl.init {
                    if calls_hooks_or_creates_jsx_in_expr(init) {
                        return true;
                    }
                }
            }
            false
        }
        oxc::Statement::BlockStatement(block) => calls_hooks_or_creates_jsx_in_stmts(&block.body),
        oxc::Statement::IfStatement(if_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&if_stmt.test)
                || calls_hooks_or_creates_jsx_in_stmt(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .map_or(false, |alt| calls_hooks_or_creates_jsx_in_stmt(alt))
        }
        oxc::Statement::ForStatement(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                match init {
                    oxc::ForStatementInit::VariableDeclaration(var_decl) => {
                        for decl in &var_decl.declarations {
                            if let Some(ref init) = decl.init {
                                if calls_hooks_or_creates_jsx_in_expr(init) {
                                    return true;
                                }
                            }
                        }
                    }
                    // An expression `ForStatementInit` is an `Expression` (the
                    // enum inherits the Expression variants).
                    expr => {
                        if let Some(expr) = expr.as_expression() {
                            if calls_hooks_or_creates_jsx_in_expr(expr) {
                                return true;
                            }
                        }
                    }
                }
            }
            if let Some(ref test) = for_stmt.test {
                if calls_hooks_or_creates_jsx_in_expr(test) {
                    return true;
                }
            }
            if let Some(ref update) = for_stmt.update {
                if calls_hooks_or_creates_jsx_in_expr(update) {
                    return true;
                }
            }
            calls_hooks_or_creates_jsx_in_stmt(&for_stmt.body)
        }
        oxc::Statement::WhileStatement(while_stmt) => {
            calls_hooks_or_creates_jsx_in_expr(&while_stmt.test)
                || calls_hooks_or_creates_jsx_in_stmt(&while_stmt.body)
        }
        oxc::Statement::DoWhileStatement(do_while) => {
            calls_hooks_or_creates_jsx_in_stmt(&do_while.body)
                || calls_hooks_or_creates_jsx_in_expr(&do_while.test)
        }
        oxc::Statement::ForInStatement(for_in) => {
            calls_hooks_or_creates_jsx_in_expr(&for_in.right)
                || calls_hooks_or_creates_jsx_in_stmt(&for_in.body)
        }
        oxc::Statement::ForOfStatement(for_of) => {
            calls_hooks_or_creates_jsx_in_expr(&for_of.right)
                || calls_hooks_or_creates_jsx_in_stmt(&for_of.body)
        }
        oxc::Statement::SwitchStatement(switch) => {
            if calls_hooks_or_creates_jsx_in_expr(&switch.discriminant) {
                return true;
            }
            for case in &switch.cases {
                if let Some(ref test) = case.test {
                    if calls_hooks_or_creates_jsx_in_expr(test) {
                        return true;
                    }
                }
                if calls_hooks_or_creates_jsx_in_stmts(&case.consequent) {
                    return true;
                }
            }
            false
        }
        oxc::Statement::ThrowStatement(throw) => {
            calls_hooks_or_creates_jsx_in_expr(&throw.argument)
        }
        oxc::Statement::TryStatement(try_stmt) => {
            if calls_hooks_or_creates_jsx_in_stmts(&try_stmt.block.body) {
                return true;
            }
            if let Some(ref handler) = try_stmt.handler {
                if calls_hooks_or_creates_jsx_in_stmts(&handler.body.body) {
                    return true;
                }
            }
            if let Some(ref finalizer) = try_stmt.finalizer {
                if calls_hooks_or_creates_jsx_in_stmts(&finalizer.body) {
                    return true;
                }
            }
            false
        }
        oxc::Statement::LabeledStatement(labeled) => {
            calls_hooks_or_creates_jsx_in_stmt(&labeled.body)
        }
        oxc::Statement::WithStatement(with) => {
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

fn calls_hooks_or_creates_jsx_in_expr(expr: &oxc::Expression) -> bool {
    /// Whether an argument expression is a nested function (skipped by the walk).
    fn is_nested_fn(arg: &oxc::Expression) -> bool {
        matches!(
            arg,
            oxc::Expression::ArrowFunctionExpression(_) | oxc::Expression::FunctionExpression(_)
        )
    }

    match expr {
        // JSX creates
        oxc::Expression::JSXElement(_) | oxc::Expression::JSXFragment(_) => true,

        // Hook calls. Only a "regular" (non-optional) call's callee is hook-checked;
        // optional calls (Babel `OptionalCallExpression`) never count as hooks, but
        // their callee/args are still searched.
        oxc::Expression::CallExpression(call) => {
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
                } else if let oxc::Argument::SpreadElement(s) = arg {
                    if calls_hooks_or_creates_jsx_in_expr(&s.argument) {
                        return true;
                    }
                }
            }
            false
        }
        // Optional chaining (`a?.b`, `a?.()`): Babel modeled these as
        // Optional{Member,Call}Expression. Recurse into the inner element, where
        // per-call hook checks honor the optional flag via `is_regular_call`.
        oxc::Expression::ChainExpression(chain) => {
            calls_hooks_or_creates_jsx_in_chain_element(&chain.expression)
        }

        // Binary/logical
        oxc::Expression::BinaryExpression(bin) => {
            calls_hooks_or_creates_jsx_in_expr(&bin.left)
                || calls_hooks_or_creates_jsx_in_expr(&bin.right)
        }
        oxc::Expression::LogicalExpression(log) => {
            calls_hooks_or_creates_jsx_in_expr(&log.left)
                || calls_hooks_or_creates_jsx_in_expr(&log.right)
        }
        oxc::Expression::ConditionalExpression(cond) => {
            calls_hooks_or_creates_jsx_in_expr(&cond.test)
                || calls_hooks_or_creates_jsx_in_expr(&cond.consequent)
                || calls_hooks_or_creates_jsx_in_expr(&cond.alternate)
        }
        oxc::Expression::AssignmentExpression(assign) => {
            calls_hooks_or_creates_jsx_in_expr(&assign.right)
        }
        oxc::Expression::SequenceExpression(seq) => {
            seq.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        oxc::Expression::UnaryExpression(unary) => {
            calls_hooks_or_creates_jsx_in_expr(&unary.argument)
        }
        oxc::Expression::UpdateExpression(update) => match &update.argument {
            oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
            target => {
                target.as_member_expression().map_or(false, calls_hooks_or_creates_jsx_in_member)
            }
        },
        oxc::Expression::StaticMemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
        }
        oxc::Expression::ComputedMemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
                || calls_hooks_or_creates_jsx_in_expr(&member.expression)
        }
        oxc::Expression::PrivateFieldExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
        }
        oxc::Expression::AwaitExpression(await_expr) => {
            calls_hooks_or_creates_jsx_in_expr(&await_expr.argument)
        }
        oxc::Expression::YieldExpression(yield_expr) => yield_expr
            .argument
            .as_ref()
            .map_or(false, |arg| calls_hooks_or_creates_jsx_in_expr(arg)),
        oxc::Expression::TaggedTemplateExpression(tagged) => {
            calls_hooks_or_creates_jsx_in_expr(&tagged.tag)
                || tagged.quasi.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        oxc::Expression::TemplateLiteral(tl) => {
            tl.expressions.iter().any(calls_hooks_or_creates_jsx_in_expr)
        }
        oxc::Expression::ArrayExpression(arr) => arr.elements.iter().any(|e| match e {
            oxc::ArrayExpressionElement::SpreadElement(s) => {
                calls_hooks_or_creates_jsx_in_expr(&s.argument)
            }
            oxc::ArrayExpressionElement::Elision(_) => false,
            other => other.as_expression().map_or(false, calls_hooks_or_creates_jsx_in_expr),
        }),
        oxc::Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| match prop {
            oxc::ObjectPropertyKind::SpreadProperty(s) => {
                calls_hooks_or_creates_jsx_in_expr(&s.argument)
            }
            // Object methods (`{ foo() {} }`, getters/setters): Babel modeled these
            // as `ObjectMethod` and traversed their body statements. Regular
            // properties traverse their value (nested functions are skipped).
            oxc::ObjectPropertyKind::ObjectProperty(p) => {
                if p.method || matches!(p.kind, oxc::PropertyKind::Get | oxc::PropertyKind::Set) {
                    if let oxc::Expression::FunctionExpression(func) = &p.value {
                        if let Some(body) = &func.body {
                            return calls_hooks_or_creates_jsx_in_stmts(&body.statements);
                        }
                    }
                    false
                } else {
                    calls_hooks_or_creates_jsx_in_expr(&p.value)
                }
            }
        }),
        oxc::Expression::ParenthesizedExpression(paren) => {
            calls_hooks_or_creates_jsx_in_expr(&paren.expression)
        }
        oxc::Expression::TSAsExpression(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        oxc::Expression::TSSatisfiesExpression(ts) => {
            calls_hooks_or_creates_jsx_in_expr(&ts.expression)
        }
        oxc::Expression::TSNonNullExpression(ts) => {
            calls_hooks_or_creates_jsx_in_expr(&ts.expression)
        }
        oxc::Expression::TSTypeAssertion(ts) => calls_hooks_or_creates_jsx_in_expr(&ts.expression),
        oxc::Expression::TSInstantiationExpression(ts) => {
            calls_hooks_or_creates_jsx_in_expr(&ts.expression)
        }
        oxc::Expression::NewExpression(new) => {
            if calls_hooks_or_creates_jsx_in_expr(&new.callee) {
                return true;
            }
            new.arguments.iter().any(|a| {
                if let Some(a) = a.as_expression() {
                    if is_nested_fn(a) {
                        return false;
                    }
                    calls_hooks_or_creates_jsx_in_expr(a)
                } else if let oxc::Argument::SpreadElement(s) = a {
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
fn calls_hooks_or_creates_jsx_in_chain_element(element: &oxc::ChainElement) -> bool {
    match element {
        oxc::ChainElement::CallExpression(call) => {
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
                        oxc::Expression::ArrowFunctionExpression(_)
                            | oxc::Expression::FunctionExpression(_)
                    ) && calls_hooks_or_creates_jsx_in_expr(arg)
                } else if let oxc::Argument::SpreadElement(s) = arg {
                    calls_hooks_or_creates_jsx_in_expr(&s.argument)
                } else {
                    false
                }
            })
        }
        oxc::ChainElement::StaticMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
        oxc::ChainElement::ComputedMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
                || calls_hooks_or_creates_jsx_in_expr(&m.expression)
        }
        oxc::ChainElement::PrivateFieldExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
        oxc::ChainElement::TSNonNullExpression(t) => {
            calls_hooks_or_creates_jsx_in_expr(&t.expression)
        }
    }
}

/// Search a member expression (object side) for hook calls / JSX.
fn calls_hooks_or_creates_jsx_in_member(member: &oxc::MemberExpression) -> bool {
    match member {
        oxc::MemberExpression::StaticMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
        oxc::MemberExpression::ComputedMemberExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
                || calls_hooks_or_creates_jsx_in_expr(&m.expression)
        }
        oxc::MemberExpression::PrivateFieldExpression(m) => {
            calls_hooks_or_creates_jsx_in_expr(&m.object)
        }
    }
}

/// Check if a function body calls hooks or creates JSX.
fn calls_hooks_or_creates_jsx(params: &oxc::FormalParameters, body: &FunctionBody) -> bool {
    // Check default param values (TS traverses the whole function node including params)
    if calls_hooks_or_creates_jsx_in_params(params) {
        return true;
    }
    match body {
        FunctionBody::Block(block) => calls_hooks_or_creates_jsx_in_stmts(&block.statements),
        FunctionBody::Expression(expr) => calls_hooks_or_creates_jsx_in_expr(expr),
    }
}

/// Check if any parameter default values contain hooks or JSX.
///
/// Babel traversed the whole function node including params; default values live
/// on `FormalParameter.initializer`, and the binding pattern may itself contain
/// nested defaults.
fn calls_hooks_or_creates_jsx_in_params(params: &oxc::FormalParameters) -> bool {
    for param in &params.items {
        if let Some(init) = &param.initializer {
            if calls_hooks_or_creates_jsx_in_expr(init) {
                return true;
            }
        }
        if calls_hooks_or_creates_jsx_in_binding(&param.pattern) {
            return true;
        }
    }
    if let Some(rest) = &params.rest {
        if calls_hooks_or_creates_jsx_in_binding(&rest.rest.argument) {
            return true;
        }
    }
    false
}

fn calls_hooks_or_creates_jsx_in_binding(pattern: &oxc::BindingPattern) -> bool {
    match pattern {
        oxc::BindingPattern::BindingIdentifier(_) => false,
        oxc::BindingPattern::ObjectPattern(obj) => {
            obj.properties.iter().any(|p| calls_hooks_or_creates_jsx_in_binding(&p.value))
                || obj
                    .rest
                    .as_ref()
                    .map_or(false, |r| calls_hooks_or_creates_jsx_in_binding(&r.argument))
        }
        oxc::BindingPattern::ArrayPattern(arr) => {
            arr.elements
                .iter()
                .any(|e| e.as_ref().map_or(false, calls_hooks_or_creates_jsx_in_binding))
                || arr
                    .rest
                    .as_ref()
                    .map_or(false, |r| calls_hooks_or_creates_jsx_in_binding(&r.argument))
        }
        oxc::BindingPattern::AssignmentPattern(assign) => {
            calls_hooks_or_creates_jsx_in_expr(&assign.right)
                || calls_hooks_or_creates_jsx_in_binding(&assign.left)
        }
    }
}

/// Check if a parameter's type annotation is valid for a React component prop.
/// Returns false for primitive type annotations that indicate this is NOT a component.
fn is_valid_props_annotation(type_annotation: Option<&oxc::TSTypeAnnotation>) -> bool {
    let Some(annotation) = type_annotation else {
        return true; // No annotation = valid
    };
    // Mirrors the Babel-bridge `babel_ts_type_name` of the param's annotation; the
    // disallowed TS keyword/type set. (Flow types never appear in the oxc AST.)
    !matches!(
        &annotation.type_annotation,
        oxc::TSType::TSArrayType(_)
            | oxc::TSType::TSBigIntKeyword(_)
            | oxc::TSType::TSBooleanKeyword(_)
            | oxc::TSType::TSConstructorType(_)
            | oxc::TSType::TSFunctionType(_)
            | oxc::TSType::TSLiteralType(_)
            | oxc::TSType::TSNeverKeyword(_)
            | oxc::TSType::TSNumberKeyword(_)
            | oxc::TSType::TSStringKeyword(_)
            | oxc::TSType::TSSymbolKeyword(_)
            | oxc::TSType::TSTupleType(_)
    )
}

/// Check if the function parameters are valid for a React component.
/// Components can have 0 params, 1 param (props), or 2 params (props + ref).
///
/// The Babel reference treated the rest parameter as a trailing `RestElement` in
/// the flat params list; in oxc the rest lives in `params.rest`. So the logical
/// param count is `items.len() + rest.is_some()`.
fn is_valid_component_params(params: &oxc::FormalParameters) -> bool {
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
            oxc::BindingPattern::BindingIdentifier(id) => {
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
enum FunctionBody<'a> {
    Block(&'a oxc::FunctionBody<'a>),
    Expression(&'a oxc::Expression<'a>),
}

// -----------------------------------------------------------------------
// Function type detection
// -----------------------------------------------------------------------

/// Determine the React function type for a function, given the compilation mode
/// and the function's name and context.
///
/// This is the Rust equivalent of `getReactFunctionType` in Program.ts.
fn get_react_function_type(
    name: Option<&str>,
    params: &oxc::FormalParameters,
    body: &FunctionBody,
    body_directives: &[String],
    is_declaration: bool,
    parent_callee_name: Option<&str>,
    opts: &PluginOptions,
    is_component_declaration: bool,
    is_hook_declaration: bool,
) -> Option<ReactFunctionType> {
    // Check for opt-in directives in the function body
    if let FunctionBody::Block(_) = body {
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

    match opts.compilation_mode.as_str() {
        "annotation" => {
            // opt-ins were checked above
            None
        }
        "infer" => {
            // Check if this is a component or hook-like function
            component_syntax_type
                .or_else(|| get_component_or_hook_like(name, params, body, parent_callee_name))
        }
        "syntax" => {
            // In syntax mode, only compile declared components/hooks
            component_syntax_type
        }
        "all" => Some(
            get_component_or_hook_like(name, params, body, parent_callee_name)
                .unwrap_or(ReactFunctionType::Other),
        ),
        _ => None,
    }
}

/// Determine if a function looks like a React component or hook based on
/// naming conventions and code patterns.
///
/// Adapted from the ESLint rule at
/// https://github.com/facebook/react/blob/main/packages/eslint-plugin-react-hooks/src/RulesOfHooks.js
fn get_component_or_hook_like(
    name: Option<&str>,
    params: &oxc::FormalParameters,
    body: &FunctionBody,
    parent_callee_name: Option<&str>,
) -> Option<ReactFunctionType> {
    if let Some(fn_name) = name {
        if is_component_name(fn_name) {
            // Check if it actually looks like a component
            let is_component = calls_hooks_or_creates_jsx(params, body)
                && is_valid_component_params(params)
                && !returns_non_node_fn(params, body);
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
    if let Some(callee_name) = parent_callee_name {
        if callee_name == "forwardRef" || callee_name == "memo" {
            return if calls_hooks_or_creates_jsx(params, body) {
                Some(ReactFunctionType::Component)
            } else {
                None
            };
        }
    }

    None
}

/// Extract the callee name from a CallExpression if it's a React API call
/// (forwardRef, memo, React.forwardRef, React.memo).
fn get_callee_name_if_react_api<'e>(callee: &'e oxc::Expression) -> Option<&'e str> {
    match callee {
        oxc::Expression::Identifier(id) => {
            if id.name == "forwardRef" || id.name == "memo" {
                Some(id.name.as_str())
            } else {
                None
            }
        }
        oxc::Expression::StaticMemberExpression(member) => {
            if let oxc::Expression::Identifier(obj) = &member.object {
                if obj.name == "React"
                    && (member.property.name == "forwardRef" || member.property.name == "memo")
                {
                    return Some(member.property.name.as_str());
                }
            }
            None
        }
        _ => None,
    }
}

// -----------------------------------------------------------------------
// Error handling
// -----------------------------------------------------------------------

/// Push a compiler error's per-detail diagnostics onto the context.
fn log_error(err: &CompilerError, fn_loc: Option<Span>, context: &mut ProgramContext) {
    // Detect simulated unknown exception (throwUnknownException__testonly). In TS,
    // non-CompilerError exceptions surface as a pipeline error carrying the error
    // message rather than a per-detail compiler error.
    let is_simulated_unknown = err.details.len() == 1
        && err.details.iter().all(|d| match d {
            CompilerErrorOrDiagnostic::ErrorDetail(d) => {
                d.category == ErrorCategory::Invariant && d.reason == "unexpected error"
            }
            _ => false,
        });
    if is_simulated_unknown {
        let mut diagnostic =
            OxcDiagnostic::error("[ReactCompiler] Pipeline error: Error: unexpected error");
        if let Some(span) = fn_loc {
            diagnostic = diagnostic.with_label(span);
        }
        context.diagnostics.push(diagnostic);
        return;
    }

    for detail in &err.details {
        if let Some(diagnostic) = crate::diagnostics::detail_to_diagnostic(detail, fn_loc) {
            context.diagnostics.push(diagnostic);
        }
    }
}

/// Handle an error according to the panicThreshold setting.
/// Returns Some(CompileResult::Error) if the error should be surfaced as fatal,
/// otherwise returns None (error was logged only).
fn handle_error<'a>(
    err: &CompilerError,
    fn_loc: Option<Span>,
    context: &mut ProgramContext,
) -> Option<CompileResult<'a>> {
    // Log the error
    log_error(err, fn_loc, context);

    let should_panic = match context.opts.panic_threshold.as_str() {
        "all_errors" => true,
        "critical_errors" => err.has_errors(),
        _ => false,
    };

    // Config errors always cause a panic
    let is_config_error = err.details.iter().any(|d| match d {
        CompilerErrorOrDiagnostic::Diagnostic(d) => d.category == ErrorCategory::Config,
        CompilerErrorOrDiagnostic::ErrorDetail(d) => d.category == ErrorCategory::Config,
    });

    if should_panic || is_config_error {
        // The per-detail diagnostics were already pushed by `log_error`; the fatal
        // result just carries them. (The old JS-shim summary is dropped.)
        Some(CompileResult::Error {
            diagnostics: std::mem::take(&mut context.diagnostics),
            ordered_log: std::mem::take(&mut context.ordered_log),
        })
    } else {
        None
    }
}

// -----------------------------------------------------------------------
// Compilation pipeline stubs
// -----------------------------------------------------------------------

/// Attempt to compile a single function.
///
/// Returns `CodegenFunction` on success or `CompilerError` on failure.
/// Debug log entries are accumulated on `context.debug_logs`.
fn try_compile_function<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    source: &CompileSource,
    scope_info: &ScopeInfo,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
) -> Result<CodegenFunction<'a>, CompilerError> {
    // Check for suppressions that affect this function
    if let (Some(start), Some(end)) = (source.fn_start, source.fn_end) {
        let affecting = filter_suppressions_that_affect_function(&context.suppressions, start, end);
        if !affecting.is_empty() {
            let owned: Vec<SuppressionRange> = affecting.into_iter().cloned().collect();
            let mut err = suppressions_to_compiler_error(&owned);
            // Suppression errors are returned (not thrown), so they should NOT
            // trigger CompileUnexpectedThrow.
            err.is_thrown = false;
            return Err(err);
        }
    }

    // Run the compilation pipeline directly on the oxc function node discovered
    // during the program walk.
    pipeline::compile_fn(
        ast,
        &source.fn_node,
        source.fn_name.as_deref(),
        scope_info,
        source.fn_type,
        output_mode,
        env_config,
        context,
    )
}

/// Process a single function: check directives, attempt compilation, handle results.
///
/// Returns `Ok(Some(codegen_fn))` when the function was compiled and should be applied,
/// `Ok(None)` when the function was skipped or lint-only,
/// or `Err(CompileResult)` if a fatal error should short-circuit the program.
fn process_fn<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    source: &CompileSource,
    scope_info: &ScopeInfo,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
) -> Result<Option<CodegenFunction<'a>>, CompileResult<'a>> {
    // Parse directives from the function body
    let opt_in_result =
        try_find_directive_enabling_memoization(&source.body_directives, &context.opts);
    let opt_out = find_directive_disabling_memoization(&source.body_directives, &context.opts);

    // If parsing opt-in directive fails, handle the error and skip
    let opt_in = match opt_in_result {
        Ok(d) => d,
        Err(err) => {
            // Apply panic threshold logic (same as compilation errors)
            if let Some(result) = handle_error(&err, source.fn_ast_loc, context) {
                return Err(result);
            }
            return Ok(None);
        }
    };

    // Attempt compilation
    let compile_result =
        try_compile_function(ast, source, scope_info, output_mode, env_config, context);

    match compile_result {
        Err(err) => {
            // Surface errors "thrown" from a pass (not accumulated via
            // env.record_error) that have all non-Invariant details, matching TS
            // tryCompileFunction()'s catch block.
            if err.is_thrown && err.is_all_non_invariant() {
                let mut diagnostic = OxcDiagnostic::error(format!(
                    "[ReactCompiler] Unexpected error: {}",
                    err.to_string_for_event()
                ));
                if let Some(span) = source.fn_ast_loc {
                    diagnostic = diagnostic.with_label(span);
                }
                context.diagnostics.push(diagnostic);
            }

            if opt_out.is_some() {
                // If there's an opt-out, just log the error (don't escalate)
                log_error(&err, source.fn_ast_loc, context);
            } else {
                // Apply panic threshold logic
                if let Some(result) = handle_error(&err, source.fn_ast_loc, context) {
                    return Err(result);
                }
            }
            Ok(None)
        }
        Ok(codegen_fn) => {
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
            if context.opts.compilation_mode == "annotation" && opt_in.is_none() {
                return Ok(None);
            }

            Ok(Some(codegen_fn))
        }
    }
}

// -----------------------------------------------------------------------
// Import checking
// -----------------------------------------------------------------------

/// Check if the program already has a `c` import from the React Compiler runtime module.
/// If so, the file was already compiled and should be skipped.
fn has_memo_cache_function_import(program: &oxc::Program, module_name: &str) -> bool {
    for stmt in &program.body {
        if let oxc::Statement::ImportDeclaration(import) = stmt {
            if import.source.value == module_name {
                if let Some(specifiers) = &import.specifiers {
                    for specifier in specifiers {
                        if let oxc::ImportDeclarationSpecifier::ImportSpecifier(data) = specifier {
                            let imported_name = match &data.imported {
                                oxc::ModuleExportName::IdentifierName(id) => Some(id.name.as_str()),
                                oxc::ModuleExportName::IdentifierReference(id) => {
                                    Some(id.name.as_str())
                                }
                                oxc::ModuleExportName::StringLiteral(s) => Some(s.value.as_str()),
                            };
                            if imported_name == Some("c") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if compilation should be skipped for this program.
fn should_skip_compilation(program: &oxc::Program, options: &PluginOptions) -> bool {
    let runtime_module = get_react_compiler_runtime_module(&options.target);
    has_memo_cache_function_import(program, &runtime_module)
}

// -----------------------------------------------------------------------
// Function discovery
// -----------------------------------------------------------------------

/// Collect the directive value strings of a function body (for opt-in/opt-out
/// checks). Matches the Babel-bridge directive values (`d.expression.value`).
fn body_directive_values(body: &oxc::FunctionBody) -> Vec<String> {
    body.directives.iter().map(|d| d.expression.value.to_string()).collect()
}

/// Try to create a `CompileSource` from a discovered oxc function node.
///
/// `fn_node` is the oxc function node, `span` its byte range (`span.start` is the
/// node id), `name` the inferred function name, `original_kind` the syntactic kind,
/// and `parent_callee_name` the enclosing forwardRef/memo callee (if any).
fn try_make_compile_source<'a>(
    fn_node: FunctionNode<'a>,
    name: Option<String>,
    original_kind: OriginalFnKind,
    parent_callee_name: Option<String>,
    opts: &PluginOptions,
    context: &mut ProgramContext,
) -> Option<CompileSource<'a>> {
    let (params, body, span, body_directives) = match fn_node {
        FunctionNode::Function(f) => {
            // Bodyless functions (`declare function`, overload signatures,
            // `TSDeclareFunction`) are never compiled; Babel modeled these as
            // separate, non-traversed statement kinds.
            let block = f.body.as_deref()?;
            (&f.params, FunctionBody::Block(block), f.span, body_directive_values(block))
        }
        FunctionNode::Arrow(a) => {
            let (body, directives) = if a.expression {
                // Expression-bodied arrow: the single body statement is an
                // `ExpressionStatement` wrapping the expression.
                let expr = a.get_expression().expect("expression-bodied arrow has an expression");
                (FunctionBody::Expression(expr), Vec::new())
            } else {
                (FunctionBody::Block(&a.body), body_directive_values(&a.body))
            };
            (&a.params, body, a.span, directives)
        }
    };

    let node_id = span.start;

    // Skip if already compiled (identified by node_id).
    if context.is_already_compiled(node_id) {
        return None;
    }

    let fn_type = get_react_function_type(
        name.as_deref(),
        params,
        &body,
        &body_directives,
        // Flow `component`/`hook` declaration syntax never appears in the oxc AST.
        false,
        parent_callee_name.as_deref(),
        opts,
        false,
        false,
    )?;

    context.mark_compiled(node_id);

    Some(CompileSource {
        kind: CompileSourceKind::Original,
        original_kind,
        fn_name: name,
        // The function source location flows into compile-error diagnostics as the
        // fallback labeled span (offset/length). Only the byte `index` is
        // load-bearing; line/column/filename never reach the example's output.
        fn_ast_loc: Some(span),
        fn_start: Some(span.start),
        fn_end: Some(span.end),
        fn_node_id: Some(node_id),
        fn_type,
        fn_node,
        body_directives,
    })
}

/// Get the variable declarator name (for inferring function names from
/// `const Foo = () => {}`).
fn get_declarator_name(decl: &oxc::VariableDeclarator) -> Option<String> {
    match &decl.id {
        oxc::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
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
struct DiscoveryWalker<'a, 'ast> {
    scope_info: &'a ScopeInfo,
    opts: &'a PluginOptions,
    context: &'a mut ProgramContext,
    queue: Vec<CompileSource<'ast>>,
    scope_stack: Vec<ScopeId>,
    loop_expression_depth: usize,
    current_declarator_name: Option<String>,
    parent_callee_stack: Vec<Option<String>>,
}

impl<'a, 'ast> DiscoveryWalker<'a, 'ast> {
    fn new(
        scope_info: &'a ScopeInfo,
        opts: &'a PluginOptions,
        context: &'a mut ProgramContext,
    ) -> Self {
        Self {
            scope_info,
            opts,
            context,
            queue: Vec::new(),
            scope_stack: Vec::new(),
            loop_expression_depth: 0,
            current_declarator_name: None,
            parent_callee_stack: Vec::new(),
        }
    }

    /// Try to push the scope a node creates. Returns whether one was pushed.
    fn try_push_scope(&mut self, span: Span) -> bool {
        if let Some(scope_id) = self.scope_info.resolve_scope_for_node(Some(span.start)) {
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
        self.opts.compilation_mode == "all"
            && (self.scope_stack.len() > 2 || self.loop_expression_depth > 0)
    }

    fn current_parent_callee(&self) -> Option<String> {
        self.parent_callee_stack.last().and_then(|opt| opt.clone())
    }

    fn walk_program(&mut self, program: &'ast oxc::Program<'ast>) {
        let pushed = self.try_push_scope(program.span);
        for stmt in &program.body {
            self.walk_statement(stmt);
        }
        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_block(&mut self, block: &'ast oxc::BlockStatement<'ast>) {
        let pushed = self.try_push_scope(block.span);
        for stmt in &block.body {
            self.walk_statement(stmt);
        }
        if pushed {
            self.scope_stack.pop();
        }
    }

    fn walk_function_body_block(&mut self, body: &'ast oxc::FunctionBody<'ast>) {
        // A function body BlockStatement shares the function's scope in the Babel
        // model and never gets its own scope entry, so do not push a scope here.
        for stmt in &body.statements {
            self.walk_statement(stmt);
        }
    }

    fn walk_statement(&mut self, stmt: &'ast oxc::Statement<'ast>) {
        match stmt {
            oxc::Statement::BlockStatement(node) => self.walk_block(node),
            oxc::Statement::ReturnStatement(node) => {
                if let Some(arg) = &node.argument {
                    self.walk_expression(arg);
                }
            }
            oxc::Statement::ExpressionStatement(node) => self.walk_expression(&node.expression),
            oxc::Statement::IfStatement(node) => {
                self.walk_expression(&node.test);
                self.walk_statement(&node.consequent);
                if let Some(alt) = &node.alternate {
                    self.walk_statement(alt);
                }
            }
            oxc::Statement::ForStatement(node) => {
                let pushed = self.try_push_scope(node.span);
                if let Some(init) = &node.init {
                    match init {
                        oxc::ForStatementInit::VariableDeclaration(decl) => {
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
            oxc::Statement::WhileStatement(node) => {
                self.loop_expression_depth += 1;
                self.walk_expression(&node.test);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
            }
            oxc::Statement::DoWhileStatement(node) => {
                self.walk_statement(&node.body);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.test);
                self.loop_expression_depth -= 1;
            }
            oxc::Statement::ForInStatement(node) => {
                let pushed = self.try_push_scope(node.span);
                self.walk_for_left(&node.left);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.right);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
                if pushed {
                    self.scope_stack.pop();
                }
            }
            oxc::Statement::ForOfStatement(node) => {
                let pushed = self.try_push_scope(node.span);
                self.walk_for_left(&node.left);
                self.loop_expression_depth += 1;
                self.walk_expression(&node.right);
                self.loop_expression_depth -= 1;
                self.walk_statement(&node.body);
                if pushed {
                    self.scope_stack.pop();
                }
            }
            oxc::Statement::SwitchStatement(node) => {
                let pushed = self.try_push_scope(node.span);
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
            oxc::Statement::ThrowStatement(node) => self.walk_expression(&node.argument),
            oxc::Statement::TryStatement(node) => {
                self.walk_block(&node.block);
                if let Some(handler) = &node.handler {
                    let pushed = self.try_push_scope(handler.span);
                    self.walk_block(&handler.body);
                    if pushed {
                        self.scope_stack.pop();
                    }
                }
                if let Some(finalizer) = &node.finalizer {
                    self.walk_block(finalizer);
                }
            }
            oxc::Statement::LabeledStatement(node) => self.walk_statement(&node.body),
            oxc::Statement::VariableDeclaration(node) => self.walk_variable_declaration(node),
            oxc::Statement::FunctionDeclaration(node) => self.walk_function(node, None),
            oxc::Statement::WithStatement(node) => {
                self.walk_expression(&node.object);
                self.walk_statement(&node.body);
            }
            oxc::Statement::ExportNamedDeclaration(node) => {
                if let Some(decl) = &node.declaration {
                    self.walk_declaration(decl);
                }
            }
            oxc::Statement::ExportDefaultDeclaration(node) => {
                self.walk_export_default(&node.declaration);
            }
            // Classes are not descended into (their bodies hold no functions for
            // discovery); all remaining statements (TS-only declarations, imports,
            // break/continue, etc.) carry no compilable functions.
            _ => {}
        }
    }

    fn walk_for_left(&mut self, left: &'ast oxc::ForStatementLeft<'ast>) {
        if let oxc::ForStatementLeft::VariableDeclaration(decl) = left {
            self.walk_variable_declaration(decl);
        }
        // Assignment-target lefts contain no functions to discover.
    }

    fn walk_variable_declaration(&mut self, decl: &'ast oxc::VariableDeclaration<'ast>) {
        for declarator in &decl.declarations {
            // Only infer the declarator name when the init is a direct function
            // expression, arrow, or call expression (for forwardRef/memo wrappers).
            if let Some(init) = &declarator.init {
                if matches!(
                    init,
                    oxc::Expression::FunctionExpression(_)
                        | oxc::Expression::ArrowFunctionExpression(_)
                        | oxc::Expression::CallExpression(_)
                ) {
                    self.current_declarator_name = get_declarator_name(declarator);
                }
                self.walk_expression(init);
            }
            self.current_declarator_name = None;
        }
    }

    fn walk_declaration(&mut self, decl: &'ast oxc::Declaration<'ast>) {
        match decl {
            oxc::Declaration::FunctionDeclaration(node) => self.walk_function(node, None),
            oxc::Declaration::VariableDeclaration(node) => self.walk_variable_declaration(node),
            // TS-only declarations have no runtime expressions / functions.
            _ => {}
        }
    }

    fn walk_export_default(&mut self, decl: &'ast oxc::ExportDefaultDeclarationKind<'ast>) {
        match decl {
            oxc::ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
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
    fn walk_function(&mut self, func: &'ast oxc::Function<'ast>, inferred_name: Option<String>) {
        let pushed = self.try_push_scope(func.span);

        let original_kind = match func.r#type {
            oxc::FunctionType::FunctionDeclaration | oxc::FunctionType::TSDeclareFunction => {
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
                self.context,
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
        arrow: &'ast oxc::ArrowFunctionExpression<'ast>,
        inferred_name: Option<String>,
    ) {
        let pushed = self.try_push_scope(arrow.span);

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
                self.context,
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

    fn walk_expression(&mut self, expr: &'ast oxc::Expression<'ast>) {
        match expr {
            oxc::Expression::FunctionExpression(node) => {
                // The declarator name flows into the function expression and is
                // consumed here (so siblings don't inherit it).
                let name = self.current_declarator_name.take();
                self.walk_function(node, name);
            }
            oxc::Expression::ArrowFunctionExpression(node) => {
                let name = self.current_declarator_name.take();
                self.walk_arrow(node, name);
            }
            oxc::Expression::CallExpression(node) => {
                let callee_name = get_callee_name_if_react_api(&node.callee).map(|s| s.to_string());
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
            oxc::Expression::ChainExpression(node) => self.walk_chain_element(&node.expression),
            oxc::Expression::StaticMemberExpression(node) => self.walk_expression(&node.object),
            oxc::Expression::ComputedMemberExpression(node) => {
                self.walk_expression(&node.object);
                self.walk_expression(&node.expression);
            }
            oxc::Expression::PrivateFieldExpression(node) => self.walk_expression(&node.object),
            oxc::Expression::BinaryExpression(node) => {
                self.walk_expression(&node.left);
                self.walk_expression(&node.right);
            }
            oxc::Expression::LogicalExpression(node) => {
                self.walk_expression(&node.left);
                self.walk_expression(&node.right);
            }
            oxc::Expression::UnaryExpression(node) => self.walk_expression(&node.argument),
            oxc::Expression::UpdateExpression(node) => {
                if let Some(member) = node.argument.as_member_expression() {
                    self.walk_member_expression(member);
                }
            }
            oxc::Expression::ConditionalExpression(node) => {
                self.walk_expression(&node.test);
                self.walk_expression(&node.consequent);
                self.walk_expression(&node.alternate);
            }
            oxc::Expression::AssignmentExpression(node) => self.walk_expression(&node.right),
            oxc::Expression::SequenceExpression(node) => {
                for e in &node.expressions {
                    self.walk_expression(e);
                }
            }
            oxc::Expression::ObjectExpression(node) => {
                for prop in &node.properties {
                    self.walk_object_property(prop);
                }
            }
            oxc::Expression::ArrayExpression(node) => {
                for el in &node.elements {
                    match el {
                        oxc::ArrayExpressionElement::SpreadElement(s) => {
                            self.walk_expression(&s.argument)
                        }
                        oxc::ArrayExpressionElement::Elision(_) => {}
                        other => {
                            if let Some(e) = other.as_expression() {
                                self.walk_expression(e);
                            }
                        }
                    }
                }
            }
            oxc::Expression::NewExpression(node) => {
                self.walk_expression(&node.callee);
                for arg in &node.arguments {
                    self.walk_argument(arg);
                }
            }
            oxc::Expression::TemplateLiteral(node) => {
                for e in &node.expressions {
                    self.walk_expression(e);
                }
            }
            oxc::Expression::TaggedTemplateExpression(node) => {
                self.walk_expression(&node.tag);
                for e in &node.quasi.expressions {
                    self.walk_expression(e);
                }
            }
            oxc::Expression::AwaitExpression(node) => self.walk_expression(&node.argument),
            oxc::Expression::YieldExpression(node) => {
                if let Some(arg) = &node.argument {
                    self.walk_expression(arg);
                }
            }
            oxc::Expression::ParenthesizedExpression(node) => {
                self.walk_expression(&node.expression)
            }
            oxc::Expression::JSXElement(node) => self.walk_jsx_element(node),
            oxc::Expression::JSXFragment(node) => self.walk_jsx_children(&node.children),
            oxc::Expression::TSAsExpression(node) => self.walk_expression(&node.expression),
            oxc::Expression::TSSatisfiesExpression(node) => self.walk_expression(&node.expression),
            oxc::Expression::TSNonNullExpression(node) => self.walk_expression(&node.expression),
            oxc::Expression::TSTypeAssertion(node) => self.walk_expression(&node.expression),
            oxc::Expression::TSInstantiationExpression(node) => {
                self.walk_expression(&node.expression)
            }
            // ClassExpression bodies and leaf expressions are not descended.
            _ => {}
        }
    }

    fn walk_argument(&mut self, arg: &'ast oxc::Argument<'ast>) {
        if let Some(expr) = arg.as_expression() {
            self.walk_expression(expr);
        } else if let oxc::Argument::SpreadElement(s) = arg {
            self.walk_expression(&s.argument);
        }
    }

    fn walk_member_expression(&mut self, member: &'ast oxc::MemberExpression<'ast>) {
        match member {
            oxc::MemberExpression::StaticMemberExpression(m) => self.walk_expression(&m.object),
            oxc::MemberExpression::ComputedMemberExpression(m) => {
                self.walk_expression(&m.object);
                self.walk_expression(&m.expression);
            }
            oxc::MemberExpression::PrivateFieldExpression(m) => self.walk_expression(&m.object),
        }
    }

    fn walk_chain_element(&mut self, element: &'ast oxc::ChainElement<'ast>) {
        match element {
            oxc::ChainElement::CallExpression(node) => {
                self.walk_expression(&node.callee);
                for arg in &node.arguments {
                    self.walk_argument(arg);
                }
            }
            oxc::ChainElement::StaticMemberExpression(m) => self.walk_expression(&m.object),
            oxc::ChainElement::ComputedMemberExpression(m) => {
                self.walk_expression(&m.object);
                self.walk_expression(&m.expression);
            }
            oxc::ChainElement::PrivateFieldExpression(m) => self.walk_expression(&m.object),
            oxc::ChainElement::TSNonNullExpression(t) => self.walk_expression(&t.expression),
        }
    }

    fn walk_object_property(&mut self, prop: &'ast oxc::ObjectPropertyKind<'ast>) {
        match prop {
            oxc::ObjectPropertyKind::SpreadProperty(s) => self.walk_expression(&s.argument),
            oxc::ObjectPropertyKind::ObjectProperty(p) => {
                if p.computed {
                    self.walk_property_key(&p.key);
                }
                // Object methods (`{ foo() {} }`, getters/setters): Babel modeled
                // these as `ObjectMethod` and walked the body without queuing the
                // method itself; the body is the value FunctionExpression's body.
                let is_method =
                    p.method || matches!(p.kind, oxc::PropertyKind::Get | oxc::PropertyKind::Set);
                if is_method {
                    if let oxc::Expression::FunctionExpression(func) = &p.value {
                        let pushed = self.try_push_scope(func.span);
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

    fn walk_property_key(&mut self, key: &'ast oxc::PropertyKey<'ast>) {
        if let Some(expr) = key.as_expression() {
            self.walk_expression(expr);
        }
    }

    fn walk_jsx_element(&mut self, node: &'ast oxc::JSXElement<'ast>) {
        for attr in &node.opening_element.attributes {
            match attr {
                oxc::JSXAttributeItem::Attribute(a) => {
                    if let Some(value) = &a.value {
                        match value {
                            oxc::JSXAttributeValue::ExpressionContainer(c) => {
                                self.walk_jsx_container(c)
                            }
                            oxc::JSXAttributeValue::Element(el) => self.walk_jsx_element(el),
                            oxc::JSXAttributeValue::Fragment(f) => {
                                self.walk_jsx_children(&f.children)
                            }
                            oxc::JSXAttributeValue::StringLiteral(_) => {}
                        }
                    }
                }
                oxc::JSXAttributeItem::SpreadAttribute(a) => self.walk_expression(&a.argument),
            }
        }
        self.walk_jsx_children(&node.children);
    }

    fn walk_jsx_children(&mut self, children: &'ast oxc_allocator::Vec<'ast, oxc::JSXChild<'ast>>) {
        for child in children {
            match child {
                oxc::JSXChild::Element(el) => self.walk_jsx_element(el),
                oxc::JSXChild::Fragment(f) => self.walk_jsx_children(&f.children),
                oxc::JSXChild::ExpressionContainer(c) => self.walk_jsx_container(c),
                oxc::JSXChild::Spread(s) => self.walk_expression(&s.expression),
                oxc::JSXChild::Text(_) => {}
            }
        }
    }

    fn walk_jsx_container(&mut self, container: &'ast oxc::JSXExpressionContainer<'ast>) {
        if let Some(expr) = container.expression.as_expression() {
            self.walk_expression(expr);
        }
    }
}

/// Find all functions in the program that should be compiled by walking the oxc
/// `Program` directly. See [`DiscoveryWalker`] for the traversal semantics.
fn find_functions_to_compile<'ast>(
    program: &'ast oxc::Program<'ast>,
    opts: &PluginOptions,
    context: &mut ProgramContext,
    scope: &ScopeInfo,
) -> Vec<CompileSource<'ast>> {
    let mut walker = DiscoveryWalker::new(scope, opts, context);
    walker.walk_program(program);
    walker.queue
}

struct CompiledFunction<'a, 'p, 's> {
    #[allow(dead_code)]
    kind: CompileSourceKind,
    #[allow(dead_code)]
    source: &'s CompileSource<'p>,
    #[allow(dead_code)]
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
// oxc splice
//
// Builds the final oxc `Program` by substituting each compiled oxc function (from
// codegen) for its original — matched by `span.start == fn_node_id` — applying
// gating, inserting outlined functions, and adding the memo-cache / gating imports.
// =============================================================================

/// An owned, oxc-shaped compiled function ready to splice into the program.
struct OxcReplacement<'a> {
    fn_node_id: Option<u32>,
    original_kind: OriginalFnKind,
    codegen_fn: CodegenFunction<'a>,
    gating: Option<GatingConfig>,
}

/// Copy the TS metadata (type annotation, decorators, optional/modifier flags)
/// from a function's original parameters onto the compiled replacement parameters,
/// matched positionally. Mirrors the Babel reference's signature restoration for
/// functions that are not memoized: the parameter bindings are unchanged, so their
/// types carry through. The compiled params (from codegen) never carry types.
fn copy_param_ts_metadata<'a>(
    allocator: &'a oxc_allocator::Allocator,
    new_params: &mut oxc_ast::ast::FormalParameters<'a>,
    source_params: &oxc_ast::ast::FormalParameters<'a>,
) {
    use oxc_allocator::CloneIn;
    for (param, source) in new_params.items.iter_mut().zip(source_params.items.iter()) {
        param.decorators = source.decorators.clone_in(allocator);
        param.type_annotation = source.type_annotation.clone_in(allocator);
        param.optional = source.optional;
        param.accessibility = source.accessibility;
        param.readonly = source.readonly;
        param.r#override = source.r#override;
    }
    if let (Some(rest), Some(source_rest)) = (&mut new_params.rest, &source_params.rest) {
        rest.decorators = source_rest.decorators.clone_in(allocator);
        rest.type_annotation = source_rest.type_annotation.clone_in(allocator);
    }
}

/// Build an oxc `Function` from a compiled codegen function. `r#type` selects
/// declaration vs expression. Mirrors the Babel `ReplaceFnVisitor` field copy.
fn ox_build_function<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    fn_type: oxc_ast::ast::FunctionType,
) -> oxc_allocator::Box<'a, oxc_ast::ast::Function<'a>> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;
    oxc_ast::ast::Function::boxed(
        SPAN,
        fn_type,
        codegen.id.clone_in(ast.allocator()),
        codegen.generator,
        codegen.is_async,
        false,
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
        None::<oxc_allocator::Box<oxc_ast::ast::TSThisParameter>>,
        codegen.params.clone_in(ast.allocator()),
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
        Some(codegen.body.clone_in(ast.allocator())),
        ast,
    )
}

/// Build the compiled replacement as an `Expression`, matching the original node
/// kind (arrow vs function expression). Mirrors `build_compiled_expression_matching_kind`.
fn ox_build_compiled_expression<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    original_kind: OriginalFnKind,
) -> oxc_ast::ast::Expression<'a> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;
    match original_kind {
        OriginalFnKind::ArrowFunctionExpression => {
            oxc_ast::ast::Expression::ArrowFunctionExpression(
                oxc_ast::ast::ArrowFunctionExpression::boxed(
                    SPAN,
                    false,
                    codegen.is_async,
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
                    codegen.params.clone_in(ast.allocator()),
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                    codegen.body.clone_in(ast.allocator()),
                    ast,
                ),
            )
        }
        _ => oxc_ast::ast::Expression::FunctionExpression(ox_build_function(
            ast,
            codegen,
            oxc_ast::ast::FunctionType::FunctionExpression,
        )),
    }
}

/// Visitor that replaces a compiled function in the oxc AST by matching `span.start`.
/// Mirrors the Babel `ReplaceFnVisitor`.
struct OxcReplaceFnVisitor<'a, 'b> {
    ast: &'b oxc_ast::builder::AstBuilder<'a>,
    node_id: u32,
    codegen: &'b CodegenFunction<'a>,
    done: bool,
}

impl<'a, 'b> oxc_ast_visit::VisitMut<'a> for OxcReplaceFnVisitor<'a, 'b> {
    fn visit_function(
        &mut self,
        func: &mut oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        if self.done {
            return;
        }
        if func.span.start == self.node_id {
            use oxc_allocator::CloneIn;
            // When the compiled function does not initialize a memo cache, the body is
            // left essentially intact, so the original TS signature (type parameters,
            // `this` parameter, return type, and per-parameter type annotations) is
            // preserved. Functions that memoize drop these types, mirroring Babel.
            let keep_types = self.codegen.memo_slots_used == 0;
            let mut params = self.codegen.params.clone_in(self.ast.allocator());
            if keep_types {
                copy_param_ts_metadata(self.ast.allocator(), &mut params, &func.params);
            } else {
                func.type_parameters = None;
                func.return_type = None;
                func.this_param = None;
            }
            func.id = self.codegen.id.clone_in(self.ast.allocator());
            func.params = params;
            func.body = Some(self.codegen.body.clone_in(self.ast.allocator()));
            func.generator = self.codegen.generator;
            func.r#async = self.codegen.is_async;
            func.declare = false;
            self.done = true;
            return;
        }
        oxc_ast_visit::walk_mut::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &mut oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        if self.done {
            return;
        }
        if arrow.span.start == self.node_id {
            use oxc_allocator::CloneIn;
            let keep_types = self.codegen.memo_slots_used == 0;
            let mut params = self.codegen.params.clone_in(self.ast.allocator());
            if keep_types {
                copy_param_ts_metadata(self.ast.allocator(), &mut params, &arrow.params);
            } else {
                arrow.type_parameters = None;
                arrow.return_type = None;
            }
            arrow.params = params;
            arrow.body = self.codegen.body.clone_in(self.ast.allocator());
            arrow.r#async = self.codegen.is_async;
            arrow.expression = false;
            self.done = true;
            return;
        }
        oxc_ast_visit::walk_mut::walk_arrow_function_expression(self, arrow);
    }
}

/// Visitor that replaces a function (matched by `span.start`) with a gated
/// conditional expression. Mirrors the Babel `ReplaceWithGatedVisitor`.
struct OxcReplaceWithGatedVisitor<'a, 'b> {
    ast: &'b oxc_ast::builder::AstBuilder<'a>,
    node_id: u32,
    gating_expression: &'b oxc_ast::ast::Expression<'a>,
    /// Pending `export default Name;` to insert after a named export-default fn.
    export_default_name: Option<String>,
    done: bool,
}

impl<'a, 'b> OxcReplaceWithGatedVisitor<'a, 'b> {
    /// Build `const <name> = <gating_expression>;`
    fn build_const_decl(&self, name: &str) -> oxc_ast::ast::Statement<'a> {
        use oxc_allocator::CloneIn;
        use oxc_span::SPAN;
        let declarator = oxc_ast::ast::VariableDeclarator::new(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            oxc_ast::ast::BindingPattern::new_binding_identifier(
                SPAN,
                ox_atom(self.ast, name),
                self.ast,
            ),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
            Some(self.gating_expression.clone_in(self.ast.allocator())),
            false,
            self.ast,
        );
        oxc_ast::ast::Statement::VariableDeclaration(oxc_ast::ast::VariableDeclaration::boxed(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            oxc_allocator::ArenaVec::from_value_in(declarator, self.ast),
            false,
            self.ast,
        ))
    }
}

impl<'a, 'b> oxc_ast_visit::VisitMut<'a> for OxcReplaceWithGatedVisitor<'a, 'b> {
    fn visit_statements(
        &mut self,
        stmts: &mut oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>,
    ) {
        use oxc_ast::ast::Statement;
        let mut i = 0;
        while i < stmts.len() {
            if self.done {
                break;
            }
            // FunctionDeclaration → `const Foo = gating() ? ... : ...;`
            let replace_name: Option<Option<String>> = match &stmts[i] {
                Statement::FunctionDeclaration(f) if f.span.start == self.node_id => {
                    Some(f.id.as_ref().map(|id| id.name.to_string()))
                }
                Statement::ExportNamedDeclaration(e) => match &e.declaration {
                    Some(oxc_ast::ast::Declaration::FunctionDeclaration(f))
                        if f.span.start == self.node_id =>
                    {
                        Some(f.id.as_ref().map(|id| id.name.to_string()))
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some(name) = replace_name {
                let name = name.unwrap_or_else(|| "anonymous".to_string());
                let is_export = matches!(stmts[i], Statement::ExportNamedDeclaration(_));
                let const_decl = self.build_const_decl(&name);
                if is_export {
                    use oxc_span::SPAN;
                    let decl = match const_decl {
                        Statement::VariableDeclaration(d) => {
                            oxc_ast::ast::Declaration::VariableDeclaration(d)
                        }
                        _ => unreachable!(),
                    };
                    stmts[i] = oxc_ast::ast::Statement::ExportNamedDeclaration(
                        oxc_ast::ast::ExportNamedDeclaration::boxed(
                            SPAN,
                            Some(decl),
                            oxc_allocator::ArenaVec::new_in(self.ast),
                            None,
                            oxc_ast::ast::ImportOrExportKind::Value,
                            None::<oxc_allocator::Box<oxc_ast::ast::WithClause>>,
                            self.ast,
                        ),
                    );
                } else {
                    stmts[i] = const_decl;
                }
                self.done = true;
                break;
            }
            // ExportDefaultDeclaration with FunctionDeclaration
            if let Statement::ExportDefaultDeclaration(e) = &stmts[i] {
                if let oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(f) =
                    &e.declaration
                {
                    if f.span.start == self.node_id {
                        if let Some(id) = f.id.as_ref().map(|id| id.name.to_string()) {
                            stmts[i] = self.build_const_decl(&id);
                            self.export_default_name = Some(id);
                        } else {
                            use oxc_allocator::CloneIn;
                            use oxc_span::SPAN;
                            stmts[i] = oxc_ast::ast::Statement::ExportDefaultDeclaration(
                                oxc_ast::ast::ExportDefaultDeclaration::boxed(
                                    SPAN,
                                    oxc_ast::ast::ExportDefaultDeclarationKind::from(
                                        self.gating_expression.clone_in(self.ast.allocator()),
                                    ),
                                    self.ast,
                                ),
                            );
                        }
                        self.done = true;
                        break;
                    }
                }
            }
            self.visit_statement(&mut stmts[i]);
            i += 1;
        }

        // Insert `export default Name;` right after the replaced declaration.
        if let Some(name) = self.export_default_name.take() {
            use oxc_span::SPAN;
            let ident =
                oxc_ast::ast::Expression::new_identifier(SPAN, ox_atom(self.ast, &name), self.ast);
            let export = oxc_ast::ast::Statement::ExportDefaultDeclaration(
                oxc_ast::ast::ExportDefaultDeclaration::boxed(
                    SPAN,
                    oxc_ast::ast::ExportDefaultDeclarationKind::from(ident),
                    self.ast,
                ),
            );
            // Find the const decl we just inserted (it has name `name`); insert after.
            let pos = stmts.iter().position(|s| {
                matches!(s, oxc_ast::ast::Statement::VariableDeclaration(d)
                    if d.declarations.first().is_some_and(|decl| matches!(&decl.id,
                        oxc_ast::ast::BindingPattern::BindingIdentifier(b) if b.name.as_str() == name)))
            });
            if let Some(pos) = pos {
                stmts.insert(pos + 1, export);
            } else {
                stmts.push(export);
            }
        }
    }

    fn visit_expression(&mut self, expr: &mut oxc_ast::ast::Expression<'a>) {
        if self.done {
            return;
        }
        let matched = match expr {
            oxc_ast::ast::Expression::FunctionExpression(f) => f.span.start == self.node_id,
            oxc_ast::ast::Expression::ArrowFunctionExpression(f) => f.span.start == self.node_id,
            _ => false,
        };
        if matched {
            use oxc_allocator::CloneIn;
            *expr = self.gating_expression.clone_in(self.ast.allocator());
            self.done = true;
            return;
        }
        oxc_ast_visit::walk_mut::walk_expression(self, expr);
    }
}

/// Visitor that renames every identifier reference matching `old_name` to `new_name`.
/// Mirrors the Babel `RenameIdentifierVisitor` (used to rename `useMemoCache`).
struct OxcRenameIdentifierVisitor<'a, 'b> {
    ast: &'b oxc_ast::builder::AstBuilder<'a>,
    old_name: &'b str,
    new_name: &'b str,
}

impl<'a, 'b> oxc_ast_visit::VisitMut<'a> for OxcRenameIdentifierVisitor<'a, 'b> {
    fn visit_identifier_reference(&mut self, ident: &mut oxc_ast::ast::IdentifierReference<'a>) {
        if ident.name == self.old_name {
            ident.name = ox_atom(self.ast, self.new_name).into();
        }
    }
}

/// Allocate a `&'a str` in the arena (satisfies the builders' `Into<Ident>` /
/// `IntoIn` slots; convert to `Atom` via `.into()` where a bare `Atom` is needed).
fn ox_atom<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, s: &str) -> &'a str {
    oxc_allocator::StringBuilder::from_str_in(s, ast.allocator()).into_str()
}

/// Build `<callee_name>()` as an oxc call expression.
fn ox_gating_call<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    callee_name: &str,
) -> oxc_ast::ast::Expression<'a> {
    use oxc_span::SPAN;
    oxc_ast::ast::Expression::new_call_expression(
        SPAN,
        oxc_ast::ast::Expression::new_identifier(SPAN, ox_atom(ast, callee_name), ast),
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
        oxc_allocator::ArenaVec::new_in(ast),
        false,
        ast,
    )
}

/// Apply the conditional gating pattern to the oxc program. Mirrors
/// `apply_gated_function_conditional`.
fn ox_apply_gated_conditional<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    program: &mut oxc_ast::ast::Program<'a>,
    replacement: &OxcReplacement<'a>,
    gating_config: &GatingConfig,
    context: &mut ProgramContext,
) {
    let node_id = match replacement.fn_node_id {
        Some(nid) => nid,
        None => return,
    };

    let gating_import = context.add_import_specifier(
        &gating_config.source,
        &gating_config.import_specifier_name,
        None,
    );
    let gating_callee_name = gating_import.name;

    // Clone the original function (matched by node_id) as the fallback expression
    // BEFORE replacing it.
    let original_expr = ox_clone_original_fn_as_expression(ast, program, node_id);
    let original_expr = match original_expr {
        Some(e) => e,
        None => return,
    };

    let compiled_expr =
        ox_build_compiled_expression(ast, &replacement.codegen_fn, replacement.original_kind);

    // gating() ? compiled : original
    use oxc_span::SPAN;
    let gating_expression = oxc_ast::ast::Expression::new_conditional_expression(
        SPAN,
        ox_gating_call(ast, &gating_callee_name),
        compiled_expr,
        original_expr,
        ast,
    );

    let mut visitor = OxcReplaceWithGatedVisitor {
        ast,
        node_id,
        gating_expression: &gating_expression,
        export_default_name: None,
        done: false,
    };
    oxc_ast_visit::VisitMut::visit_program(&mut visitor, program);
}

/// Clone the original function at `node_id` as an `Expression` (FunctionDeclaration
/// becomes a FunctionExpression). Mirrors `clone_original_fn_as_expression`.
fn ox_clone_original_fn_as_expression<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    program: &oxc_ast::ast::Program<'a>,
    node_id: u32,
) -> Option<oxc_ast::ast::Expression<'a>> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;

    struct Finder<'a, 'b> {
        ast: &'b oxc_ast::builder::AstBuilder<'a>,
        node_id: u32,
        found: Option<oxc_ast::ast::Expression<'a>>,
    }
    impl<'a, 'b> oxc_ast_visit::Visit<'a> for Finder<'a, 'b> {
        fn visit_function(
            &mut self,
            func: &oxc_ast::ast::Function<'a>,
            flags: oxc_syntax::scope::ScopeFlags,
        ) {
            if self.found.is_some() {
                return;
            }
            if func.span.start == self.node_id {
                use oxc_allocator::CloneIn;
                use oxc_span::SPAN;
                let f = oxc_ast::ast::Function::boxed(
                    SPAN,
                    oxc_ast::ast::FunctionType::FunctionExpression,
                    func.id.clone_in(self.ast.allocator()),
                    func.generator,
                    func.r#async,
                    false,
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
                    None::<oxc_allocator::Box<oxc_ast::ast::TSThisParameter>>,
                    func.params.clone_in(self.ast.allocator()),
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                    func.body.clone_in(self.ast.allocator()),
                    self.ast,
                );
                self.found = Some(oxc_ast::ast::Expression::FunctionExpression(f));
                return;
            }
            oxc_ast_visit::walk::walk_function(self, func, flags);
        }
        fn visit_arrow_function_expression(
            &mut self,
            arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
        ) {
            if self.found.is_some() {
                return;
            }
            if arrow.span.start == self.node_id {
                self.found = Some(oxc_ast::ast::Expression::ArrowFunctionExpression(
                    oxc_allocator::ArenaBox::new_in(arrow.clone_in(self.ast.allocator()), self.ast),
                ));
                return;
            }
            oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
        }
    }
    let _ = (SPAN, ast.allocator());
    let mut finder = Finder { ast, node_id, found: None };
    oxc_ast_visit::Visit::visit_program(&mut finder, program);
    finder.found.map(|e| e.clone_in(ast.allocator()))
}

/// Splice every compiled oxc function into a clone of the original oxc program and
/// add the required imports. Returns the final memoized program.
fn ox_splice_program<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    oxc_program: &oxc_ast::ast::Program<'a>,
    replacements: &[OxcReplacement<'a>],
    context: &mut ProgramContext,
) -> oxc_ast::ast::Program<'a> {
    use oxc_allocator::CloneIn;

    let mut program = oxc_program.clone_in(ast.allocator());

    // Outlined function declarations are placed differently depending on the
    // original function's syntactic kind, mirroring `insertNewOutlinedFunctionNode`
    // in TS `Program.ts`:
    //   - FunctionDeclaration originals: inserted as a sibling immediately after the
    //     original function (Babel `insertAfter`).
    //   - (Arrow)FunctionExpression originals: appended at the end of the program
    //     body (Babel `pushContainer('body', ...)`), since inserting as a sibling
    //     would corrupt the parent expression.
    let mut appended_outlined_decls: Vec<oxc_ast::ast::Statement<'a>> = Vec::new();

    for replacement in replacements {
        let mut sibling_outlined_decls: Vec<oxc_ast::ast::Statement<'a>> = Vec::new();
        let insert_as_sibling = replacement.original_kind == OriginalFnKind::FunctionDeclaration;
        for outlined in &replacement.codegen_fn.outlined {
            let func = ox_build_function(
                ast,
                &outlined.func,
                oxc_ast::ast::FunctionType::FunctionDeclaration,
            );
            let stmt = oxc_ast::ast::Statement::FunctionDeclaration(func);
            if insert_as_sibling {
                sibling_outlined_decls.push(stmt);
            } else {
                appended_outlined_decls.push(stmt);
            }
        }

        if let Some(ref gating_config) = replacement.gating {
            ox_apply_gated_conditional(ast, &mut program, replacement, gating_config, context);
        } else if let Some(node_id) = replacement.fn_node_id {
            let mut visitor =
                OxcReplaceFnVisitor { ast, node_id, codegen: &replacement.codegen_fn, done: false };
            oxc_ast_visit::VisitMut::visit_program(&mut visitor, &mut program);
        }

        if !sibling_outlined_decls.is_empty() {
            if let Some(node_id) = replacement.fn_node_id {
                ox_insert_outlined_after(&mut program, node_id, sibling_outlined_decls);
            }
        }
    }

    // Append outlined function declarations (from expression-parented originals) at
    // the top level.
    program.body.extend(appended_outlined_decls);

    // Register the memo cache import and rename `useMemoCache` references.
    let needs_memo_import = replacements.iter().any(|r| r.codegen_fn.memo_slots_used > 0);
    if needs_memo_import {
        let import_spec = context.add_memo_cache_import();
        let local_name = import_spec.name;
        let mut visitor =
            OxcRenameIdentifierVisitor { ast, old_name: "useMemoCache", new_name: &local_name };
        oxc_ast_visit::VisitMut::visit_program(&mut visitor, &mut program);
    }

    ox_add_imports_to_program(ast, &mut program, context);

    program
}

/// Insert outlined function declarations immediately after the top-level statement
/// that declares the function identified by `node_id`. Mirrors Babel's
/// `originalFn.insertAfter(...)` for `FunctionDeclaration` originals. The statement
/// may be a bare `FunctionDeclaration` or one wrapped in an `export`.
fn ox_insert_outlined_after<'a>(
    program: &mut oxc_ast::ast::Program<'a>,
    node_id: u32,
    outlined_decls: Vec<oxc_ast::ast::Statement<'a>>,
) {
    use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Statement};

    let matches = |stmt: &Statement<'a>| -> bool {
        match stmt {
            Statement::FunctionDeclaration(f) => f.span.start == node_id,
            Statement::ExportNamedDeclaration(e) => {
                matches!(&e.declaration, Some(Declaration::FunctionDeclaration(f)) if f.span.start == node_id)
            }
            Statement::ExportDefaultDeclaration(e) => {
                matches!(&e.declaration, ExportDefaultDeclarationKind::FunctionDeclaration(f) if f.span.start == node_id)
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
    ast: &oxc_ast::builder::AstBuilder<'a>,
    program: &mut oxc_ast::ast::Program<'a>,
    context: &ProgramContext,
) {
    use oxc_span::SPAN;
    if !context.has_pending_imports() {
        return;
    }
    let imports = context.imports();

    // Existing non-namespaced value imports, by module name.
    let mut existing_import_indices: FxHashMap<String, usize> = FxHashMap::default();
    for (idx, stmt) in program.body.iter().enumerate() {
        if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt {
            if ox_is_non_namespaced_import(import) {
                existing_import_indices.entry(import.source.value.to_string()).or_insert(idx);
            }
        }
    }

    let mut sorted_modules: Vec<_> = imports.iter().collect();
    sorted_modules.sort_by(|(a, _), (b, _)| a.to_lowercase().cmp(&b.to_lowercase()));

    let is_module = matches!(program.source_type.module_kind(), oxc_span::ModuleKind::Module);

    let mut new_stmts: Vec<oxc_ast::ast::Statement<'a>> = Vec::new();

    for (module_name, imports_map) in sorted_modules {
        let mut sorted_imports: Vec<_> = imports_map.values().collect();
        sorted_imports.sort_by(|a, b| a.imported.cmp(&b.imported));

        if let Some(&idx) = existing_import_indices.get(module_name) {
            // Merge into the existing import declaration.
            if let oxc_ast::ast::Statement::ImportDeclaration(import) = &mut program.body[idx] {
                let specifiers =
                    import.specifiers.get_or_insert_with(|| oxc_allocator::ArenaVec::new_in(ast));
                for spec in &sorted_imports {
                    specifiers.push(ox_make_import_specifier(ast, spec));
                }
            }
        } else if is_module {
            // ESM: import { imported as local, ... } from 'module'
            let mut specifiers = oxc_allocator::ArenaVec::new_in(ast);
            for spec in &sorted_imports {
                specifiers.push(ox_make_import_specifier(ast, spec));
            }
            let source =
                oxc_ast::ast::StringLiteral::new(SPAN, ox_atom(ast, module_name), None, ast);
            let import = oxc_ast::ast::ImportDeclaration::boxed(
                SPAN,
                Some(specifiers),
                source,
                None,
                None::<oxc_allocator::Box<oxc_ast::ast::WithClause>>,
                oxc_ast::ast::ImportOrExportKind::Value,
                ast,
            );
            new_stmts.push(oxc_ast::ast::Statement::ImportDeclaration(import));
        } else {
            // CommonJS: const { imported: local, ... } = require('module')
            let mut props = oxc_allocator::ArenaVec::new_in(ast);
            for spec in &sorted_imports {
                let key = oxc_ast::ast::PropertyKey::new_static_identifier(
                    SPAN,
                    ox_atom(ast, &spec.imported),
                    ast,
                );
                let value = oxc_ast::ast::BindingPattern::new_binding_identifier(
                    SPAN,
                    ox_atom(ast, &spec.name),
                    ast,
                );
                props.push(oxc_ast::ast::BindingProperty::new(SPAN, key, value, false, false, ast));
            }
            let object_pattern = oxc_ast::ast::BindingPattern::new_object_pattern(
                SPAN,
                props,
                None::<oxc_allocator::Box<oxc_ast::ast::BindingRestElement>>,
                ast,
            );
            let require_call = oxc_ast::ast::Expression::new_call_expression(
                SPAN,
                oxc_ast::ast::Expression::new_identifier(SPAN, "require", ast),
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
                oxc_allocator::ArenaVec::from_value_in(
                    oxc_ast::ast::Argument::from(oxc_ast::ast::Expression::new_string_literal(
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
            let declarator = oxc_ast::ast::VariableDeclarator::new(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                object_pattern,
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                Some(require_call),
                false,
                ast,
            );
            let decl = oxc_ast::ast::VariableDeclaration::boxed(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                oxc_allocator::ArenaVec::from_value_in(declarator, ast),
                false,
                ast,
            );
            new_stmts.push(oxc_ast::ast::Statement::VariableDeclaration(decl));
        }
    }

    if !new_stmts.is_empty() {
        let old_body = std::mem::replace(&mut program.body, oxc_allocator::ArenaVec::new_in(ast));
        program.body.extend(new_stmts);
        program.body.extend(old_body);
    }
}

/// Build an oxc named import specifier `imported as local`. Mirrors `make_import_specifier`.
fn ox_make_import_specifier<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    spec: &super::imports::NonLocalImportSpecifier,
) -> oxc_ast::ast::ImportDeclarationSpecifier<'a> {
    use oxc_span::SPAN;
    let imported = oxc_ast::ast::ModuleExportName::IdentifierName(
        oxc_ast::ast::IdentifierName::new(SPAN, ox_atom(ast, &spec.imported), ast),
    );
    let local = oxc_ast::ast::BindingIdentifier::new(SPAN, ox_atom(ast, &spec.name), ast);
    oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(oxc_ast::ast::ImportSpecifier::boxed(
        SPAN,
        imported,
        local,
        oxc_ast::ast::ImportOrExportKind::Value,
        ast,
    ))
}

/// Whether an import declaration is a non-namespaced value import. Mirrors
/// `is_non_namespaced_import`.
fn ox_is_non_namespaced_import(import: &oxc_ast::ast::ImportDeclaration) -> bool {
    if !matches!(import.import_kind, oxc_ast::ast::ImportOrExportKind::Value) {
        return false;
    }
    match &import.specifiers {
        None => true,
        Some(specifiers) => specifiers
            .iter()
            .all(|s| matches!(s, oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(_))),
    }
}

/// Main entry point for the React Compiler.
///
/// Receives a full program AST, scope information (unused for now), and resolved options.
/// Returns a CompileResult indicating whether the AST was modified,
/// along with any logger events.
///
/// This function implements the logic from the TS entrypoint (Program.ts):
/// - shouldSkipCompilation: check for existing runtime imports
/// - validateRestrictedImports: check for blocklisted imports
/// - findProgramSuppressions: find eslint/flow suppression comments
/// - findFunctionsToCompile: traverse program to find components and hooks
/// - processFn: per-function compilation with directive and suppression handling
/// - applyCompiledFunctions: replace original functions with compiled versions
pub fn compile_program<'a, 'p>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    oxc_program: &'p oxc_ast::ast::Program<'a>,
    scope: ScopeInfo,
    options: PluginOptions,
) -> CompileResult<'a> {
    // Compute output mode once, up front
    let output_mode = CompilerOutputMode::from_opts(&options);

    // Debug log accumulated on early-return paths (before the full context exists).
    let mut early_ordered_log: Vec<OrderedLogItem> = Vec::new();

    // Log environment config for debugLogIRs
    if options.debug {
        early_ordered_log.push(OrderedLogItem::Debug {
            entry: DebugLogEntry::new("EnvironmentConfig", format!("{:#?}", options.environment)),
        });
    }

    let program = oxc_program;

    // Check for existing runtime imports (file already compiled)
    if should_skip_compilation(program, &options) {
        return CompileResult::Success {
            ast: None,
            diagnostics: Diagnostics::new(),
            ordered_log: early_ordered_log,
            renames: Vec::new(),
        };
    }

    // Validate restricted imports from the environment config
    let restricted_imports = options.environment.validate_blocklisted_imports.clone();

    // Determine if we should check for eslint suppressions
    let validate_exhaustive = options.environment.validate_exhaustive_memoization_dependencies;
    let validate_hooks = options.environment.validate_hooks_usage;

    let eslint_rules: Option<Vec<String>> =
        if validate_exhaustive && validate_hooks {
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
    let module_directives: Vec<String> =
        program.directives.iter().map(|d| d.expression.value.to_string()).collect();
    let has_module_scope_opt_out =
        find_directive_disabling_memoization(&module_directives, &options).is_some();

    // Create program context
    let mut context = ProgramContext::new(
        options.clone(),
        // Source text feeds the line-offset table (diagnostic line/col) and the fast
        // refresh hash. The oxc front-end derives locations from it on demand.
        Some(program.source_text.to_string()),
        suppressions,
        has_module_scope_opt_out,
    );

    // Initialize known referenced names from scope bindings for UID collision detection
    context.init_from_scope(&scope);

    // Seed context with early ordered log entries
    context.ordered_log.extend(early_ordered_log);

    // Validate restricted imports (needs context for handle_error)
    if let Some(err) = validate_restricted_imports(program, &restricted_imports) {
        if let Some(result) = handle_error(&err, None, &mut context) {
            return result;
        }
        return CompileResult::Success {
            ast: None,
            diagnostics: context.diagnostics,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
        };
    }

    // Pre-register instrumentation imports to get stable local names.
    // These are needed before compilation so codegen can use the correct names.
    let instrument_fn_name: Option<String>;
    let instrument_gating_name: Option<String>;
    let hook_guard_name: Option<String>;

    if let Some(ref instrument_config) = options.environment.enable_emit_instrument_forget {
        let fn_spec = context.add_import_specifier(
            &instrument_config.fn_.source,
            &instrument_config.fn_.import_specifier_name,
            None,
        );
        instrument_fn_name = Some(fn_spec.name.clone());
        instrument_gating_name = instrument_config.gating.as_ref().map(|g| {
            let spec = context.add_import_specifier(&g.source, &g.import_specifier_name, None);
            spec.name.clone()
        });
    } else {
        instrument_fn_name = None;
        instrument_gating_name = None;
    }

    if let Some(ref hook_guard_config) = options.environment.enable_emit_hook_guards {
        let spec = context.add_import_specifier(
            &hook_guard_config.source,
            &hook_guard_config.import_specifier_name,
            None,
        );
        hook_guard_name = Some(spec.name.clone());
    } else {
        hook_guard_name = None;
    }

    // Store pre-resolved names on context for pipeline access
    context.instrument_fn_name = instrument_fn_name;
    context.instrument_gating_name = instrument_gating_name;
    context.hook_guard_name = hook_guard_name;

    // Find all functions to compile
    let queue = find_functions_to_compile(program, &options, &mut context, &scope);

    // Clone env_config once for all function compilations (avoids per-function clone
    // while satisfying the borrow checker — compile_fn needs &mut context + &env_config)
    let env_config = options.environment.clone();

    // Process each function and collect compiled results
    let mut compiled_fns: Vec<CompiledFunction<'_, '_, '_>> = Vec::new();

    for source in &queue {
        match process_fn(ast, source, &scope, output_mode, &env_config, &mut context) {
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
            let mut err = CompilerError::new();
            err.push_error_detail(CompilerErrorDetail::new(
                ErrorCategory::Invariant,
                "Unexpected compiled functions when module scope opt-out is present",
            ));
            handle_error(&err, None, &mut context);
        }
        return CompileResult::Success {
            ast: None,
            diagnostics: context.diagnostics,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
        };
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
                fn_node_id: cf.source.fn_node_id,
                original_kind: cf.source.original_kind,
                codegen_fn: cf.codegen_fn,
                gating,
            }
        })
        .collect();

    // Drop the discovery results (and their borrows of `file.program`).
    drop(queue);

    if replacements.is_empty() {
        // No functions to replace. Return renames for the Babel plugin to apply
        // (e.g., variable shadowing renames in lint mode). Imports are NOT added
        // when there are no replacements — matching TS behavior where
        // addImportsToProgram is only called when compiledFns.length > 0.
        return CompileResult::Success {
            ast: None,
            diagnostics: context.diagnostics,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
        };
    }

    // Build the memoized oxc program: splice each compiled oxc function in for its
    // original (matched by `span.start == fn_node_id`), apply gating, insert outlined
    // functions, and add the memo-cache / gating imports.
    let compiled_program = ox_splice_program(ast, oxc_program, &replacements, &mut context);

    CompileResult::Success {
        ast: Some(compiled_program),
        diagnostics: context.diagnostics,
        ordered_log: context.ordered_log,
        renames: convert_renames(&context.renames),
    }
}

/// Convert internal BindingRename structs to the serializable BindingRenameInfo format.
fn convert_renames(
    renames: &[crate::react_compiler_hir::environment::BindingRename],
) -> Vec<BindingRenameInfo> {
    renames
        .iter()
        .map(|r| BindingRenameInfo {
            original: r.original.clone(),
            renamed: r.renamed.clone(),
            declaration_start: r.declaration_start,
        })
        .collect()
}
