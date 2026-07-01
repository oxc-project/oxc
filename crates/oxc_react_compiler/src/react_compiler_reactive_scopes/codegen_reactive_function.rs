// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Code generation pass: converts a `ReactiveFunction` tree back into a Babel-compatible
//! AST with memoization (useMemoCache) wired in.
//!
//! This is the final pass in the compilation pipeline.
//!
//! Corresponds to `src/ReactiveScopes/CodegenReactiveFunction.ts` in the TS compiler.

use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use crate::react_compiler_diagnostics::CompilerDiagnostic;
use crate::react_compiler_diagnostics::CompilerDiagnosticDetail;
use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_diagnostics::SourceLocation as DiagSourceLocation;
use crate::react_compiler_hir::ArrayElement;
use crate::react_compiler_hir::ArrayPattern;
use crate::react_compiler_hir::BlockId;
use crate::react_compiler_hir::DeclarationId;
use crate::react_compiler_hir::FunctionExpressionType;
use crate::react_compiler_hir::IdentifierId;
use crate::react_compiler_hir::InstructionKind;
use crate::react_compiler_hir::InstructionValue;
use crate::react_compiler_hir::JsxAttribute;
use crate::react_compiler_hir::JsxTag;
use crate::react_compiler_hir::LogicalOperator;
use crate::react_compiler_hir::ObjectPattern;
use crate::react_compiler_hir::ObjectPropertyKey;
use crate::react_compiler_hir::ObjectPropertyOrSpread;
use crate::react_compiler_hir::ObjectPropertyType;
use crate::react_compiler_hir::ParamPattern;
use crate::react_compiler_hir::Pattern;
use crate::react_compiler_hir::Place;
use crate::react_compiler_hir::PlaceOrSpread;
use crate::react_compiler_hir::PrimitiveValue;
use crate::react_compiler_hir::PropertyLiteral;
use crate::react_compiler_hir::ScopeId;
use crate::react_compiler_hir::SpreadPattern;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::reactive::PrunedReactiveScopeBlock;
use crate::react_compiler_hir::reactive::ReactiveBlock;
use crate::react_compiler_hir::reactive::ReactiveFunction;
use crate::react_compiler_hir::reactive::ReactiveInstruction;
use crate::react_compiler_hir::reactive::ReactiveScopeBlock;
use crate::react_compiler_hir::reactive::ReactiveStatement;
use crate::react_compiler_hir::reactive::ReactiveTerminal;
use crate::react_compiler_hir::reactive::ReactiveTerminalTargetKind;
use crate::react_compiler_hir::reactive::ReactiveValue;

use crate::react_compiler_reactive_scopes::build_reactive_function::build_reactive_function;
use crate::react_compiler_reactive_scopes::prune_hoisted_contexts::prune_hoisted_contexts;
use crate::react_compiler_reactive_scopes::prune_unused_labels::prune_unused_labels;
use crate::react_compiler_reactive_scopes::prune_unused_lvalues::prune_unused_lvalues;
use crate::react_compiler_reactive_scopes::rename_variables::rename_variables;
use crate::react_compiler_reactive_scopes::visitors::ReactiveFunctionVisitor;
use crate::react_compiler_reactive_scopes::visitors::visit_reactive_function;

// =============================================================================
// Public API
// =============================================================================

pub const MEMO_CACHE_SENTINEL: &str = "react.memo_cache_sentinel";
pub const EARLY_RETURN_SENTINEL: &str = "react.early_return_sentinel";

/// FBT tags whose children get special codegen treatment.
const SINGLE_CHILD_FBT_TAGS: &[&str] = &["fbt:param", "fbs:param"];

/// Computes the Fast Refresh source hash used to bust the memo cache when the
/// source file changes. Matches the TS compiler's
/// `createHmac('sha256', code).digest('hex')`: an HMAC-SHA256 keyed by the
/// source code, hashing empty data.
///
/// Not yet wired into the oxc emission path (Fast Refresh hashing is deferred);
/// kept with its verified test as the primitive the port will reuse.
#[allow(dead_code)]
fn source_file_hash(code: &str) -> String {
    hmac_sha256::HMAC::mac(b"", code.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Top-level entry point: produces an oxc-shaped
/// [`crate::react_compiler::entrypoint::compile_result::CodegenFunction`] from a
/// reactive function, building oxc AST directly via [`oxc_ast::builder::AstBuilder`].
pub fn codegen_function<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &ReactiveFunction,
    env: &mut Environment,
    unique_identifiers: FxHashSet<String>,
    fbt_operands: FxHashSet<IdentifierId>,
) -> Result<crate::react_compiler::entrypoint::compile_result::CodegenFunction<'a>, CompilerError> {
    use crate::react_compiler::entrypoint::compile_result::CodegenFunction as OxcCodegenFunction;
    use oxc_span::SPAN;

    let fn_name = func.id.as_deref().unwrap_or("[[ anonymous ]]");
    // Outlined functions reuse the same `fbtOperands` set as the main function
    // (see TS `codegenFunction`), so keep a copy before it is moved into the context.
    let fbt_operands_for_outlined = fbt_operands.clone();
    let mut cx = OxcContext::new(oxc_ast::builder::AstBuilder::new(ast.allocator()), env, fn_name.to_string(), unique_identifiers, fbt_operands);

    // The value-emission port covers most instruction kinds, but a few sub-emitters
    // (function/object/JSX expressions, hook-guard wrapping, TS-type reparse) are
    // deferred to later batches and currently raise an invariant error. Until they
    // land — and until the emitted body is actually spliced into the program — fall
    // back to an empty body on any emission error, preserving the pre-batch behavior
    // (the original program is returned un-memoized by `compile_program`) without
    // surfacing spurious diagnostics. This shim is removed once emission is complete.
    let mut compiled = match ox_codegen_reactive_function(&mut cx, func) {
        Ok(compiled) => compiled,
        Err(_) => OxcCompiledFunction {
            params: ast.alloc_formal_parameters(
                SPAN,
                oxc_ast::ast::FormalParameterKind::FormalParameter,
                ast.vec(),
                None::<oxc_allocator::Box<oxc_ast::ast::FormalParameterRest>>,
            ),
            body: ast.alloc_function_body(SPAN, ast.vec(), ast.vec()),
            generator: false,
            is_async: false,
            memo_slots_used: 0,
            memo_blocks: 0,
            memo_values: 0,
            pruned_memo_blocks: 0,
            pruned_memo_values: 0,
        },
    };

    let cache_count = compiled.memo_slots_used;
    if cache_count != 0 {
        let cache_name = cx.synthesize_name("$");
        // const $ = useMemoCache(N)
        let use_memo_cache = ast.expression_call(
            SPAN,
            ast.expression_identifier(SPAN, "useMemoCache"),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
            ast.vec1(oxc_ast::ast::Argument::from(ox_number(ast, cache_count as f64))),
            false,
        );
        let declarator = ast.variable_declarator(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.binding_pattern_binding_identifier(SPAN, ox_str(ast, &cache_name)),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
            Some(use_memo_cache),
            false,
        );
        let preface = oxc_ast::ast::Statement::VariableDeclaration(ast.alloc_variable_declaration(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            ast.vec1(declarator),
            false,
        ));
        let body_stmts = std::mem::replace(&mut compiled.body.statements, ast.vec());
        let mut new_body = ast.vec1(preface);
        new_body.extend(body_stmts);
        compiled.body.statements = new_body;
    }

    let id = func.id.as_deref().map(|name| ast.binding_identifier(SPAN, ox_str(ast, name)));

    // Release the borrow of `env` held by `cx` so the outlined functions can be
    // compiled with fresh contexts (mirrors TS `codegenFunction`).
    drop(cx);

    let outlined = ox_codegen_outlined(ast, env, fbt_operands_for_outlined)?;

    Ok(OxcCodegenFunction {
        loc: func.loc,
        id,
        name_hint: func.name_hint.clone(),
        params: compiled.params,
        body: compiled.body,
        generator: func.generator,
        is_async: func.is_async,
        memo_slots_used: compiled.memo_slots_used,
        memo_blocks: compiled.memo_blocks,
        memo_values: compiled.memo_values,
        pruned_memo_blocks: compiled.pruned_memo_blocks,
        pruned_memo_values: compiled.pruned_memo_values,
        outlined,
    })
}

/// Compile the functions accumulated during the outlining passes (stored on the
/// `Environment`) into `CodegenFunction`s. Mirrors the `outlined` loop in TS
/// `codegenFunction`: for each entry build its reactive function, run the same
/// prune passes + variable renaming, then codegen it with a fresh context.
fn ox_codegen_outlined<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    env: &mut Environment,
    fbt_operands: FxHashSet<IdentifierId>,
) -> Result<
    Vec<crate::react_compiler::entrypoint::compile_result::OutlinedFunction<'a>>,
    CompilerError,
> {
    use crate::react_compiler::entrypoint::compile_result::OutlinedFunction as OxcOutlinedFunction;

    let entries = env.take_outlined_functions();
    let mut outlined = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut reactive_function = build_reactive_function(&entry.func, env).map_err(|diag| {
            let loc = diag.primary_location().cloned();
            let mut err = CompilerError::new();
            err.push_error_detail(crate::react_compiler_diagnostics::CompilerErrorDetail {
                category: diag.category,
                reason: diag.reason,
                description: diag.description,
                loc,
                suggestions: diag.suggestions,
            });
            err
        })?;
        prune_unused_labels(&mut reactive_function, env)?;
        prune_unused_lvalues(&mut reactive_function, env);
        prune_hoisted_contexts(&mut reactive_function, env)?;

        let identifiers = rename_variables(&mut reactive_function, env);

        let func = codegen_function(ast, &reactive_function, env, identifiers, fbt_operands.clone())?;
        outlined.push(OxcOutlinedFunction { func, fn_type: entry.fn_type });
    }
    Ok(outlined)
}

// =============================================================================
// oxc codegen orchestration
//
// Walks the reactive function tree and builds oxc nodes via `AstBuilder`. The
// HIR-driven control flow mirrors the TS compiler's `CodegenReactiveFunction`.
// =============================================================================

use oxc_ast::ast as oxc;
use oxc_span::SPAN;
use oxc_allocator::GetAllocator;
use crate::react_compiler_reactive_scopes::old_builder_ext::OldBuilderExt;

// Temp value tracking. Maps a temporary's declaration to its emitted oxc value
// (`None` for params/catch bindings that are declared but have no inlinable value).
// oxc nodes are not `Clone`; the snapshot/restore in block codegen and the per-place
// read both clone into the arena via [`CloneIn`] (see `ox_clone_temporaries` /
// `ox_codegen_place`).
type OxcTemporaries<'a> = FxHashMap<DeclarationId, Option<OxValue<'a>>>;

use oxc_allocator::CloneIn;

/// oxc analog of the Babel `ExpressionOrJsxText`: an instruction value is usually an
/// expression, but JSX children codegen can produce raw `JSXText`, which is not an
/// `Expression` in oxc.
enum OxValue<'a> {
    Expression(oxc::Expression<'a>),
    JsxText(oxc_allocator::Box<'a, oxc::JSXText<'a>>),
}

impl<'a> OxValue<'a> {
    fn clone_in(&self, allocator: &'a oxc_allocator::Allocator) -> OxValue<'a> {
        match self {
            OxValue::Expression(e) => OxValue::Expression(e.clone_in(allocator)),
            OxValue::JsxText(t) => OxValue::JsxText(t.clone_in(allocator)),
        }
    }
}

/// Clone the temporaries map, cloning each oxc value into the arena.
fn ox_clone_temporaries<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    temp: &OxcTemporaries<'a>,
) -> OxcTemporaries<'a> {
    temp.iter().map(|(id, v)| (*id, v.as_ref().map(|v| v.clone_in(ast.allocator())))).collect()
}

struct OxcContext<'a, 'env> {
    ast: oxc_ast::builder::AstBuilder<'a>,
    env: &'env mut Environment,
    #[allow(dead_code)]
    fn_name: String,
    next_cache_index: u32,
    declarations: FxHashSet<DeclarationId>,
    temp: OxcTemporaries<'a>,
    object_methods: FxHashMap<
        IdentifierId,
        (InstructionValue, Option<crate::react_compiler_diagnostics::SourceLocation>),
    >,
    unique_identifiers: FxHashSet<String>,
    #[allow(dead_code)]
    fbt_operands: FxHashSet<IdentifierId>,
    synthesized_names: FxHashMap<String, String>,
}

impl<'a, 'env> OxcContext<'a, 'env> {
    fn new(
        ast: oxc_ast::builder::AstBuilder<'a>,
        env: &'env mut Environment,
        fn_name: String,
        unique_identifiers: FxHashSet<String>,
        fbt_operands: FxHashSet<IdentifierId>,
    ) -> Self {
        OxcContext {
            ast,
            env,
            fn_name,
            next_cache_index: 0,
            declarations: FxHashSet::default(),
            temp: FxHashMap::default(),
            object_methods: FxHashMap::default(),
            unique_identifiers,
            fbt_operands,
            synthesized_names: FxHashMap::default(),
        }
    }

    fn alloc_cache_index(&mut self) -> u32 {
        let idx = self.next_cache_index;
        self.next_cache_index += 1;
        idx
    }

    fn declare(&mut self, identifier_id: IdentifierId) {
        let ident = &self.env.identifiers[identifier_id.0 as usize];
        self.declarations.insert(ident.declaration_id);
    }

    fn has_declared(&self, identifier_id: IdentifierId) -> bool {
        let ident = &self.env.identifiers[identifier_id.0 as usize];
        self.declarations.contains(&ident.declaration_id)
    }

    fn synthesize_name(&mut self, name: &str) -> String {
        if let Some(prev) = self.synthesized_names.get(name) {
            return prev.clone();
        }
        let mut validated = name.to_string();
        let mut index = 0u32;
        while self.unique_identifiers.contains(&validated) {
            validated = format!("{name}{index}");
            index += 1;
        }
        self.unique_identifiers.insert(validated.clone());
        self.synthesized_names.insert(name.to_string(), validated.clone());
        validated
    }

    #[allow(dead_code)]
    fn record_error(&mut self, detail: CompilerErrorDetail) -> Result<(), CompilerError> {
        self.env.record_error(detail)
    }
}

/// Intermediate oxc-shaped function: like the Babel `CodegenFunction`, but holding
/// the arena-allocated oxc params/body so `codegen_function` can splice the memo cache.
struct OxcCompiledFunction<'a> {
    params: oxc_allocator::Box<'a, oxc::FormalParameters<'a>>,
    body: oxc_allocator::Box<'a, oxc::FunctionBody<'a>>,
    generator: bool,
    is_async: bool,
    memo_slots_used: u32,
    memo_blocks: u32,
    memo_values: u32,
    pruned_memo_blocks: u32,
    pruned_memo_values: u32,
}

/// JSX text children containing any of these characters must be wrapped in an
/// expression container (`{"..."}`) rather than emitted as raw JSX text.
const JSX_TEXT_CHILD_REQUIRES_EXPR_CONTAINER_PATTERN: &[char] = &['<', '>', '&', '{', '}'];
/// JSX string attribute values containing these characters must be wrapped in an
/// expression container.
const STRING_REQUIRES_EXPR_CONTAINER_CHARS: &str = "\"\\";

/// Reference to an lvalue target during pattern codegen.
enum LvalueRef<'a> {
    Place(&'a Place),
    Pattern(&'a Pattern),
    // Constructed once nested spread/rest lvalue emission is ported; the match arm
    // in `ox_codegen_lvalue` already handles it.
    #[allow(dead_code)]
    Spread(&'a SpreadPattern),
}

fn ox_number<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, value: f64) -> oxc::Expression<'a> {
    ast.expression_numeric_literal(SPAN, value, None, oxc::NumberBase::Decimal)
}

/// Allocate a `&'a str` in the arena (satisfies the builders' `IntoIn` slots for
/// both `Atom` and `Str`).
fn ox_str<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, s: &str) -> &'a str {
    oxc_allocator::StringBuilder::from_str_in(s, ast.allocator()).into_str()
}

/// Re-parse a TS type annotation from its original source span (recorded on the
/// `TypeCastExpression`'s `RawNode` as `type_start`/`type_end`). The lowering only
/// stores the span, so codegen recovers the actual `TSType` AST by re-parsing the
/// source slice. Returns `None` if the source / span is unavailable or unparsable.
fn ox_reparse_ts_type<'a>(
    cx: &OxcContext<'a, '_>,
    raw: &crate::react_compiler_ast::common::RawNode,
) -> Option<oxc::TSType<'a>> {
    let source = cx.env.code.as_deref()?;
    let start = raw.type_start? as usize;
    let end = raw.type_end? as usize;
    if start >= source.len() || end > source.len() || start >= end {
        return None;
    }
    let slice = &source[start..end];
    // Apply identifier renames recorded on the `RawNode` (e.g. `typeof field` ->
    // `typeof field_0` when the value binding was renamed) as text edits, right to
    // left so earlier offsets stay valid, before re-parsing. Mirrors
    // `convert_ast_reverse::convert_type_from_raw`.
    let mut edits: Vec<(usize, usize, &str)> = raw
        .idents
        .iter()
        .filter_map(|id| {
            let renamed = id.renamed_to.as_deref()?;
            let rel = (id.start as usize).checked_sub(start)?;
            Some((rel, id.name.len(), renamed))
        })
        .collect();
    let edited_type: std::borrow::Cow<str> = if edits.is_empty() {
        std::borrow::Cow::Borrowed(slice)
    } else {
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        let mut text = slice.to_string();
        for (rel, old_len, renamed) in edits {
            if rel + old_len <= text.len() {
                text.replace_range(rel..rel + old_len, renamed);
            }
        }
        std::borrow::Cow::Owned(text)
    };
    // Wrap the type in a cast so the parser yields a `TSAsExpression` whose
    // `type_annotation` is exactly the parsed type.
    let wrapped = oxc_allocator::StringBuilder::from_strs_array_in(
        ["let __oxc_t = null as ", &edited_type, ";"],
        cx.ast.allocator(),
    )
    .into_str();
    let parsed =
        oxc_parser::Parser::new(cx.ast.allocator(), wrapped, oxc_span::SourceType::tsx()).parse();
    if parsed.panicked {
        return None;
    }
    let stmt = parsed.program.body.into_iter().next()?;
    let oxc::Statement::VariableDeclaration(decl) = stmt else { return None };
    let init = decl.unbox().declarations.into_iter().next()?.init?;
    let oxc::Expression::TSAsExpression(ts_as) = init else { return None };
    Some(ts_as.unbox().type_annotation)
}

/// Record binding renames on the identifiers inside a type-annotation `RawNode`,
/// so `ox_reparse_ts_type` rewrites them when re-parsing the type from source
/// (`typeof x` -> `typeof x_0`). Only identifiers that are real references
/// (`reference_node_ids`) are considered; for each, the nearest enclosing
/// rename (largest `declaration_start` not past the use site) wins. Mirrors the
/// baseline (Babel-path) `set_raw_type_renames`.
fn set_raw_type_renames(
    raw: &mut crate::react_compiler_ast::common::RawNode,
    renames: &[crate::react_compiler_hir::environment::BindingRename],
    reference_node_ids: &rustc_hash::FxHashSet<u32>,
) {
    if renames.is_empty() {
        return;
    }
    for id in &mut raw.idents {
        if id.node_id == 0 || !reference_node_ids.contains(&id.node_id) {
            continue;
        }
        if let Some(rename) = renames
            .iter()
            .filter(|r| r.original == id.name && r.declaration_start <= id.start)
            .max_by_key(|r| r.declaration_start)
        {
            id.renamed_to = Some(rename.renamed.clone());
        }
    }
}

/// Re-parse a full statement from its original source span. Used to re-emit
/// statement-position `UnsupportedNode`s (e.g. inline TS `enum` declarations)
/// verbatim, mirroring `convert_ast_reverse`'s `extract_source_stmt`.
fn ox_reparse_source_stmt<'a>(
    cx: &OxcContext<'a, '_>,
    base: &crate::react_compiler_ast::common::BaseNode,
) -> Option<oxc::Statement<'a>> {
    let source = cx.env.code.as_deref()?;
    let start = base.start? as usize;
    let end = base.end? as usize;
    if start >= source.len() || end > source.len() || start >= end {
        return None;
    }
    let slice = &source[start..end];
    let text = oxc_allocator::StringBuilder::from_str_in(slice, cx.ast.allocator()).into_str();
    let parsed =
        oxc_parser::Parser::new(cx.ast.allocator(), text, oxc_span::SourceType::tsx()).parse();
    if parsed.panicked || parsed.program.body.is_empty() {
        return None;
    }
    parsed.program.body.into_iter().next()
}

/// Byte-offset span source for a Babel-shaped statement, used to re-parse
/// statement-position `UnsupportedNode`s from the original source.
fn ox_statement_base(
    stmt: &crate::react_compiler_ast::statements::Statement,
) -> &crate::react_compiler_ast::common::BaseNode {
    use crate::react_compiler_ast::statements::Statement as S;
    match stmt {
        S::BlockStatement(s) => &s.base,
        S::ReturnStatement(s) => &s.base,
        S::IfStatement(s) => &s.base,
        S::ForStatement(s) => &s.base,
        S::WhileStatement(s) => &s.base,
        S::DoWhileStatement(s) => &s.base,
        S::ForInStatement(s) => &s.base,
        S::ForOfStatement(s) => &s.base,
        S::SwitchStatement(s) => &s.base,
        S::ThrowStatement(s) => &s.base,
        S::TryStatement(s) => &s.base,
        S::BreakStatement(s) => &s.base,
        S::ContinueStatement(s) => &s.base,
        S::LabeledStatement(s) => &s.base,
        S::ExpressionStatement(s) => &s.base,
        S::EmptyStatement(s) => &s.base,
        S::DebuggerStatement(s) => &s.base,
        S::WithStatement(s) => &s.base,
        S::VariableDeclaration(s) => &s.base,
        S::FunctionDeclaration(s) => &s.base,
        S::ClassDeclaration(s) => &s.base,
        S::ImportDeclaration(s) => &s.base,
        S::ExportNamedDeclaration(s) => &s.base,
        S::ExportDefaultDeclaration(s) => &s.base,
        S::ExportAllDeclaration(s) => &s.base,
        S::TSTypeAliasDeclaration(s) => &s.base,
        S::TSInterfaceDeclaration(s) => &s.base,
        S::TSEnumDeclaration(s) => &s.base,
        S::TSModuleDeclaration(s) => &s.base,
        S::TSDeclareFunction(s) => &s.base,
        S::TypeAlias(s) => &s.base,
        S::OpaqueType(s) => &s.base,
        S::InterfaceDeclaration(s) => &s.base,
        S::DeclareVariable(s) => &s.base,
        S::DeclareFunction(s) => &s.base,
        S::DeclareClass(s) => &s.base,
        S::DeclareModule(s) => &s.base,
        S::DeclareModuleExports(s) => &s.base,
        S::DeclareExportDeclaration(s) => &s.base,
        S::DeclareExportAllDeclaration(s) => &s.base,
        S::DeclareInterface(s) => &s.base,
        S::DeclareTypeAlias(s) => &s.base,
        S::DeclareOpaqueType(s) => &s.base,
        S::EnumDeclaration(s) => &s.base,
        S::Unknown(s) => s.base(),
    }
}

/// Build `Symbol.for("<name>")`.
fn ox_symbol_for<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, name: &str) -> oxc::Expression<'a> {
    let callee = oxc::Expression::from(ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, "Symbol"),
        ast.identifier_name(SPAN, "for"),
        false,
    ));
    ast.expression_call(
        SPAN,
        callee,
        None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
        ast.vec1(oxc::Argument::from(ast.expression_string_literal(SPAN, ox_str(ast, name), None))),
        false,
    )
}

/// `$[index]` computed member expression.
fn ox_cache_index<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    cache_name: &str,
    index: u32,
) -> oxc::Expression<'a> {
    oxc::Expression::from(ast.member_expression_computed(
        SPAN,
        ast.expression_identifier(SPAN, ox_str(ast, cache_name)),
        ox_number(ast, index as f64),
        false,
    ))
}

fn ox_codegen_reactive_function<'a>(
    cx: &mut OxcContext<'a, '_>,
    func: &ReactiveFunction,
) -> Result<OxcCompiledFunction<'a>, CompilerError> {
    // Register parameters
    for param in &func.params {
        let place = match param {
            ParamPattern::Place(p) => p,
            ParamPattern::Spread(sp) => &sp.place,
        };
        let ident = &cx.env.identifiers[place.identifier.0 as usize];
        cx.temp.insert(ident.declaration_id, None);
        cx.declare(place.identifier);
    }

    let params = ox_convert_parameters(cx, &func.params)?;
    let mut statements = ox_codegen_block(cx, &func.body)?;

    // Directives
    let directives = cx.ast.vec_from_iter(func.directives.iter().map(|d| {
        cx.ast.directive(
            SPAN,
            cx.ast.string_literal(SPAN, ox_str(&cx.ast, d), None),
            ox_str(&cx.ast, d),
        )
    }));

    // Remove trailing `return undefined`
    if let Some(oxc::Statement::ReturnStatement(ret)) = statements.last() {
        if ret.argument.is_none() {
            statements.pop();
        }
    }

    let (memo_blocks, memo_values, pruned_memo_blocks, pruned_memo_values) =
        count_memo_blocks(func, cx.env);

    let body = cx.ast.alloc_function_body(SPAN, directives, statements);

    Ok(OxcCompiledFunction {
        params,
        body,
        generator: func.generator,
        is_async: func.is_async,
        memo_slots_used: cx.next_cache_index,
        memo_blocks,
        memo_values,
        pruned_memo_blocks,
        pruned_memo_values,
    })
}

fn ox_convert_parameters<'a>(
    cx: &mut OxcContext<'a, '_>,
    params: &[ParamPattern],
) -> Result<oxc_allocator::Box<'a, oxc::FormalParameters<'a>>, CompilerError> {
    let mut items: Vec<oxc::FormalParameter<'a>> = Vec::new();
    let mut rest: Option<oxc::FormalParameterRest<'a>> = None;
    for param in params {
        match param {
            ParamPattern::Place(place) => {
                let binding = ox_binding_for_identifier(cx, place.identifier)?;
                items.push(cx.ast.formal_parameter(
                    SPAN,
                    cx.ast.vec(),
                    binding,
                    None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                    None::<oxc_allocator::Box<oxc::Expression>>,
                    false,
                    None,
                    false,
                    false,
                ));
            }
            ParamPattern::Spread(spread) => {
                let binding = ox_binding_for_identifier(cx, spread.place.identifier)?;
                let rest_elem = cx.ast.binding_rest_element(SPAN, binding);
                rest = Some(cx.ast.formal_parameter_rest(
                    SPAN,
                    cx.ast.vec(),
                    rest_elem,
                    None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                ));
            }
        }
    }
    let items_vec = cx.ast.vec_from_iter(items);
    Ok(cx.ast.alloc_formal_parameters(
        SPAN,
        oxc::FormalParameterKind::FormalParameter,
        items_vec,
        rest,
    ))
}

fn ox_binding_for_identifier<'a>(
    cx: &OxcContext<'a, '_>,
    identifier_id: IdentifierId,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let name = ox_identifier_name(cx.env, identifier_id)?;
    Ok(cx.ast.binding_pattern_binding_identifier(SPAN, ox_str(&cx.ast, &name)))
}

fn ox_identifier_name(
    env: &Environment,
    identifier_id: IdentifierId,
) -> Result<String, CompilerError> {
    let ident = &env.identifiers[identifier_id.0 as usize];
    match &ident.name {
        Some(crate::react_compiler_hir::IdentifierName::Named(n)) => Ok(n.clone()),
        Some(crate::react_compiler_hir::IdentifierName::Promoted(n)) => Ok(n.clone()),
        None => Err(invariant_err(
            "Expected temporaries to be promoted to named identifiers in an earlier pass",
            None,
        )),
    }
}

// =============================================================================
// Block codegen (oxc)
// =============================================================================

fn ox_codegen_block<'a>(
    cx: &mut OxcContext<'a, '_>,
    block: &ReactiveBlock,
) -> Result<oxc_allocator::Vec<'a, oxc::Statement<'a>>, CompilerError> {
    let temp_snapshot = ox_clone_temporaries(&cx.ast, &cx.temp);
    let result = ox_codegen_block_no_reset(cx, block)?;
    cx.temp = temp_snapshot;
    Ok(result)
}

fn ox_codegen_block_no_reset<'a>(
    cx: &mut OxcContext<'a, '_>,
    block: &ReactiveBlock,
) -> Result<oxc_allocator::Vec<'a, oxc::Statement<'a>>, CompilerError> {
    let mut statements: oxc_allocator::Vec<'a, oxc::Statement<'a>> = cx.ast.vec();
    for item in block {
        match item {
            ReactiveStatement::Instruction(instr) => {
                if let Some(stmt) = ox_codegen_instruction_nullable(cx, instr)? {
                    statements.push(stmt);
                }
            }
            ReactiveStatement::PrunedScope(PrunedReactiveScopeBlock { instructions, .. }) => {
                let scope_block = ox_codegen_block_no_reset(cx, instructions)?;
                statements.extend(scope_block);
            }
            ReactiveStatement::Scope(ReactiveScopeBlock { scope, instructions }) => {
                let temp_snapshot = ox_clone_temporaries(&cx.ast, &cx.temp);
                ox_codegen_reactive_scope(cx, &mut statements, *scope, instructions)?;
                cx.temp = temp_snapshot;
            }
            ReactiveStatement::Terminal(term_stmt) => {
                let stmt = ox_codegen_terminal(cx, &term_stmt.terminal)?;
                let Some(stmt) = stmt else {
                    continue;
                };
                if let Some(ref label) = term_stmt.label {
                    if !label.implicit {
                        let inner = match stmt {
                            oxc::Statement::BlockStatement(mut bs) if bs.body.len() == 1 => {
                                bs.body.pop().unwrap()
                            }
                            other => other,
                        };
                        let label_ident = cx
                            .ast
                            .label_identifier(SPAN, ox_str(&cx.ast, &codegen_label(label.id)));
                        statements.push(cx.ast.statement_labeled(SPAN, label_ident, inner));
                    } else if let oxc::Statement::BlockStatement(bs) = stmt {
                        let bs = bs.unbox();
                        statements.extend(bs.body);
                    } else {
                        statements.push(stmt);
                    }
                } else if let oxc::Statement::BlockStatement(bs) = stmt {
                    let bs = bs.unbox();
                    statements.extend(bs.body);
                } else {
                    statements.push(stmt);
                }
            }
        }
    }
    Ok(statements)
}

fn ox_codegen_block_statement<'a>(
    cx: &mut OxcContext<'a, '_>,
    block: &ReactiveBlock,
) -> Result<oxc::BlockStatement<'a>, CompilerError> {
    let body = ox_codegen_block(cx, block)?;
    Ok(cx.ast.block_statement(SPAN, body))
}

// =============================================================================
// Reactive scope codegen (memoization) (oxc)
// =============================================================================

fn ox_codegen_reactive_scope<'a>(
    cx: &mut OxcContext<'a, '_>,
    statements: &mut oxc_allocator::Vec<'a, oxc::Statement<'a>>,
    scope_id: ScopeId,
    block: &ReactiveBlock,
) -> Result<(), CompilerError> {
    let scope_deps = cx.env.scopes[scope_id.0 as usize].dependencies.clone();
    let scope_decls = cx.env.scopes[scope_id.0 as usize].declarations.clone();
    let scope_reassignments = cx.env.scopes[scope_id.0 as usize].reassignments.clone();

    let mut cache_store_stmts: oxc_allocator::Vec<'a, oxc::Statement<'a>> = cx.ast.vec();
    let mut cache_load_stmts: oxc_allocator::Vec<'a, oxc::Statement<'a>> = cx.ast.vec();
    let mut cache_loads: Vec<(String, u32)> = Vec::new();
    let mut change_exprs: Vec<oxc::Expression<'a>> = Vec::new();

    let mut deps = scope_deps;
    deps.sort_by(|a, b| compare_scope_dependency(a, b, cx.env));

    for dep in &deps {
        let index = cx.alloc_cache_index();
        let cache_name = cx.synthesize_name("$");
        let dep_expr = ox_codegen_dependency(cx, dep)?;
        let comparison = cx.ast.expression_binary(
            SPAN,
            ox_cache_index(&cx.ast, &cache_name, index),
            oxc::BinaryOperator::StrictInequality,
            dep_expr,
        );
        change_exprs.push(comparison);

        let dep_value = ox_codegen_dependency(cx, dep)?;
        let store = cx.ast.expression_assignment(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(ast_member_target(
                &cx.ast,
                &cache_name,
                index,
            ))),
            dep_value,
        );
        cache_store_stmts.push(cx.ast.statement_expression(SPAN, store));
    }

    let mut first_output_index: Option<u32> = None;

    let mut decls = scope_decls;
    decls.sort_by(|(_id_a, a), (_id_b, b)| compare_scope_declaration(a, b, cx.env));

    for (_ident_id, decl) in &decls {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }
        let name = ox_identifier_name(cx.env, decl.identifier)?;
        if !cx.has_declared(decl.identifier) {
            let declarator = cx.ast.variable_declarator(
                SPAN,
                oxc::VariableDeclarationKind::Let,
                cx.ast.binding_pattern_binding_identifier(SPAN, ox_str(&cx.ast, &name)),
                None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                None,
                false,
            );
            statements.push(oxc::Statement::VariableDeclaration(
                cx.ast.alloc_variable_declaration(
                    SPAN,
                    oxc::VariableDeclarationKind::Let,
                    cx.ast.vec1(declarator),
                    false,
                ),
            ));
        }
        cache_loads.push((name, index));
        cx.declare(decl.identifier);
    }

    for reassignment_id in scope_reassignments {
        let index = cx.alloc_cache_index();
        if first_output_index.is_none() {
            first_output_index = Some(index);
        }
        let name = ox_identifier_name(cx.env, reassignment_id)?;
        cache_loads.push((name, index));
    }

    let test_condition = if change_exprs.is_empty() {
        let first_idx = first_output_index.ok_or_else(|| {
            invariant_err("Expected scope to have at least one declaration", None)
        })?;
        let cache_name = cx.synthesize_name("$");
        cx.ast.expression_binary(
            SPAN,
            ox_cache_index(&cx.ast, &cache_name, first_idx),
            oxc::BinaryOperator::StrictEquality,
            ox_symbol_for(&cx.ast, MEMO_CACHE_SENTINEL),
        )
    } else {
        change_exprs
            .into_iter()
            .reduce(|acc, expr| {
                cx.ast.expression_logical(SPAN, acc, oxc::LogicalOperator::Or, expr)
            })
            .unwrap()
    };

    let mut computation_body = ox_codegen_block(cx, block)?;

    for (name, index) in &cache_loads {
        let cache_name = cx.synthesize_name("$");
        // $[index] = name
        let store = cx.ast.expression_assignment(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(ast_member_target(
                &cx.ast,
                &cache_name,
                *index,
            ))),
            cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, name)),
        );
        cache_store_stmts.push(cx.ast.statement_expression(SPAN, store));
        // name = $[index]
        let load = cx.ast.expression_assignment(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::AssignmentTargetIdentifier(
                cx.ast.alloc_identifier_reference(SPAN, ox_str(&cx.ast, name)),
            ),
            ox_cache_index(&cx.ast, &cache_name, *index),
        );
        cache_load_stmts.push(cx.ast.statement_expression(SPAN, load));
    }

    computation_body.extend(cache_store_stmts);

    let memo_stmt = cx.ast.statement_if(
        SPAN,
        test_condition,
        cx.ast.statement_block(SPAN, computation_body),
        Some(cx.ast.statement_block(SPAN, cache_load_stmts)),
    );
    statements.push(memo_stmt);

    // Early return
    let early_return_value = cx.env.scopes[scope_id.0 as usize].early_return_value.clone();
    if let Some(ref early_return) = early_return_value {
        let early_ident = &cx.env.identifiers[early_return.value.0 as usize];
        let name = match &early_ident.name {
            Some(crate::react_compiler_hir::IdentifierName::Named(n)) => n.clone(),
            Some(crate::react_compiler_hir::IdentifierName::Promoted(n)) => n.clone(),
            None => {
                return Err(invariant_err(
                    "Expected early return value to be promoted to a named variable",
                    early_return.loc,
                ));
            }
        };
        let test = cx.ast.expression_binary(
            SPAN,
            cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, &name)),
            oxc::BinaryOperator::StrictInequality,
            ox_symbol_for(&cx.ast, EARLY_RETURN_SENTINEL),
        );
        let return_stmt = cx.ast.statement_return(
            SPAN,
            Some(cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, &name))),
        );
        let consequent = cx.ast.statement_block(SPAN, cx.ast.vec1(return_stmt));
        statements.push(cx.ast.statement_if(SPAN, test, consequent, None));
    }

    Ok(())
}

/// Build `$[index]` as a `MemberExpression` for use as an assignment target.
fn ast_member_target<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    cache_name: &str,
    index: u32,
) -> oxc::MemberExpression<'a> {
    ast.member_expression_computed(
        SPAN,
        ast.expression_identifier(SPAN, ox_str(ast, cache_name)),
        ox_number(ast, index as f64),
        false,
    )
}

// =============================================================================
// Terminal codegen (oxc)
// =============================================================================

fn ox_codegen_terminal<'a>(
    cx: &mut OxcContext<'a, '_>,
    terminal: &ReactiveTerminal,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    match terminal {
        ReactiveTerminal::Break { target, target_kind, .. } => {
            if *target_kind == ReactiveTerminalTargetKind::Implicit {
                return Ok(None);
            }
            let label = if *target_kind == ReactiveTerminalTargetKind::Labeled {
                Some(cx.ast.label_identifier(SPAN, ox_str(&cx.ast, &codegen_label(*target))))
            } else {
                None
            };
            Ok(Some(cx.ast.statement_break(SPAN, label)))
        }
        ReactiveTerminal::Continue { target, target_kind, .. } => {
            if *target_kind == ReactiveTerminalTargetKind::Implicit {
                return Ok(None);
            }
            let label = if *target_kind == ReactiveTerminalTargetKind::Labeled {
                Some(cx.ast.label_identifier(SPAN, ox_str(&cx.ast, &codegen_label(*target))))
            } else {
                None
            };
            Ok(Some(cx.ast.statement_continue(SPAN, label)))
        }
        ReactiveTerminal::Return { value, .. } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            if let oxc::Expression::Identifier(ref ident) = expr {
                if ident.name == "undefined" {
                    return Ok(Some(cx.ast.statement_return(SPAN, None)));
                }
            }
            Ok(Some(cx.ast.statement_return(SPAN, Some(expr))))
        }
        ReactiveTerminal::Throw { value, .. } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            Ok(Some(cx.ast.statement_throw(SPAN, expr)))
        }
        ReactiveTerminal::If { test, consequent, alternate, .. } => {
            let test_expr = ox_codegen_place_to_expression(cx, test)?;
            let consequent_block = ox_codegen_block_statement(cx, consequent)?;
            let consequent = oxc::Statement::BlockStatement(cx.ast.alloc(consequent_block));
            let alternate = if let Some(alt) = alternate {
                let block = ox_codegen_block_statement(cx, alt)?;
                if block.body.is_empty() {
                    None
                } else {
                    Some(oxc::Statement::BlockStatement(cx.ast.alloc(block)))
                }
            } else {
                None
            };
            Ok(Some(cx.ast.statement_if(SPAN, test_expr, consequent, alternate)))
        }
        ReactiveTerminal::Switch { test, cases, .. } => {
            let test_expr = ox_codegen_place_to_expression(cx, test)?;
            let mut switch_cases: oxc_allocator::Vec<'a, oxc::SwitchCase<'a>> = cx.ast.vec();
            for case in cases {
                let case_test = case
                    .test
                    .as_ref()
                    .map(|t| ox_codegen_place_to_expression(cx, t))
                    .transpose()?;
                let block =
                    case.block.as_ref().map(|b| ox_codegen_block_statement(cx, b)).transpose()?;
                let consequent: oxc_allocator::Vec<'a, oxc::Statement<'a>> = match block {
                    Some(b) if b.body.is_empty() => cx.ast.vec(),
                    Some(b) => cx.ast.vec1(oxc::Statement::BlockStatement(cx.ast.alloc(b))),
                    None => cx.ast.vec(),
                };
                switch_cases.push(cx.ast.switch_case(SPAN, case_test, consequent));
            }
            Ok(Some(cx.ast.statement_switch(SPAN, test_expr, switch_cases)))
        }
        ReactiveTerminal::DoWhile { loop_block, test, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body = oxc::Statement::BlockStatement(cx.ast.alloc(body));
            Ok(Some(cx.ast.statement_do_while(SPAN, body, test_expr)))
        }
        ReactiveTerminal::While { test, loop_block, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body = oxc::Statement::BlockStatement(cx.ast.alloc(body));
            Ok(Some(cx.ast.statement_while(SPAN, test_expr, body)))
        }
        ReactiveTerminal::For { init, test, update, loop_block, .. } => {
            let init_val = ox_codegen_for_init(cx, init)?;
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let update_expr = update
                .as_ref()
                .map(|u| ox_codegen_instruction_value_to_expression(cx, u))
                .transpose()?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body = oxc::Statement::BlockStatement(cx.ast.alloc(body));
            Ok(Some(cx.ast.statement_for(SPAN, init_val, Some(test_expr), update_expr, body)))
        }
        ReactiveTerminal::ForIn { init, loop_block, loc, .. } => {
            ox_codegen_for_in(cx, init, loop_block, *loc)
        }
        ReactiveTerminal::ForOf { init, test, loop_block, loc, .. } => {
            ox_codegen_for_of(cx, init, test, loop_block, *loc)
        }
        ReactiveTerminal::Label { block, .. } => {
            let body = ox_codegen_block_statement(cx, block)?;
            Ok(Some(oxc::Statement::BlockStatement(cx.ast.alloc(body))))
        }
        ReactiveTerminal::Try { block, handler_binding, handler, .. } => {
            let catch_param = match handler_binding.as_ref() {
                Some(binding) => {
                    let ident = &cx.env.identifiers[binding.identifier.0 as usize];
                    cx.temp.insert(ident.declaration_id, None);
                    let pattern = ox_binding_for_identifier(cx, binding.identifier)?;
                    Some(cx.ast.catch_parameter(
                        SPAN,
                        pattern,
                        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                    ))
                }
                None => None,
            };
            let try_block = ox_codegen_block_statement(cx, block)?;
            let handler_block = ox_codegen_block_statement(cx, handler)?;
            let handler = cx.ast.catch_clause(SPAN, catch_param, handler_block);
            Ok(Some(cx.ast.statement_try(
                SPAN,
                try_block,
                Some(handler),
                None::<oxc_allocator::Box<oxc::BlockStatement>>,
            )))
        }
    }
}

fn ox_codegen_for_in<'a>(
    cx: &mut OxcContext<'a, '_>,
    init: &ReactiveValue,
    loop_block: &ReactiveBlock,
    loc: Option<DiagSourceLocation>,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    let ReactiveValue::SequenceExpression { instructions, .. } = init else {
        return Err(invariant_err("Expected a sequence expression init for for..in", None));
    };
    if instructions.len() != 2 {
        cx.record_error(CompilerErrorDetail {
            category: ErrorCategory::Todo,
            reason: "Support non-trivial for..in inits".to_string(),
            description: None,
            loc,
            suggestions: None,
        })?;
        return Ok(Some(cx.ast.statement_empty(SPAN)));
    }
    let iterable_collection = &instructions[0];
    let iterable_item = &instructions[1];
    let instr_value = get_instruction_value(&iterable_item.value)?;
    let (lval, var_decl_kind) = ox_extract_for_in_of_lval(cx, instr_value, "for..in", loc)?;
    let right = ox_codegen_instruction_value_to_expression(cx, &iterable_collection.value)?;
    let body = ox_codegen_block_statement(cx, loop_block)?;
    let body = oxc::Statement::BlockStatement(cx.ast.alloc(body));
    let declarator = cx.ast.variable_declarator(
        SPAN,
        var_decl_kind,
        lval,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        None,
        false,
    );
    let decl =
        cx.ast.alloc_variable_declaration(SPAN, var_decl_kind, cx.ast.vec1(declarator), false);
    let left = oxc::ForStatementLeft::VariableDeclaration(decl);
    Ok(Some(cx.ast.statement_for_in(SPAN, left, right, body)))
}

fn ox_codegen_for_of<'a>(
    cx: &mut OxcContext<'a, '_>,
    init: &ReactiveValue,
    test: &ReactiveValue,
    loop_block: &ReactiveBlock,
    loc: Option<DiagSourceLocation>,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    let ReactiveValue::SequenceExpression { instructions: init_instrs, .. } = init else {
        return Err(invariant_err("Expected a sequence expression init for for..of", None));
    };
    if init_instrs.len() != 1 {
        return Err(invariant_err(
            "Expected a single-expression sequence expression init for for..of",
            None,
        ));
    }
    let get_iter_value = get_instruction_value(&init_instrs[0].value)?;
    let InstructionValue::GetIterator { collection, .. } = get_iter_value else {
        return Err(invariant_err("Expected GetIterator in for..of init", None));
    };

    let ReactiveValue::SequenceExpression { instructions: test_instrs, .. } = test else {
        return Err(invariant_err("Expected a sequence expression test for for..of", None));
    };
    if test_instrs.len() != 2 {
        cx.record_error(CompilerErrorDetail {
            category: ErrorCategory::Todo,
            reason: "Support non-trivial for..of inits".to_string(),
            description: None,
            loc,
            suggestions: None,
        })?;
        return Ok(Some(cx.ast.statement_empty(SPAN)));
    }
    let iterable_item = &test_instrs[1];
    let instr_value = get_instruction_value(&iterable_item.value)?;
    let (lval, var_decl_kind) = ox_extract_for_in_of_lval(cx, instr_value, "for..of", loc)?;

    let right = ox_codegen_place_to_expression(cx, collection)?;
    let body = ox_codegen_block_statement(cx, loop_block)?;
    let body = oxc::Statement::BlockStatement(cx.ast.alloc(body));
    let declarator = cx.ast.variable_declarator(
        SPAN,
        var_decl_kind,
        lval,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        None,
        false,
    );
    let decl =
        cx.ast.alloc_variable_declaration(SPAN, var_decl_kind, cx.ast.vec1(declarator), false);
    let left = oxc::ForStatementLeft::VariableDeclaration(decl);
    Ok(Some(cx.ast.statement_for_of(SPAN, false, left, right, body)))
}

fn ox_extract_for_in_of_lval<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr_value: &InstructionValue,
    context_name: &str,
    loc: Option<DiagSourceLocation>,
) -> Result<(oxc::BindingPattern<'a>, oxc::VariableDeclarationKind), CompilerError> {
    let (lval, kind) = match instr_value {
        InstructionValue::StoreLocal { lvalue, .. } => {
            (ox_codegen_lvalue(cx, &LvalueRef::Place(&lvalue.place))?, lvalue.kind)
        }
        InstructionValue::Destructure { lvalue, .. } => {
            (ox_codegen_lvalue(cx, &LvalueRef::Pattern(&lvalue.pattern))?, lvalue.kind)
        }
        InstructionValue::StoreContext { .. } => {
            cx.record_error(CompilerErrorDetail {
                category: ErrorCategory::Todo,
                reason: format!("Support non-trivial {} inits", context_name),
                description: None,
                loc,
                suggestions: None,
            })?;
            return Ok((
                cx.ast.binding_pattern_binding_identifier(SPAN, "_"),
                oxc::VariableDeclarationKind::Let,
            ));
        }
        _ => {
            return Err(invariant_err(
                &format!(
                    "Expected a StoreLocal or Destructure in {} collection, found {:?}",
                    context_name,
                    std::mem::discriminant(instr_value)
                ),
                None,
            ));
        }
    };
    let var_decl_kind = match kind {
        InstructionKind::Const => oxc::VariableDeclarationKind::Const,
        InstructionKind::Let => oxc::VariableDeclarationKind::Let,
        _ => {
            return Err(invariant_err(
                &format!("Unexpected {:?} variable in {} collection", kind, context_name),
                None,
            ));
        }
    };
    Ok((lval, var_decl_kind))
}

fn ox_codegen_for_init<'a>(
    cx: &mut OxcContext<'a, '_>,
    init: &ReactiveValue,
) -> Result<Option<oxc::ForStatementInit<'a>>, CompilerError> {
    if let ReactiveValue::SequenceExpression { instructions, .. } = init {
        let block_items: Vec<ReactiveStatement> =
            instructions.iter().map(|i| ReactiveStatement::Instruction(i.clone())).collect();
        let body = ox_codegen_block(cx, &block_items)?;
        let mut declarators: oxc_allocator::Vec<'a, oxc::VariableDeclarator<'a>> = cx.ast.vec();
        let mut kind = oxc::VariableDeclarationKind::Const;
        for stmt in body {
            // Fold `name = init` assignment into the last declarator when possible.
            if let oxc::Statement::ExpressionStatement(ref expr_stmt) = stmt {
                if let oxc::Expression::AssignmentExpression(ref assign) = expr_stmt.expression {
                    if matches!(assign.operator, oxc::AssignmentOperator::Assign) {
                        if let oxc::AssignmentTarget::AssignmentTargetIdentifier(ref left_ident) =
                            assign.left
                        {
                            if let Some(top) = declarators.last_mut() {
                                if let oxc::BindingPattern::BindingIdentifier(ref top_ident) =
                                    top.id
                                {
                                    if top_ident.name == left_ident.name && top.init.is_none() {
                                        // Move the assignment's right-hand side into the declarator.
                                        if let oxc::Statement::ExpressionStatement(expr_stmt) = stmt
                                        {
                                            if let oxc::Expression::AssignmentExpression(assign) =
                                                expr_stmt.unbox().expression
                                            {
                                                top.init = Some(assign.unbox().right);
                                            }
                                        }
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let oxc::Statement::VariableDeclaration(var_decl) = stmt {
                let var_decl = var_decl.unbox();
                match var_decl.kind {
                    oxc::VariableDeclarationKind::Let | oxc::VariableDeclarationKind::Const => {}
                    _ => {
                        return Err(invariant_err(
                            "Expected a let or const variable declaration",
                            None,
                        ));
                    }
                }
                if matches!(var_decl.kind, oxc::VariableDeclarationKind::Let) {
                    kind = oxc::VariableDeclarationKind::Let;
                }
                declarators.extend(var_decl.declarations);
            } else {
                return Err(invariant_err("Expected a variable declaration", None));
            }
        }
        if declarators.is_empty() {
            return Err(invariant_err("Expected a variable declaration in for-init", None));
        }
        let decl = cx.ast.alloc_variable_declaration(SPAN, kind, declarators, false);
        Ok(Some(oxc::ForStatementInit::VariableDeclaration(decl)))
    } else {
        let expr = ox_codegen_instruction_value_to_expression(cx, init)?;
        Ok(Some(oxc::ForStatementInit::from(expr)))
    }
}

// =============================================================================
// Per-instruction value emission (oxc).
//
// Ports the Babel reference value tree-walk (`codegen_instruction*`,
// `codegen_store_or_declare`, `emit_store`, `codegen_instruction_value`,
// `codegen_base_instruction_value`, `codegen_place`, `codegen_lvalue`,
// `codegen_argument`, `codegen_dependency`) to build oxc nodes via `AstBuilder`.
// The HIR-driven control flow is identical; only node construction differs. Since
// oxc tracks positions by `Span` (not Babel-style locs), the per-node loc
// propagation (`apply_loc_to_value` / place-loc overrides) collapses to `SPAN`.
//
// `FunctionExpression` / `ObjectExpression` / JSX / non-trivial `TypeCastExpression`
// emission are deferred to later batches and currently raise an invariant error
// (which fails compilation of that function and falls back to the original program,
// matching the current differential floor).
// =============================================================================

fn ox_convert_value_to_expression<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    value: OxValue<'a>,
) -> oxc::Expression<'a> {
    match value {
        OxValue::Expression(e) => e,
        OxValue::JsxText(text) => ast.expression_string_literal(SPAN, text.value.as_str(), None),
    }
}

fn ox_codegen_instruction_nullable<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr: &ReactiveInstruction,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    if let ReactiveValue::Instruction(ref value) = instr.value {
        match value {
            InstructionValue::StoreLocal { .. }
            | InstructionValue::StoreContext { .. }
            | InstructionValue::Destructure { .. }
            | InstructionValue::DeclareLocal { .. }
            | InstructionValue::DeclareContext { .. } => {
                return ox_codegen_store_or_declare(cx, instr, value);
            }
            InstructionValue::StartMemoize { .. } | InstructionValue::FinishMemoize { .. } => {
                return Ok(None);
            }
            InstructionValue::Debugger { .. } => {
                return Ok(Some(cx.ast.statement_debugger(SPAN)));
            }
            InstructionValue::ObjectMethod { loc, .. } => {
                invariant(
                    instr.lvalue.is_some(),
                    "Expected object methods to have a temp lvalue",
                    None,
                )?;
                let lvalue = instr.lvalue.as_ref().unwrap();
                cx.object_methods.insert(lvalue.identifier, (value.clone(), *loc));
                return Ok(None);
            }
            InstructionValue::UnsupportedNode {
                original_node: Some(crate::react_compiler_ast::OriginalNode::Statement(stmt)),
                ..
            } => {
                // Statement-position unsupported node (e.g. an inline TS `enum`
                // declaration): re-emit it verbatim by re-parsing its original
                // source span into an oxc statement, mirroring the Babel path's
                // `return node` for non-expression original nodes.
                let reparsed = ox_reparse_source_stmt(cx, ox_statement_base(stmt));
                return match reparsed {
                    Some(oxc_stmt) => Ok(Some(oxc_stmt)),
                    None => Err(invariant_err(
                        "Failed to re-parse unsupported statement node from source",
                        None,
                    )),
                };
            }
            _ => {}
        }
    }
    let expr_value = ox_codegen_instruction_value(cx, &instr.value)?;
    let stmt = ox_codegen_instruction(cx, instr, expr_value)?;
    if matches!(stmt, oxc::Statement::EmptyStatement(_)) { Ok(None) } else { Ok(Some(stmt)) }
}

fn ox_codegen_store_or_declare<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr: &ReactiveInstruction,
    value: &InstructionValue,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    match value {
        InstructionValue::StoreLocal { lvalue, value: val, .. } => {
            let mut kind = lvalue.kind;
            if cx.has_declared(lvalue.place.identifier) {
                kind = InstructionKind::Reassign;
            }
            let rhs = ox_codegen_place_to_expression(cx, val)?;
            ox_emit_store(cx, instr, kind, &LvalueRef::Place(&lvalue.place), Some(rhs))
        }
        InstructionValue::StoreContext { lvalue, value: val, .. } => {
            let rhs = ox_codegen_place_to_expression(cx, val)?;
            ox_emit_store(cx, instr, lvalue.kind, &LvalueRef::Place(&lvalue.place), Some(rhs))
        }
        InstructionValue::DeclareLocal { lvalue, .. }
        | InstructionValue::DeclareContext { lvalue, .. } => {
            if cx.has_declared(lvalue.place.identifier) {
                return Ok(None);
            }
            ox_emit_store(cx, instr, lvalue.kind, &LvalueRef::Place(&lvalue.place), None)
        }
        InstructionValue::Destructure { lvalue, value: val, .. } => {
            let kind = lvalue.kind;
            for place in crate::react_compiler_hir::visitors::each_pattern_operand(&lvalue.pattern)
            {
                let ident = &cx.env.identifiers[place.identifier.0 as usize];
                if kind != InstructionKind::Reassign && ident.name.is_none() {
                    cx.temp.insert(ident.declaration_id, None);
                }
            }
            let rhs = ox_codegen_place_to_expression(cx, val)?;
            ox_emit_store(cx, instr, kind, &LvalueRef::Pattern(&lvalue.pattern), Some(rhs))
        }
        _ => unreachable!(),
    }
}

fn ox_emit_store<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr: &ReactiveInstruction,
    kind: InstructionKind,
    lvalue: &LvalueRef,
    value: Option<oxc::Expression<'a>>,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    match kind {
        InstructionKind::Const => {
            if instr.lvalue.is_some() {
                return Err(invariant_err_with_detail_message(
                    "Const declaration cannot be referenced as an expression",
                    "this is Const",
                    instr.loc,
                ));
            }
            let lval = ox_codegen_lvalue(cx, lvalue)?;
            Ok(Some(ox_make_var_decl(cx, oxc::VariableDeclarationKind::Const, lval, value)))
        }
        InstructionKind::Function => {
            let lval = ox_codegen_lvalue(cx, lvalue)?;
            let oxc::BindingPattern::BindingIdentifier(fn_id) = lval else {
                return Err(invariant_err(
                    "Expected an identifier as function declaration lvalue",
                    None,
                ));
            };
            let Some(rhs) = value else {
                return Err(invariant_err(
                    "Expected a function value for function declaration",
                    None,
                ));
            };
            match rhs {
                oxc::Expression::FunctionExpression(func_expr) => {
                    let func_expr = func_expr.unbox();
                    let decl = cx.ast.alloc_function(
                        SPAN,
                        oxc::FunctionType::FunctionDeclaration,
                        Some(fn_id.unbox()),
                        func_expr.generator,
                        func_expr.r#async,
                        false,
                        func_expr.type_parameters,
                        func_expr.this_param,
                        func_expr.params,
                        func_expr.return_type,
                        func_expr.body,
                    );
                    Ok(Some(oxc::Statement::FunctionDeclaration(decl)))
                }
                _ => Err(invariant_err(
                    "Expected a function expression for function declaration",
                    None,
                )),
            }
        }
        InstructionKind::Let => {
            if instr.lvalue.is_some() {
                return Err(invariant_err_with_detail_message(
                    "Const declaration cannot be referenced as an expression",
                    "this is Let",
                    instr.loc,
                ));
            }
            let lval = ox_codegen_lvalue(cx, lvalue)?;
            Ok(Some(ox_make_var_decl(cx, oxc::VariableDeclarationKind::Let, lval, value)))
        }
        InstructionKind::Reassign => {
            let Some(rhs) = value else {
                return Err(invariant_err("Expected a value for reassignment", None));
            };
            let lval = ox_codegen_lvalue(cx, lvalue)?;
            let target = ox_binding_pattern_to_assignment_target(cx, lval)?;
            let expr =
                cx.ast.expression_assignment(SPAN, oxc::AssignmentOperator::Assign, target, rhs);
            if let Some(ref lvalue_place) = instr.lvalue {
                let is_store_context = matches!(
                    &instr.value,
                    ReactiveValue::Instruction(InstructionValue::StoreContext { .. })
                );
                if !is_store_context {
                    let ident = &cx.env.identifiers[lvalue_place.identifier.0 as usize];
                    cx.temp.insert(ident.declaration_id, Some(OxValue::Expression(expr)));
                    return Ok(None);
                }
                let stmt = ox_codegen_instruction(cx, instr, OxValue::Expression(expr))?;
                if matches!(stmt, oxc::Statement::EmptyStatement(_)) {
                    return Ok(None);
                }
                return Ok(Some(stmt));
            }
            Ok(Some(cx.ast.statement_expression(SPAN, expr)))
        }
        InstructionKind::Catch => Ok(Some(cx.ast.statement_empty(SPAN))),
        InstructionKind::HoistedLet
        | InstructionKind::HoistedConst
        | InstructionKind::HoistedFunction => Err(invariant_err(
            &format!("Expected {:?} to have been pruned in PruneHoistedContexts", kind),
            None,
        )),
    }
}

/// Build `kind id = init;` (or `kind id;` when `init` is `None`).
fn ox_make_var_decl<'a>(
    cx: &OxcContext<'a, '_>,
    kind: oxc::VariableDeclarationKind,
    id: oxc::BindingPattern<'a>,
    init: Option<oxc::Expression<'a>>,
) -> oxc::Statement<'a> {
    let declarator = cx.ast.variable_declarator(
        SPAN,
        kind,
        id,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        init,
        false,
    );
    oxc::Statement::VariableDeclaration(cx.ast.alloc_variable_declaration(
        SPAN,
        kind,
        cx.ast.vec1(declarator),
        false,
    ))
}

fn ox_codegen_instruction<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr: &ReactiveInstruction,
    value: OxValue<'a>,
) -> Result<oxc::Statement<'a>, CompilerError> {
    let Some(ref lvalue) = instr.lvalue else {
        let expr = ox_convert_value_to_expression(&cx.ast, value);
        return Ok(cx.ast.statement_expression(SPAN, expr));
    };
    let ident = &cx.env.identifiers[lvalue.identifier.0 as usize];
    if ident.name.is_none() {
        cx.temp.insert(ident.declaration_id, Some(value));
        return Ok(cx.ast.statement_empty(SPAN));
    }
    let expr_value = ox_convert_value_to_expression(&cx.ast, value);
    let name = ox_identifier_name(cx.env, lvalue.identifier)?;
    if cx.has_declared(lvalue.identifier) {
        let target = oxc::AssignmentTarget::AssignmentTargetIdentifier(
            cx.ast.alloc_identifier_reference(SPAN, ox_str(&cx.ast, &name)),
        );
        let expr =
            cx.ast.expression_assignment(SPAN, oxc::AssignmentOperator::Assign, target, expr_value);
        Ok(cx.ast.statement_expression(SPAN, expr))
    } else {
        let id = cx.ast.binding_pattern_binding_identifier(SPAN, ox_str(&cx.ast, &name));
        Ok(ox_make_var_decl(cx, oxc::VariableDeclarationKind::Const, id, Some(expr_value)))
    }
}

// =============================================================================
// Instruction value codegen (oxc)
// =============================================================================

fn ox_codegen_instruction_value_to_expression<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr_value: &ReactiveValue,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let value = ox_codegen_instruction_value(cx, instr_value)?;
    Ok(ox_convert_value_to_expression(&cx.ast, value))
}

fn ox_codegen_instruction_value<'a>(
    cx: &mut OxcContext<'a, '_>,
    instr_value: &ReactiveValue,
) -> Result<OxValue<'a>, CompilerError> {
    match instr_value {
        ReactiveValue::Instruction(iv) => ox_codegen_base_instruction_value(cx, iv),
        ReactiveValue::LogicalExpression { operator, left, right, .. } => {
            let left_expr = ox_codegen_instruction_value_to_expression(cx, left)?;
            let right_expr = ox_codegen_instruction_value_to_expression(cx, right)?;
            Ok(OxValue::Expression(cx.ast.expression_logical(
                SPAN,
                left_expr,
                ox_convert_logical_operator(operator),
                right_expr,
            )))
        }
        ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let cons_expr = ox_codegen_instruction_value_to_expression(cx, consequent)?;
            let alt_expr = ox_codegen_instruction_value_to_expression(cx, alternate)?;
            Ok(OxValue::Expression(
                cx.ast.expression_conditional(SPAN, test_expr, cons_expr, alt_expr),
            ))
        }
        ReactiveValue::SequenceExpression { instructions, value, .. } => {
            let block_items: Vec<ReactiveStatement> =
                instructions.iter().map(|i| ReactiveStatement::Instruction(i.clone())).collect();
            let body = ox_codegen_block_no_reset(cx, &block_items)?;
            let mut expressions: oxc_allocator::Vec<'a, oxc::Expression<'a>> = cx.ast.vec();
            for stmt in body {
                match stmt {
                    oxc::Statement::ExpressionStatement(es) => {
                        expressions.push(es.unbox().expression);
                    }
                    oxc::Statement::VariableDeclaration(_) => {
                        cx.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "(CodegenReactiveFunction::codegenInstructionValue) Cannot declare variables in a value block".to_string(),
                            description: None,
                            loc: None,
                            suggestions: None,
                        })?;
                        expressions.push(cx.ast.expression_string_literal(
                            SPAN,
                            "TODO handle declaration",
                            None,
                        ));
                    }
                    _ => {
                        cx.record_error(CompilerErrorDetail {
                            category: ErrorCategory::Todo,
                            reason: "(CodegenReactiveFunction::codegenInstructionValue) Handle conversion of statement to expression".to_string(),
                            description: None,
                            loc: None,
                            suggestions: None,
                        })?;
                        expressions.push(cx.ast.expression_string_literal(
                            SPAN,
                            "TODO handle statement",
                            None,
                        ));
                    }
                }
            }
            let final_expr = ox_codegen_instruction_value_to_expression(cx, value)?;
            if expressions.is_empty() {
                Ok(OxValue::Expression(final_expr))
            } else {
                expressions.push(final_expr);
                Ok(OxValue::Expression(cx.ast.expression_sequence(SPAN, expressions)))
            }
        }
        ReactiveValue::OptionalExpression { value, optional, .. } => {
            let opt_value = ox_codegen_instruction_value_to_expression(cx, value)?;
            ox_make_optional(cx, opt_value, *optional)
        }
    }
}

/// Strip a `ChainExpression` wrapper from a sub-expression so its inner element can
/// be folded into the enclosing optional chain. In oxc, `a?.b.c(d)` is a single
/// `ChainExpression` wrapping the outermost member/call; inner optional members are
/// plain members with `optional: true`. When the callee/object of an outer chain
/// element is itself a `ChainExpression`, it must be unwrapped to avoid emitting
/// spurious parens (e.g. `(a?.b)(d)` instead of `a?.b(d)`).
fn ox_unwrap_chain(expr: oxc::Expression<'_>) -> oxc::Expression<'_> {
    match expr {
        oxc::Expression::ChainExpression(chain) => {
            let chain = chain.unbox();
            match chain.expression {
                oxc::ChainElement::CallExpression(call) => oxc::Expression::CallExpression(call),
                oxc::ChainElement::ComputedMemberExpression(m) => {
                    oxc::Expression::ComputedMemberExpression(m)
                }
                oxc::ChainElement::StaticMemberExpression(m) => {
                    oxc::Expression::StaticMemberExpression(m)
                }
                oxc::ChainElement::PrivateFieldExpression(m) => {
                    oxc::Expression::PrivateFieldExpression(m)
                }
                oxc::ChainElement::TSNonNullExpression(e) => {
                    oxc::Expression::TSNonNullExpression(e)
                }
            }
        }
        other => other,
    }
}

/// Re-wrap a call/member expression as an optional-chaining element, mirroring the
/// Babel reference's `OptionalExpression` arm.
fn ox_make_optional<'a>(
    cx: &mut OxcContext<'a, '_>,
    expr: oxc::Expression<'a>,
    optional: bool,
) -> Result<OxValue<'a>, CompilerError> {
    let chain_element: oxc::ChainElement<'a> =
        match expr {
            oxc::Expression::ChainExpression(chain) => {
                // Already a chain; update the optional flag on the head element.
                let chain = chain.unbox();
                match chain.expression {
                    oxc::ChainElement::CallExpression(call) => {
                        let mut call = call.unbox();
                        call.optional = optional;
                        oxc::ChainElement::CallExpression(cx.ast.alloc(call))
                    }
                    oxc::ChainElement::ComputedMemberExpression(m) => {
                        let mut m = m.unbox();
                        m.optional = optional;
                        oxc::ChainElement::ComputedMemberExpression(cx.ast.alloc(m))
                    }
                    oxc::ChainElement::StaticMemberExpression(m) => {
                        let mut m = m.unbox();
                        m.optional = optional;
                        oxc::ChainElement::StaticMemberExpression(cx.ast.alloc(m))
                    }
                    other => other,
                }
            }
            oxc::Expression::CallExpression(call) => {
                let mut call = call.unbox();
                call.callee = ox_unwrap_chain(call.callee);
                oxc::ChainElement::CallExpression(cx.ast.alloc_call_expression(
                    SPAN,
                    call.callee,
                    call.type_arguments,
                    call.arguments,
                    optional,
                ))
            }
            oxc::Expression::ComputedMemberExpression(m) => {
                let m = m.unbox();
                oxc::ChainElement::ComputedMemberExpression(
                    cx.ast.alloc_computed_member_expression(
                        SPAN,
                        ox_unwrap_chain(m.object),
                        m.expression,
                        optional,
                    ),
                )
            }
            oxc::Expression::StaticMemberExpression(m) => {
                let m = m.unbox();
                oxc::ChainElement::StaticMemberExpression(
                    cx.ast.alloc_static_member_expression(
                        SPAN,
                        ox_unwrap_chain(m.object),
                        m.property,
                        optional,
                    ),
                )
            }
            _ => {
                return Err(invariant_err(
                    "Expected optional value to resolve to call or member expression",
                    None,
                ));
            }
        };
    Ok(OxValue::Expression(cx.ast.expression_chain(SPAN, chain_element)))
}

fn ox_codegen_base_instruction_value<'a>(
    cx: &mut OxcContext<'a, '_>,
    iv: &InstructionValue,
) -> Result<OxValue<'a>, CompilerError> {
    match iv {
        InstructionValue::Primitive { value, .. } => {
            Ok(OxValue::Expression(ox_codegen_primitive_value(&cx.ast, value)))
        }
        InstructionValue::BinaryExpression { operator, left, right, .. } => {
            let left_expr = ox_codegen_place_to_expression(cx, left)?;
            let right_expr = ox_codegen_place_to_expression(cx, right)?;
            Ok(OxValue::Expression(cx.ast.expression_binary(
                SPAN,
                left_expr,
                ox_convert_binary_operator(operator),
                right_expr,
            )))
        }
        InstructionValue::UnaryExpression { operator, value, .. } => {
            let arg = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(cx.ast.expression_unary(
                SPAN,
                ox_convert_unary_operator(operator),
                arg,
            )))
        }
        InstructionValue::LoadLocal { place, .. } | InstructionValue::LoadContext { place, .. } => {
            let expr = ox_codegen_place_to_expression(cx, place)?;
            Ok(OxValue::Expression(expr))
        }
        InstructionValue::LoadGlobal { binding, .. } => Ok(OxValue::Expression(
            cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, binding.name())),
        )),
        InstructionValue::CallExpression { callee, args, .. } => {
            let callee_expr = ox_codegen_place_to_expression(cx, callee)?;
            let arguments = ox_codegen_arguments(cx, args)?;
            let call_expr = cx.ast.expression_call(
                SPAN,
                callee_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
                false,
            );
            let result = ox_maybe_wrap_hook_call(cx, call_expr, callee.identifier)?;
            Ok(OxValue::Expression(result))
        }
        InstructionValue::MethodCall { property, args, .. } => {
            let member_expr = ox_codegen_place_to_expression(cx, property)?;
            if !ox_is_member_like(&member_expr) {
                let msg = format!("Got: '{}'", ox_expression_type_name(&member_expr));
                let mut err = CompilerError::new();
                err.push_diagnostic(
                    CompilerDiagnostic::new(
                        ErrorCategory::Invariant,
                        "[Codegen] Internal error: MethodCall::property must be an unpromoted + unmemoized MemberExpression",
                        None,
                    )
                    .with_detail(CompilerDiagnosticDetail::Error {
                        loc: property.loc,
                        message: Some(msg),
                        identifier_name: None,
                    }),
                );
                return Err(err);
            }
            let arguments = ox_codegen_arguments(cx, args)?;
            let call_expr = cx.ast.expression_call(
                SPAN,
                member_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
                false,
            );
            let result = ox_maybe_wrap_hook_call(cx, call_expr, property.identifier)?;
            Ok(OxValue::Expression(result))
        }
        InstructionValue::NewExpression { callee, args, .. } => {
            let callee_expr = ox_codegen_place_to_expression(cx, callee)?;
            let arguments = ox_codegen_arguments(cx, args)?;
            Ok(OxValue::Expression(cx.ast.expression_new(
                SPAN,
                callee_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
            )))
        }
        InstructionValue::ArrayExpression { elements, .. } => {
            let mut elems: oxc_allocator::Vec<'a, oxc::ArrayExpressionElement<'a>> = cx.ast.vec();
            for el in elements {
                match el {
                    ArrayElement::Place(place) => {
                        let expr = ox_codegen_place_to_expression(cx, place)?;
                        elems.push(oxc::ArrayExpressionElement::from(expr));
                    }
                    ArrayElement::Spread(spread) => {
                        let arg = ox_codegen_place_to_expression(cx, &spread.place)?;
                        elems.push(oxc::ArrayExpressionElement::SpreadElement(
                            cx.ast.alloc_spread_element(SPAN, arg),
                        ));
                    }
                    ArrayElement::Hole => {
                        elems.push(cx.ast.array_expression_element_elision(SPAN));
                    }
                }
            }
            Ok(OxValue::Expression(cx.ast.expression_array(SPAN, elems)))
        }
        InstructionValue::ObjectExpression { properties, .. } => {
            ox_codegen_object_expression(cx, properties)
        }
        InstructionValue::PropertyLoad { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let member = ox_property_member(cx, obj, property);
            Ok(OxValue::Expression(oxc::Expression::from(member)))
        }
        InstructionValue::PropertyStore { object, property, value, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let member = ox_property_member(cx, obj, property);
            let val = ox_codegen_place_to_expression(cx, value)?;
            let target = oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(member));
            Ok(OxValue::Expression(cx.ast.expression_assignment(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                val,
            )))
        }
        InstructionValue::PropertyDelete { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let member = ox_property_member(cx, obj, property);
            Ok(OxValue::Expression(cx.ast.expression_unary(
                SPAN,
                oxc::UnaryOperator::Delete,
                oxc::Expression::from(member),
            )))
        }
        InstructionValue::ComputedLoad { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = cx.ast.member_expression_computed(SPAN, obj, prop, false);
            Ok(OxValue::Expression(oxc::Expression::from(member)))
        }
        InstructionValue::ComputedStore { object, property, value, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = cx.ast.member_expression_computed(SPAN, obj, prop, false);
            let val = ox_codegen_place_to_expression(cx, value)?;
            let target = oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(member));
            Ok(OxValue::Expression(cx.ast.expression_assignment(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                val,
            )))
        }
        InstructionValue::ComputedDelete { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = cx.ast.member_expression_computed(SPAN, obj, prop, false);
            Ok(OxValue::Expression(cx.ast.expression_unary(
                SPAN,
                oxc::UnaryOperator::Delete,
                oxc::Expression::from(member),
            )))
        }
        InstructionValue::RegExpLiteral { pattern, flags, .. } => {
            let regex_flags = ox_parse_regexp_flags(flags);
            let regex = oxc::RegExp {
                pattern: oxc::RegExpPattern {
                    text: ox_str(&cx.ast, pattern).into(),
                    pattern: None,
                },
                flags: regex_flags,
            };
            Ok(OxValue::Expression(cx.ast.expression_reg_exp_literal(SPAN, regex, None)))
        }
        InstructionValue::MetaProperty { meta, property, .. } => {
            let meta_ident = cx.ast.identifier_name(SPAN, ox_str(&cx.ast, meta));
            let prop_ident = cx.ast.identifier_name(SPAN, ox_str(&cx.ast, property));
            Ok(OxValue::Expression(cx.ast.expression_meta_property(SPAN, meta_ident, prop_ident)))
        }
        InstructionValue::Await { value, .. } => {
            let arg = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(cx.ast.expression_await(SPAN, arg)))
        }
        InstructionValue::GetIterator { collection, .. } => {
            let expr = ox_codegen_place_to_expression(cx, collection)?;
            Ok(OxValue::Expression(expr))
        }
        InstructionValue::IteratorNext { iterator, .. } => {
            let expr = ox_codegen_place_to_expression(cx, iterator)?;
            Ok(OxValue::Expression(expr))
        }
        InstructionValue::NextPropertyOf { value, .. } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(expr))
        }
        InstructionValue::PostfixUpdate { operation, lvalue, .. } => {
            let arg = ox_codegen_place_to_expression(cx, lvalue)?;
            let target = ox_expression_to_simple_assignment_target(cx, arg)?;
            Ok(OxValue::Expression(cx.ast.expression_update(
                SPAN,
                ox_convert_update_operator(operation),
                false,
                target,
            )))
        }
        InstructionValue::PrefixUpdate { operation, lvalue, .. } => {
            let arg = ox_codegen_place_to_expression(cx, lvalue)?;
            let target = ox_expression_to_simple_assignment_target(cx, arg)?;
            Ok(OxValue::Expression(cx.ast.expression_update(
                SPAN,
                ox_convert_update_operator(operation),
                true,
                target,
            )))
        }
        InstructionValue::StoreLocal { lvalue, value, .. } => {
            invariant(
                lvalue.kind == InstructionKind::Reassign,
                "Unexpected StoreLocal in codegenInstructionValue",
                None,
            )?;
            let lval = ox_codegen_lvalue(cx, &LvalueRef::Place(&lvalue.place))?;
            let target = ox_binding_pattern_to_assignment_target(cx, lval)?;
            let rhs = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(cx.ast.expression_assignment(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                rhs,
            )))
        }
        InstructionValue::StoreGlobal { name, value, .. } => {
            let rhs = ox_codegen_place_to_expression(cx, value)?;
            let target = oxc::AssignmentTarget::AssignmentTargetIdentifier(
                cx.ast.alloc_identifier_reference(SPAN, ox_str(&cx.ast, name)),
            );
            Ok(OxValue::Expression(cx.ast.expression_assignment(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                rhs,
            )))
        }
        InstructionValue::FunctionExpression {
            name, name_hint, lowered_func, expr_type, ..
        } => ox_codegen_function_expression(cx, name, name_hint, lowered_func, expr_type),
        InstructionValue::TaggedTemplateExpression { tag, quasis, subexprs, .. } => {
            let tag_expr = ox_codegen_place_to_expression(cx, tag)?;
            let mut exprs: oxc_allocator::Vec<'a, oxc::Expression<'a>> = cx.ast.vec();
            for p in subexprs {
                exprs.push(ox_codegen_place_to_expression(cx, p)?);
            }
            let quasi = ox_template_literal(cx, quasis, exprs);
            Ok(OxValue::Expression(cx.ast.expression_tagged_template(
                SPAN,
                tag_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                quasi,
            )))
        }
        InstructionValue::TemplateLiteral { subexprs, quasis, .. } => {
            let mut exprs: oxc_allocator::Vec<'a, oxc::Expression<'a>> = cx.ast.vec();
            for p in subexprs {
                exprs.push(ox_codegen_place_to_expression(cx, p)?);
            }
            let template = ox_template_literal(cx, quasis, exprs);
            Ok(OxValue::Expression(oxc::Expression::TemplateLiteral(cx.ast.alloc(template))))
        }
        InstructionValue::TypeCastExpression {
            value, type_annotation_kind, type_annotation, ..
        } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            // Recover the TS type from its original source span and re-wrap the
            // inner expression, matching the baseline output. If the type can't be
            // recovered, fall back to the unwrapped expression.
            let wrapped = match (type_annotation_kind.as_deref(), type_annotation) {
                (Some("satisfies"), Some(ta)) => {
                    let mut ta = ta.clone();
                    set_raw_type_renames(&mut ta, &cx.env.renames, &cx.env.reference_node_ids);
                    match ox_reparse_ts_type(cx, &ta) {
                        Some(ty) => cx.ast.expression_ts_satisfies(SPAN, expr, ty),
                        None => expr,
                    }
                }
                (Some("as"), Some(ta)) => {
                    let mut ta = ta.clone();
                    set_raw_type_renames(&mut ta, &cx.env.renames, &cx.env.reference_node_ids);
                    match ox_reparse_ts_type(cx, &ta) {
                        Some(ty) => cx.ast.expression_ts_as(SPAN, expr, ty),
                        None => expr,
                    }
                }
                _ => expr,
            };
            Ok(OxValue::Expression(wrapped))
        }
        InstructionValue::JSXText { value, .. } => {
            Ok(OxValue::JsxText(cx.ast.alloc_jsx_text(SPAN, ox_str(&cx.ast, value), None)))
        }
        InstructionValue::JsxExpression { tag, props, children, .. } => {
            ox_codegen_jsx_expression(cx, tag, props, children)
        }
        InstructionValue::JsxFragment { children, .. } => {
            let mut child_nodes: oxc_allocator::Vec<'a, oxc::JSXChild<'a>> = cx.ast.vec();
            for child in children {
                child_nodes.push(ox_codegen_jsx_element(cx, child)?);
            }
            let opening = cx.ast.jsx_opening_fragment(SPAN);
            let closing = cx.ast.jsx_closing_fragment(SPAN);
            let fragment = cx.ast.jsx_fragment(SPAN, opening, child_nodes, closing);
            Ok(OxValue::Expression(oxc::Expression::JSXFragment(cx.ast.alloc(fragment))))
        }
        InstructionValue::StartMemoize { .. }
        | InstructionValue::FinishMemoize { .. }
        | InstructionValue::Debugger { .. }
        | InstructionValue::DeclareLocal { .. }
        | InstructionValue::DeclareContext { .. }
        | InstructionValue::Destructure { .. }
        | InstructionValue::ObjectMethod { .. }
        | InstructionValue::StoreContext { .. }
        | InstructionValue::UnsupportedNode { .. } => Err(invariant_err(
            &format!("Unexpected {:?} in codegenInstructionValue", std::mem::discriminant(iv)),
            None,
        )),
    }
}

/// Build `obj.prop` / `obj[prop]` member expression from a `PropertyLiteral`.
fn ox_property_member<'a>(
    cx: &OxcContext<'a, '_>,
    object: oxc::Expression<'a>,
    property: &PropertyLiteral,
) -> oxc::MemberExpression<'a> {
    match property {
        PropertyLiteral::String(s) => cx.ast.member_expression_static(
            SPAN,
            object,
            cx.ast.identifier_name(SPAN, ox_str(&cx.ast, s)),
            false,
        ),
        PropertyLiteral::Number(n) => {
            cx.ast.member_expression_computed(SPAN, object, ox_number(&cx.ast, n.value()), false)
        }
    }
}

fn ox_template_literal<'a>(
    cx: &OxcContext<'a, '_>,
    quasis: &[crate::react_compiler_hir::TemplateQuasi],
    expressions: oxc_allocator::Vec<'a, oxc::Expression<'a>>,
) -> oxc::TemplateLiteral<'a> {
    let mut quasi_vec: oxc_allocator::Vec<'a, oxc::TemplateElement<'a>> = cx.ast.vec();
    let len = quasis.len();
    for (i, q) in quasis.iter().enumerate() {
        let value = oxc::TemplateElementValue {
            raw: ox_str(&cx.ast, &q.raw).into(),
            cooked: q.cooked.as_deref().map(|c| ox_str(&cx.ast, c).into()),
        };
        quasi_vec.push(cx.ast.template_element(SPAN, value, i == len - 1));
    }
    cx.ast.template_literal(SPAN, quasi_vec, expressions)
}

fn ox_codegen_arguments<'a>(
    cx: &mut OxcContext<'a, '_>,
    args: &[PlaceOrSpread],
) -> Result<oxc_allocator::Vec<'a, oxc::Argument<'a>>, CompilerError> {
    let mut out: oxc_allocator::Vec<'a, oxc::Argument<'a>> = cx.ast.vec();
    for arg in args {
        out.push(ox_codegen_argument(cx, arg)?);
    }
    Ok(out)
}

fn ox_codegen_argument<'a>(
    cx: &mut OxcContext<'a, '_>,
    arg: &PlaceOrSpread,
) -> Result<oxc::Argument<'a>, CompilerError> {
    match arg {
        PlaceOrSpread::Place(place) => {
            Ok(oxc::Argument::from(ox_codegen_place_to_expression(cx, place)?))
        }
        PlaceOrSpread::Spread(spread) => {
            let expr = ox_codegen_place_to_expression(cx, &spread.place)?;
            Ok(oxc::Argument::SpreadElement(cx.ast.alloc_spread_element(SPAN, expr)))
        }
    }
}

fn ox_is_member_like(expr: &oxc::Expression) -> bool {
    matches!(
        expr,
        oxc::Expression::StaticMemberExpression(_)
            | oxc::Expression::ComputedMemberExpression(_)
            | oxc::Expression::PrivateFieldExpression(_)
            | oxc::Expression::ChainExpression(_)
    )
}

fn ox_expression_type_name(expr: &oxc::Expression) -> &'static str {
    match expr {
        oxc::Expression::Identifier(_) => "Identifier",
        _ => "unknown",
    }
}

// =============================================================================
// Place / lvalue / dependency codegen (oxc)
// =============================================================================

fn ox_codegen_place_to_expression<'a>(
    cx: &mut OxcContext<'a, '_>,
    place: &Place,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    Ok(ox_convert_value_to_expression(&cx.ast, value))
}

fn ox_codegen_place<'a>(
    cx: &mut OxcContext<'a, '_>,
    place: &Place,
) -> Result<OxValue<'a>, CompilerError> {
    let ident = &cx.env.identifiers[place.identifier.0 as usize];
    let declaration_id = ident.declaration_id;
    if let Some(tmp) = cx.temp.get(&declaration_id) {
        if let Some(val) = tmp {
            return Ok(val.clone_in(cx.ast.allocator()));
        }
    } else if ident.name.is_none() {
        return Err(invariant_err(
            &format!(
                "[Codegen] No value found for temporary, identifier id={}",
                place.identifier.0
            ),
            place.loc,
        ));
    }
    let name = ox_identifier_name(cx.env, place.identifier)?;
    Ok(OxValue::Expression(cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, &name))))
}

fn ox_codegen_lvalue<'a>(
    cx: &mut OxcContext<'a, '_>,
    pattern: &LvalueRef,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    match pattern {
        LvalueRef::Place(place) => ox_binding_for_identifier(cx, place.identifier),
        LvalueRef::Pattern(pat) => match pat {
            Pattern::Array(arr) => ox_codegen_array_pattern(cx, arr),
            Pattern::Object(obj) => ox_codegen_object_pattern(cx, obj),
        },
        LvalueRef::Spread(spread) => ox_binding_for_identifier(cx, spread.place.identifier),
    }
}

fn ox_codegen_array_pattern<'a>(
    cx: &mut OxcContext<'a, '_>,
    pattern: &ArrayPattern,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let mut elements: oxc_allocator::Vec<'a, Option<oxc::BindingPattern<'a>>> = cx.ast.vec();
    let mut rest: Option<oxc::BindingRestElement<'a>> = None;
    for item in &pattern.items {
        match item {
            crate::react_compiler_hir::ArrayPatternElement::Place(place) => {
                elements.push(Some(ox_binding_for_identifier(cx, place.identifier)?));
            }
            crate::react_compiler_hir::ArrayPatternElement::Spread(spread) => {
                let inner = ox_binding_for_identifier(cx, spread.place.identifier)?;
                rest = Some(cx.ast.binding_rest_element(SPAN, inner));
            }
            crate::react_compiler_hir::ArrayPatternElement::Hole => {
                elements.push(None);
            }
        }
    }
    Ok(cx.ast.binding_pattern_array_pattern(SPAN, elements, rest))
}

fn ox_codegen_object_pattern<'a>(
    cx: &mut OxcContext<'a, '_>,
    pattern: &ObjectPattern,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let mut properties: oxc_allocator::Vec<'a, oxc::BindingProperty<'a>> = cx.ast.vec();
    let mut rest: Option<oxc::BindingRestElement<'a>> = None;
    for prop in &pattern.properties {
        match prop {
            ObjectPropertyOrSpread::Property(obj_prop) => {
                let (key, computed) = ox_codegen_object_property_key(cx, &obj_prop.key)?;
                let value = ox_binding_for_identifier(cx, obj_prop.place.identifier)?;
                let shorthand = !computed
                    && matches!(
                        (&key, &value),
                        (
                            oxc::PropertyKey::StaticIdentifier(k),
                            oxc::BindingPattern::BindingIdentifier(v),
                        ) if k.name == v.name
                    );
                properties.push(cx.ast.binding_property(SPAN, key, value, shorthand, computed));
            }
            ObjectPropertyOrSpread::Spread(spread) => {
                let inner = ox_binding_for_identifier(cx, spread.place.identifier)?;
                rest = Some(cx.ast.binding_rest_element(SPAN, inner));
            }
        }
    }
    Ok(cx.ast.binding_pattern_object_pattern(SPAN, properties, rest))
}

/// Build an object pattern key, returning `(key, computed)`.
fn ox_codegen_object_property_key<'a>(
    cx: &mut OxcContext<'a, '_>,
    key: &ObjectPropertyKey,
) -> Result<(oxc::PropertyKey<'a>, bool), CompilerError> {
    match key {
        ObjectPropertyKey::String { name } => Ok((
            oxc::PropertyKey::from(cx.ast.expression_string_literal(
                SPAN,
                ox_str(&cx.ast, name),
                None,
            )),
            false,
        )),
        ObjectPropertyKey::Identifier { name } => {
            Ok((cx.ast.property_key_static_identifier(SPAN, ox_str(&cx.ast, name)), false))
        }
        ObjectPropertyKey::Computed { name } => {
            let expr = ox_codegen_place_to_expression(cx, name)?;
            Ok((oxc::PropertyKey::from(expr), true))
        }
        ObjectPropertyKey::Number { name } => {
            Ok((oxc::PropertyKey::from(ox_number(&cx.ast, name.value())), false))
        }
    }
}

fn ox_codegen_dependency<'a>(
    cx: &mut OxcContext<'a, '_>,
    dep: &crate::react_compiler_hir::ReactiveScopeDependency,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let name = ox_identifier_name(cx.env, dep.identifier)?;
    let mut object = cx.ast.expression_identifier(SPAN, ox_str(&cx.ast, &name));
    if !dep.path.is_empty() {
        let has_optional = dep.path.iter().any(|p| p.optional);
        // Build every member as a plain member expression carrying its own optional
        // flag. In oxc, an optional chain like `a.b?.c.d?.e` is a single
        // `ChainExpression` wrapping the outermost member, with inner members being
        // plain members whose `optional` flags are preserved. Wrapping each step in
        // its own `ChainExpression` would force spurious parens such as `(((a.b)?.c).d)?.e`.
        for path_entry in &dep.path {
            let member = ox_property_member(cx, object, &path_entry.property);
            object = match member {
                oxc::MemberExpression::StaticMemberExpression(m) => {
                    let m = m.unbox();
                    oxc::Expression::StaticMemberExpression(cx.ast.alloc_static_member_expression(
                        SPAN,
                        m.object,
                        m.property,
                        path_entry.optional,
                    ))
                }
                oxc::MemberExpression::ComputedMemberExpression(m) => {
                    let m = m.unbox();
                    oxc::Expression::ComputedMemberExpression(
                        cx.ast.alloc_computed_member_expression(
                            SPAN,
                            m.object,
                            m.expression,
                            path_entry.optional,
                        ),
                    )
                }
                oxc::MemberExpression::PrivateFieldExpression(m) => {
                    oxc::Expression::PrivateFieldExpression(m)
                }
            };
        }
        // Wrap the whole access path in a single chain only when it actually contains
        // an optional access.
        if has_optional {
            let chain = match object {
                oxc::Expression::StaticMemberExpression(m) => {
                    oxc::ChainElement::StaticMemberExpression(m)
                }
                oxc::Expression::ComputedMemberExpression(m) => {
                    oxc::ChainElement::ComputedMemberExpression(m)
                }
                oxc::Expression::PrivateFieldExpression(m) => {
                    oxc::ChainElement::PrivateFieldExpression(m)
                }
                other => return Ok(other),
            };
            object = cx.ast.expression_chain(SPAN, chain);
        }
    }
    Ok(object)
}

/// Convert a `BindingPattern` (from `ox_codegen_lvalue`) into an `AssignmentTarget`
/// for reassignment / `StoreLocal` emission.
fn ox_binding_pattern_to_assignment_target<'a>(
    cx: &OxcContext<'a, '_>,
    pattern: oxc::BindingPattern<'a>,
) -> Result<oxc::AssignmentTarget<'a>, CompilerError> {
    match pattern {
        oxc::BindingPattern::BindingIdentifier(id) => {
            let id = id.unbox();
            Ok(oxc::AssignmentTarget::AssignmentTargetIdentifier(
                cx.ast.alloc_identifier_reference(SPAN, id.name),
            ))
        }
        _ => {
            Err(invariant_err("Destructuring reassignment targets are not yet ported to oxc", None))
        }
    }
}

/// Convert an expression to a `SimpleAssignmentTarget` for update expressions.
fn ox_expression_to_simple_assignment_target<'a>(
    cx: &OxcContext<'a, '_>,
    expr: oxc::Expression<'a>,
) -> Result<oxc::SimpleAssignmentTarget<'a>, CompilerError> {
    match expr {
        oxc::Expression::Identifier(id) => {
            let id = id.unbox();
            Ok(oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(
                cx.ast.alloc_identifier_reference(SPAN, id.name),
            ))
        }
        oxc::Expression::StaticMemberExpression(m) => {
            Ok(oxc::SimpleAssignmentTarget::from(oxc::MemberExpression::StaticMemberExpression(m)))
        }
        oxc::Expression::ComputedMemberExpression(m) => Ok(oxc::SimpleAssignmentTarget::from(
            oxc::MemberExpression::ComputedMemberExpression(m),
        )),
        _ => Err(invariant_err("Expected a simple assignment target for update expression", None)),
    }
}

// =============================================================================
// Deferred sub-emitters (later batches): function/object/jsx expression codegen.
// =============================================================================

fn ox_codegen_function_expression<'a>(
    cx: &mut OxcContext<'a, '_>,
    name: &Option<String>,
    name_hint: &Option<String>,
    lowered_func: &crate::react_compiler_hir::LoweredFunction,
    expr_type: &FunctionExpressionType,
) -> Result<OxValue<'a>, CompilerError> {
    let func = cx.env.functions[lowered_func.func.0 as usize].clone();
    let mut reactive_fn = build_reactive_function(&func, cx.env)?;
    prune_unused_labels(&mut reactive_fn, cx.env)?;
    prune_unused_lvalues(&mut reactive_fn, cx.env);
    prune_hoisted_contexts(&mut reactive_fn, cx.env)?;

    let fn_result = ox_codegen_inner_function(cx, &reactive_fn)?;

    let value = match expr_type {
        FunctionExpressionType::ArrowFunctionExpression => {
            let mut fn_result = fn_result;
            // Optimize single-return arrow functions into expression bodies.
            let single_return_arg = if fn_result.body.statements.len() == 1
                && reactive_fn.directives.is_empty()
                && matches!(
                    fn_result.body.statements.last(),
                    Some(oxc::Statement::ReturnStatement(ret)) if ret.argument.is_some()
                ) {
                let stmt = fn_result.body.statements.pop().unwrap();
                let oxc::Statement::ReturnStatement(ret) = stmt else { unreachable!() };
                ret.unbox().argument
            } else {
                None
            };
            match single_return_arg {
                Some(arg) => {
                    let stmts = cx.ast.vec1(cx.ast.statement_expression(SPAN, arg));
                    let body = cx.ast.alloc_function_body(SPAN, cx.ast.vec(), stmts);
                    ox_build_arrow(cx, fn_result.params, body, fn_result.is_async, true)
                }
                None => {
                    ox_build_arrow(cx, fn_result.params, fn_result.body, fn_result.is_async, false)
                }
            }
        }
        _ => {
            let id = name.as_ref().map(|n| cx.ast.binding_identifier(SPAN, ox_str(&cx.ast, n)));
            let func = cx.ast.function(
                SPAN,
                oxc::FunctionType::FunctionExpression,
                id,
                fn_result.generator,
                fn_result.is_async,
                false,
                None::<oxc_allocator::Box<oxc::TSTypeParameterDeclaration>>,
                None::<oxc_allocator::Box<oxc::TSThisParameter>>,
                fn_result.params,
                None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                Some(fn_result.body),
            );
            oxc::Expression::FunctionExpression(cx.ast.alloc(func))
        }
    };

    // enableNameAnonymousFunctions: `({ "<hint>": <fn> })["<hint>"]`
    if cx.env.config.enable_name_anonymous_functions && name.is_none() && name_hint.is_some() {
        let hint = name_hint.as_ref().unwrap().clone();
        let key = oxc::PropertyKey::from(cx.ast.expression_string_literal(
            SPAN,
            ox_str(&cx.ast, &hint),
            None,
        ));
        let prop =
            cx.ast.object_property(SPAN, oxc::PropertyKind::Init, key, value, false, false, false);
        let props = cx.ast.vec1(oxc::ObjectPropertyKind::ObjectProperty(cx.ast.alloc(prop)));
        let object = cx.ast.expression_object(SPAN, props);
        let member = cx.ast.member_expression_computed(
            SPAN,
            object,
            cx.ast.expression_string_literal(SPAN, ox_str(&cx.ast, &hint), None),
            false,
        );
        return Ok(OxValue::Expression(oxc::Expression::from(member)));
    }

    Ok(OxValue::Expression(value))
}

fn ox_build_arrow<'a>(
    cx: &OxcContext<'a, '_>,
    params: oxc_allocator::Box<'a, oxc::FormalParameters<'a>>,
    body: oxc_allocator::Box<'a, oxc::FunctionBody<'a>>,
    is_async: bool,
    expression: bool,
) -> oxc::Expression<'a> {
    cx.ast.expression_arrow_function(
        SPAN,
        expression,
        is_async,
        None::<oxc_allocator::Box<oxc::TSTypeParameterDeclaration>>,
        params,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        body,
    )
}

/// Run the inner-function codegen with a fresh context (mirrors the Babel reference's
/// `Context::new` + `codegen_reactive_function` for function/object-method expressions).
fn ox_codegen_inner_function<'a>(
    cx: &mut OxcContext<'a, '_>,
    reactive_fn: &ReactiveFunction,
) -> Result<OxcCompiledFunction<'a>, CompilerError> {
    let fn_name = reactive_fn.id.as_deref().unwrap_or("[[ anonymous ]]").to_string();
    let mut inner_cx = OxcContext::new(
        oxc_ast::builder::AstBuilder::new(cx.ast.allocator()),
        cx.env,
        fn_name,
        cx.unique_identifiers.clone(),
        cx.fbt_operands.clone(),
    );
    inner_cx.temp = ox_clone_temporaries(&cx.ast, &cx.temp);
    ox_codegen_reactive_function(&mut inner_cx, reactive_fn)
}

fn ox_codegen_object_expression<'a>(
    cx: &mut OxcContext<'a, '_>,
    properties: &[ObjectPropertyOrSpread],
) -> Result<OxValue<'a>, CompilerError> {
    let mut props: oxc_allocator::Vec<'a, oxc::ObjectPropertyKind<'a>> = cx.ast.vec();
    for prop in properties {
        match prop {
            ObjectPropertyOrSpread::Property(obj_prop) => {
                let (key, key_computed) = ox_codegen_object_property_key(cx, &obj_prop.key)?;
                match obj_prop.property_type {
                    ObjectPropertyType::Property => {
                        let value = ox_codegen_place_to_expression(cx, &obj_prop.place)?;
                        let shorthand = !key_computed
                            && matches!(
                                (&key, &value),
                                (
                                    oxc::PropertyKey::StaticIdentifier(k),
                                    oxc::Expression::Identifier(v),
                                ) if k.name == v.name
                            );
                        let p = cx.ast.object_property(
                            SPAN,
                            oxc::PropertyKind::Init,
                            key,
                            value,
                            false,
                            shorthand,
                            key_computed,
                        );
                        props.push(oxc::ObjectPropertyKind::ObjectProperty(cx.ast.alloc(p)));
                    }
                    ObjectPropertyType::Method => {
                        let method_data =
                            cx.object_methods.get(&obj_prop.place.identifier).cloned();
                        let Some((InstructionValue::ObjectMethod { lowered_func, .. }, _)) =
                            method_data
                        else {
                            return Err(invariant_err("Expected ObjectMethod instruction", None));
                        };

                        let func = cx.env.functions[lowered_func.func.0 as usize].clone();
                        let mut reactive_fn = build_reactive_function(&func, cx.env)?;
                        prune_unused_labels(&mut reactive_fn, cx.env)?;
                        prune_unused_lvalues(&mut reactive_fn, cx.env);

                        let fn_result = ox_codegen_inner_function(cx, &reactive_fn)?;
                        let method = cx.ast.function(
                            SPAN,
                            oxc::FunctionType::FunctionExpression,
                            None,
                            fn_result.generator,
                            fn_result.is_async,
                            false,
                            None::<oxc_allocator::Box<oxc::TSTypeParameterDeclaration>>,
                            None::<oxc_allocator::Box<oxc::TSThisParameter>>,
                            fn_result.params,
                            None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                            Some(fn_result.body),
                        );
                        let func_expr = oxc::Expression::FunctionExpression(cx.ast.alloc(method));
                        let p = cx.ast.object_property(
                            SPAN,
                            oxc::PropertyKind::Init,
                            key,
                            func_expr,
                            true,
                            false,
                            key_computed,
                        );
                        props.push(oxc::ObjectPropertyKind::ObjectProperty(cx.ast.alloc(p)));
                    }
                }
            }
            ObjectPropertyOrSpread::Spread(spread) => {
                let arg = ox_codegen_place_to_expression(cx, &spread.place)?;
                let spread_el = cx.ast.spread_element(SPAN, arg);
                props.push(oxc::ObjectPropertyKind::SpreadProperty(cx.ast.alloc(spread_el)));
            }
        }
    }
    Ok(OxValue::Expression(cx.ast.expression_object(SPAN, props)))
}

// =============================================================================
// JSX codegen (oxc)
// =============================================================================

fn ox_codegen_jsx_expression<'a>(
    cx: &mut OxcContext<'a, '_>,
    tag: &JsxTag,
    props: &[JsxAttribute],
    children: &Option<Vec<Place>>,
) -> Result<OxValue<'a>, CompilerError> {
    let mut attributes: oxc_allocator::Vec<'a, oxc::JSXAttributeItem<'a>> = cx.ast.vec();
    for attr in props {
        attributes.push(ox_codegen_jsx_attribute(cx, attr)?);
    }

    let (tag_value, is_fbt_tag) = match tag {
        JsxTag::Place(place) => (ox_codegen_place_to_expression(cx, place)?, false),
        JsxTag::Builtin(builtin) => {
            let is_fbt = SINGLE_CHILD_FBT_TAGS.contains(&builtin.name.as_str());
            (cx.ast.expression_string_literal(SPAN, ox_str(&cx.ast, &builtin.name), None), is_fbt)
        }
    };

    let opening_name = ox_expression_to_jsx_tag(cx, &tag_value)?;

    let mut child_nodes: oxc_allocator::Vec<'a, oxc::JSXChild<'a>> = cx.ast.vec();
    if let Some(c) = children {
        for child in c {
            if is_fbt_tag {
                child_nodes.push(ox_codegen_jsx_fbt_child_element(cx, child)?);
            } else {
                child_nodes.push(ox_codegen_jsx_element(cx, child)?);
            }
        }
    }

    let is_self_closing = children.is_none();
    let opening = cx.ast.jsx_opening_element(
        SPAN,
        opening_name,
        None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
        attributes,
    );
    let closing = if is_self_closing {
        None
    } else {
        let closing_name = ox_expression_to_jsx_tag(cx, &tag_value)?;
        Some(cx.ast.jsx_closing_element(SPAN, closing_name))
    };
    let element = cx.ast.jsx_element(SPAN, opening, child_nodes, closing);
    Ok(OxValue::Expression(oxc::Expression::JSXElement(cx.ast.alloc(element))))
}

fn ox_string_requires_expr_container(s: &str) -> bool {
    for c in s.chars() {
        if STRING_REQUIRES_EXPR_CONTAINER_CHARS.contains(c) {
            return true;
        }
        let code = c as u32;
        if code <= 0x1F || code == 0x7F || (0x80..=0x9F).contains(&code) || code >= 0xA0 {
            return true;
        }
    }
    false
}

fn ox_encode_jsx_text(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '{' => escaped.push_str("&#123;"),
            '}' => escaped.push_str("&#125;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn ox_codegen_jsx_attribute<'a>(
    cx: &mut OxcContext<'a, '_>,
    attr: &JsxAttribute,
) -> Result<oxc::JSXAttributeItem<'a>, CompilerError> {
    match attr {
        JsxAttribute::Attribute { name, place } => {
            let prop_name = if name.contains(':') {
                let parts: Vec<&str> = name.splitn(2, ':').collect();
                let namespace = cx.ast.jsx_identifier(SPAN, ox_str(&cx.ast, parts[0]));
                let local = cx.ast.jsx_identifier(SPAN, ox_str(&cx.ast, parts[1]));
                cx.ast.jsx_attribute_name_namespaced_name(SPAN, namespace, local)
            } else {
                cx.ast.jsx_attribute_name_identifier(SPAN, ox_str(&cx.ast, name))
            };

            let is_fbt_operand = cx.fbt_operands.contains(&place.identifier);
            let inner_value = ox_codegen_place_to_expression(cx, place)?;
            let attr_value = match inner_value {
                oxc::Expression::StringLiteral(ref s)
                    if !ox_string_requires_expr_container(s.value.as_str()) || is_fbt_operand =>
                {
                    let value = s.value;
                    Some(cx.ast.jsx_attribute_value_string_literal(SPAN, value, None))
                }
                _ => {
                    let expr = oxc::JSXExpression::from(inner_value);
                    Some(cx.ast.jsx_attribute_value_expression_container(SPAN, expr))
                }
            };
            Ok(cx.ast.jsx_attribute_item_attribute(SPAN, prop_name, attr_value))
        }
        JsxAttribute::SpreadAttribute { argument } => {
            let expr = ox_codegen_place_to_expression(cx, argument)?;
            Ok(cx.ast.jsx_attribute_item_spread_attribute(SPAN, expr))
        }
    }
}

fn ox_codegen_jsx_element<'a>(
    cx: &mut OxcContext<'a, '_>,
    place: &Place,
) -> Result<oxc::JSXChild<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    match value {
        OxValue::JsxText(text) => {
            let raw = text.value.as_str();
            if raw.contains(JSX_TEXT_CHILD_REQUIRES_EXPR_CONTAINER_PATTERN) {
                let lit = cx.ast.expression_string_literal(SPAN, ox_str(&cx.ast, raw), None);
                Ok(cx.ast.jsx_child_expression_container(SPAN, oxc::JSXExpression::from(lit)))
            } else {
                let encoded = ox_encode_jsx_text(raw);
                Ok(cx.ast.jsx_child_text(SPAN, ox_str(&cx.ast, &encoded), None))
            }
        }
        OxValue::Expression(oxc::Expression::JSXElement(elem)) => {
            let elem = elem.unbox();
            Ok(cx.ast.jsx_child_element(
                SPAN,
                elem.opening_element,
                elem.children,
                elem.closing_element,
            ))
        }
        OxValue::Expression(oxc::Expression::JSXFragment(frag)) => {
            let frag = frag.unbox();
            Ok(cx.ast.jsx_child_fragment(
                SPAN,
                frag.opening_fragment,
                frag.children,
                frag.closing_fragment,
            ))
        }
        OxValue::Expression(expr) => {
            Ok(cx.ast.jsx_child_expression_container(SPAN, oxc::JSXExpression::from(expr)))
        }
    }
}

fn ox_codegen_jsx_fbt_child_element<'a>(
    cx: &mut OxcContext<'a, '_>,
    place: &Place,
) -> Result<oxc::JSXChild<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    match value {
        OxValue::JsxText(text) => {
            let encoded = ox_encode_jsx_text(text.value.as_str());
            Ok(cx.ast.jsx_child_text(SPAN, ox_str(&cx.ast, &encoded), None))
        }
        OxValue::Expression(oxc::Expression::JSXElement(elem)) => {
            let elem = elem.unbox();
            Ok(cx.ast.jsx_child_element(
                SPAN,
                elem.opening_element,
                elem.children,
                elem.closing_element,
            ))
        }
        OxValue::Expression(expr) => {
            Ok(cx.ast.jsx_child_expression_container(SPAN, oxc::JSXExpression::from(expr)))
        }
    }
}

/// Build a `JSXElementName` from a tag expression following the TS compiler's
/// identifier-reference rule (uppercase / contains-`.` names become references).
fn ox_expression_to_jsx_tag<'a>(
    cx: &OxcContext<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<oxc::JSXElementName<'a>, CompilerError> {
    match expr {
        oxc::Expression::Identifier(ident) => Ok(ox_jsx_element_name_from_ident(cx, &ident.name)),
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_) => {
            let member = ox_convert_member_expression_to_jsx(cx, expr)?;
            Ok(cx.ast.jsx_element_name_member_expression(SPAN, member.0, member.1))
        }
        oxc::Expression::StringLiteral(s) => {
            let tag_text = s.value.as_str();
            if tag_text.contains(':') {
                let parts: Vec<&str> = tag_text.splitn(2, ':').collect();
                let namespace = cx.ast.jsx_identifier(SPAN, ox_str(&cx.ast, parts[0]));
                let name = cx.ast.jsx_identifier(SPAN, ox_str(&cx.ast, parts[1]));
                Ok(cx.ast.jsx_element_name_namespaced_name(SPAN, namespace, name))
            } else {
                Ok(ox_jsx_element_name_from_ident(cx, tag_text))
            }
        }
        _ => Err(invariant_err("Expected JSX tag to be an identifier or string", None)),
    }
}

fn ox_jsx_element_name_from_ident<'a>(
    cx: &OxcContext<'a, '_>,
    name: &str,
) -> oxc::JSXElementName<'a> {
    let first_char = name.chars().next().unwrap_or('a');
    if first_char.is_uppercase() || name.contains('.') {
        cx.ast.jsx_element_name_identifier_reference(SPAN, ox_str(&cx.ast, name))
    } else {
        cx.ast.jsx_element_name_identifier(SPAN, ox_str(&cx.ast, name))
    }
}

/// Convert an oxc member expression into a JSX member expression's
/// `(object, property)` pair.
fn ox_convert_member_expression_to_jsx<'a>(
    cx: &OxcContext<'a, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<(oxc::JSXMemberExpressionObject<'a>, oxc::JSXIdentifier<'a>), CompilerError> {
    let oxc::Expression::StaticMemberExpression(me) = expr else {
        return Err(invariant_err("Expected JSX member expression property to be a string", None));
    };
    let property = cx.ast.jsx_identifier(SPAN, ox_str(&cx.ast, me.property.name.as_str()));
    let object = match &me.object {
        oxc::Expression::Identifier(ident) => cx
            .ast
            .jsx_member_expression_object_identifier_reference(SPAN, ox_str(&cx.ast, &ident.name)),
        oxc::Expression::StaticMemberExpression(_) => {
            let inner = ox_convert_member_expression_to_jsx(cx, &me.object)?;
            cx.ast.jsx_member_expression_object_member_expression(SPAN, inner.0, inner.1)
        }
        _ => {
            return Err(invariant_err(
                "Expected JSX member expression to be an identifier or nested member expression",
                None,
            ));
        }
    };
    Ok((object, property))
}

fn ox_maybe_wrap_hook_call<'a>(
    cx: &OxcContext<'a, '_>,
    call_expr: oxc::Expression<'a>,
    _callee_id: IdentifierId,
) -> Result<oxc::Expression<'a>, CompilerError> {
    // enableEmitHookGuards wrapping is deferred to a later batch; the guard is
    // off by default, so unwrapped calls match the differential floor.
    if cx.env.hook_guard_name.is_some()
        && cx.env.output_mode == crate::react_compiler_hir::environment::OutputMode::Client
    {
        return Err(invariant_err(
            "Hook guard wrapping in oxc codegen is not yet ported (deferred to a later batch)",
            None,
        ));
    }
    Ok(call_expr)
}

// =============================================================================
// Operator conversions (HIR -> oxc)
// =============================================================================

fn ox_convert_binary_operator(
    op: &crate::react_compiler_hir::BinaryOperator,
) -> oxc::BinaryOperator {
    use crate::react_compiler_hir::BinaryOperator as Hir;
    use oxc::BinaryOperator as Ox;
    match op {
        Hir::Equal => Ox::Equality,
        Hir::NotEqual => Ox::Inequality,
        Hir::StrictEqual => Ox::StrictEquality,
        Hir::StrictNotEqual => Ox::StrictInequality,
        Hir::LessThan => Ox::LessThan,
        Hir::LessEqual => Ox::LessEqualThan,
        Hir::GreaterThan => Ox::GreaterThan,
        Hir::GreaterEqual => Ox::GreaterEqualThan,
        Hir::ShiftLeft => Ox::ShiftLeft,
        Hir::ShiftRight => Ox::ShiftRight,
        Hir::UnsignedShiftRight => Ox::ShiftRightZeroFill,
        Hir::Add => Ox::Addition,
        Hir::Subtract => Ox::Subtraction,
        Hir::Multiply => Ox::Multiplication,
        Hir::Divide => Ox::Division,
        Hir::Modulo => Ox::Remainder,
        Hir::Exponent => Ox::Exponential,
        Hir::BitwiseOr => Ox::BitwiseOR,
        Hir::BitwiseXor => Ox::BitwiseXOR,
        Hir::BitwiseAnd => Ox::BitwiseAnd,
        Hir::In => Ox::In,
        Hir::InstanceOf => Ox::Instanceof,
    }
}

fn ox_convert_unary_operator(op: &crate::react_compiler_hir::UnaryOperator) -> oxc::UnaryOperator {
    use crate::react_compiler_hir::UnaryOperator as Hir;
    use oxc::UnaryOperator as Ox;
    match op {
        Hir::Minus => Ox::UnaryNegation,
        Hir::Plus => Ox::UnaryPlus,
        Hir::Not => Ox::LogicalNot,
        Hir::BitwiseNot => Ox::BitwiseNot,
        Hir::TypeOf => Ox::Typeof,
        Hir::Void => Ox::Void,
    }
}

fn ox_convert_logical_operator(op: &LogicalOperator) -> oxc::LogicalOperator {
    match op {
        LogicalOperator::And => oxc::LogicalOperator::And,
        LogicalOperator::Or => oxc::LogicalOperator::Or,
        LogicalOperator::NullishCoalescing => oxc::LogicalOperator::Coalesce,
    }
}

fn ox_convert_update_operator(
    op: &crate::react_compiler_hir::UpdateOperator,
) -> oxc::UpdateOperator {
    match op {
        crate::react_compiler_hir::UpdateOperator::Increment => oxc::UpdateOperator::Increment,
        crate::react_compiler_hir::UpdateOperator::Decrement => oxc::UpdateOperator::Decrement,
    }
}

// =============================================================================
// Primitive / literal helpers (oxc)
// =============================================================================

fn ox_codegen_primitive_value<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    value: &PrimitiveValue,
) -> oxc::Expression<'a> {
    match value {
        PrimitiveValue::Number(n) => {
            let f = n.value();
            if f.is_nan() {
                ast.expression_identifier(SPAN, "NaN")
            } else if f.is_infinite() {
                if f > 0.0 {
                    ast.expression_identifier(SPAN, "Infinity")
                } else {
                    ast.expression_unary(
                        SPAN,
                        oxc::UnaryOperator::UnaryNegation,
                        ast.expression_identifier(SPAN, "Infinity"),
                    )
                }
            } else if f < 0.0 {
                ast.expression_unary(SPAN, oxc::UnaryOperator::UnaryNegation, ox_number(ast, -f))
            } else {
                ox_number(ast, f)
            }
        }
        PrimitiveValue::Boolean(b) => ast.expression_boolean_literal(SPAN, *b),
        PrimitiveValue::String(s) => {
            ast.expression_string_literal(SPAN, ox_str(ast, &s.to_string_lossy()), None)
        }
        PrimitiveValue::Null => ast.expression_null_literal(SPAN),
        PrimitiveValue::Undefined => ast.expression_identifier(SPAN, "undefined"),
    }
}

fn ox_parse_regexp_flags(flags_str: &str) -> oxc::RegExpFlags {
    let mut flags = oxc::RegExpFlags::empty();
    for c in flags_str.chars() {
        match c {
            'g' => flags |= oxc::RegExpFlags::G,
            'i' => flags |= oxc::RegExpFlags::I,
            'm' => flags |= oxc::RegExpFlags::M,
            's' => flags |= oxc::RegExpFlags::S,
            'u' => flags |= oxc::RegExpFlags::U,
            'y' => flags |= oxc::RegExpFlags::Y,
            'd' => flags |= oxc::RegExpFlags::D,
            'v' => flags |= oxc::RegExpFlags::V,
            _ => {}
        }
    }
    flags
}

// =============================================================================
// CountMemoBlockVisitor — uses ReactiveFunctionVisitor trait
// =============================================================================

/// Counts memo blocks and pruned memo blocks in a reactive function.
/// TS: `class CountMemoBlockVisitor extends ReactiveFunctionVisitor<void>`
struct CountMemoBlockVisitor<'a> {
    env: &'a Environment,
}

struct CountMemoBlockState {
    memo_blocks: u32,
    memo_values: u32,
    pruned_memo_blocks: u32,
    pruned_memo_values: u32,
}

impl<'a> ReactiveFunctionVisitor for CountMemoBlockVisitor<'a> {
    type State = CountMemoBlockState;

    fn env(&self) -> &Environment {
        self.env
    }

    fn visit_scope(&self, scope_block: &ReactiveScopeBlock, state: &mut CountMemoBlockState) {
        state.memo_blocks += 1;
        let scope = &self.env.scopes[scope_block.scope.0 as usize];
        state.memo_values += scope.declarations.len() as u32;
        self.traverse_scope(scope_block, state);
    }

    fn visit_pruned_scope(
        &self,
        scope_block: &PrunedReactiveScopeBlock,
        state: &mut CountMemoBlockState,
    ) {
        state.pruned_memo_blocks += 1;
        let scope = &self.env.scopes[scope_block.scope.0 as usize];
        state.pruned_memo_values += scope.declarations.len() as u32;
        self.traverse_pruned_scope(scope_block, state);
    }
}

fn count_memo_blocks(func: &ReactiveFunction, env: &Environment) -> (u32, u32, u32, u32) {
    let visitor = CountMemoBlockVisitor { env };
    let mut state = CountMemoBlockState {
        memo_blocks: 0,
        memo_values: 0,
        pruned_memo_blocks: 0,
        pruned_memo_values: 0,
    };
    visit_reactive_function(func, &visitor, &mut state);
    (state.memo_blocks, state.memo_values, state.pruned_memo_blocks, state.pruned_memo_values)
}

fn codegen_label(id: BlockId) -> String {
    format!("bb{}", id.0)
}

fn get_instruction_value(
    reactive_value: &ReactiveValue,
) -> Result<&InstructionValue, CompilerError> {
    match reactive_value {
        ReactiveValue::Instruction(iv) => Ok(iv),
        _ => Err(invariant_err("Expected base instruction value", None)),
    }
}

fn invariant(
    condition: bool,
    reason: &str,
    loc: Option<DiagSourceLocation>,
) -> Result<(), CompilerError> {
    if !condition { Err(invariant_err(reason, loc)) } else { Ok(()) }
}

fn invariant_err(reason: &str, loc: Option<DiagSourceLocation>) -> CompilerError {
    // Use CompilerDiagnostic (with details array) to match TS CompilerError.invariant()
    let mut err = CompilerError::new();
    err.push_diagnostic(
        CompilerDiagnostic::new(ErrorCategory::Invariant, reason, None::<String>).with_detail(
            CompilerDiagnosticDetail::Error {
                loc,
                message: Some(reason.to_string()),
                identifier_name: None,
            },
        ),
    );
    err
}

fn invariant_err_with_detail_message(
    reason: &str,
    message: &str,
    loc: Option<DiagSourceLocation>,
) -> CompilerError {
    let mut err = CompilerError::new();
    let diagnostic = crate::react_compiler_diagnostics::CompilerDiagnostic::new(
        ErrorCategory::Invariant,
        reason,
        None::<String>,
    )
    .with_detail(crate::react_compiler_diagnostics::CompilerDiagnosticDetail::Error {
        loc,
        message: Some(message.to_string()),
        identifier_name: None,
    });
    err.push_diagnostic(diagnostic);
    err
}

fn compare_scope_dependency(
    a: &crate::react_compiler_hir::ReactiveScopeDependency,
    b: &crate::react_compiler_hir::ReactiveScopeDependency,
    env: &Environment,
) -> std::cmp::Ordering {
    let a_name = dep_to_sort_key(a, env);
    let b_name = dep_to_sort_key(b, env);
    a_name.cmp(&b_name)
}

fn dep_to_sort_key(
    dep: &crate::react_compiler_hir::ReactiveScopeDependency,
    env: &Environment,
) -> String {
    let ident = &env.identifiers[dep.identifier.0 as usize];
    let base = match &ident.name {
        Some(crate::react_compiler_hir::IdentifierName::Named(n)) => n.clone(),
        Some(crate::react_compiler_hir::IdentifierName::Promoted(n)) => n.clone(),
        None => format!("_t{}", dep.identifier.0),
    };
    let mut parts = vec![base];
    for entry in &dep.path {
        let prefix = if entry.optional { "?" } else { "" };
        let prop = match &entry.property {
            PropertyLiteral::String(s) => s.clone(),
            PropertyLiteral::Number(n) => format!("{}", n),
        };
        parts.push(format!("{prefix}{prop}"));
    }
    parts.join(".")
}

fn compare_scope_declaration(
    a: &crate::react_compiler_hir::ReactiveScopeDeclaration,
    b: &crate::react_compiler_hir::ReactiveScopeDeclaration,
    env: &Environment,
) -> std::cmp::Ordering {
    let a_name = ident_sort_key(a.identifier, env);
    let b_name = ident_sort_key(b.identifier, env);
    a_name.cmp(&b_name)
}

fn ident_sort_key(id: IdentifierId, env: &Environment) -> String {
    let ident = &env.identifiers[id.0 as usize];
    match &ident.name {
        Some(crate::react_compiler_hir::IdentifierName::Named(n)) => n.clone(),
        Some(crate::react_compiler_hir::IdentifierName::Promoted(n)) => n.clone(),
        None => format!("_t{}", id.0),
    }
}

#[cfg(test)]
mod tests {
    /// The Fast Refresh source hash must match Node's
    /// `createHmac('sha256', code).digest('hex')` byte-for-byte, or hot-reload
    /// cache invalidation would diverge from the TS compiler. Reference values
    /// were computed with Node's `crypto` module.
    #[test]
    fn source_file_hash_matches_node_create_hmac() {
        use super::source_file_hash;
        assert_eq!(
            source_file_hash("hello world"),
            "0de8bee5d7f9c5d209f8c6fabed0ea84cb3fca1244e8ed38079a61b599a84c47"
        );
        assert_eq!(
            source_file_hash(""),
            "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"
        );
        assert_eq!(
            source_file_hash("function App(){}"),
            "d637acb4985c789d6622c70197db2b62dda282f16f3276aa810b598d6e6cab7b"
        );
    }
}
