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

use rustc_hash::FxHashMap;

use crate::react_compiler_ast::File;
use crate::react_compiler_ast::Program;
use crate::react_compiler_ast::common::BaseNode;
use crate::react_compiler_ast::declarations::ImportSpecifier;
use crate::react_compiler_ast::declarations::ModuleExportName;
use crate::react_compiler_ast::expressions::*;
use crate::react_compiler_ast::patterns::PatternLike;
use crate::react_compiler_ast::scope::ScopeId;
use crate::react_compiler_ast::scope::ScopeInfo;
use crate::react_compiler_ast::statements::*;
use crate::react_compiler_ast::visitor::AstWalker;
use crate::react_compiler_ast::visitor::Visitor;
use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::CompilerErrorOrDiagnostic;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_diagnostics::SourceLocation;
use crate::react_compiler_hir::ReactFunctionType;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
use crate::react_compiler_lowering::FunctionNode;

use super::compile_result::BindingRenameInfo;
use super::compile_result::CodegenFunction;
use super::compile_result::CompileResult;
use super::compile_result::CompilerErrorDetailInfo;
use super::compile_result::CompilerErrorInfo;
use super::compile_result::CompilerErrorItemInfo;
use super::compile_result::DebugLogEntry;
use super::compile_result::LoggerEvent;
use super::compile_result::LoggerPosition;
use super::compile_result::LoggerSourceLocation;
use super::compile_result::LoggerSuggestionInfo;
use super::compile_result::LoggerSuggestionOp;
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

/// A function found in the program that should be compiled
#[allow(dead_code)]
struct CompileSource {
    kind: CompileSourceKind,
    original_kind: OriginalFnKind,
    /// Location of this function in the AST for logging
    fn_name: Option<String>,
    fn_loc: Option<SourceLocation>,
    /// Original AST source location (with index and filename) for logger events.
    fn_ast_loc: Option<crate::react_compiler_ast::common::SourceLocation>,
    fn_start: Option<u32>,
    fn_end: Option<u32>,
    fn_node_id: Option<u32>,
    fn_type: ReactFunctionType,
    /// Directives from the function body (for opt-in/opt-out checks)
    body_directives: Vec<Directive>,
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
/// Returns the first matching directive, or None.
///
/// Also checks for dynamic gating directives (`use memo if(...)`)
fn try_find_directive_enabling_memoization<'a>(
    directives: &'a [Directive],
    opts: &PluginOptions,
) -> Result<Option<&'a Directive>, CompilerError> {
    // Check standard opt-in directives
    let opt_in = directives.iter().find(|d| OPT_IN_DIRECTIVES.contains(&d.value.value.as_str()));
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
    directives: &'a [Directive],
    opts: &PluginOptions,
) -> Option<&'a Directive> {
    if let Some(ref custom_directives) = opts.custom_opt_out_directives {
        directives.iter().find(|d| custom_directives.contains(&d.value.value))
    } else {
        directives.iter().find(|d| OPT_OUT_DIRECTIVES.contains(&d.value.value.as_str()))
    }
}

/// Result of a dynamic gating directive parse.
struct DynamicGatingResult<'a> {
    #[allow(dead_code)]
    directive: &'a Directive,
    gating: GatingConfig,
}

/// Check for dynamic gating directives like `use memo if(identifier)`.
/// Returns the directive and gating config if found, or an error if malformed.
fn find_directives_dynamic_gating<'a>(
    directives: &'a [Directive],
    opts: &PluginOptions,
) -> Result<Option<DynamicGatingResult<'a>>, CompilerError> {
    let dynamic_gating = match &opts.dynamic_gating {
        Some(dg) => dg,
        None => return Ok(None),
    };

    let mut errors: Vec<CompilerErrorDetail> = Vec::new();
    let mut matches: Vec<(&'a Directive, String)> = Vec::new();

    for directive in directives {
        if let Some(ident) = parse_dynamic_gating_directive(&directive.value.value) {
            if is_valid_identifier(ident) {
                matches.push((directive, ident.to_string()));
            } else {
                let mut detail = CompilerErrorDetail::new(
                    ErrorCategory::Gating,
                    "Dynamic gating directive is not a valid JavaScript identifier",
                )
                .with_description(format!("Found '{}'", directive.value.value));
                detail.loc = directive.base.loc.as_ref().map(convert_loc);
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
        let names: Vec<String> = matches.iter().map(|(d, _)| d.value.value.clone()).collect();
        let mut err = CompilerError::new();
        let mut detail = CompilerErrorDetail::new(
            ErrorCategory::Gating,
            "Multiple dynamic gating directives found",
        )
        .with_description(format!("Expected a single directive but found [{}]", names.join(", ")));
        detail.loc = matches[0].0.base.loc.as_ref().map(convert_loc);
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
fn expr_is_hook(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(id) => is_hook_name(&id.name),
        Expression::MemberExpression(member) => {
            if member.computed {
                return false;
            }
            // Property must be a hook name
            if !expr_is_hook(&member.property) {
                return false;
            }
            // Object must be a PascalCase identifier
            if let Expression::Identifier(obj) = member.object.as_ref() {
                obj.name.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if an expression is a React API call (e.g., `forwardRef` or `React.forwardRef`).
#[allow(dead_code)]
fn is_react_api(expr: &Expression, function_name: &str) -> bool {
    match expr {
        Expression::Identifier(id) => id.name == function_name,
        Expression::MemberExpression(member) => {
            if let Expression::Identifier(obj) = member.object.as_ref() {
                if obj.name == "React" {
                    if let Expression::Identifier(prop) = member.property.as_ref() {
                        return prop.name == function_name;
                    }
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
fn get_function_name_from_id(id: Option<&Identifier>) -> Option<String> {
    id.map(|id| id.name.clone())
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
        // Skip nested function/class declarations -- they have their own returns
        Statement::FunctionDeclaration(_) | Statement::ClassDeclaration(_) => {}
        // Unmodeled statements are opaque to return analysis; functions
        // containing them bail out in lowering before this matters.
        Statement::Unknown(_) => {}
        _ => {}
    }
}

/// Check if a function returns non-node values.
/// For arrow functions with expression body, checks the expression directly.
/// For block bodies, walks the statements.
fn returns_non_node_fn(params: &[PatternLike], body: &FunctionBody) -> bool {
    let _ = params;
    match body {
        FunctionBody::Block(block) => returns_non_node_in_stmts(&block.body),
        FunctionBody::Expression(expr) => is_non_node(expr),
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
                if let Some(ref init) = decl.init {
                    if calls_hooks_or_creates_jsx_in_expr(init) {
                        return true;
                    }
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
                    .map_or(false, |alt| calls_hooks_or_creates_jsx_in_stmt(alt))
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                match init.as_ref() {
                    ForInit::Expression(expr) => {
                        if calls_hooks_or_creates_jsx_in_expr(expr) {
                            return true;
                        }
                    }
                    ForInit::VariableDeclaration(var_decl) => {
                        for decl in &var_decl.declarations {
                            if let Some(ref init) = decl.init {
                                if calls_hooks_or_creates_jsx_in_expr(init) {
                                    return true;
                                }
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
        Statement::ThrowStatement(throw) => calls_hooks_or_creates_jsx_in_expr(&throw.argument),
        Statement::TryStatement(try_stmt) => {
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
        Statement::LabeledStatement(labeled) => calls_hooks_or_creates_jsx_in_stmt(&labeled.body),
        Statement::WithStatement(with) => {
            calls_hooks_or_creates_jsx_in_expr(&with.object)
                || calls_hooks_or_creates_jsx_in_stmt(&with.body)
        }
        // Recurse into class body to find JSX/hooks in methods (matching TS behavior
        // where Babel's traverse enters class bodies, only skipping nested functions)
        Statement::FunctionDeclaration(_) => false,
        Statement::ClassDeclaration(class) => calls_hooks_or_creates_jsx_in_class_body(&class.body),
        // Unmodeled statements are preserved verbatim and never compiled, so
        // hook/JSX content inside them cannot affect compilation decisions.
        Statement::Unknown(_) => false,
        _ => false,
    }
}

fn calls_hooks_or_creates_jsx_in_expr(expr: &Expression) -> bool {
    match expr {
        // JSX creates
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,

        // Hook calls
        Expression::CallExpression(call) => {
            if expr_is_hook(&call.callee) {
                return true;
            }
            // Also check arguments for JSX/hooks (but not nested functions)
            if calls_hooks_or_creates_jsx_in_expr(&call.callee) {
                return true;
            }
            for arg in &call.arguments {
                // Skip function arguments -- they are nested functions
                if matches!(
                    arg,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                    continue;
                }
                if calls_hooks_or_creates_jsx_in_expr(arg) {
                    return true;
                }
            }
            false
        }
        Expression::OptionalCallExpression(call) => {
            // Note: OptionalCallExpression is NOT treated as a hook call for
            // the purpose of determining function type. The TS code only checks
            // regular CallExpression nodes in callsHooksOrCreatesJsx.
            // We still recurse into the callee and arguments to find other
            // hook calls or JSX.
            if calls_hooks_or_creates_jsx_in_expr(&call.callee) {
                return true;
            }
            for arg in &call.arguments {
                if matches!(
                    arg,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                    continue;
                }
                if calls_hooks_or_creates_jsx_in_expr(arg) {
                    return true;
                }
            }
            false
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
            seq.expressions.iter().any(|e| calls_hooks_or_creates_jsx_in_expr(e))
        }
        Expression::UnaryExpression(unary) => calls_hooks_or_creates_jsx_in_expr(&unary.argument),
        Expression::UpdateExpression(update) => {
            calls_hooks_or_creates_jsx_in_expr(&update.argument)
        }
        Expression::MemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
                || calls_hooks_or_creates_jsx_in_expr(&member.property)
        }
        Expression::OptionalMemberExpression(member) => {
            calls_hooks_or_creates_jsx_in_expr(&member.object)
                || calls_hooks_or_creates_jsx_in_expr(&member.property)
        }
        Expression::SpreadElement(spread) => calls_hooks_or_creates_jsx_in_expr(&spread.argument),
        Expression::AwaitExpression(await_expr) => {
            calls_hooks_or_creates_jsx_in_expr(&await_expr.argument)
        }
        Expression::YieldExpression(yield_expr) => yield_expr
            .argument
            .as_ref()
            .map_or(false, |arg| calls_hooks_or_creates_jsx_in_expr(arg)),
        Expression::TaggedTemplateExpression(tagged) => {
            calls_hooks_or_creates_jsx_in_expr(&tagged.tag)
                || tagged.quasi.expressions.iter().any(|e| calls_hooks_or_creates_jsx_in_expr(e))
        }
        Expression::TemplateLiteral(tl) => {
            tl.expressions.iter().any(|e| calls_hooks_or_creates_jsx_in_expr(e))
        }
        Expression::ArrayExpression(arr) => arr
            .elements
            .iter()
            .any(|e| e.as_ref().map_or(false, |e| calls_hooks_or_creates_jsx_in_expr(e))),
        Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| match prop {
            ObjectExpressionProperty::ObjectProperty(p) => {
                calls_hooks_or_creates_jsx_in_expr(&p.value)
            }
            ObjectExpressionProperty::SpreadElement(s) => {
                calls_hooks_or_creates_jsx_in_expr(&s.argument)
            }
            // ObjectMethod: traverse into its body to find hooks/JSX.
            // This matches the TS behavior where Babel's traverse enters
            // ObjectMethod (only FunctionDeclaration, FunctionExpression,
            // and ArrowFunctionExpression are skipped).
            ObjectExpressionProperty::ObjectMethod(m) => {
                calls_hooks_or_creates_jsx_in_stmts(&m.body.body)
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
        Expression::TypeCastExpression(tc) => calls_hooks_or_creates_jsx_in_expr(&tc.expression),
        Expression::NewExpression(new) => {
            if calls_hooks_or_creates_jsx_in_expr(&new.callee) {
                return true;
            }
            new.arguments.iter().any(|a| {
                if matches!(
                    a,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                    return false;
                }
                calls_hooks_or_creates_jsx_in_expr(a)
            })
        }

        // Skip nested functions
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => false,

        // Recurse into class body to find JSX/hooks in methods
        Expression::ClassExpression(class) => calls_hooks_or_creates_jsx_in_class_body(&class.body),

        // Leaf expressions
        _ => false,
    }
}

/// Recursively search a ClassBody for JSX elements or hook calls.
/// Class body members are stored as serde_json::Value since they aren't fully typed.
/// We search the JSON tree, skipping nested function nodes (matching TS behavior where
/// Babel's traverse skips ArrowFunctionExpression, FunctionExpression, FunctionDeclaration
/// but recurses into class methods).
fn calls_hooks_or_creates_jsx_in_class_body(
    body: &crate::react_compiler_ast::expressions::ClassBody,
) -> bool {
    body.body.iter().any(|member| member.contains_hook_or_jsx)
}

/// Check if a function body calls hooks or creates JSX.
fn calls_hooks_or_creates_jsx(params: &[PatternLike], body: &FunctionBody) -> bool {
    // Check default param values (TS traverses the whole function node including params)
    if calls_hooks_or_creates_jsx_in_params(params) {
        return true;
    }
    match body {
        FunctionBody::Block(block) => calls_hooks_or_creates_jsx_in_stmts(&block.body),
        FunctionBody::Expression(expr) => calls_hooks_or_creates_jsx_in_expr(expr),
    }
}

/// Check if any parameter default values contain hooks or JSX.
fn calls_hooks_or_creates_jsx_in_params(params: &[PatternLike]) -> bool {
    for param in params {
        if calls_hooks_or_creates_jsx_in_pattern(param) {
            return true;
        }
    }
    false
}

fn calls_hooks_or_creates_jsx_in_pattern(pattern: &PatternLike) -> bool {
    match pattern {
        PatternLike::AssignmentPattern(assign) => {
            // Check the default value expression
            calls_hooks_or_creates_jsx_in_expr(&assign.right)
                || calls_hooks_or_creates_jsx_in_pattern(&assign.left)
        }
        PatternLike::ObjectPattern(obj) => obj.properties.iter().any(|prop| match prop {
            crate::react_compiler_ast::patterns::ObjectPatternProperty::ObjectProperty(p) => {
                calls_hooks_or_creates_jsx_in_pattern(&p.value)
            }
            crate::react_compiler_ast::patterns::ObjectPatternProperty::RestElement(rest) => {
                calls_hooks_or_creates_jsx_in_pattern(&rest.argument)
            }
        }),
        PatternLike::ArrayPattern(arr) => arr
            .elements
            .iter()
            .any(|elem| elem.as_ref().map_or(false, |e| calls_hooks_or_creates_jsx_in_pattern(e))),
        PatternLike::RestElement(rest) => calls_hooks_or_creates_jsx_in_pattern(&rest.argument),
        PatternLike::Identifier(_)
        | PatternLike::MemberExpression(_)
        | PatternLike::TSAsExpression(_)
        | PatternLike::TSSatisfiesExpression(_)
        | PatternLike::TSNonNullExpression(_)
        | PatternLike::TSTypeAssertion(_)
        | PatternLike::TypeCastExpression(_) => false,
    }
}

/// Check if the function parameters are valid for a React component.
/// Components can have 0 params, 1 param (props), or 2 params (props + ref).
/// Check if a parameter's type annotation is valid for a React component prop.
/// Returns false for primitive type annotations that indicate this is NOT a component.
fn is_valid_props_annotation(param: &PatternLike) -> bool {
    let type_annotation = match param {
        PatternLike::Identifier(id) => id.type_annotation.as_ref(),
        PatternLike::ObjectPattern(op) => op.type_annotation.as_ref(),
        PatternLike::ArrayPattern(ap) => ap.type_annotation.as_ref(),
        PatternLike::AssignmentPattern(ap) => ap.type_annotation.as_ref(),
        PatternLike::RestElement(re) => re.type_annotation.as_ref(),
        PatternLike::MemberExpression(_)
        | PatternLike::TSAsExpression(_)
        | PatternLike::TSSatisfiesExpression(_)
        | PatternLike::TSNonNullExpression(_)
        | PatternLike::TSTypeAssertion(_)
        | PatternLike::TypeCastExpression(_) => None,
    };
    let Some(raw) = type_annotation else {
        return true; // No annotation = valid
    };
    // `node_type` is the pre-extracted, unwrapped inner type tag. The TS and Flow
    // disallowed type names are disjoint, so one membership test covers both.
    let Some(inner_type) = raw.node_type.as_deref() else {
        return true;
    };
    !matches!(
        inner_type,
        // TS
        "TSArrayType"
            | "TSBigIntKeyword"
            | "TSBooleanKeyword"
            | "TSConstructorType"
            | "TSFunctionType"
            | "TSLiteralType"
            | "TSNeverKeyword"
            | "TSNumberKeyword"
            | "TSStringKeyword"
            | "TSSymbolKeyword"
            | "TSTupleType"
            // Flow
            | "ArrayTypeAnnotation"
            | "BooleanLiteralTypeAnnotation"
            | "BooleanTypeAnnotation"
            | "EmptyTypeAnnotation"
            | "FunctionTypeAnnotation"
            | "NullLiteralTypeAnnotation"
            | "NumberLiteralTypeAnnotation"
            | "NumberTypeAnnotation"
            | "StringLiteralTypeAnnotation"
            | "StringTypeAnnotation"
            | "SymbolTypeAnnotation"
            | "ThisTypeAnnotation"
            | "TupleTypeAnnotation"
    )
}

fn is_valid_component_params(params: &[PatternLike]) -> bool {
    if params.is_empty() {
        return true;
    }
    if params.len() > 2 {
        return false;
    }
    // First param cannot be a rest element
    if matches!(params[0], PatternLike::RestElement(_)) {
        return false;
    }
    // Check type annotation on first param
    if !is_valid_props_annotation(&params[0]) {
        return false;
    }
    if params.len() == 1 {
        return true;
    }
    // If second param exists, it should look like a ref
    if let PatternLike::Identifier(ref id) = params[1] {
        id.name.contains("ref") || id.name.contains("Ref")
    } else {
        false
    }
}

// -----------------------------------------------------------------------
// Unified function body type for traversal
// -----------------------------------------------------------------------

/// Abstraction over function body types to simplify traversal code
enum FunctionBody<'a> {
    Block(&'a BlockStatement),
    Expression(&'a Expression),
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
    params: &[PatternLike],
    body: &FunctionBody,
    body_directives: &[Directive],
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
    params: &[PatternLike],
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
fn get_callee_name_if_react_api(callee: &Expression) -> Option<&str> {
    match callee {
        Expression::Identifier(id) => {
            if id.name == "forwardRef" || id.name == "memo" {
                Some(&id.name)
            } else {
                None
            }
        }
        Expression::MemberExpression(member) => {
            if let Expression::Identifier(obj) = member.object.as_ref() {
                if obj.name == "React" {
                    if let Expression::Identifier(prop) = member.property.as_ref() {
                        if prop.name == "forwardRef" || prop.name == "memo" {
                            return Some(&prop.name);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

// -----------------------------------------------------------------------
// SourceLocation conversion
// -----------------------------------------------------------------------

/// Convert an AST SourceLocation to a diagnostics SourceLocation
fn convert_loc(loc: &crate::react_compiler_ast::common::SourceLocation) -> SourceLocation {
    SourceLocation {
        start: crate::react_compiler_diagnostics::Position {
            line: loc.start.line,
            column: loc.start.column,
            index: loc.start.index,
        },
        end: crate::react_compiler_diagnostics::Position {
            line: loc.end.line,
            column: loc.end.column,
            index: loc.end.index,
        },
    }
}

fn base_node_loc(base: &BaseNode) -> Option<SourceLocation> {
    base.loc.as_ref().map(convert_loc)
}

// -----------------------------------------------------------------------
// Error handling
// -----------------------------------------------------------------------

/// Convert CompilerDiagnostic details into serializable CompilerErrorItemInfo items.
fn diagnostic_details_to_items(
    d: &crate::react_compiler_diagnostics::CompilerDiagnostic,
    filename: Option<&str>,
) -> Option<Vec<CompilerErrorItemInfo>> {
    let items: Vec<CompilerErrorItemInfo> = d
        .details
        .iter()
        .map(|item| match item {
            crate::react_compiler_diagnostics::CompilerDiagnosticDetail::Error {
                loc,
                message,
                identifier_name,
            } => CompilerErrorItemInfo {
                kind: "error".to_string(),
                loc: loc.as_ref().map(|l| {
                    let mut logger_loc = diag_loc_to_logger_loc(l, filename);
                    logger_loc.identifier_name = identifier_name.clone();
                    logger_loc
                }),
                message: message.clone(),
            },
            crate::react_compiler_diagnostics::CompilerDiagnosticDetail::Hint { message } => {
                CompilerErrorItemInfo {
                    kind: "hint".to_string(),
                    loc: None,
                    message: Some(message.clone()),
                }
            }
        })
        .collect();
    if items.is_empty() { None } else { Some(items) }
}

/// Convert an optional AST SourceLocation to a LoggerSourceLocation with filename.
fn to_logger_loc(
    ast_loc: Option<&crate::react_compiler_ast::common::SourceLocation>,
    filename: Option<&str>,
) -> Option<LoggerSourceLocation> {
    ast_loc.map(|loc| LoggerSourceLocation {
        start: LoggerPosition {
            line: loc.start.line,
            column: loc.start.column,
            index: loc.start.index,
        },
        end: LoggerPosition { line: loc.end.line, column: loc.end.column, index: loc.end.index },
        filename: filename.map(|s| s.to_string()),
        identifier_name: loc.identifier_name.clone(),
    })
}

/// Convert a diagnostics SourceLocation to a LoggerSourceLocation with filename.
fn diag_loc_to_logger_loc(loc: &SourceLocation, filename: Option<&str>) -> LoggerSourceLocation {
    LoggerSourceLocation {
        start: LoggerPosition {
            line: loc.start.line,
            column: loc.start.column,
            index: loc.start.index,
        },
        end: LoggerPosition { line: loc.end.line, column: loc.end.column, index: loc.end.index },
        filename: filename.map(|s| s.to_string()),
        identifier_name: None,
    }
}

/// Convert diagnostic suggestions to logger suggestion infos.
fn suggestions_to_logger(
    suggestions: &Option<Vec<crate::react_compiler_diagnostics::CompilerSuggestion>>,
) -> Option<Vec<LoggerSuggestionInfo>> {
    suggestions.as_ref().map(|suggestions| {
        suggestions
            .iter()
            .map(|s| {
                let op = match s.op {
                    crate::react_compiler_diagnostics::CompilerSuggestionOperation::InsertBefore => {
                        LoggerSuggestionOp::InsertBefore
                    }
                    crate::react_compiler_diagnostics::CompilerSuggestionOperation::InsertAfter => {
                        LoggerSuggestionOp::InsertAfter
                    }
                    crate::react_compiler_diagnostics::CompilerSuggestionOperation::Remove => {
                        LoggerSuggestionOp::Remove
                    }
                    crate::react_compiler_diagnostics::CompilerSuggestionOperation::Replace => {
                        LoggerSuggestionOp::Replace
                    }
                };
                LoggerSuggestionInfo {
                    description: s.description.clone(),
                    op,
                    range: s.range,
                    text: s.text.clone(),
                }
            })
            .collect()
    })
}

/// Log an error as LoggerEvent(s) directly onto the ProgramContext.
fn log_error(
    err: &CompilerError,
    fn_ast_loc: Option<&crate::react_compiler_ast::common::SourceLocation>,
    context: &mut ProgramContext,
) {
    // Use the filename from the AST node's loc (set by parser's sourceFilename option),
    // not from plugin options (which may have a different prefix like '/').
    let source_filename = fn_ast_loc.and_then(|loc| loc.filename.as_deref());
    let fn_loc = to_logger_loc(fn_ast_loc, source_filename);

    // Detect simulated unknown exception (throwUnknownException__testonly).
    // In TS, non-CompilerError exceptions are logged as PipelineError with the
    // error message as data. Emit the same event shape.
    let is_simulated_unknown = err.details.len() == 1
        && err.details.iter().all(|d| match d {
            CompilerErrorOrDiagnostic::ErrorDetail(d) => {
                d.category == ErrorCategory::Invariant && d.reason == "unexpected error"
            }
            _ => false,
        });
    if is_simulated_unknown {
        context.log_event(LoggerEvent::PipelineError {
            fn_loc: fn_loc.clone(),
            data: "Error: unexpected error".to_string(),
        });
        return;
    }

    for detail in &err.details {
        let detail_info = match detail {
            CompilerErrorOrDiagnostic::Diagnostic(d) => CompilerErrorDetailInfo {
                category: format!("{:?}", d.category),
                reason: d.reason.clone(),
                description: d.description.clone(),
                severity: format!("{:?}", d.logged_severity()),
                suggestions: suggestions_to_logger(&d.suggestions),
                details: diagnostic_details_to_items(d, source_filename),
                loc: None,
            },
            CompilerErrorOrDiagnostic::ErrorDetail(d) => CompilerErrorDetailInfo {
                category: format!("{:?}", d.category),
                reason: d.reason.clone(),
                description: d.description.clone(),
                severity: format!("{:?}", d.logged_severity()),
                suggestions: suggestions_to_logger(&d.suggestions),
                details: None,
                loc: d.loc.as_ref().map(|l| diag_loc_to_logger_loc(l, source_filename)),
            },
        };
        // Use CompileErrorWithLoc when fn_loc is present to match TS field ordering
        if let Some(ref loc) = fn_loc {
            context.log_event(LoggerEvent::CompileErrorWithLoc {
                fn_loc: loc.clone(),
                detail: detail_info,
            });
        } else {
            context.log_event(LoggerEvent::CompileError { fn_loc: None, detail: detail_info });
        }
    }
}

/// Handle an error according to the panicThreshold setting.
/// Returns Some(CompileResult::Error) if the error should be surfaced as fatal,
/// otherwise returns None (error was logged only).
fn handle_error<'a>(
    err: &CompilerError,
    fn_ast_loc: Option<&crate::react_compiler_ast::common::SourceLocation>,
    context: &mut ProgramContext,
) -> Option<CompileResult<'a>> {
    // Log the error
    log_error(err, fn_ast_loc, context);

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
        let source_fn = context.source_filename();
        let mut error_info = compiler_error_to_info(err, source_fn.as_deref());

        // Detect simulated unknown exception (throwUnknownException__testonly).
        // In the TS compiler, this throws a plain Error('unexpected error'), not
        // a CompilerError. Set rawMessage so the JS side throws with the raw
        // message instead of formatting through formatCompilerError().
        let is_simulated_unknown = err.details.len() == 1
            && err.details.iter().all(|d| match d {
                CompilerErrorOrDiagnostic::ErrorDetail(d) => {
                    d.category == ErrorCategory::Invariant && d.reason == "unexpected error"
                }
                _ => false,
            });
        if is_simulated_unknown {
            error_info.raw_message = Some("unexpected error".to_string());
        }

        // Pre-format the error message in Rust when possible, so the JS
        // shim can use it directly instead of calling formatCompilerError().
        if error_info.raw_message.is_none() {
            if let Some(ref source) = context.code {
                error_info.formatted_message =
                    Some(crate::react_compiler_diagnostics::code_frame::format_compiler_error(
                        err,
                        source,
                        source_fn.as_deref(),
                    ));
            }
        }

        Some(CompileResult::Error {
            error: error_info,
            events: context.events.clone(),
            ordered_log: context.ordered_log.clone(),
            timing: Vec::new(),
        })
    } else {
        None
    }
}

/// Convert a diagnostics CompilerError to a serializable CompilerErrorInfo.
fn compiler_error_to_info(err: &CompilerError, filename: Option<&str>) -> CompilerErrorInfo {
    let details: Vec<CompilerErrorDetailInfo> = err
        .details
        .iter()
        .map(|d| match d {
            CompilerErrorOrDiagnostic::Diagnostic(d) => CompilerErrorDetailInfo {
                category: format!("{:?}", d.category),
                reason: d.reason.clone(),
                description: d.description.clone(),
                severity: format!("{:?}", d.severity()),
                suggestions: suggestions_to_logger(&d.suggestions),
                details: diagnostic_details_to_items(d, filename),
                loc: None,
            },
            CompilerErrorOrDiagnostic::ErrorDetail(d) => CompilerErrorDetailInfo {
                category: format!("{:?}", d.category),
                reason: d.reason.clone(),
                description: d.description.clone(),
                severity: format!("{:?}", d.severity()),
                suggestions: suggestions_to_logger(&d.suggestions),
                details: None,
                loc: d.loc.as_ref().map(|l| diag_loc_to_logger_loc(l, filename)),
            },
        })
        .collect();

    let (reason, description) = details
        .first()
        .map(|d| (d.reason.clone(), d.description.clone()))
        .unwrap_or_else(|| ("Unknown error".to_string(), None));

    CompilerErrorInfo { reason, description, details, raw_message: None, formatted_message: None }
}

// -----------------------------------------------------------------------
// Compilation pipeline stubs
// -----------------------------------------------------------------------

/// Attempt to compile a single function.
///
/// Returns `CodegenFunction` on success or `CompilerError` on failure.
/// Debug log entries are accumulated on `context.debug_logs`.
fn try_compile_function<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    source: &CompileSource,
    scope_info: &ScopeInfo,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
    fn_map: &FxHashMap<u32, FunctionNode<'_>>,
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

    // Run the compilation pipeline. The discovery records the function's
    // node_id; map it back to the oxc FunctionNode for lowering.
    let fn_node = *fn_map
        .get(&source.fn_node_id.expect("compiled function has a node id"))
        .expect("oxc FunctionNode for discovered function");
    pipeline::compile_fn(
        ast,
        &fn_node,
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
    ast: &oxc_ast::AstBuilder<'a>,
    source: &CompileSource,
    scope_info: &ScopeInfo,
    output_mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
    fn_map: &FxHashMap<u32, FunctionNode<'_>>,
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
            if let Some(result) = handle_error(&err, source.fn_ast_loc.as_ref(), context) {
                return Err(result);
            }
            return Ok(None);
        }
    };

    // Attempt compilation
    let compile_result =
        try_compile_function(ast, source, scope_info, output_mode, env_config, context, fn_map);

    match compile_result {
        Err(err) => {
            // Emit CompileUnexpectedThrow for errors that were "thrown" from a pass
            // (not accumulated via env.record_error) and have all non-Invariant details.
            // Matches TS tryCompileFunction() catch block behavior.
            if err.is_thrown && err.is_all_non_invariant() {
                let source_filename =
                    source.fn_ast_loc.as_ref().and_then(|loc| loc.filename.as_deref());
                context.log_event(LoggerEvent::CompileUnexpectedThrow {
                    fn_loc: to_logger_loc(source.fn_ast_loc.as_ref(), source_filename),
                    data: err.to_string_for_event(),
                });
            }

            if opt_out.is_some() {
                // If there's an opt-out, just log the error (don't escalate)
                log_error(&err, source.fn_ast_loc.as_ref(), context);
            } else {
                // Apply panic threshold logic
                if let Some(result) = handle_error(&err, source.fn_ast_loc.as_ref(), context) {
                    return Err(result);
                }
            }
            Ok(None)
        }
        Ok(codegen_fn) => {
            // Check opt-out
            if !context.opts.ignore_use_no_forget && opt_out.is_some() {
                let opt_out_value = &opt_out.unwrap().value.value;
                let source_filename =
                    source.fn_ast_loc.as_ref().and_then(|loc| loc.filename.as_deref());
                context.log_event(LoggerEvent::CompileSkip {
                    fn_loc: to_logger_loc(source.fn_ast_loc.as_ref(), source_filename),
                    reason: format!("Skipped due to '{}' directive.", opt_out_value),
                    loc: opt_out.and_then(|d| to_logger_loc(d.base.loc.as_ref(), source_filename)),
                });
                // The function is skipped due to opt-out. Do NOT register the memo
                // cache import here — it will be registered in apply_compiled_functions()
                // only for functions that are actually applied to the output.
                return Ok(None);
            }

            // Log success with memo stats from CodegenFunction
            let source_filename =
                source.fn_ast_loc.as_ref().and_then(|loc| loc.filename.as_deref());
            context.log_event(LoggerEvent::CompileSuccess {
                fn_loc: to_logger_loc(source.fn_ast_loc.as_ref(), source_filename),
                fn_name: codegen_fn.id.as_ref().map(|id| id.name.to_string()),
                memo_slots: codegen_fn.memo_slots_used,
                memo_blocks: codegen_fn.memo_blocks,
                memo_values: codegen_fn.memo_values,
                pruned_memo_blocks: codegen_fn.pruned_memo_blocks,
                pruned_memo_values: codegen_fn.pruned_memo_values,
            });

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
fn has_memo_cache_function_import(program: &Program, module_name: &str) -> bool {
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            if import.source.value == module_name {
                for specifier in &import.specifiers {
                    if let ImportSpecifier::ImportSpecifier(data) = specifier {
                        let imported_name = match &data.imported {
                            ModuleExportName::Identifier(id) => Some(id.name.as_str()),
                            ModuleExportName::StringLiteral(s) => s.value.as_str(),
                        };
                        if imported_name == Some("c") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if compilation should be skipped for this program.
fn should_skip_compilation(program: &Program, options: &PluginOptions) -> bool {
    let runtime_module = get_react_compiler_runtime_module(&options.target);
    has_memo_cache_function_import(program, &runtime_module)
}

// -----------------------------------------------------------------------
// Function discovery
// -----------------------------------------------------------------------

/// Information about an expression that might be a function to compile
struct FunctionInfo<'a> {
    name: Option<String>,
    original_kind: OriginalFnKind,
    params: &'a [PatternLike],
    body: FunctionBody<'a>,
    body_directives: Vec<Directive>,
    base: &'a BaseNode,
    parent_callee_name: Option<String>,
    /// True if the node has `__componentDeclaration` set by the Hermes parser (Flow component syntax)
    is_component_declaration: bool,
    /// True if the node has `__hookDeclaration` set by the Hermes parser (Flow hook syntax)
    is_hook_declaration: bool,
}

/// Extract function info from a FunctionDeclaration
fn fn_info_from_decl(decl: &FunctionDeclaration) -> FunctionInfo<'_> {
    FunctionInfo {
        name: get_function_name_from_id(decl.id.as_ref()),
        original_kind: OriginalFnKind::FunctionDeclaration,
        params: &decl.params,
        body: FunctionBody::Block(&decl.body),
        body_directives: decl.body.directives.clone(),
        base: &decl.base,
        parent_callee_name: None,
        is_component_declaration: decl.component_declaration,
        is_hook_declaration: decl.hook_declaration,
    }
}

/// Extract function info from a FunctionExpression
fn fn_info_from_func_expr<'a>(
    expr: &'a FunctionExpression,
    inferred_name: Option<String>,
    parent_callee_name: Option<String>,
) -> FunctionInfo<'a> {
    FunctionInfo {
        name: inferred_name,
        original_kind: OriginalFnKind::FunctionExpression,
        params: &expr.params,
        body: FunctionBody::Block(&expr.body),
        body_directives: expr.body.directives.clone(),
        base: &expr.base,
        parent_callee_name,
        is_component_declaration: false,
        is_hook_declaration: false,
    }
}

/// Extract function info from an ArrowFunctionExpression
fn fn_info_from_arrow<'a>(
    expr: &'a ArrowFunctionExpression,
    inferred_name: Option<String>,
    parent_callee_name: Option<String>,
) -> FunctionInfo<'a> {
    let (body, directives) = match expr.body.as_ref() {
        ArrowFunctionBody::BlockStatement(block) => {
            (FunctionBody::Block(block), block.directives.clone())
        }
        ArrowFunctionBody::Expression(e) => (FunctionBody::Expression(e), Vec::new()),
    };
    FunctionInfo {
        name: inferred_name,
        original_kind: OriginalFnKind::ArrowFunctionExpression,
        params: &expr.params,
        body,
        body_directives: directives,
        base: &expr.base,
        parent_callee_name,
        is_component_declaration: false,
        is_hook_declaration: false,
    }
}

/// Try to create a CompileSource from function info
fn try_make_compile_source<'a>(
    info: FunctionInfo<'a>,
    opts: &PluginOptions,
    context: &mut ProgramContext,
) -> Option<CompileSource> {
    // Skip if already compiled (identified by node_id)
    if let Some(nid) = info.base.node_id {
        if context.is_already_compiled(nid) {
            return None;
        }
    }

    let fn_type = get_react_function_type(
        info.name.as_deref(),
        info.params,
        &info.body,
        &info.body_directives,
        info.is_component_declaration || info.is_hook_declaration,
        info.parent_callee_name.as_deref(),
        opts,
        info.is_component_declaration,
        info.is_hook_declaration,
    )?;

    // Mark as compiled
    if let Some(nid) = info.base.node_id {
        context.mark_compiled(nid);
    }

    Some(CompileSource {
        kind: CompileSourceKind::Original,
        original_kind: info.original_kind,
        fn_name: info.name,
        fn_loc: base_node_loc(info.base),
        fn_ast_loc: info.base.loc.clone(),
        fn_start: info.base.start,
        fn_end: info.base.end,
        fn_node_id: info.base.node_id,
        fn_type,
        body_directives: info.body_directives,
    })
}

/// Get the variable declarator name (for inferring function names from `const Foo = () => {}`)
fn get_declarator_name(decl: &VariableDeclarator) -> Option<String> {
    match &decl.id {
        PatternLike::Identifier(id) => Some(id.name.clone()),
        _ => None,
    }
}

// -----------------------------------------------------------------------
// FunctionDiscoveryVisitor — uses AstWalker to find compilable functions
// -----------------------------------------------------------------------

/// Visitor that discovers functions to compile, matching the TypeScript
/// compiler's Babel `program.traverse` behavior.
///
/// Dynamically controls body traversal via `traverse_function_bodies()`:
/// functions that are queued for compilation have their bodies skipped
/// (matching Babel's `fn.skip()`), while non-compiled functions have their
/// bodies traversed to find nested component/hook declarations.
///
/// Tracks parent context via:
/// - `current_declarator_name`: set by `enter_variable_declarator`, used to
///   infer function names from `const Foo = () => {}`.
/// - `parent_callee_stack`: set by `enter_call_expression`, used to detect
///   forwardRef/memo wrappers around function expressions.
///
/// In 'all' mode, uses `scope_stack.len() > 1` to reject functions that are
/// not at program scope. The walker pushes the program scope first, then
/// nested scopes for for/switch/etc. — so `len() > 1` means the function
/// is inside a nested scope (not at program level), matching Babel's
/// `fn.scope.getProgramParent() !== fn.scope.parent` check.
struct FunctionDiscoveryVisitor<'a> {
    opts: &'a PluginOptions,
    context: &'a mut ProgramContext,
    queue: Vec<CompileSource>,
    /// The inferred name from the current VariableDeclarator, if any.
    current_declarator_name: Option<String>,
    /// Stack tracking callee names of enclosing CallExpressions.
    /// `Some(name)` when the callee is a React API (forwardRef/memo),
    /// `None` for other calls.
    parent_callee_stack: Vec<Option<String>>,
    /// Depth counter for loop expression positions (while.test, for-in.right, etc.).
    /// When > 0, functions are treated as non-program-scope in 'all' mode.
    loop_expression_depth: usize,
    /// Set by enter_* hooks: true when the function was queued for compilation,
    /// meaning the walker should NOT traverse its body (matching Babel's fn.skip()).
    /// When false, the walker DOES traverse the body to find nested declarations.
    skip_body: bool,
}

impl<'a> FunctionDiscoveryVisitor<'a> {
    fn new(opts: &'a PluginOptions, context: &'a mut ProgramContext) -> Self {
        Self {
            opts,
            context,
            queue: Vec::new(),
            current_declarator_name: None,
            parent_callee_stack: Vec::new(),
            loop_expression_depth: 0,
            skip_body: false,
        }
    }

    /// Check if in 'all' mode and the function is inside a nested scope.
    /// The walker pushes the function's own scope BEFORE calling enter hooks,
    /// so scope_stack = [program, ...parents, function_scope]. A top-level
    /// function has len=2 (program + function). Anything deeper means it's
    /// inside a nested scope (for/switch/etc.) and should be rejected.
    /// Also rejects functions found in loop expression positions (while.test,
    /// for-in.right, etc.) where Babel treats the scope as non-program.
    fn is_rejected_by_scope_check(&self, scope_stack: &[ScopeId]) -> bool {
        self.opts.compilation_mode == "all"
            && (scope_stack.len() > 2 || self.loop_expression_depth > 0)
    }

    /// Get the current parent callee name (forwardRef/memo) if any.
    fn current_parent_callee(&self) -> Option<String> {
        self.parent_callee_stack.last().and_then(|opt| opt.clone())
    }
}

impl<'a, 'ast> Visitor<'ast> for FunctionDiscoveryVisitor<'a> {
    fn traverse_function_bodies(&self) -> bool {
        // Dynamic: only skip the body of functions that were queued for compilation.
        // Non-queued functions have their bodies traversed to find nested declarations
        // (matching Babel behavior where fn.skip() is only called for compiled functions).
        !self.skip_body
    }

    fn enter_loop_expression(&mut self) {
        self.loop_expression_depth += 1;
    }

    fn leave_loop_expression(&mut self) {
        self.loop_expression_depth -= 1;
    }

    fn enter_variable_declarator(
        &mut self,
        node: &'ast VariableDeclarator,
        _scope_stack: &[ScopeId],
    ) {
        // Only infer the declarator name when the init is a direct function
        // expression, arrow, or call expression (for forwardRef/memo wrappers).
        // TS checks `path.parentPath.isVariableDeclarator()` which only matches
        // when the function IS the init, not when it's nested inside an object,
        // array, or other expression.
        if let Some(ref init) = node.init {
            match init.as_ref() {
                Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::CallExpression(_) => {
                    self.current_declarator_name = get_declarator_name(node);
                }
                _ => {}
            }
        }
    }

    fn leave_variable_declarator(
        &mut self,
        _node: &'ast VariableDeclarator,
        _scope_stack: &[ScopeId],
    ) {
        self.current_declarator_name = None;
    }

    fn enter_call_expression(&mut self, node: &'ast CallExpression, _scope_stack: &[ScopeId]) {
        let callee_name = get_callee_name_if_react_api(&node.callee).map(|s| s.to_string());
        // In TS, the declarator name only flows through forwardRef/memo calls
        // (path.parentPath.isCallExpression() checks the callee). For any other
        // call expression, clear the name so nested functions don't inherit it.
        if callee_name.is_none() {
            self.current_declarator_name = None;
        }
        self.parent_callee_stack.push(callee_name);
    }

    fn leave_call_expression(&mut self, _node: &'ast CallExpression, _scope_stack: &[ScopeId]) {
        let was_react_api = self.parent_callee_stack.pop().and_then(|name| name).is_some();
        // After a forwardRef/memo call finishes, clear the declarator name.
        // The name is only valid within the call's arguments — if a function
        // inside consumed it via .take(), great; if not, it shouldn't leak
        // to sibling or subsequent expressions.
        if was_react_api {
            self.current_declarator_name = None;
        }
    }

    fn enter_function_declaration(
        &mut self,
        node: &'ast FunctionDeclaration,
        scope_stack: &[ScopeId],
    ) {
        self.skip_body = false;
        if self.is_rejected_by_scope_check(scope_stack) {
            return;
        }
        let info = fn_info_from_decl(node);
        if let Some(source) = try_make_compile_source(info, self.opts, self.context) {
            self.queue.push(source);
            self.skip_body = true;
        }
    }

    fn enter_function_expression(
        &mut self,
        node: &'ast FunctionExpression,
        scope_stack: &[ScopeId],
    ) {
        self.skip_body = false;
        if self.is_rejected_by_scope_check(scope_stack) {
            return;
        }
        // TS getFunctionName for FunctionExpressions only returns names from parent
        // context (VariableDeclarator, AssignmentExpression, Property) — never from
        // the expression's own `id`. So we only use current_declarator_name here.
        let inferred_name = self.current_declarator_name.take();
        let parent_callee = self.current_parent_callee();
        let info = fn_info_from_func_expr(node, inferred_name, parent_callee);
        if let Some(source) = try_make_compile_source(info, self.opts, self.context) {
            self.queue.push(source);
            self.skip_body = true;
        }
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: &'ast ArrowFunctionExpression,
        scope_stack: &[ScopeId],
    ) {
        self.skip_body = false;
        if self.is_rejected_by_scope_check(scope_stack) {
            return;
        }
        let inferred_name = self.current_declarator_name.take();
        let parent_callee = self.current_parent_callee();
        let info = fn_info_from_arrow(node, inferred_name, parent_callee);
        if let Some(source) = try_make_compile_source(info, self.opts, self.context) {
            self.queue.push(source);
            self.skip_body = true;
        }
    }

    fn enter_object_method(
        &mut self,
        _node: &'ast crate::react_compiler_ast::expressions::ObjectMethod,
        _scope_stack: &[ScopeId],
    ) {
        self.skip_body = false;
    }
}

/// Find all functions in the program that should be compiled.
///
/// Uses the `AstWalker` with a `FunctionDiscoveryVisitor` to traverse
/// the entire program, discovering functions at any depth. The visitor
/// dynamically controls body traversal: compiled functions have their
/// bodies skipped (matching Babel's `fn.skip()`), while non-compiled
/// functions have their bodies traversed to find nested declarations.
///
/// The visitor tracks parent context (VariableDeclarator names for
/// `const Foo = () => {}`, CallExpression callees for forwardRef/memo
/// wrappers) via enter/leave hooks.
///
/// Skips classes and their contents (the walker does not recurse into
/// class bodies).
fn find_functions_to_compile<'a>(
    program: &'a Program,
    opts: &PluginOptions,
    context: &mut ProgramContext,
    scope: &ScopeInfo,
) -> Vec<CompileSource> {
    let mut visitor = FunctionDiscoveryVisitor::new(opts, context);
    let mut walker = AstWalker::new(scope);
    walker.walk_program(&mut visitor, program);
    visitor.queue
}

// -----------------------------------------------------------------------
// Main entry point
// -----------------------------------------------------------------------

/// A successfully compiled function, ready to be applied to the AST.
///
/// `'a` is the arena lifetime of the compiled oxc nodes; `'s` borrows the
/// discovery's `CompileSource`.
struct CompiledFunction<'a, 's> {
    #[allow(dead_code)]
    kind: CompileSourceKind,
    #[allow(dead_code)]
    source: &'s CompileSource,
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
    ast: &oxc_ast::AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    fn_type: oxc_ast::ast::FunctionType,
) -> oxc_allocator::Box<'a, oxc_ast::ast::Function<'a>> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;
    ast.alloc_function(
        SPAN,
        fn_type,
        codegen.id.clone_in(ast.allocator),
        codegen.generator,
        codegen.is_async,
        false,
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
        None::<oxc_allocator::Box<oxc_ast::ast::TSThisParameter>>,
        codegen.params.clone_in(ast.allocator),
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
        Some(codegen.body.clone_in(ast.allocator)),
    )
}

/// Build the compiled replacement as an `Expression`, matching the original node
/// kind (arrow vs function expression). Mirrors `build_compiled_expression_matching_kind`.
fn ox_build_compiled_expression<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    codegen: &CodegenFunction<'a>,
    original_kind: OriginalFnKind,
) -> oxc_ast::ast::Expression<'a> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;
    match original_kind {
        OriginalFnKind::ArrowFunctionExpression => {
            oxc_ast::ast::Expression::ArrowFunctionExpression(ast.alloc_arrow_function_expression(
                SPAN,
                false,
                codegen.is_async,
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
                codegen.params.clone_in(ast.allocator),
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                codegen.body.clone_in(ast.allocator),
            ))
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
    ast: &'b oxc_ast::AstBuilder<'a>,
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
            let mut params = self.codegen.params.clone_in(self.ast.allocator);
            if keep_types {
                copy_param_ts_metadata(self.ast.allocator, &mut params, &func.params);
            } else {
                func.type_parameters = None;
                func.return_type = None;
                func.this_param = None;
            }
            func.id = self.codegen.id.clone_in(self.ast.allocator);
            func.params = params;
            func.body = Some(self.codegen.body.clone_in(self.ast.allocator));
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
            let mut params = self.codegen.params.clone_in(self.ast.allocator);
            if keep_types {
                copy_param_ts_metadata(self.ast.allocator, &mut params, &arrow.params);
            } else {
                arrow.type_parameters = None;
                arrow.return_type = None;
            }
            arrow.params = params;
            arrow.body = self.codegen.body.clone_in(self.ast.allocator);
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
    ast: &'b oxc_ast::AstBuilder<'a>,
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
        let declarator = self.ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            self.ast.binding_pattern_binding_identifier(SPAN, ox_atom(self.ast, name)),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
            Some(self.gating_expression.clone_in(self.ast.allocator)),
            false,
        );
        oxc_ast::ast::Statement::VariableDeclaration(self.ast.alloc_variable_declaration(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            self.ast.vec1(declarator),
            false,
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
                        self.ast.alloc_export_named_declaration(
                            SPAN,
                            Some(decl),
                            self.ast.vec(),
                            None,
                            oxc_ast::ast::ImportOrExportKind::Value,
                            None::<oxc_allocator::Box<oxc_ast::ast::WithClause>>,
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
                                self.ast.alloc_export_default_declaration(
                                    SPAN,
                                    oxc_ast::ast::ExportDefaultDeclarationKind::from(
                                        self.gating_expression.clone_in(self.ast.allocator),
                                    ),
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
            let ident = self.ast.expression_identifier(SPAN, ox_atom(self.ast, &name));
            let export = oxc_ast::ast::Statement::ExportDefaultDeclaration(
                self.ast.alloc_export_default_declaration(
                    SPAN,
                    oxc_ast::ast::ExportDefaultDeclarationKind::from(ident),
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
            *expr = self.gating_expression.clone_in(self.ast.allocator);
            self.done = true;
            return;
        }
        oxc_ast_visit::walk_mut::walk_expression(self, expr);
    }
}

/// Visitor that renames every identifier reference matching `old_name` to `new_name`.
/// Mirrors the Babel `RenameIdentifierVisitor` (used to rename `useMemoCache`).
struct OxcRenameIdentifierVisitor<'a, 'b> {
    ast: &'b oxc_ast::AstBuilder<'a>,
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
fn ox_atom<'a>(ast: &oxc_ast::AstBuilder<'a>, s: &str) -> &'a str {
    oxc_allocator::StringBuilder::from_str_in(s, ast.allocator).into_str()
}

/// Build `<callee_name>()` as an oxc call expression.
fn ox_gating_call<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    callee_name: &str,
) -> oxc_ast::ast::Expression<'a> {
    use oxc_span::SPAN;
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ox_atom(ast, callee_name)),
        None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
        ast.vec(),
        false,
    )
}

/// Apply the conditional gating pattern to the oxc program. Mirrors
/// `apply_gated_function_conditional`.
fn ox_apply_gated_conditional<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
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
    let gating_expression = ast.expression_conditional(
        SPAN,
        ox_gating_call(ast, &gating_callee_name),
        compiled_expr,
        original_expr,
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
    ast: &oxc_ast::AstBuilder<'a>,
    program: &oxc_ast::ast::Program<'a>,
    node_id: u32,
) -> Option<oxc_ast::ast::Expression<'a>> {
    use oxc_allocator::CloneIn;
    use oxc_span::SPAN;

    struct Finder<'a, 'b> {
        ast: &'b oxc_ast::AstBuilder<'a>,
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
                let f = self.ast.alloc_function(
                    SPAN,
                    oxc_ast::ast::FunctionType::FunctionExpression,
                    func.id.clone_in(self.ast.allocator),
                    func.generator,
                    func.r#async,
                    false,
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
                    None::<oxc_allocator::Box<oxc_ast::ast::TSThisParameter>>,
                    func.params.clone_in(self.ast.allocator),
                    None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                    func.body.clone_in(self.ast.allocator),
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
                    self.ast.alloc(arrow.clone_in(self.ast.allocator)),
                ));
                return;
            }
            oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
        }
    }
    let _ = (SPAN, ast.allocator);
    let mut finder = Finder { ast, node_id, found: None };
    oxc_ast_visit::Visit::visit_program(&mut finder, program);
    finder.found.map(|e| e.clone_in(ast.allocator))
}

/// Splice every compiled oxc function into a clone of the original oxc program and
/// add the required imports. Returns the final memoized program.
fn ox_splice_program<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    oxc_program: &oxc_ast::ast::Program<'a>,
    replacements: &[OxcReplacement<'a>],
    context: &mut ProgramContext,
) -> oxc_ast::ast::Program<'a> {
    use oxc_allocator::CloneIn;

    let mut program = oxc_program.clone_in(ast.allocator);

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
        let insert_as_sibling =
            replacement.original_kind == OriginalFnKind::FunctionDeclaration;
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
    ast: &oxc_ast::AstBuilder<'a>,
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
                let specifiers = import.specifiers.get_or_insert_with(|| ast.vec());
                for spec in &sorted_imports {
                    specifiers.push(ox_make_import_specifier(ast, spec));
                }
            }
        } else if is_module {
            // ESM: import { imported as local, ... } from 'module'
            let mut specifiers = ast.vec();
            for spec in &sorted_imports {
                specifiers.push(ox_make_import_specifier(ast, spec));
            }
            let source = ast.string_literal(SPAN, ox_atom(ast, module_name), None);
            let import = ast.alloc_import_declaration(
                SPAN,
                Some(specifiers),
                source,
                None,
                None::<oxc_allocator::Box<oxc_ast::ast::WithClause>>,
                oxc_ast::ast::ImportOrExportKind::Value,
            );
            new_stmts.push(oxc_ast::ast::Statement::ImportDeclaration(import));
        } else {
            // CommonJS: const { imported: local, ... } = require('module')
            let mut props = ast.vec();
            for spec in &sorted_imports {
                let key = ast.property_key_static_identifier(SPAN, ox_atom(ast, &spec.imported));
                let value = ast.binding_pattern_binding_identifier(SPAN, ox_atom(ast, &spec.name));
                props.push(ast.binding_property(SPAN, key, value, false, false));
            }
            let object_pattern = ast.binding_pattern_object_pattern(
                SPAN,
                props,
                None::<oxc_allocator::Box<oxc_ast::ast::BindingRestElement>>,
            );
            let require_call = ast.expression_call(
                SPAN,
                ast.expression_identifier(SPAN, "require"),
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
                ast.vec1(oxc_ast::ast::Argument::from(ast.expression_string_literal(
                    SPAN,
                    ox_atom(ast, module_name),
                    None,
                ))),
                false,
            );
            let declarator = ast.variable_declarator(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                object_pattern,
                None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
                Some(require_call),
                false,
            );
            let decl = ast.alloc_variable_declaration(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                ast.vec1(declarator),
                false,
            );
            new_stmts.push(oxc_ast::ast::Statement::VariableDeclaration(decl));
        }
    }

    if !new_stmts.is_empty() {
        let old_body = std::mem::replace(&mut program.body, ast.vec());
        program.body.extend(new_stmts);
        program.body.extend(old_body);
    }
}

/// Build an oxc named import specifier `imported as local`. Mirrors `make_import_specifier`.
fn ox_make_import_specifier<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    spec: &super::imports::NonLocalImportSpecifier,
) -> oxc_ast::ast::ImportDeclarationSpecifier<'a> {
    use oxc_span::SPAN;
    let imported = oxc_ast::ast::ModuleExportName::IdentifierName(
        ast.identifier_name(SPAN, ox_atom(ast, &spec.imported)),
    );
    let local = ast.binding_identifier(SPAN, ox_atom(ast, &spec.name));
    oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(ast.alloc_import_specifier(
        SPAN,
        imported,
        local,
        oxc_ast::ast::ImportOrExportKind::Value,
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
pub fn compile_program<'a>(
    ast: &oxc_ast::AstBuilder<'a>,
    oxc_program: &oxc_ast::ast::Program<'a>,
    file: File,
    scope: ScopeInfo,
    options: PluginOptions,
    fn_map: &FxHashMap<u32, FunctionNode<'_>>,
) -> CompileResult<'a> {
    // Compute output mode once, up front
    let output_mode = CompilerOutputMode::from_opts(&options);

    // Create a temporary context for early-return paths (before full context is set up)
    let early_events: Vec<LoggerEvent> = Vec::new();
    let mut early_ordered_log: Vec<OrderedLogItem> = Vec::new();

    // Log environment config for debugLogIRs
    if options.debug {
        early_ordered_log.push(OrderedLogItem::Debug {
            entry: DebugLogEntry::new("EnvironmentConfig", format!("{:#?}", options.environment)),
        });
    }

    // Check if we should compile this file at all (pre-resolved by JS shim)
    if !options.should_compile {
        return CompileResult::Success {
            ast: None,
            events: early_events,
            ordered_log: early_ordered_log,
            renames: Vec::new(),
            timing: Vec::new(),
        };
    }

    let program = &file.program;

    // Check for existing runtime imports (file already compiled)
    if should_skip_compilation(program, &options) {
        return CompileResult::Success {
            ast: None,
            events: early_events,
            ordered_log: early_ordered_log,
            renames: Vec::new(),
            timing: Vec::new(),
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
        &file.comments,
        eslint_rules.as_deref(),
        options.flow_suppressions,
    );

    // Check for module-scope opt-out directive
    let has_module_scope_opt_out =
        find_directive_disabling_memoization(&program.directives, &options).is_some();

    // Create program context
    let mut context = ProgramContext::new(
        options.clone(),
        options.filename.clone(),
        // Pass the source code for fast refresh hash computation.
        options.source_code.clone(),
        suppressions,
        has_module_scope_opt_out,
    );

    // Extract the source filename from the AST (set by parser's sourceFilename option).
    // This is the bare filename (e.g., "foo.ts") without path prefixes, which the TS
    // compiler uses in logger event source locations.
    let source_filename =
        program.base.loc.as_ref().and_then(|loc| loc.filename.clone()).or_else(|| {
            // Fallback: try the first statement's loc
            program.body.first().and_then(|stmt| {
                let base = match stmt {
                    crate::react_compiler_ast::statements::Statement::ExpressionStatement(s) => {
                        &s.base
                    }
                    crate::react_compiler_ast::statements::Statement::VariableDeclaration(s) => {
                        &s.base
                    }
                    crate::react_compiler_ast::statements::Statement::FunctionDeclaration(s) => {
                        &s.base
                    }
                    _ => return None,
                };
                base.loc.as_ref().and_then(|loc| loc.filename.clone())
            })
        });
    context.set_source_filename(source_filename);

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
            events: context.events,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
            timing: Vec::new(),
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
    let mut compiled_fns: Vec<CompiledFunction<'_, '_>> = Vec::new();

    for source in &queue {
        match process_fn(ast, source, &scope, output_mode, &env_config, &mut context, fn_map) {
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

    // Emit CompileSuccess events for JSX-outlined functions (fn_type.is_some()).
    // In TS, outlined functions from outlineJSX are appended to the compilation queue
    // and processed after all original functions, so their events appear at the end.
    // Regular outlined functions (from OutlineFunctions pass) don't get separate events.
    for compiled in &compiled_fns {
        for outlined in &compiled.codegen_fn.outlined {
            if outlined.fn_type.is_some() {
                context.log_event(LoggerEvent::CompileSuccess {
                    fn_loc: None,
                    fn_name: outlined.func.id.as_ref().map(|id| id.name.to_string()),
                    memo_slots: outlined.func.memo_slots_used,
                    memo_blocks: outlined.func.memo_blocks,
                    memo_values: outlined.func.memo_values,
                    pruned_memo_blocks: outlined.func.pruned_memo_blocks,
                    pruned_memo_values: outlined.func.pruned_memo_values,
                });
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
            events: context.events,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
            timing: Vec::new(),
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
            events: context.events,
            ordered_log: context.ordered_log,
            renames: convert_renames(&context.renames),
            timing: Vec::new(),
        };
    }

    // Build the memoized oxc program: splice each compiled oxc function in for its
    // original (matched by `span.start == fn_node_id`), apply gating, insert outlined
    // functions, and add the memo-cache / gating imports.
    let compiled_program = ox_splice_program(ast, oxc_program, &replacements, &mut context);

    let timing_entries = context.timing.into_entries();

    CompileResult::Success {
        ast: Some(compiled_program),
        events: context.events,
        ordered_log: context.ordered_log,
        renames: convert_renames(&context.renames),
        timing: timing_entries,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hook_name() {
        assert!(is_hook_name("useState"));
        assert!(is_hook_name("useEffect"));
        assert!(is_hook_name("use0Something"));
        assert!(!is_hook_name("use"));
        assert!(!is_hook_name("useless")); // lowercase after use
        assert!(!is_hook_name("foo"));
        assert!(!is_hook_name(""));
    }

    #[test]
    fn test_is_component_name() {
        assert!(is_component_name("MyComponent"));
        assert!(is_component_name("App"));
        assert!(!is_component_name("myComponent"));
        assert!(!is_component_name("app"));
        assert!(!is_component_name(""));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("$baz"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier("foo bar"));
    }

    #[test]
    fn test_is_valid_component_params_empty() {
        assert!(is_valid_component_params(&[]));
    }

    #[test]
    fn test_is_valid_component_params_one_identifier() {
        let params = vec![PatternLike::Identifier(Identifier {
            base: BaseNode::default(),
            name: "props".to_string(),
            type_annotation: None,
            optional: None,
            decorators: None,
        })];
        assert!(is_valid_component_params(&params));
    }

    #[test]
    fn test_is_valid_component_params_too_many() {
        let params = vec![
            PatternLike::Identifier(Identifier {
                base: BaseNode::default(),
                name: "a".to_string(),
                type_annotation: None,
                optional: None,
                decorators: None,
            }),
            PatternLike::Identifier(Identifier {
                base: BaseNode::default(),
                name: "b".to_string(),
                type_annotation: None,
                optional: None,
                decorators: None,
            }),
            PatternLike::Identifier(Identifier {
                base: BaseNode::default(),
                name: "c".to_string(),
                type_annotation: None,
                optional: None,
                decorators: None,
            }),
        ];
        assert!(!is_valid_component_params(&params));
    }

    #[test]
    fn test_is_valid_component_params_with_ref() {
        let params = vec![
            PatternLike::Identifier(Identifier {
                base: BaseNode::default(),
                name: "props".to_string(),
                type_annotation: None,
                optional: None,
                decorators: None,
            }),
            PatternLike::Identifier(Identifier {
                base: BaseNode::default(),
                name: "ref".to_string(),
                type_annotation: None,
                optional: None,
                decorators: None,
            }),
        ];
        assert!(is_valid_component_params(&params));
    }

    #[test]
    fn test_should_skip_compilation_no_import() {
        let program = Program {
            base: BaseNode::default(),
            body: vec![],
            directives: vec![],
            source_type: crate::react_compiler_ast::SourceType::Module,
            interpreter: None,
            source_file: None,
        };
        let options = PluginOptions {
            should_compile: true,
            enable_reanimated: false,
            is_dev: false,
            filename: None,
            compilation_mode: "infer".to_string(),
            panic_threshold: "none".to_string(),
            target: super::super::plugin_options::CompilerTarget::Version("19".to_string()),
            gating: None,
            dynamic_gating: None,
            no_emit: false,
            output_mode: None,
            eslint_suppression_rules: None,
            flow_suppressions: true,
            ignore_use_no_forget: false,
            custom_opt_out_directives: None,
            environment: EnvironmentConfig::default(),
            source_code: None,
            profiling: false,
            debug: false,
        };
        assert!(!should_skip_compilation(&program, &options));
    }
}
