use oxc_ast::ast::{
    self, BindingPattern, Expression, FormalParameters, ImportDeclarationSpecifier, Statement,
    TSType, TSTypeAnnotation,
};

use crate::{
    compiler_error::{CompilerError, ErrorCategory},
    entrypoint::options::{
        CompilationMode, CompilerReactTarget, OPT_IN_DIRECTIVES, OPT_OUT_DIRECTIVES, PanicThreshold,
    },
    hir::{ReactFunctionType, build_hir::LowerableFunction},
    utils::{component_declaration::is_component_name, hook_declaration::is_hook_name},
};

/// Result of compiling a program.
#[derive(Debug)]
pub struct ProgramCompilationResult {
    /// Number of functions that were successfully compiled.
    pub compiled: u32,
    /// Number of functions that were skipped.
    pub skipped: u32,
    /// Number of functions that errored.
    pub errored: u32,
}

/// Compile a configured `hookPattern` regex once.
///
/// Returns:
/// - `Ok(None)` if no pattern is configured.
/// - `Ok(Some(regex))` if the pattern compiles cleanly.
/// - `Err(CompilerError)` if the pattern is not a valid regex — propagating the
///   same `invalid_config` error that `Environment::new` would later raise, but
///   *before* a classification pass silently swallows it.
///
/// Mirrors upstream `Environment.ts:939` / `Program.ts:1010-1015`, which lazily
/// `new RegExp(this.config.hookPattern)` on every lookup. We compile once and
/// reuse to avoid the per-call cost (and to surface bad config eagerly).
///
/// # Errors
/// Returns an `invalid_config` `CompilerError` if `hook_pattern` is `Some(_)`
/// but the contained string does not parse as a regex.
pub fn compile_hook_pattern(
    hook_pattern: Option<&str>,
) -> Result<Option<lazy_regex::Regex>, CompilerError> {
    match hook_pattern {
        None => Ok(None),
        Some(pat) => lazy_regex::Regex::new(pat).map(Some).map_err(|err| {
            CompilerError::invalid_config(
                "Invalid `hookPattern` regex",
                Some(&err.to_string()),
                None,
            )
        }),
    }
}

/// Determine if a function should be compiled based on the compilation mode.
///
/// Port of `getReactFunctionType` from Program.ts (lines 818-864).
///
/// This function does NOT check opt-out directives (`'use no memo'` / `'use no forget'`).
/// In the TS reference, `getReactFunctionType` only determines the function type;
/// opt-out directives are checked by the caller (`processFn`) AFTER compilation,
/// allowing validation/lint errors to still be reported.
///
/// `is_memo_or_forwardref_arg` should be true when the function is the callback
/// argument of `React.memo()`, `memo()`, `React.forwardRef()`, or `forwardRef()`.
///
/// `hook_pattern` is a pre-compiled regex matching the configured
/// `EnvironmentConfig::hook_pattern`. When `Some`, the default `^use[A-Z0-9]`
/// hook-name detection is overridden — mirroring upstream
/// `Program.ts:1010 isHookName`. Callers MUST compile the user-provided pattern
/// via [`compile_hook_pattern`] beforehand and surface any compile failure as
/// an invalid-config diagnostic; classification silently treats `None` here as
/// "use the built-in convention".
pub fn should_compile_function(
    function: &LowerableFunction<'_>,
    name: Option<&str>,
    directives: &[String],
    mode: CompilationMode,
    is_memo_or_forwardref_arg: bool,
    has_dynamic_gating: bool,
    hook_pattern: Option<&lazy_regex::Regex>,
) -> Option<ReactFunctionType> {
    // Check for opt-in directives (TS lines 822-830)
    // Static opt-ins: "use forget", "use memo"
    // Dynamic gating directives: "use memo if(...)" — only recognized when
    // dynamicGating is configured (Program.ts:87-97 returns early if null).
    let has_opt_in = directives.iter().any(|d| {
        OPT_IN_DIRECTIVES.contains(&d.as_str())
            || (has_dynamic_gating && parse_dynamic_gating_directive(d).is_some())
    });
    if has_opt_in {
        return Some(
            get_component_or_hook_like(function, name, is_memo_or_forwardref_arg, hook_pattern)
                .unwrap_or(ReactFunctionType::Other),
        );
    }

    // componentSyntaxType: In the TS reference, this checks for Flow component/hook
    // syntax declarations (isComponentDeclaration/isHookDeclaration). Since we don't
    // support Flow syntax in Rust, componentSyntaxType is always None.

    match mode {
        CompilationMode::Annotation => {
            // opt-ins are checked above
            None
        }
        CompilationMode::Infer => {
            // Check if this is a component or hook-like function
            // TS: return componentSyntaxType ?? getComponentOrHookLike(fn);
            get_component_or_hook_like(function, name, is_memo_or_forwardref_arg, hook_pattern)
        }
        CompilationMode::Syntax => {
            // TS: return componentSyntaxType;
            // No Flow syntax support in Rust, always None.
            None
        }
        CompilationMode::All => {
            // TS: return getComponentOrHookLike(fn) ?? "Other";
            Some(
                get_component_or_hook_like(function, name, is_memo_or_forwardref_arg, hook_pattern)
                    .unwrap_or(ReactFunctionType::Other),
            )
        }
    }
}

/// Check if a name matches the hook naming convention, optionally with a
/// user-provided regex override.
///
/// Port of `Program.ts:isHookName` (lines 1010-1015):
///   function isHookName(s, hookPattern) {
///     if (hookPattern !== null) return new RegExp(hookPattern).test(s);
///     return /^use[A-Z0-9]/.test(s);
///   }
#[inline]
fn is_hook_name_with_pattern(name: &str, hook_pattern: Option<&lazy_regex::Regex>) -> bool {
    if let Some(re) = hook_pattern { re.is_match(name) } else { is_hook_name(name) }
}

/// Port of `getComponentOrHookLike` from Program.ts (lines 955-982).
///
/// Determines whether a function is a Component, Hook, or neither based on:
/// - Function name (component-like or hook-like)
/// - Behavioral analysis (calls hooks or creates JSX)
/// - Parameter validation (for components)
/// - Return type checking (for components)
/// - memo/forwardRef wrapping (for anonymous functions)
pub fn get_component_or_hook_like(
    func: &LowerableFunction<'_>,
    name: Option<&str>,
    is_memo_or_forwardref_arg: bool,
    hook_pattern: Option<&lazy_regex::Regex>,
) -> Option<ReactFunctionType> {
    // Check if the name is component or hook like
    if let Some(n) = name {
        if is_component_name(n) {
            // TS lines 961-965: component name requires all three checks
            let is_component = calls_hooks_or_creates_jsx(func, hook_pattern)
                && is_valid_component_params(get_params(func))
                && !returns_non_node(func);
            return if is_component { Some(ReactFunctionType::Component) } else { None };
        }
        if is_hook_name_with_pattern(n, hook_pattern) {
            // TS lines 966-968: hooks just need hook invocations or JSX
            return if calls_hooks_or_creates_jsx(func, hook_pattern) {
                Some(ReactFunctionType::Hook)
            } else {
                None
            };
        }
    }

    // For function/arrow expressions, check if they appear as the argument
    // to React.forwardRef() or React.memo() (TS lines 975-979)
    if is_memo_or_forwardref_arg {
        return if calls_hooks_or_creates_jsx(func, hook_pattern) {
            Some(ReactFunctionType::Component)
        } else {
            None
        };
    }

    None
}

/// Get the formal parameters from a `LowerableFunction`.
fn get_params<'a>(func: &'a LowerableFunction<'a>) -> &'a FormalParameters<'a> {
    match func {
        LowerableFunction::Function(f) => &f.params,
        LowerableFunction::ArrowFunction(a) => &a.params,
    }
}

/// Check if a function body directly calls hooks or creates JSX.
///
/// Port of `callsHooksOrCreatesJsx` from Program.ts (lines 996-1018).
/// Traverses the function body but skips nested function declarations/expressions/arrows.
///
/// `hook_pattern` is forwarded from upstream `isHook(path, hookPattern)`. When
/// `Some`, any name that matches the regex is treated as a hook even if it
/// does not satisfy the default `^use[A-Z0-9]` convention.
pub fn calls_hooks_or_creates_jsx(
    func: &LowerableFunction,
    hook_pattern: Option<&lazy_regex::Regex>,
) -> bool {
    fn check_expr(expr: &Expression, hook_pattern: Option<&lazy_regex::Regex>) -> bool {
        match expr {
            // JSX
            Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
            // Hook calls: call expressions where callee is a hook name
            Expression::CallExpression(call) => {
                let is_hook = match &call.callee {
                    Expression::Identifier(id) => is_hook_name_with_pattern(&id.name, hook_pattern),
                    Expression::StaticMemberExpression(member) => {
                        is_hook_name_with_pattern(&member.property.name, hook_pattern)
                            && matches!(&member.object, Expression::Identifier(obj) if obj.name.starts_with(|c: char| c.is_ascii_uppercase()))
                    }
                    _ => false,
                };
                if is_hook {
                    return true;
                }
                // Check arguments (but not nested functions in them)
                for arg in &call.arguments {
                    if let Some(e) = arg.as_expression() {
                        // Skip function expressions/arrows in arguments
                        if !matches!(
                            e,
                            Expression::FunctionExpression(_)
                                | Expression::ArrowFunctionExpression(_)
                        ) && check_expr(e, hook_pattern)
                        {
                            return true;
                        }
                    }
                }
                // Check callee (but skip if it's a function expression)
                if matches!(
                    &call.callee,
                    Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                ) {
                    false
                } else {
                    check_expr(&call.callee, hook_pattern)
                }
            }
            // Recurse into other expressions
            Expression::ParenthesizedExpression(paren) => {
                check_expr(&paren.expression, hook_pattern)
            }
            Expression::SequenceExpression(seq) => {
                seq.expressions.iter().any(|e| check_expr(e, hook_pattern))
            }
            Expression::ConditionalExpression(cond) => {
                check_expr(&cond.test, hook_pattern)
                    || check_expr(&cond.consequent, hook_pattern)
                    || check_expr(&cond.alternate, hook_pattern)
            }
            Expression::LogicalExpression(log) => {
                check_expr(&log.left, hook_pattern) || check_expr(&log.right, hook_pattern)
            }
            Expression::BinaryExpression(bin) => {
                check_expr(&bin.left, hook_pattern) || check_expr(&bin.right, hook_pattern)
            }
            Expression::UnaryExpression(un) => check_expr(&un.argument, hook_pattern),
            Expression::AssignmentExpression(assign) => check_expr(&assign.right, hook_pattern),
            Expression::TaggedTemplateExpression(tag) => check_expr(&tag.tag, hook_pattern),
            Expression::TemplateLiteral(tl) => {
                tl.expressions.iter().any(|e| check_expr(e, hook_pattern))
            }
            Expression::ArrayExpression(arr) => arr
                .elements
                .iter()
                .any(|el| el.as_expression().is_some_and(|e| check_expr(e, hook_pattern))),
            Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| {
                if let ast::ObjectPropertyKind::ObjectProperty(p) = prop {
                    check_expr(&p.value, hook_pattern)
                } else {
                    false
                }
            }),
            Expression::StaticMemberExpression(member) => check_expr(&member.object, hook_pattern),
            Expression::ComputedMemberExpression(member) => {
                check_expr(&member.object, hook_pattern)
                    || check_expr(&member.expression, hook_pattern)
            }
            Expression::AwaitExpression(aw) => check_expr(&aw.argument, hook_pattern),
            Expression::YieldExpression(y) => {
                y.argument.as_ref().is_some_and(|a| check_expr(a, hook_pattern))
            }
            // TS type expression wrappers: recurse into inner expression.
            Expression::TSNonNullExpression(e) => check_expr(&e.expression, hook_pattern),
            Expression::TSAsExpression(e) => check_expr(&e.expression, hook_pattern),
            Expression::TSSatisfiesExpression(e) => check_expr(&e.expression, hook_pattern),
            Expression::TSTypeAssertion(e) => check_expr(&e.expression, hook_pattern),
            Expression::TSInstantiationExpression(e) => check_expr(&e.expression, hook_pattern),
            _ => false,
        }
    }

    fn check_stmt(stmt: &Statement, hook_pattern: Option<&lazy_regex::Regex>) -> bool {
        match stmt {
            Statement::ExpressionStatement(es) => check_expr(&es.expression, hook_pattern),
            Statement::ReturnStatement(ret) => {
                ret.argument.as_ref().is_some_and(|e| check_expr(e, hook_pattern))
            }
            Statement::VariableDeclaration(decl) => decl
                .declarations
                .iter()
                .any(|d| d.init.as_ref().is_some_and(|e| check_expr(e, hook_pattern))),
            Statement::IfStatement(ifs) => {
                check_expr(&ifs.test, hook_pattern)
                    || check_stmt(&ifs.consequent, hook_pattern)
                    || ifs.alternate.as_ref().is_some_and(|a| check_stmt(a, hook_pattern))
            }
            Statement::BlockStatement(block) => {
                block.body.iter().any(|s| check_stmt(s, hook_pattern))
            }
            Statement::ForStatement(f) => check_stmt(&f.body, hook_pattern),
            Statement::ForInStatement(f) => check_stmt(&f.body, hook_pattern),
            Statement::ForOfStatement(f) => check_stmt(&f.body, hook_pattern),
            Statement::WhileStatement(w) => check_stmt(&w.body, hook_pattern),
            Statement::DoWhileStatement(d) => check_stmt(&d.body, hook_pattern),
            Statement::SwitchStatement(s) => s
                .cases
                .iter()
                .any(|c| c.consequent.iter().any(|stmt| check_stmt(stmt, hook_pattern))),
            Statement::TryStatement(t) => {
                t.block.body.iter().any(|s| check_stmt(s, hook_pattern))
                    || t.handler
                        .as_ref()
                        .is_some_and(|h| h.body.body.iter().any(|s| check_stmt(s, hook_pattern)))
                    || t.finalizer
                        .as_ref()
                        .is_some_and(|f| f.body.iter().any(|s| check_stmt(s, hook_pattern)))
            }
            Statement::ThrowStatement(throw) => check_expr(&throw.argument, hook_pattern),
            Statement::LabeledStatement(l) => check_stmt(&l.body, hook_pattern),
            _ => false,
        }
    }

    match func {
        LowerableFunction::Function(f) => {
            if let Some(body) = &f.body {
                body.statements.iter().any(|stmt| check_stmt(stmt, hook_pattern))
            } else {
                false
            }
        }
        LowerableFunction::ArrowFunction(a) => {
            // For expression arrows, the parser stores the expression as a
            // single ExpressionStatement in body.statements, so this works
            // uniformly.
            a.body.statements.iter().any(|stmt| check_stmt(stmt, hook_pattern))
        }
    }
}

/// Port of `isValidPropsAnnotation` from Program.ts (lines 878-921).
///
/// Rejects params with primitive-type annotations that are clearly not props objects.
fn is_valid_props_annotation(annot: Option<&TSTypeAnnotation<'_>>) -> bool {
    let Some(annot) = annot else {
        return true;
    };
    // We only handle TSTypeAnnotation (no Flow TypeAnnotation support)
    !matches!(
        annot.type_annotation,
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

/// Port of `isValidComponentParams` from Program.ts (lines 923-949).
///
/// Validates that a function's parameters are consistent with being a React component:
/// - 0 params: valid
/// - 1 param: valid unless it's a rest element; also checks type annotation
/// - 2 params: second param must be identifier containing "ref" or "Ref"
/// - 3+ params: invalid
pub fn is_valid_component_params(params: &FormalParameters<'_>) -> bool {
    let count = params.items.len();

    // If there's a rest parameter, it counts as an additional param in the TS sense.
    // But in the TS reference, rest elements are checked differently: params[0].isRestElement().
    // In oxc AST, rest params are stored in params.rest, not in params.items.

    if count == 0 && params.rest.is_none() {
        return true;
    }

    if count > 2 || (count == 2 && params.rest.is_some()) {
        return false;
    }

    // Check first param's type annotation
    if count > 0 {
        let first = &params.items[0];
        // In oxc AST, FormalParameter.type_annotation holds the TS annotation.
        if !is_valid_props_annotation(first.type_annotation.as_deref()) {
            return false;
        }
    }

    if count == 0 && params.rest.is_some() {
        // Single rest param like (...args) => {} — TS treats this as params[0].isRestElement()
        return false;
    }

    if count == 1 {
        // A rest param in params.rest means we actually have (param, ...rest) or (...rest).
        // With 1 item + rest, total params > 1 check was done above.
        // With 1 item + no rest, just check the param is not somehow rest-like.
        // In oxc AST, rest params are always in params.rest, not items, so items[0]
        // is never a rest element.
        return params.rest.is_none();
    }

    if count == 2 {
        // Second param must be identifier containing "ref" or "Ref"
        if let BindingPattern::BindingIdentifier(id) = &params.items[1].pattern {
            let name = id.name.as_str();
            return name.contains("ref") || name.contains("Ref");
        }
        return false;
    }

    false
}

/// Port of `isNonNode` from Program.ts (lines 1020-1034).
///
/// Returns true if the expression is clearly not a renderable React node.
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

/// Port of `returnsNonNode` from Program.ts (lines 1036-1060).
///
/// Traverses the function body looking at return statements. Returns true if
/// ANY return value is a non-renderable type.
pub fn returns_non_node(func: &LowerableFunction) -> bool {
    fn check_stmt_for_non_node_return(stmt: &Statement) -> bool {
        match stmt {
            Statement::ReturnStatement(ret) => ret.argument.as_ref().is_none_or(|e| is_non_node(e)),
            Statement::BlockStatement(block) => {
                block.body.iter().any(|s| check_stmt_for_non_node_return(s))
            }
            Statement::IfStatement(ifs) => {
                check_stmt_for_non_node_return(&ifs.consequent)
                    || ifs.alternate.as_ref().is_some_and(|a| check_stmt_for_non_node_return(a))
            }
            Statement::ForStatement(f) => check_stmt_for_non_node_return(&f.body),
            Statement::ForInStatement(f) => check_stmt_for_non_node_return(&f.body),
            Statement::ForOfStatement(f) => check_stmt_for_non_node_return(&f.body),
            Statement::WhileStatement(w) => check_stmt_for_non_node_return(&w.body),
            Statement::DoWhileStatement(d) => check_stmt_for_non_node_return(&d.body),
            Statement::SwitchStatement(s) => s
                .cases
                .iter()
                .any(|c| c.consequent.iter().any(|stmt| check_stmt_for_non_node_return(stmt))),
            Statement::TryStatement(t) => {
                t.block.body.iter().any(|s| check_stmt_for_non_node_return(s))
                    || t.handler.as_ref().is_some_and(|h| {
                        h.body.body.iter().any(|s| check_stmt_for_non_node_return(s))
                    })
                    || t.finalizer
                        .as_ref()
                        .is_some_and(|f| f.body.iter().any(|s| check_stmt_for_non_node_return(s)))
            }
            Statement::LabeledStatement(l) => check_stmt_for_non_node_return(&l.body),
            // ExpressionStatement, VariableDeclaration, ThrowStatement, etc. — no return
            _ => false,
        }
    }

    match func {
        LowerableFunction::Function(f) => {
            if let Some(body) = &f.body {
                body.statements.iter().any(|s| check_stmt_for_non_node_return(s))
            } else {
                false
            }
        }
        LowerableFunction::ArrowFunction(a) => {
            // For expression arrows (body is not BlockStatement in TS),
            // the expression itself is the return value.
            // In oxc, expression arrows have body.statements containing one
            // ExpressionStatement, BUT the TS reference checks
            // `node.node.body.type !== "BlockStatement"` and uses `isNonNode(node.node.body)`.
            // For concise arrows, check the expression directly.
            if a.expression {
                // Concise arrow: body has a single expression as the implicit return
                if let Some(Statement::ExpressionStatement(es)) = a.body.statements.first() {
                    return is_non_node(&es.expression);
                }
            }
            // Block body arrow: traverse normally
            a.body.statements.iter().any(|s| check_stmt_for_non_node_return(s))
        }
    }
}

/// Check if a function has a directive that enables memoization.
pub fn find_directive_enabling_memoization(directives: &[String]) -> Option<String> {
    directives.iter().find(|d| OPT_IN_DIRECTIVES.contains(&d.as_str())).cloned()
}

/// Check if a function has a directive that disables memoization.
pub fn find_directive_disabling_memoization(
    directives: &[String],
    custom_opt_out: Option<&[String]>,
) -> Option<String> {
    // When custom opt-out directives are configured, ONLY check those
    // (do not fall through to standard directives). Matches TS Program.ts:73-80.
    if let Some(custom) = custom_opt_out {
        return directives.iter().find(|d| custom.contains(d)).cloned();
    }
    // Otherwise check standard opt-out directives
    directives.iter().find(|d| OPT_OUT_DIRECTIVES.contains(&d.as_str())).cloned()
}

/// Determine how to handle a compilation error based on the panic threshold.
///
/// Port of Program.ts lines 143-155. Config errors always throw regardless of threshold.
pub fn handle_compilation_error(error: &CompilerError, threshold: PanicThreshold) -> ErrorAction {
    // Config errors always throw (upstream Program.ts:150-152 `isConfigError`).
    if error.details.iter().any(|detail| detail.category() == ErrorCategory::Config) {
        return ErrorAction::Panic;
    }

    match threshold {
        PanicThreshold::AllErrors => ErrorAction::Panic,
        PanicThreshold::CriticalErrors => {
            if error.has_errors() {
                ErrorAction::Panic
            } else {
                ErrorAction::Skip
            }
        }
        PanicThreshold::None => ErrorAction::Skip,
    }
}

/// Action to take when encountering a compilation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    /// Panic (throw/propagate the error).
    Panic,
    /// Skip this function and continue.
    Skip,
}

/// Parse the `use memo if(IDENT)` directive pattern.
///
/// Port of `findDirectivesDynamicGating` regex `^use memo if\(([^\)]*)\)$`
/// from Program.ts (lines 40-120).
///
/// Returns:
/// - `None` if the directive doesn't match the `use memo if(...)` pattern at all
/// - `Some(Ok(ident))` if a valid identifier was found inside the parentheses
/// - `Some(Err(raw))` if the pattern matched but the content is not a valid JS identifier
pub fn parse_dynamic_gating_directive(directive: &str) -> Option<Result<&str, &str>> {
    let trimmed = directive.trim();
    let rest = trimmed.strip_prefix("use memo if(")?;
    let ident = rest.strip_suffix(')')?;
    let ident = ident.trim();
    if ident.is_empty() {
        return Some(Err(trimmed));
    }
    if oxc_syntax::identifier::is_identifier_name(ident) {
        Some(Ok(ident))
    } else {
        Some(Err(trimmed))
    }
}

/// Get the React compiler runtime module name for the given target.
///
/// Port of `getReactCompilerRuntimeModule` from Program.ts (lines 1291-1310).
///
/// Returns the module name from which the compiler imports the `c` (useMemoCache)
/// function:
/// - React 19: `"react/compiler-runtime"` (from react namespace)
/// - React 17/18: `"react-compiler-runtime"` (npm package)
/// - Meta-internal: custom `runtimeModule` from config
pub fn get_react_compiler_runtime_module(target: &CompilerReactTarget) -> &str {
    match target {
        CompilerReactTarget::React19 => "react/compiler-runtime",
        CompilerReactTarget::React17 | CompilerReactTarget::React18 => "react-compiler-runtime",
        CompilerReactTarget::MetaInternal { runtime_module } => runtime_module.as_str(),
    }
}

/// Returns true if the program contains an `import { c } from "<moduleName>"` declaration,
/// regardless of the local name of the `c` specifier and the presence of other specifiers
/// in the same declaration.
///
/// Port of `hasMemoCacheFunctionImport` from Program.ts (lines 866-895).
///
/// This is used to detect files that have already been compiled by the React Compiler,
/// preventing double-compilation.
pub fn has_memo_cache_function_import(body: &[Statement<'_>], module_name: &str) -> bool {
    for stmt in body {
        if let Statement::ImportDeclaration(import) = stmt
            && import.source.value.as_str() == module_name
            && import.specifiers.as_ref().is_some_and(|specs| {
                specs.iter().any(|spec| {
                    matches!(spec, ImportDeclarationSpecifier::ImportSpecifier(s)
                        if s.imported.name() == "c")
                })
            })
        {
            return true;
        }
    }
    false
}
