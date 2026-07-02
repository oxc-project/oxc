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
pub fn codegen_function<'a, 'h>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &ReactiveFunction<'h>,
    env: &mut Environment<'h>,
    unique_identifiers: FxHashSet<String>,
    fbt_operands: FxHashSet<IdentifierId>,
) -> Result<crate::react_compiler::entrypoint::compile_result::CodegenFunction<'a>, CompilerError> {
    use crate::react_compiler::entrypoint::compile_result::CodegenFunction as OxcCodegenFunction;
    use oxc_span::SPAN;

    let fn_name = func.id.as_deref().unwrap_or("[[ anonymous ]]");
    // Outlined functions reuse the same `fbtOperands` set as the main function
    // (see TS `codegenFunction`), so keep a copy before it is moved into the context.
    let fbt_operands_for_outlined = fbt_operands.clone();
    let mut cx = OxcContext::new(
        oxc_ast::builder::AstBuilder::new(ast.allocator()),
        env,
        fn_name.to_string(),
        unique_identifiers,
        fbt_operands,
    );

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
            params: oxc_ast::ast::FormalParameters::boxed(
                SPAN,
                oxc_ast::ast::FormalParameterKind::FormalParameter,
                oxc_allocator::ArenaVec::new_in(ast),
                None::<oxc_allocator::Box<oxc_ast::ast::FormalParameterRest>>,
                ast,
            ),
            body: oxc_ast::ast::FunctionBody::boxed(
                SPAN,
                oxc_allocator::ArenaVec::new_in(ast),
                oxc_allocator::ArenaVec::new_in(ast),
                ast,
            ),
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
        let use_memo_cache = oxc_ast::ast::Expression::new_call_expression(
            SPAN,
            oxc_ast::ast::Expression::new_identifier(SPAN, "useMemoCache", ast),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeParameterInstantiation>>,
            oxc_allocator::ArenaVec::from_value_in(
                oxc_ast::ast::Argument::from(ox_number(ast, cache_count as f64)),
                ast,
            ),
            false,
            ast,
        );
        let declarator = oxc_ast::ast::VariableDeclarator::new(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Const,
            oxc_ast::ast::BindingPattern::new_binding_identifier(
                SPAN,
                ox_str(ast, &cache_name),
                ast,
            ),
            None::<oxc_allocator::Box<oxc_ast::ast::TSTypeAnnotation>>,
            Some(use_memo_cache),
            false,
            ast,
        );
        let preface =
            oxc_ast::ast::Statement::VariableDeclaration(oxc_ast::ast::VariableDeclaration::boxed(
                SPAN,
                oxc_ast::ast::VariableDeclarationKind::Const,
                oxc_allocator::ArenaVec::from_value_in(declarator, ast),
                false,
                ast,
            ));
        let body_stmts =
            std::mem::replace(&mut compiled.body.statements, oxc_allocator::ArenaVec::new_in(ast));
        let mut new_body = oxc_allocator::ArenaVec::from_value_in(preface, ast);
        new_body.extend(body_stmts);
        compiled.body.statements = new_body;
    }

    let id = func
        .id
        .as_deref()
        .map(|name| oxc_ast::ast::BindingIdentifier::new(SPAN, ox_str(ast, name), ast));

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

        let func =
            codegen_function(ast, &reactive_function, env, identifiers, fbt_operands.clone())?;
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

use oxc_allocator::GetAllocator;
use oxc_allocator::IntoIn;
use oxc_ast::ast as oxc;
use oxc_span::SPAN;

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

struct OxcContext<'a, 'env, 'h> {
    ast: oxc_ast::builder::AstBuilder<'a>,
    env: &'env mut Environment<'h>,
    #[allow(dead_code)]
    fn_name: String,
    next_cache_index: u32,
    declarations: FxHashSet<DeclarationId>,
    temp: OxcTemporaries<'a>,
    object_methods: FxHashMap<
        IdentifierId,
        (InstructionValue<'h>, Option<crate::react_compiler_diagnostics::SourceLocation>),
    >,
    unique_identifiers: FxHashSet<String>,
    #[allow(dead_code)]
    fbt_operands: FxHashSet<IdentifierId>,
    synthesized_names: FxHashMap<String, String>,
}

impl<'a, 'env, 'h> OxcContext<'a, 'env, 'h> {
    fn new(
        ast: oxc_ast::builder::AstBuilder<'a>,
        env: &'env mut Environment<'h>,
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
    oxc_ast::ast::Expression::new_numeric_literal(SPAN, value, None, oxc::NumberBase::Decimal, ast)
}

/// Allocate a `&'a str` in the arena (satisfies the builders' `IntoIn` slots for
/// both `Atom` and `Str`).
fn ox_str<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, s: &str) -> &'a str {
    oxc_allocator::StringBuilder::from_str_in(s, ast.allocator()).into_str()
}

/// Re-emit a TS type annotation stored on a `TypeCastExpression` into the output
/// allocator. The lowering stores the original `&TSType` AST node directly, so this
/// is a `clone_in` — no parser.
///
/// When some identifier reference inside the type has a binding rename (e.g. a
/// `typeof field` whose value binding was renamed to `field_3`), the matching
/// references in the clone are renamed in place. `clone_in` preserves spans, so the
/// renames are keyed by source offset, exactly like the reference set that selects
/// them.
fn ox_reemit_ts_type<'a>(cx: &OxcContext<'a, '_, '_>, ty: &oxc::TSType<'_>) -> oxc::TSType<'a> {
    // Which identifier references inside the type get a binding rename, keyed by
    // absolute source offset. An ident is renamed only if it is an actual reference
    // (its offset is in `reference_node_ids`, excluding type labels / property keys)
    // and a binding rename applies for the nearest enclosing declaration. Without
    // this, a `typeof field` keeps the pre-rename name while the value binding was
    // renamed to `field_3`.
    let renames_by_offset: FxHashMap<u32, String> = if cx.env.renames.is_empty() {
        FxHashMap::default()
    } else {
        struct Collector {
            out: Vec<(u32, String)>,
        }
        impl<'v> oxc_ast_visit::Visit<'v> for Collector {
            fn visit_identifier_reference(&mut self, it: &oxc::IdentifierReference<'v>) {
                self.out.push((it.span.start, it.name.to_string()));
            }
            fn visit_identifier_name(&mut self, it: &oxc::IdentifierName<'v>) {
                self.out.push((it.span.start, it.name.to_string()));
            }
        }
        let mut collector = Collector { out: Vec::new() };
        oxc_ast_visit::Visit::visit_ts_type(&mut collector, ty);

        let mut renames = FxHashMap::default();
        for (offset, name) in &collector.out {
            if *offset == 0 || !cx.env.reference_node_ids.contains(offset) {
                continue;
            }
            if let Some(rename) = cx
                .env
                .renames
                .iter()
                .filter(|r| &r.original == name && r.declaration_start <= *offset)
                .max_by_key(|r| r.declaration_start)
            {
                renames.insert(*offset, rename.renamed.clone());
            }
        }
        renames
    };

    // Clone the stored type into the output allocator. Common case: no renames apply,
    // so the clone is the whole answer.
    let mut cloned = ty.clone_in(cx.ast.allocator());
    if renames_by_offset.is_empty() {
        return cloned;
    }

    // Rename case: rewrite the matching identifier references in the clone, keyed by
    // the preserved source offset.
    struct Renamer<'a> {
        allocator: &'a oxc_allocator::Allocator,
        renames_by_offset: FxHashMap<u32, String>,
    }
    impl<'a> oxc_ast_visit::VisitMut<'a> for Renamer<'a> {
        fn visit_identifier_reference(&mut self, it: &mut oxc::IdentifierReference<'a>) {
            if let Some(renamed) = self.renames_by_offset.get(&it.span.start) {
                it.name = renamed.as_str().into_in(self.allocator);
            }
        }
        fn visit_identifier_name(&mut self, it: &mut oxc::IdentifierName<'a>) {
            if let Some(renamed) = self.renames_by_offset.get(&it.span.start) {
                it.name = renamed.as_str().into_in(self.allocator);
            }
        }
    }
    let mut renamer = Renamer { allocator: cx.ast.allocator(), renames_by_offset };
    oxc_ast_visit::VisitMut::visit_ts_type(&mut renamer, &mut cloned);
    cloned
}

/// Build `Symbol.for("<name>")`.
fn ox_symbol_for<'a>(ast: &oxc_ast::builder::AstBuilder<'a>, name: &str) -> oxc::Expression<'a> {
    let callee =
        oxc::Expression::from(oxc_ast::ast::MemberExpression::new_static_member_expression(
            SPAN,
            oxc_ast::ast::Expression::new_identifier(SPAN, "Symbol", ast),
            oxc_ast::ast::IdentifierName::new(SPAN, "for", ast),
            false,
            ast,
        ));
    oxc_ast::ast::Expression::new_call_expression(
        SPAN,
        callee,
        None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
        oxc_allocator::ArenaVec::from_value_in(
            oxc::Argument::from(oxc_ast::ast::Expression::new_string_literal(
                SPAN,
                ox_str(ast, name),
                None,
                ast,
            )),
            ast,
        ),
        false,
        ast,
    )
}

/// `$[index]` computed member expression.
fn ox_cache_index<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    cache_name: &str,
    index: u32,
) -> oxc::Expression<'a> {
    oxc::Expression::from(oxc_ast::ast::MemberExpression::new_computed_member_expression(
        SPAN,
        oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(ast, cache_name), ast),
        ox_number(ast, index as f64),
        false,
        ast,
    ))
}

fn ox_codegen_reactive_function<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    func: &ReactiveFunction<'h>,
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
    let directives = oxc_allocator::ArenaVec::from_iter_in(
        func.directives.iter().map(|d| {
            oxc_ast::ast::Directive::new(
                SPAN,
                oxc_ast::ast::StringLiteral::new(SPAN, ox_str(&cx.ast, d), None, &cx.ast),
                ox_str(&cx.ast, d),
                &cx.ast,
            )
        }),
        &cx.ast,
    );

    // Remove trailing `return undefined`
    if let Some(oxc::Statement::ReturnStatement(ret)) = statements.last() {
        if ret.argument.is_none() {
            statements.pop();
        }
    }

    let (memo_blocks, memo_values, pruned_memo_blocks, pruned_memo_values) =
        count_memo_blocks(func, cx.env);

    let body = oxc_ast::ast::FunctionBody::boxed(SPAN, directives, statements, &cx.ast);

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
    cx: &mut OxcContext<'a, '_, '_>,
    params: &[ParamPattern],
) -> Result<oxc_allocator::Box<'a, oxc::FormalParameters<'a>>, CompilerError> {
    let mut items: Vec<oxc::FormalParameter<'a>> = Vec::new();
    let mut rest: Option<oxc::FormalParameterRest<'a>> = None;
    for param in params {
        match param {
            ParamPattern::Place(place) => {
                let binding = ox_binding_for_identifier(cx, place.identifier)?;
                items.push(oxc_ast::ast::FormalParameter::new(
                    SPAN,
                    oxc_allocator::ArenaVec::new_in(&cx.ast),
                    binding,
                    None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                    None::<oxc_allocator::Box<oxc::Expression>>,
                    false,
                    None,
                    false,
                    false,
                    &cx.ast,
                ));
            }
            ParamPattern::Spread(spread) => {
                let binding = ox_binding_for_identifier(cx, spread.place.identifier)?;
                let rest_elem = oxc_ast::ast::BindingRestElement::new(SPAN, binding, &cx.ast);
                rest = Some(oxc_ast::ast::FormalParameterRest::new(
                    SPAN,
                    oxc_allocator::ArenaVec::new_in(&cx.ast),
                    rest_elem,
                    None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                    &cx.ast,
                ));
            }
        }
    }
    let items_vec = oxc_allocator::ArenaVec::from_iter_in(items, &cx.ast);
    Ok(oxc_ast::ast::FormalParameters::boxed(
        SPAN,
        oxc::FormalParameterKind::FormalParameter,
        items_vec,
        rest,
        &cx.ast,
    ))
}

fn ox_binding_for_identifier<'a>(
    cx: &OxcContext<'a, '_, '_>,
    identifier_id: IdentifierId,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let name = ox_identifier_name(cx.env, identifier_id)?;
    Ok(oxc_ast::ast::BindingPattern::new_binding_identifier(SPAN, ox_str(&cx.ast, &name), &cx.ast))
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

fn ox_codegen_block<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    block: &ReactiveBlock<'h>,
) -> Result<oxc_allocator::Vec<'a, oxc::Statement<'a>>, CompilerError> {
    let temp_snapshot = ox_clone_temporaries(&cx.ast, &cx.temp);
    let result = ox_codegen_block_no_reset(cx, block)?;
    cx.temp = temp_snapshot;
    Ok(result)
}

fn ox_codegen_block_no_reset<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    block: &ReactiveBlock<'h>,
) -> Result<oxc_allocator::Vec<'a, oxc::Statement<'a>>, CompilerError> {
    let mut statements: oxc_allocator::Vec<'a, oxc::Statement<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
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
                        let label_ident = oxc_ast::ast::LabelIdentifier::new(
                            SPAN,
                            ox_str(&cx.ast, &codegen_label(label.id)),
                            &cx.ast,
                        );
                        statements.push(oxc_ast::ast::Statement::new_labeled_statement(
                            SPAN,
                            label_ident,
                            inner,
                            &cx.ast,
                        ));
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

fn ox_codegen_block_statement<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    block: &ReactiveBlock<'h>,
) -> Result<oxc::BlockStatement<'a>, CompilerError> {
    let body = ox_codegen_block(cx, block)?;
    Ok(oxc_ast::ast::BlockStatement::new(SPAN, body, &cx.ast))
}

// =============================================================================
// Reactive scope codegen (memoization) (oxc)
// =============================================================================

fn ox_codegen_reactive_scope<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    statements: &mut oxc_allocator::Vec<'a, oxc::Statement<'a>>,
    scope_id: ScopeId,
    block: &ReactiveBlock<'h>,
) -> Result<(), CompilerError> {
    let scope_deps = cx.env.scopes[scope_id.0 as usize].dependencies.clone();
    let scope_decls = cx.env.scopes[scope_id.0 as usize].declarations.clone();
    let scope_reassignments = cx.env.scopes[scope_id.0 as usize].reassignments.clone();

    let mut cache_store_stmts: oxc_allocator::Vec<'a, oxc::Statement<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    let mut cache_load_stmts: oxc_allocator::Vec<'a, oxc::Statement<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    let mut cache_loads: Vec<(String, u32)> = Vec::new();
    let mut change_exprs: Vec<oxc::Expression<'a>> = Vec::new();

    let mut deps = scope_deps;
    deps.sort_by(|a, b| compare_scope_dependency(a, b, cx.env));

    for dep in &deps {
        let index = cx.alloc_cache_index();
        let cache_name = cx.synthesize_name("$");
        let dep_expr = ox_codegen_dependency(cx, dep)?;
        let comparison = oxc_ast::ast::Expression::new_binary_expression(
            SPAN,
            ox_cache_index(&cx.ast, &cache_name, index),
            oxc::BinaryOperator::StrictInequality,
            dep_expr,
            &cx.ast,
        );
        change_exprs.push(comparison);

        let dep_value = ox_codegen_dependency(cx, dep)?;
        let store = oxc_ast::ast::Expression::new_assignment_expression(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(ast_member_target(
                &cx.ast,
                &cache_name,
                index,
            ))),
            dep_value,
            &cx.ast,
        );
        cache_store_stmts
            .push(oxc_ast::ast::Statement::new_expression_statement(SPAN, store, &cx.ast));
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
            let declarator = oxc_ast::ast::VariableDeclarator::new(
                SPAN,
                oxc::VariableDeclarationKind::Let,
                oxc_ast::ast::BindingPattern::new_binding_identifier(
                    SPAN,
                    ox_str(&cx.ast, &name),
                    &cx.ast,
                ),
                None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                None,
                false,
                &cx.ast,
            );
            statements.push(oxc::Statement::VariableDeclaration(
                oxc_ast::ast::VariableDeclaration::boxed(
                    SPAN,
                    oxc::VariableDeclarationKind::Let,
                    oxc_allocator::ArenaVec::from_value_in(declarator, &cx.ast),
                    false,
                    &cx.ast,
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
        oxc_ast::ast::Expression::new_binary_expression(
            SPAN,
            ox_cache_index(&cx.ast, &cache_name, first_idx),
            oxc::BinaryOperator::StrictEquality,
            ox_symbol_for(&cx.ast, MEMO_CACHE_SENTINEL),
            &cx.ast,
        )
    } else {
        change_exprs
            .into_iter()
            .reduce(|acc, expr| {
                oxc_ast::ast::Expression::new_logical_expression(
                    SPAN,
                    acc,
                    oxc::LogicalOperator::Or,
                    expr,
                    &cx.ast,
                )
            })
            .unwrap()
    };

    let mut computation_body = ox_codegen_block(cx, block)?;

    for (name, index) in &cache_loads {
        let cache_name = cx.synthesize_name("$");
        // $[index] = name
        let store = oxc_ast::ast::Expression::new_assignment_expression(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(ast_member_target(
                &cx.ast,
                &cache_name,
                *index,
            ))),
            oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(&cx.ast, name), &cx.ast),
            &cx.ast,
        );
        cache_store_stmts
            .push(oxc_ast::ast::Statement::new_expression_statement(SPAN, store, &cx.ast));
        // name = $[index]
        let load = oxc_ast::ast::Expression::new_assignment_expression(
            SPAN,
            oxc::AssignmentOperator::Assign,
            oxc::AssignmentTarget::AssignmentTargetIdentifier(
                oxc_ast::ast::IdentifierReference::boxed(SPAN, ox_str(&cx.ast, name), &cx.ast),
            ),
            ox_cache_index(&cx.ast, &cache_name, *index),
            &cx.ast,
        );
        cache_load_stmts
            .push(oxc_ast::ast::Statement::new_expression_statement(SPAN, load, &cx.ast));
    }

    computation_body.extend(cache_store_stmts);

    let memo_stmt = oxc_ast::ast::Statement::new_if_statement(
        SPAN,
        test_condition,
        oxc_ast::ast::Statement::new_block_statement(SPAN, computation_body, &cx.ast),
        Some(oxc_ast::ast::Statement::new_block_statement(SPAN, cache_load_stmts, &cx.ast)),
        &cx.ast,
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
        let test = oxc_ast::ast::Expression::new_binary_expression(
            SPAN,
            oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(&cx.ast, &name), &cx.ast),
            oxc::BinaryOperator::StrictInequality,
            ox_symbol_for(&cx.ast, EARLY_RETURN_SENTINEL),
            &cx.ast,
        );
        let return_stmt = oxc_ast::ast::Statement::new_return_statement(
            SPAN,
            Some(oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(&cx.ast, &name), &cx.ast)),
            &cx.ast,
        );
        let consequent = oxc_ast::ast::Statement::new_block_statement(
            SPAN,
            oxc_allocator::ArenaVec::from_value_in(return_stmt, &cx.ast),
            &cx.ast,
        );
        statements
            .push(oxc_ast::ast::Statement::new_if_statement(SPAN, test, consequent, None, &cx.ast));
    }

    Ok(())
}

/// Build `$[index]` as a `MemberExpression` for use as an assignment target.
fn ast_member_target<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    cache_name: &str,
    index: u32,
) -> oxc::MemberExpression<'a> {
    oxc_ast::ast::MemberExpression::new_computed_member_expression(
        SPAN,
        oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(ast, cache_name), ast),
        ox_number(ast, index as f64),
        false,
        ast,
    )
}

// =============================================================================
// Terminal codegen (oxc)
// =============================================================================

fn ox_codegen_terminal<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    terminal: &ReactiveTerminal<'h>,
) -> Result<Option<oxc::Statement<'a>>, CompilerError> {
    match terminal {
        ReactiveTerminal::Break { target, target_kind, .. } => {
            if *target_kind == ReactiveTerminalTargetKind::Implicit {
                return Ok(None);
            }
            let label = if *target_kind == ReactiveTerminalTargetKind::Labeled {
                Some(oxc_ast::ast::LabelIdentifier::new(
                    SPAN,
                    ox_str(&cx.ast, &codegen_label(*target)),
                    &cx.ast,
                ))
            } else {
                None
            };
            Ok(Some(oxc_ast::ast::Statement::new_break_statement(SPAN, label, &cx.ast)))
        }
        ReactiveTerminal::Continue { target, target_kind, .. } => {
            if *target_kind == ReactiveTerminalTargetKind::Implicit {
                return Ok(None);
            }
            let label = if *target_kind == ReactiveTerminalTargetKind::Labeled {
                Some(oxc_ast::ast::LabelIdentifier::new(
                    SPAN,
                    ox_str(&cx.ast, &codegen_label(*target)),
                    &cx.ast,
                ))
            } else {
                None
            };
            Ok(Some(oxc_ast::ast::Statement::new_continue_statement(SPAN, label, &cx.ast)))
        }
        ReactiveTerminal::Return { value, .. } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            if let oxc::Expression::Identifier(ref ident) = expr {
                if ident.name == "undefined" {
                    return Ok(Some(oxc_ast::ast::Statement::new_return_statement(
                        SPAN, None, &cx.ast,
                    )));
                }
            }
            Ok(Some(oxc_ast::ast::Statement::new_return_statement(SPAN, Some(expr), &cx.ast)))
        }
        ReactiveTerminal::Throw { value, .. } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            Ok(Some(oxc_ast::ast::Statement::new_throw_statement(SPAN, expr, &cx.ast)))
        }
        ReactiveTerminal::If { test, consequent, alternate, .. } => {
            let test_expr = ox_codegen_place_to_expression(cx, test)?;
            let consequent_block = ox_codegen_block_statement(cx, consequent)?;
            let consequent = oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(
                consequent_block,
                &cx.ast,
            ));
            let alternate = if let Some(alt) = alternate {
                let block = ox_codegen_block_statement(cx, alt)?;
                if block.body.is_empty() {
                    None
                } else {
                    Some(oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(
                        block, &cx.ast,
                    )))
                }
            } else {
                None
            };
            Ok(Some(oxc_ast::ast::Statement::new_if_statement(
                SPAN, test_expr, consequent, alternate, &cx.ast,
            )))
        }
        ReactiveTerminal::Switch { test, cases, .. } => {
            let test_expr = ox_codegen_place_to_expression(cx, test)?;
            let mut switch_cases: oxc_allocator::Vec<'a, oxc::SwitchCase<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
            for case in cases {
                let case_test = case
                    .test
                    .as_ref()
                    .map(|t| ox_codegen_place_to_expression(cx, t))
                    .transpose()?;
                let block =
                    case.block.as_ref().map(|b| ox_codegen_block_statement(cx, b)).transpose()?;
                let consequent: oxc_allocator::Vec<'a, oxc::Statement<'a>> = match block {
                    Some(b) if b.body.is_empty() => oxc_allocator::ArenaVec::new_in(&cx.ast),
                    Some(b) => oxc_allocator::ArenaVec::from_value_in(
                        oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(b, &cx.ast)),
                        &cx.ast,
                    ),
                    None => oxc_allocator::ArenaVec::new_in(&cx.ast),
                };
                switch_cases
                    .push(oxc_ast::ast::SwitchCase::new(SPAN, case_test, consequent, &cx.ast));
            }
            Ok(Some(oxc_ast::ast::Statement::new_switch_statement(
                SPAN,
                test_expr,
                switch_cases,
                &cx.ast,
            )))
        }
        ReactiveTerminal::DoWhile { loop_block, test, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body =
                oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast));
            Ok(Some(oxc_ast::ast::Statement::new_do_while_statement(
                SPAN, body, test_expr, &cx.ast,
            )))
        }
        ReactiveTerminal::While { test, loop_block, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body =
                oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast));
            Ok(Some(oxc_ast::ast::Statement::new_while_statement(SPAN, test_expr, body, &cx.ast)))
        }
        ReactiveTerminal::For { init, test, update, loop_block, .. } => {
            let init_val = ox_codegen_for_init(cx, init)?;
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let update_expr = update
                .as_ref()
                .map(|u| ox_codegen_instruction_value_to_expression(cx, u))
                .transpose()?;
            let body = ox_codegen_block_statement(cx, loop_block)?;
            let body =
                oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast));
            Ok(Some(oxc_ast::ast::Statement::new_for_statement(
                SPAN,
                init_val,
                Some(test_expr),
                update_expr,
                body,
                &cx.ast,
            )))
        }
        ReactiveTerminal::ForIn { init, loop_block, loc, .. } => {
            ox_codegen_for_in(cx, init, loop_block, *loc)
        }
        ReactiveTerminal::ForOf { init, test, loop_block, loc, .. } => {
            ox_codegen_for_of(cx, init, test, loop_block, *loc)
        }
        ReactiveTerminal::Label { block, .. } => {
            let body = ox_codegen_block_statement(cx, block)?;
            Ok(Some(oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast))))
        }
        ReactiveTerminal::Try { block, handler_binding, handler, .. } => {
            let catch_param = match handler_binding.as_ref() {
                Some(binding) => {
                    let ident = &cx.env.identifiers[binding.identifier.0 as usize];
                    cx.temp.insert(ident.declaration_id, None);
                    let pattern = ox_binding_for_identifier(cx, binding.identifier)?;
                    Some(oxc_ast::ast::CatchParameter::new(
                        SPAN,
                        pattern,
                        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
                        &cx.ast,
                    ))
                }
                None => None,
            };
            let try_block = ox_codegen_block_statement(cx, block)?;
            let handler_block = ox_codegen_block_statement(cx, handler)?;
            let handler = oxc_ast::ast::CatchClause::new(SPAN, catch_param, handler_block, &cx.ast);
            Ok(Some(oxc_ast::ast::Statement::new_try_statement(
                SPAN,
                try_block,
                Some(handler),
                None::<oxc_allocator::Box<oxc::BlockStatement>>,
                &cx.ast,
            )))
        }
    }
}

fn ox_codegen_for_in<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    init: &ReactiveValue<'h>,
    loop_block: &ReactiveBlock<'h>,
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
        return Ok(Some(oxc_ast::ast::Statement::new_empty_statement(SPAN, &cx.ast)));
    }
    let iterable_collection = &instructions[0];
    let iterable_item = &instructions[1];
    let instr_value = get_instruction_value(&iterable_item.value)?;
    let (lval, var_decl_kind) = ox_extract_for_in_of_lval(cx, instr_value, "for..in", loc)?;
    let right = ox_codegen_instruction_value_to_expression(cx, &iterable_collection.value)?;
    let body = ox_codegen_block_statement(cx, loop_block)?;
    let body = oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast));
    let declarator = oxc_ast::ast::VariableDeclarator::new(
        SPAN,
        var_decl_kind,
        lval,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        None,
        false,
        &cx.ast,
    );
    let decl = oxc_ast::ast::VariableDeclaration::boxed(
        SPAN,
        var_decl_kind,
        oxc_allocator::ArenaVec::from_value_in(declarator, &cx.ast),
        false,
        &cx.ast,
    );
    let left = oxc::ForStatementLeft::VariableDeclaration(decl);
    Ok(Some(oxc_ast::ast::Statement::new_for_in_statement(SPAN, left, right, body, &cx.ast)))
}

fn ox_codegen_for_of<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    init: &ReactiveValue<'h>,
    test: &ReactiveValue<'h>,
    loop_block: &ReactiveBlock<'h>,
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
        return Ok(Some(oxc_ast::ast::Statement::new_empty_statement(SPAN, &cx.ast)));
    }
    let iterable_item = &test_instrs[1];
    let instr_value = get_instruction_value(&iterable_item.value)?;
    let (lval, var_decl_kind) = ox_extract_for_in_of_lval(cx, instr_value, "for..of", loc)?;

    let right = ox_codegen_place_to_expression(cx, collection)?;
    let body = ox_codegen_block_statement(cx, loop_block)?;
    let body = oxc::Statement::BlockStatement(oxc_allocator::ArenaBox::new_in(body, &cx.ast));
    let declarator = oxc_ast::ast::VariableDeclarator::new(
        SPAN,
        var_decl_kind,
        lval,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        None,
        false,
        &cx.ast,
    );
    let decl = oxc_ast::ast::VariableDeclaration::boxed(
        SPAN,
        var_decl_kind,
        oxc_allocator::ArenaVec::from_value_in(declarator, &cx.ast),
        false,
        &cx.ast,
    );
    let left = oxc::ForStatementLeft::VariableDeclaration(decl);
    Ok(Some(oxc_ast::ast::Statement::new_for_of_statement(SPAN, false, left, right, body, &cx.ast)))
}

fn ox_extract_for_in_of_lval<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
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
                oxc_ast::ast::BindingPattern::new_binding_identifier(SPAN, "_", &cx.ast),
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

fn ox_codegen_for_init<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    init: &ReactiveValue<'h>,
) -> Result<Option<oxc::ForStatementInit<'a>>, CompilerError> {
    if let ReactiveValue::SequenceExpression { instructions, .. } = init {
        let block_items: Vec<ReactiveStatement> =
            instructions.iter().map(|i| ReactiveStatement::Instruction(i.clone())).collect();
        let body = ox_codegen_block(cx, &block_items)?;
        let mut declarators: oxc_allocator::Vec<'a, oxc::VariableDeclarator<'a>> =
            oxc_allocator::ArenaVec::new_in(&cx.ast);
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
        let decl =
            oxc_ast::ast::VariableDeclaration::boxed(SPAN, kind, declarators, false, &cx.ast);
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
        OxValue::JsxText(text) => {
            oxc_ast::ast::Expression::new_string_literal(SPAN, text.value.as_str(), None, ast)
        }
    }
}

fn ox_codegen_instruction_nullable<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr: &ReactiveInstruction<'h>,
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
                return Ok(Some(oxc_ast::ast::Statement::new_debugger_statement(SPAN, &cx.ast)));
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
            InstructionValue::UnsupportedNode { stmt, .. } => {
                // Statement-position unsupported node (e.g. an inline TS `enum`
                // declaration): re-emit it verbatim by cloning the borrowed oxc
                // statement into the output allocator, mirroring the Babel path's
                // `return node` for non-expression original nodes.
                return Ok(Some(stmt.clone_in(cx.ast.allocator())));
            }
            _ => {}
        }
    }
    let expr_value = ox_codegen_instruction_value(cx, &instr.value)?;
    let stmt = ox_codegen_instruction(cx, instr, expr_value)?;
    if matches!(stmt, oxc::Statement::EmptyStatement(_)) { Ok(None) } else { Ok(Some(stmt)) }
}

fn ox_codegen_store_or_declare<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr: &ReactiveInstruction<'h>,
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

fn ox_emit_store<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr: &ReactiveInstruction<'h>,
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
                    let decl = oxc_ast::ast::Function::boxed(
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
                        &cx.ast,
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
            let expr = oxc_ast::ast::Expression::new_assignment_expression(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                rhs,
                &cx.ast,
            );
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
            Ok(Some(oxc_ast::ast::Statement::new_expression_statement(SPAN, expr, &cx.ast)))
        }
        InstructionKind::Catch => {
            Ok(Some(oxc_ast::ast::Statement::new_empty_statement(SPAN, &cx.ast)))
        }
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
    cx: &OxcContext<'a, '_, '_>,
    kind: oxc::VariableDeclarationKind,
    id: oxc::BindingPattern<'a>,
    init: Option<oxc::Expression<'a>>,
) -> oxc::Statement<'a> {
    let declarator = oxc_ast::ast::VariableDeclarator::new(
        SPAN,
        kind,
        id,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        init,
        false,
        &cx.ast,
    );
    oxc::Statement::VariableDeclaration(oxc_ast::ast::VariableDeclaration::boxed(
        SPAN,
        kind,
        oxc_allocator::ArenaVec::from_value_in(declarator, &cx.ast),
        false,
        &cx.ast,
    ))
}

fn ox_codegen_instruction<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr: &ReactiveInstruction<'h>,
    value: OxValue<'a>,
) -> Result<oxc::Statement<'a>, CompilerError> {
    let Some(ref lvalue) = instr.lvalue else {
        let expr = ox_convert_value_to_expression(&cx.ast, value);
        return Ok(oxc_ast::ast::Statement::new_expression_statement(SPAN, expr, &cx.ast));
    };
    let ident = &cx.env.identifiers[lvalue.identifier.0 as usize];
    if ident.name.is_none() {
        cx.temp.insert(ident.declaration_id, Some(value));
        return Ok(oxc_ast::ast::Statement::new_empty_statement(SPAN, &cx.ast));
    }
    let expr_value = ox_convert_value_to_expression(&cx.ast, value);
    let name = ox_identifier_name(cx.env, lvalue.identifier)?;
    if cx.has_declared(lvalue.identifier) {
        let target = oxc::AssignmentTarget::AssignmentTargetIdentifier(
            oxc_ast::ast::IdentifierReference::boxed(SPAN, ox_str(&cx.ast, &name), &cx.ast),
        );
        let expr = oxc_ast::ast::Expression::new_assignment_expression(
            SPAN,
            oxc::AssignmentOperator::Assign,
            target,
            expr_value,
            &cx.ast,
        );
        Ok(oxc_ast::ast::Statement::new_expression_statement(SPAN, expr, &cx.ast))
    } else {
        let id = oxc_ast::ast::BindingPattern::new_binding_identifier(
            SPAN,
            ox_str(&cx.ast, &name),
            &cx.ast,
        );
        Ok(ox_make_var_decl(cx, oxc::VariableDeclarationKind::Const, id, Some(expr_value)))
    }
}

// =============================================================================
// Instruction value codegen (oxc)
// =============================================================================

fn ox_codegen_instruction_value_to_expression<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr_value: &ReactiveValue<'h>,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let value = ox_codegen_instruction_value(cx, instr_value)?;
    Ok(ox_convert_value_to_expression(&cx.ast, value))
}

fn ox_codegen_instruction_value<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    instr_value: &ReactiveValue<'h>,
) -> Result<OxValue<'a>, CompilerError> {
    match instr_value {
        ReactiveValue::Instruction(iv) => ox_codegen_base_instruction_value(cx, iv),
        ReactiveValue::LogicalExpression { operator, left, right, .. } => {
            let left_expr = ox_codegen_instruction_value_to_expression(cx, left)?;
            let right_expr = ox_codegen_instruction_value_to_expression(cx, right)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_logical_expression(
                SPAN,
                left_expr,
                ox_convert_logical_operator(operator),
                right_expr,
                &cx.ast,
            )))
        }
        ReactiveValue::ConditionalExpression { test, consequent, alternate, .. } => {
            let test_expr = ox_codegen_instruction_value_to_expression(cx, test)?;
            let cons_expr = ox_codegen_instruction_value_to_expression(cx, consequent)?;
            let alt_expr = ox_codegen_instruction_value_to_expression(cx, alternate)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_conditional_expression(
                SPAN, test_expr, cons_expr, alt_expr, &cx.ast,
            )))
        }
        ReactiveValue::SequenceExpression { instructions, value, .. } => {
            let block_items: Vec<ReactiveStatement> =
                instructions.iter().map(|i| ReactiveStatement::Instruction(i.clone())).collect();
            let body = ox_codegen_block_no_reset(cx, &block_items)?;
            let mut expressions: oxc_allocator::Vec<'a, oxc::Expression<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
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
                        expressions.push(oxc_ast::ast::Expression::new_string_literal(
                            SPAN,
                            "TODO handle declaration",
                            None,
                            &cx.ast,
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
                        expressions.push(oxc_ast::ast::Expression::new_string_literal(
                            SPAN,
                            "TODO handle statement",
                            None,
                            &cx.ast,
                        ));
                    }
                }
            }
            let final_expr = ox_codegen_instruction_value_to_expression(cx, value)?;
            if expressions.is_empty() {
                Ok(OxValue::Expression(final_expr))
            } else {
                expressions.push(final_expr);
                Ok(OxValue::Expression(oxc_ast::ast::Expression::new_sequence_expression(
                    SPAN,
                    expressions,
                    &cx.ast,
                )))
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
    cx: &mut OxcContext<'a, '_, '_>,
    expr: oxc::Expression<'a>,
    optional: bool,
) -> Result<OxValue<'a>, CompilerError> {
    let chain_element: oxc::ChainElement<'a> = match expr {
        oxc::Expression::ChainExpression(chain) => {
            // Already a chain; update the optional flag on the head element.
            let chain = chain.unbox();
            match chain.expression {
                oxc::ChainElement::CallExpression(call) => {
                    let mut call = call.unbox();
                    call.optional = optional;
                    oxc::ChainElement::CallExpression(oxc_allocator::ArenaBox::new_in(
                        call, &cx.ast,
                    ))
                }
                oxc::ChainElement::ComputedMemberExpression(m) => {
                    let mut m = m.unbox();
                    m.optional = optional;
                    oxc::ChainElement::ComputedMemberExpression(oxc_allocator::ArenaBox::new_in(
                        m, &cx.ast,
                    ))
                }
                oxc::ChainElement::StaticMemberExpression(m) => {
                    let mut m = m.unbox();
                    m.optional = optional;
                    oxc::ChainElement::StaticMemberExpression(oxc_allocator::ArenaBox::new_in(
                        m, &cx.ast,
                    ))
                }
                other => other,
            }
        }
        oxc::Expression::CallExpression(call) => {
            let mut call = call.unbox();
            call.callee = ox_unwrap_chain(call.callee);
            oxc::ChainElement::CallExpression(oxc_ast::ast::CallExpression::boxed(
                SPAN,
                call.callee,
                call.type_arguments,
                call.arguments,
                optional,
                &cx.ast,
            ))
        }
        oxc::Expression::ComputedMemberExpression(m) => {
            let m = m.unbox();
            oxc::ChainElement::ComputedMemberExpression(
                oxc_ast::ast::ComputedMemberExpression::boxed(
                    SPAN,
                    ox_unwrap_chain(m.object),
                    m.expression,
                    optional,
                    &cx.ast,
                ),
            )
        }
        oxc::Expression::StaticMemberExpression(m) => {
            let m = m.unbox();
            oxc::ChainElement::StaticMemberExpression(oxc_ast::ast::StaticMemberExpression::boxed(
                SPAN,
                ox_unwrap_chain(m.object),
                m.property,
                optional,
                &cx.ast,
            ))
        }
        _ => {
            return Err(invariant_err(
                "Expected optional value to resolve to call or member expression",
                None,
            ));
        }
    };
    Ok(OxValue::Expression(oxc_ast::ast::Expression::new_chain_expression(
        SPAN,
        chain_element,
        &cx.ast,
    )))
}

fn ox_codegen_base_instruction_value<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    iv: &InstructionValue,
) -> Result<OxValue<'a>, CompilerError> {
    match iv {
        InstructionValue::Primitive { value, .. } => {
            Ok(OxValue::Expression(ox_codegen_primitive_value(&cx.ast, value)))
        }
        InstructionValue::BinaryExpression { operator, left, right, .. } => {
            let left_expr = ox_codegen_place_to_expression(cx, left)?;
            let right_expr = ox_codegen_place_to_expression(cx, right)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_binary_expression(
                SPAN,
                left_expr,
                ox_convert_binary_operator(operator),
                right_expr,
                &cx.ast,
            )))
        }
        InstructionValue::UnaryExpression { operator, value, .. } => {
            let arg = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_unary_expression(
                SPAN,
                ox_convert_unary_operator(operator),
                arg,
                &cx.ast,
            )))
        }
        InstructionValue::LoadLocal { place, .. } | InstructionValue::LoadContext { place, .. } => {
            let expr = ox_codegen_place_to_expression(cx, place)?;
            Ok(OxValue::Expression(expr))
        }
        InstructionValue::LoadGlobal { binding, .. } => {
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_identifier(
                SPAN,
                ox_str(&cx.ast, binding.name()),
                &cx.ast,
            )))
        }
        InstructionValue::CallExpression { callee, args, .. } => {
            let callee_expr = ox_codegen_place_to_expression(cx, callee)?;
            let arguments = ox_codegen_arguments(cx, args)?;
            let call_expr = oxc_ast::ast::Expression::new_call_expression(
                SPAN,
                callee_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
                false,
                &cx.ast,
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
            let call_expr = oxc_ast::ast::Expression::new_call_expression(
                SPAN,
                member_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
                false,
                &cx.ast,
            );
            let result = ox_maybe_wrap_hook_call(cx, call_expr, property.identifier)?;
            Ok(OxValue::Expression(result))
        }
        InstructionValue::NewExpression { callee, args, .. } => {
            let callee_expr = ox_codegen_place_to_expression(cx, callee)?;
            let arguments = ox_codegen_arguments(cx, args)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_new_expression(
                SPAN,
                callee_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                arguments,
                &cx.ast,
            )))
        }
        InstructionValue::ArrayExpression { elements, .. } => {
            let mut elems: oxc_allocator::Vec<'a, oxc::ArrayExpressionElement<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
            for el in elements {
                match el {
                    ArrayElement::Place(place) => {
                        let expr = ox_codegen_place_to_expression(cx, place)?;
                        elems.push(oxc::ArrayExpressionElement::from(expr));
                    }
                    ArrayElement::Spread(spread) => {
                        let arg = ox_codegen_place_to_expression(cx, &spread.place)?;
                        elems.push(oxc::ArrayExpressionElement::SpreadElement(
                            oxc_ast::ast::SpreadElement::boxed(SPAN, arg, &cx.ast),
                        ));
                    }
                    ArrayElement::Hole => {
                        elems
                            .push(oxc_ast::ast::ArrayExpressionElement::new_elision(SPAN, &cx.ast));
                    }
                }
            }
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_array_expression(
                SPAN, elems, &cx.ast,
            )))
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
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_assignment_expression(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                val,
                &cx.ast,
            )))
        }
        InstructionValue::PropertyDelete { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let member = ox_property_member(cx, obj, property);
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_unary_expression(
                SPAN,
                oxc::UnaryOperator::Delete,
                oxc::Expression::from(member),
                &cx.ast,
            )))
        }
        InstructionValue::ComputedLoad { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = oxc_ast::ast::MemberExpression::new_computed_member_expression(
                SPAN, obj, prop, false, &cx.ast,
            );
            Ok(OxValue::Expression(oxc::Expression::from(member)))
        }
        InstructionValue::ComputedStore { object, property, value, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = oxc_ast::ast::MemberExpression::new_computed_member_expression(
                SPAN, obj, prop, false, &cx.ast,
            );
            let val = ox_codegen_place_to_expression(cx, value)?;
            let target = oxc::AssignmentTarget::from(oxc::SimpleAssignmentTarget::from(member));
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_assignment_expression(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                val,
                &cx.ast,
            )))
        }
        InstructionValue::ComputedDelete { object, property, .. } => {
            let obj = ox_codegen_place_to_expression(cx, object)?;
            let prop = ox_codegen_place_to_expression(cx, property)?;
            let member = oxc_ast::ast::MemberExpression::new_computed_member_expression(
                SPAN, obj, prop, false, &cx.ast,
            );
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_unary_expression(
                SPAN,
                oxc::UnaryOperator::Delete,
                oxc::Expression::from(member),
                &cx.ast,
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
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_reg_exp_literal(
                SPAN, regex, None, &cx.ast,
            )))
        }
        InstructionValue::MetaProperty { meta, property, .. } => {
            let meta_ident =
                oxc_ast::ast::IdentifierName::new(SPAN, ox_str(&cx.ast, meta), &cx.ast);
            let prop_ident =
                oxc_ast::ast::IdentifierName::new(SPAN, ox_str(&cx.ast, property), &cx.ast);
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_meta_property(
                SPAN, meta_ident, prop_ident, &cx.ast,
            )))
        }
        InstructionValue::Await { value, .. } => {
            let arg = ox_codegen_place_to_expression(cx, value)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_await_expression(
                SPAN, arg, &cx.ast,
            )))
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
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_update_expression(
                SPAN,
                ox_convert_update_operator(operation),
                false,
                target,
                &cx.ast,
            )))
        }
        InstructionValue::PrefixUpdate { operation, lvalue, .. } => {
            let arg = ox_codegen_place_to_expression(cx, lvalue)?;
            let target = ox_expression_to_simple_assignment_target(cx, arg)?;
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_update_expression(
                SPAN,
                ox_convert_update_operator(operation),
                true,
                target,
                &cx.ast,
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
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_assignment_expression(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                rhs,
                &cx.ast,
            )))
        }
        InstructionValue::StoreGlobal { name, value, .. } => {
            let rhs = ox_codegen_place_to_expression(cx, value)?;
            let target = oxc::AssignmentTarget::AssignmentTargetIdentifier(
                oxc_ast::ast::IdentifierReference::boxed(SPAN, ox_str(&cx.ast, name), &cx.ast),
            );
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_assignment_expression(
                SPAN,
                oxc::AssignmentOperator::Assign,
                target,
                rhs,
                &cx.ast,
            )))
        }
        InstructionValue::FunctionExpression {
            name, name_hint, lowered_func, expr_type, ..
        } => ox_codegen_function_expression(cx, name, name_hint, lowered_func, expr_type),
        InstructionValue::TaggedTemplateExpression { tag, quasis, subexprs, .. } => {
            let tag_expr = ox_codegen_place_to_expression(cx, tag)?;
            let mut exprs: oxc_allocator::Vec<'a, oxc::Expression<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
            for p in subexprs {
                exprs.push(ox_codegen_place_to_expression(cx, p)?);
            }
            let quasi = ox_template_literal(cx, quasis, exprs);
            Ok(OxValue::Expression(oxc_ast::ast::Expression::new_tagged_template_expression(
                SPAN,
                tag_expr,
                None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
                quasi,
                &cx.ast,
            )))
        }
        InstructionValue::TemplateLiteral { subexprs, quasis, .. } => {
            let mut exprs: oxc_allocator::Vec<'a, oxc::Expression<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
            for p in subexprs {
                exprs.push(ox_codegen_place_to_expression(cx, p)?);
            }
            let template = ox_template_literal(cx, quasis, exprs);
            Ok(OxValue::Expression(oxc::Expression::TemplateLiteral(
                oxc_allocator::ArenaBox::new_in(template, &cx.ast),
            )))
        }
        InstructionValue::TypeCastExpression {
            value,
            type_annotation_kind,
            type_annotation,
            ..
        } => {
            let expr = ox_codegen_place_to_expression(cx, value)?;
            // Re-emit the stored TS type into the output allocator (a `clone_in`, with
            // any binding renames applied to identifier references inside the type) and
            // re-wrap the inner expression, matching the baseline output.
            let wrapped = match (type_annotation_kind.as_deref(), type_annotation) {
                (Some("satisfies"), Some(ta)) => {
                    oxc_ast::ast::Expression::new_ts_satisfies_expression(
                        SPAN,
                        expr,
                        ox_reemit_ts_type(cx, ta),
                        &cx.ast,
                    )
                }
                (Some("as"), Some(ta)) => oxc_ast::ast::Expression::new_ts_as_expression(
                    SPAN,
                    expr,
                    ox_reemit_ts_type(cx, ta),
                    &cx.ast,
                ),
                _ => expr,
            };
            Ok(OxValue::Expression(wrapped))
        }
        InstructionValue::JSXText { value, .. } => Ok(OxValue::JsxText(
            oxc_ast::ast::JSXText::boxed(SPAN, ox_str(&cx.ast, value), None, &cx.ast),
        )),
        InstructionValue::JsxExpression { tag, props, children, .. } => {
            ox_codegen_jsx_expression(cx, tag, props, children)
        }
        InstructionValue::JsxFragment { children, .. } => {
            let mut child_nodes: oxc_allocator::Vec<'a, oxc::JSXChild<'a>> =
                oxc_allocator::ArenaVec::new_in(&cx.ast);
            for child in children {
                child_nodes.push(ox_codegen_jsx_element(cx, child)?);
            }
            let opening = oxc_ast::ast::JSXOpeningFragment::new(SPAN, &cx.ast);
            let closing = oxc_ast::ast::JSXClosingFragment::new(SPAN, &cx.ast);
            let fragment =
                oxc_ast::ast::JSXFragment::new(SPAN, opening, child_nodes, closing, &cx.ast);
            Ok(OxValue::Expression(oxc::Expression::JSXFragment(oxc_allocator::ArenaBox::new_in(
                fragment, &cx.ast,
            ))))
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
    cx: &OxcContext<'a, '_, '_>,
    object: oxc::Expression<'a>,
    property: &PropertyLiteral,
) -> oxc::MemberExpression<'a> {
    match property {
        PropertyLiteral::String(s) => oxc_ast::ast::MemberExpression::new_static_member_expression(
            SPAN,
            object,
            oxc_ast::ast::IdentifierName::new(SPAN, ox_str(&cx.ast, s), &cx.ast),
            false,
            &cx.ast,
        ),
        PropertyLiteral::Number(n) => {
            oxc_ast::ast::MemberExpression::new_computed_member_expression(
                SPAN,
                object,
                ox_number(&cx.ast, n.value()),
                false,
                &cx.ast,
            )
        }
    }
}

fn ox_template_literal<'a>(
    cx: &OxcContext<'a, '_, '_>,
    quasis: &[crate::react_compiler_hir::TemplateQuasi],
    expressions: oxc_allocator::Vec<'a, oxc::Expression<'a>>,
) -> oxc::TemplateLiteral<'a> {
    let mut quasi_vec: oxc_allocator::Vec<'a, oxc::TemplateElement<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    let len = quasis.len();
    for (i, q) in quasis.iter().enumerate() {
        let value = oxc::TemplateElementValue {
            raw: ox_str(&cx.ast, &q.raw).into(),
            cooked: q.cooked.as_deref().map(|c| ox_str(&cx.ast, c).into()),
        };
        quasi_vec.push(oxc_ast::ast::TemplateElement::new(SPAN, value, i == len - 1, &cx.ast));
    }
    oxc_ast::ast::TemplateLiteral::new(SPAN, quasi_vec, expressions, &cx.ast)
}

fn ox_codegen_arguments<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    args: &[PlaceOrSpread],
) -> Result<oxc_allocator::Vec<'a, oxc::Argument<'a>>, CompilerError> {
    let mut out: oxc_allocator::Vec<'a, oxc::Argument<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    for arg in args {
        out.push(ox_codegen_argument(cx, arg)?);
    }
    Ok(out)
}

fn ox_codegen_argument<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    arg: &PlaceOrSpread,
) -> Result<oxc::Argument<'a>, CompilerError> {
    match arg {
        PlaceOrSpread::Place(place) => {
            Ok(oxc::Argument::from(ox_codegen_place_to_expression(cx, place)?))
        }
        PlaceOrSpread::Spread(spread) => {
            let expr = ox_codegen_place_to_expression(cx, &spread.place)?;
            Ok(oxc::Argument::SpreadElement(oxc_ast::ast::SpreadElement::boxed(
                SPAN, expr, &cx.ast,
            )))
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
    cx: &mut OxcContext<'a, '_, '_>,
    place: &Place,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    Ok(ox_convert_value_to_expression(&cx.ast, value))
}

fn ox_codegen_place<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
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
    Ok(OxValue::Expression(oxc_ast::ast::Expression::new_identifier(
        SPAN,
        ox_str(&cx.ast, &name),
        &cx.ast,
    )))
}

fn ox_codegen_lvalue<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
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
    cx: &mut OxcContext<'a, '_, '_>,
    pattern: &ArrayPattern,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let mut elements: oxc_allocator::Vec<'a, Option<oxc::BindingPattern<'a>>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    let mut rest: Option<oxc::BindingRestElement<'a>> = None;
    for item in &pattern.items {
        match item {
            crate::react_compiler_hir::ArrayPatternElement::Place(place) => {
                elements.push(Some(ox_binding_for_identifier(cx, place.identifier)?));
            }
            crate::react_compiler_hir::ArrayPatternElement::Spread(spread) => {
                let inner = ox_binding_for_identifier(cx, spread.place.identifier)?;
                rest = Some(oxc_ast::ast::BindingRestElement::new(SPAN, inner, &cx.ast));
            }
            crate::react_compiler_hir::ArrayPatternElement::Hole => {
                elements.push(None);
            }
        }
    }
    Ok(oxc_ast::ast::BindingPattern::new_array_pattern(SPAN, elements, rest, &cx.ast))
}

fn ox_codegen_object_pattern<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    pattern: &ObjectPattern,
) -> Result<oxc::BindingPattern<'a>, CompilerError> {
    let mut properties: oxc_allocator::Vec<'a, oxc::BindingProperty<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
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
                properties.push(oxc_ast::ast::BindingProperty::new(
                    SPAN, key, value, shorthand, computed, &cx.ast,
                ));
            }
            ObjectPropertyOrSpread::Spread(spread) => {
                let inner = ox_binding_for_identifier(cx, spread.place.identifier)?;
                rest = Some(oxc_ast::ast::BindingRestElement::new(SPAN, inner, &cx.ast));
            }
        }
    }
    Ok(oxc_ast::ast::BindingPattern::new_object_pattern(SPAN, properties, rest, &cx.ast))
}

/// Build an object pattern key, returning `(key, computed)`.
fn ox_codegen_object_property_key<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    key: &ObjectPropertyKey,
) -> Result<(oxc::PropertyKey<'a>, bool), CompilerError> {
    match key {
        ObjectPropertyKey::String { name } => Ok((
            oxc::PropertyKey::from(oxc_ast::ast::Expression::new_string_literal(
                SPAN,
                ox_str(&cx.ast, name),
                None,
                &cx.ast,
            )),
            false,
        )),
        ObjectPropertyKey::Identifier { name } => Ok((
            oxc_ast::ast::PropertyKey::new_static_identifier(SPAN, ox_str(&cx.ast, name), &cx.ast),
            false,
        )),
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
    cx: &mut OxcContext<'a, '_, '_>,
    dep: &crate::react_compiler_hir::ReactiveScopeDependency,
) -> Result<oxc::Expression<'a>, CompilerError> {
    let name = ox_identifier_name(cx.env, dep.identifier)?;
    let mut object =
        oxc_ast::ast::Expression::new_identifier(SPAN, ox_str(&cx.ast, &name), &cx.ast);
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
                    oxc::Expression::StaticMemberExpression(
                        oxc_ast::ast::StaticMemberExpression::boxed(
                            SPAN,
                            m.object,
                            m.property,
                            path_entry.optional,
                            &cx.ast,
                        ),
                    )
                }
                oxc::MemberExpression::ComputedMemberExpression(m) => {
                    let m = m.unbox();
                    oxc::Expression::ComputedMemberExpression(
                        oxc_ast::ast::ComputedMemberExpression::boxed(
                            SPAN,
                            m.object,
                            m.expression,
                            path_entry.optional,
                            &cx.ast,
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
            object = oxc_ast::ast::Expression::new_chain_expression(SPAN, chain, &cx.ast);
        }
    }
    Ok(object)
}

/// Convert a `BindingPattern` (from `ox_codegen_lvalue`) into an `AssignmentTarget`
/// for reassignment / `StoreLocal` emission.
fn ox_binding_pattern_to_assignment_target<'a>(
    cx: &OxcContext<'a, '_, '_>,
    pattern: oxc::BindingPattern<'a>,
) -> Result<oxc::AssignmentTarget<'a>, CompilerError> {
    match pattern {
        oxc::BindingPattern::BindingIdentifier(id) => {
            let id = id.unbox();
            Ok(oxc::AssignmentTarget::AssignmentTargetIdentifier(
                oxc_ast::ast::IdentifierReference::boxed(SPAN, id.name, &cx.ast),
            ))
        }
        _ => {
            Err(invariant_err("Destructuring reassignment targets are not yet ported to oxc", None))
        }
    }
}

/// Convert an expression to a `SimpleAssignmentTarget` for update expressions.
fn ox_expression_to_simple_assignment_target<'a>(
    cx: &OxcContext<'a, '_, '_>,
    expr: oxc::Expression<'a>,
) -> Result<oxc::SimpleAssignmentTarget<'a>, CompilerError> {
    match expr {
        oxc::Expression::Identifier(id) => {
            let id = id.unbox();
            Ok(oxc::SimpleAssignmentTarget::AssignmentTargetIdentifier(
                oxc_ast::ast::IdentifierReference::boxed(SPAN, id.name, &cx.ast),
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
    cx: &mut OxcContext<'a, '_, '_>,
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
                    let stmts = oxc_allocator::ArenaVec::from_value_in(
                        oxc_ast::ast::Statement::new_expression_statement(SPAN, arg, &cx.ast),
                        &cx.ast,
                    );
                    let body = oxc_ast::ast::FunctionBody::boxed(
                        SPAN,
                        oxc_allocator::ArenaVec::new_in(&cx.ast),
                        stmts,
                        &cx.ast,
                    );
                    ox_build_arrow(cx, fn_result.params, body, fn_result.is_async, true)
                }
                None => {
                    ox_build_arrow(cx, fn_result.params, fn_result.body, fn_result.is_async, false)
                }
            }
        }
        _ => {
            let id = name
                .as_ref()
                .map(|n| oxc_ast::ast::BindingIdentifier::new(SPAN, ox_str(&cx.ast, n), &cx.ast));
            let func = oxc_ast::ast::Function::new(
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
                &cx.ast,
            );
            oxc::Expression::FunctionExpression(oxc_allocator::ArenaBox::new_in(func, &cx.ast))
        }
    };

    // enableNameAnonymousFunctions: `({ "<hint>": <fn> })["<hint>"]`
    if cx.env.config.enable_name_anonymous_functions && name.is_none() && name_hint.is_some() {
        let hint = name_hint.as_ref().unwrap().clone();
        let key = oxc::PropertyKey::from(oxc_ast::ast::Expression::new_string_literal(
            SPAN,
            ox_str(&cx.ast, &hint),
            None,
            &cx.ast,
        ));
        let prop = oxc_ast::ast::ObjectProperty::new(
            SPAN,
            oxc::PropertyKind::Init,
            key,
            value,
            false,
            false,
            false,
            &cx.ast,
        );
        let props = oxc_allocator::ArenaVec::from_value_in(
            oxc::ObjectPropertyKind::ObjectProperty(oxc_allocator::ArenaBox::new_in(prop, &cx.ast)),
            &cx.ast,
        );
        let object = oxc_ast::ast::Expression::new_object_expression(SPAN, props, &cx.ast);
        let member = oxc_ast::ast::MemberExpression::new_computed_member_expression(
            SPAN,
            object,
            oxc_ast::ast::Expression::new_string_literal(
                SPAN,
                ox_str(&cx.ast, &hint),
                None,
                &cx.ast,
            ),
            false,
            &cx.ast,
        );
        return Ok(OxValue::Expression(oxc::Expression::from(member)));
    }

    Ok(OxValue::Expression(value))
}

fn ox_build_arrow<'a>(
    cx: &OxcContext<'a, '_, '_>,
    params: oxc_allocator::Box<'a, oxc::FormalParameters<'a>>,
    body: oxc_allocator::Box<'a, oxc::FunctionBody<'a>>,
    is_async: bool,
    expression: bool,
) -> oxc::Expression<'a> {
    oxc_ast::ast::Expression::new_arrow_function_expression(
        SPAN,
        expression,
        is_async,
        None::<oxc_allocator::Box<oxc::TSTypeParameterDeclaration>>,
        params,
        None::<oxc_allocator::Box<oxc::TSTypeAnnotation>>,
        body,
        &cx.ast,
    )
}

/// Run the inner-function codegen with a fresh context (mirrors the Babel reference's
/// `Context::new` + `codegen_reactive_function` for function/object-method expressions).
fn ox_codegen_inner_function<'a, 'h>(
    cx: &mut OxcContext<'a, '_, 'h>,
    reactive_fn: &ReactiveFunction<'h>,
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
    cx: &mut OxcContext<'a, '_, '_>,
    properties: &[ObjectPropertyOrSpread],
) -> Result<OxValue<'a>, CompilerError> {
    let mut props: oxc_allocator::Vec<'a, oxc::ObjectPropertyKind<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
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
                        let p = oxc_ast::ast::ObjectProperty::new(
                            SPAN,
                            oxc::PropertyKind::Init,
                            key,
                            value,
                            false,
                            shorthand,
                            key_computed,
                            &cx.ast,
                        );
                        props.push(oxc::ObjectPropertyKind::ObjectProperty(
                            oxc_allocator::ArenaBox::new_in(p, &cx.ast),
                        ));
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
                        let method = oxc_ast::ast::Function::new(
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
                            &cx.ast,
                        );
                        let func_expr = oxc::Expression::FunctionExpression(
                            oxc_allocator::ArenaBox::new_in(method, &cx.ast),
                        );
                        let p = oxc_ast::ast::ObjectProperty::new(
                            SPAN,
                            oxc::PropertyKind::Init,
                            key,
                            func_expr,
                            true,
                            false,
                            key_computed,
                            &cx.ast,
                        );
                        props.push(oxc::ObjectPropertyKind::ObjectProperty(
                            oxc_allocator::ArenaBox::new_in(p, &cx.ast),
                        ));
                    }
                }
            }
            ObjectPropertyOrSpread::Spread(spread) => {
                let arg = ox_codegen_place_to_expression(cx, &spread.place)?;
                let spread_el = oxc_ast::ast::SpreadElement::new(SPAN, arg, &cx.ast);
                props.push(oxc::ObjectPropertyKind::SpreadProperty(
                    oxc_allocator::ArenaBox::new_in(spread_el, &cx.ast),
                ));
            }
        }
    }
    Ok(OxValue::Expression(oxc_ast::ast::Expression::new_object_expression(SPAN, props, &cx.ast)))
}

// =============================================================================
// JSX codegen (oxc)
// =============================================================================

fn ox_codegen_jsx_expression<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    tag: &JsxTag,
    props: &[JsxAttribute],
    children: &Option<Vec<Place>>,
) -> Result<OxValue<'a>, CompilerError> {
    let mut attributes: oxc_allocator::Vec<'a, oxc::JSXAttributeItem<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
    for attr in props {
        attributes.push(ox_codegen_jsx_attribute(cx, attr)?);
    }

    let (tag_value, is_fbt_tag) = match tag {
        JsxTag::Place(place) => (ox_codegen_place_to_expression(cx, place)?, false),
        JsxTag::Builtin(builtin) => {
            let is_fbt = SINGLE_CHILD_FBT_TAGS.contains(&builtin.name.as_str());
            (
                oxc_ast::ast::Expression::new_string_literal(
                    SPAN,
                    ox_str(&cx.ast, &builtin.name),
                    None,
                    &cx.ast,
                ),
                is_fbt,
            )
        }
    };

    let opening_name = ox_expression_to_jsx_tag(cx, &tag_value)?;

    let mut child_nodes: oxc_allocator::Vec<'a, oxc::JSXChild<'a>> =
        oxc_allocator::ArenaVec::new_in(&cx.ast);
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
    let opening = oxc_ast::ast::JSXOpeningElement::new(
        SPAN,
        opening_name,
        None::<oxc_allocator::Box<oxc::TSTypeParameterInstantiation>>,
        attributes,
        &cx.ast,
    );
    let closing = if is_self_closing {
        None
    } else {
        let closing_name = ox_expression_to_jsx_tag(cx, &tag_value)?;
        Some(oxc_ast::ast::JSXClosingElement::new(SPAN, closing_name, &cx.ast))
    };
    let element = oxc_ast::ast::JSXElement::new(SPAN, opening, child_nodes, closing, &cx.ast);
    Ok(OxValue::Expression(oxc::Expression::JSXElement(oxc_allocator::ArenaBox::new_in(
        element, &cx.ast,
    ))))
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
    cx: &mut OxcContext<'a, '_, '_>,
    attr: &JsxAttribute,
) -> Result<oxc::JSXAttributeItem<'a>, CompilerError> {
    match attr {
        JsxAttribute::Attribute { name, place } => {
            let prop_name = if name.contains(':') {
                let parts: Vec<&str> = name.splitn(2, ':').collect();
                let namespace =
                    oxc_ast::ast::JSXIdentifier::new(SPAN, ox_str(&cx.ast, parts[0]), &cx.ast);
                let local =
                    oxc_ast::ast::JSXIdentifier::new(SPAN, ox_str(&cx.ast, parts[1]), &cx.ast);
                oxc_ast::ast::JSXAttributeName::new_namespaced_name(SPAN, namespace, local, &cx.ast)
            } else {
                oxc_ast::ast::JSXAttributeName::new_identifier(SPAN, ox_str(&cx.ast, name), &cx.ast)
            };

            let is_fbt_operand = cx.fbt_operands.contains(&place.identifier);
            let inner_value = ox_codegen_place_to_expression(cx, place)?;
            let attr_value = match inner_value {
                oxc::Expression::StringLiteral(ref s)
                    if !ox_string_requires_expr_container(s.value.as_str()) || is_fbt_operand =>
                {
                    let value = s.value;
                    Some(oxc_ast::ast::JSXAttributeValue::new_string_literal(
                        SPAN, value, None, &cx.ast,
                    ))
                }
                _ => {
                    let expr = oxc::JSXExpression::from(inner_value);
                    Some(oxc_ast::ast::JSXAttributeValue::new_expression_container(
                        SPAN, expr, &cx.ast,
                    ))
                }
            };
            Ok(oxc_ast::ast::JSXAttributeItem::new_attribute(SPAN, prop_name, attr_value, &cx.ast))
        }
        JsxAttribute::SpreadAttribute { argument } => {
            let expr = ox_codegen_place_to_expression(cx, argument)?;
            Ok(oxc_ast::ast::JSXAttributeItem::new_spread_attribute(SPAN, expr, &cx.ast))
        }
    }
}

fn ox_codegen_jsx_element<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    place: &Place,
) -> Result<oxc::JSXChild<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    match value {
        OxValue::JsxText(text) => {
            let raw = text.value.as_str();
            if raw.contains(JSX_TEXT_CHILD_REQUIRES_EXPR_CONTAINER_PATTERN) {
                let lit = oxc_ast::ast::Expression::new_string_literal(
                    SPAN,
                    ox_str(&cx.ast, raw),
                    None,
                    &cx.ast,
                );
                Ok(oxc_ast::ast::JSXChild::new_expression_container(
                    SPAN,
                    oxc::JSXExpression::from(lit),
                    &cx.ast,
                ))
            } else {
                let encoded = ox_encode_jsx_text(raw);
                Ok(oxc_ast::ast::JSXChild::new_text(SPAN, ox_str(&cx.ast, &encoded), None, &cx.ast))
            }
        }
        OxValue::Expression(oxc::Expression::JSXElement(elem)) => {
            let elem = elem.unbox();
            Ok(oxc_ast::ast::JSXChild::new_element(
                SPAN,
                elem.opening_element,
                elem.children,
                elem.closing_element,
                &cx.ast,
            ))
        }
        OxValue::Expression(oxc::Expression::JSXFragment(frag)) => {
            let frag = frag.unbox();
            Ok(oxc_ast::ast::JSXChild::new_fragment(
                SPAN,
                frag.opening_fragment,
                frag.children,
                frag.closing_fragment,
                &cx.ast,
            ))
        }
        OxValue::Expression(expr) => Ok(oxc_ast::ast::JSXChild::new_expression_container(
            SPAN,
            oxc::JSXExpression::from(expr),
            &cx.ast,
        )),
    }
}

fn ox_codegen_jsx_fbt_child_element<'a>(
    cx: &mut OxcContext<'a, '_, '_>,
    place: &Place,
) -> Result<oxc::JSXChild<'a>, CompilerError> {
    let value = ox_codegen_place(cx, place)?;
    match value {
        OxValue::JsxText(text) => {
            let encoded = ox_encode_jsx_text(text.value.as_str());
            Ok(oxc_ast::ast::JSXChild::new_text(SPAN, ox_str(&cx.ast, &encoded), None, &cx.ast))
        }
        OxValue::Expression(oxc::Expression::JSXElement(elem)) => {
            let elem = elem.unbox();
            Ok(oxc_ast::ast::JSXChild::new_element(
                SPAN,
                elem.opening_element,
                elem.children,
                elem.closing_element,
                &cx.ast,
            ))
        }
        OxValue::Expression(expr) => Ok(oxc_ast::ast::JSXChild::new_expression_container(
            SPAN,
            oxc::JSXExpression::from(expr),
            &cx.ast,
        )),
    }
}

/// Build a `JSXElementName` from a tag expression following the TS compiler's
/// identifier-reference rule (uppercase / contains-`.` names become references).
fn ox_expression_to_jsx_tag<'a>(
    cx: &OxcContext<'a, '_, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<oxc::JSXElementName<'a>, CompilerError> {
    match expr {
        oxc::Expression::Identifier(ident) => Ok(ox_jsx_element_name_from_ident(cx, &ident.name)),
        oxc::Expression::StaticMemberExpression(_)
        | oxc::Expression::ComputedMemberExpression(_) => {
            let member = ox_convert_member_expression_to_jsx(cx, expr)?;
            Ok(oxc_ast::ast::JSXElementName::new_member_expression(
                SPAN, member.0, member.1, &cx.ast,
            ))
        }
        oxc::Expression::StringLiteral(s) => {
            let tag_text = s.value.as_str();
            if tag_text.contains(':') {
                let parts: Vec<&str> = tag_text.splitn(2, ':').collect();
                let namespace =
                    oxc_ast::ast::JSXIdentifier::new(SPAN, ox_str(&cx.ast, parts[0]), &cx.ast);
                let name =
                    oxc_ast::ast::JSXIdentifier::new(SPAN, ox_str(&cx.ast, parts[1]), &cx.ast);
                Ok(oxc_ast::ast::JSXElementName::new_namespaced_name(
                    SPAN, namespace, name, &cx.ast,
                ))
            } else {
                Ok(ox_jsx_element_name_from_ident(cx, tag_text))
            }
        }
        _ => Err(invariant_err("Expected JSX tag to be an identifier or string", None)),
    }
}

fn ox_jsx_element_name_from_ident<'a>(
    cx: &OxcContext<'a, '_, '_>,
    name: &str,
) -> oxc::JSXElementName<'a> {
    let first_char = name.chars().next().unwrap_or('a');
    if first_char.is_uppercase() || name.contains('.') {
        oxc_ast::ast::JSXElementName::new_identifier_reference(SPAN, ox_str(&cx.ast, name), &cx.ast)
    } else {
        oxc_ast::ast::JSXElementName::new_identifier(SPAN, ox_str(&cx.ast, name), &cx.ast)
    }
}

/// Convert an oxc member expression into a JSX member expression's
/// `(object, property)` pair.
fn ox_convert_member_expression_to_jsx<'a>(
    cx: &OxcContext<'a, '_, '_>,
    expr: &oxc::Expression<'a>,
) -> Result<(oxc::JSXMemberExpressionObject<'a>, oxc::JSXIdentifier<'a>), CompilerError> {
    let oxc::Expression::StaticMemberExpression(me) = expr else {
        return Err(invariant_err("Expected JSX member expression property to be a string", None));
    };
    let property =
        oxc_ast::ast::JSXIdentifier::new(SPAN, ox_str(&cx.ast, me.property.name.as_str()), &cx.ast);
    let object = match &me.object {
        oxc::Expression::Identifier(ident) => {
            oxc_ast::ast::JSXMemberExpressionObject::new_identifier_reference(
                SPAN,
                ox_str(&cx.ast, &ident.name),
                &cx.ast,
            )
        }
        oxc::Expression::StaticMemberExpression(_) => {
            let inner = ox_convert_member_expression_to_jsx(cx, &me.object)?;
            oxc_ast::ast::JSXMemberExpressionObject::new_member_expression(
                SPAN, inner.0, inner.1, &cx.ast,
            )
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
    cx: &OxcContext<'a, '_, '_>,
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
                oxc_ast::ast::Expression::new_identifier(SPAN, "NaN", ast)
            } else if f.is_infinite() {
                if f > 0.0 {
                    oxc_ast::ast::Expression::new_identifier(SPAN, "Infinity", ast)
                } else {
                    oxc_ast::ast::Expression::new_unary_expression(
                        SPAN,
                        oxc::UnaryOperator::UnaryNegation,
                        oxc_ast::ast::Expression::new_identifier(SPAN, "Infinity", ast),
                        ast,
                    )
                }
            } else if f < 0.0 {
                oxc_ast::ast::Expression::new_unary_expression(
                    SPAN,
                    oxc::UnaryOperator::UnaryNegation,
                    ox_number(ast, -f),
                    ast,
                )
            } else {
                ox_number(ast, f)
            }
        }
        PrimitiveValue::Boolean(b) => oxc_ast::ast::Expression::new_boolean_literal(SPAN, *b, ast),
        PrimitiveValue::String(s) => oxc_ast::ast::Expression::new_string_literal(
            SPAN,
            ox_str(ast, &s.to_string_lossy()),
            None,
            ast,
        ),
        PrimitiveValue::Null => oxc_ast::ast::Expression::new_null_literal(SPAN, ast),
        PrimitiveValue::Undefined => {
            oxc_ast::ast::Expression::new_identifier(SPAN, "undefined", ast)
        }
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
struct CountMemoBlockVisitor<'a, 'e> {
    env: &'e Environment<'a>,
}

struct CountMemoBlockState {
    memo_blocks: u32,
    memo_values: u32,
    pruned_memo_blocks: u32,
    pruned_memo_values: u32,
}

impl<'a, 'e> ReactiveFunctionVisitor<'a> for CountMemoBlockVisitor<'a, 'e> {
    type State = CountMemoBlockState;

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    fn visit_scope(&self, scope_block: &ReactiveScopeBlock<'a>, state: &mut CountMemoBlockState) {
        state.memo_blocks += 1;
        let scope = &self.env.scopes[scope_block.scope.0 as usize];
        state.memo_values += scope.declarations.len() as u32;
        self.traverse_scope(scope_block, state);
    }

    fn visit_pruned_scope(
        &self,
        scope_block: &PrunedReactiveScopeBlock<'a>,
        state: &mut CountMemoBlockState,
    ) {
        state.pruned_memo_blocks += 1;
        let scope = &self.env.scopes[scope_block.scope.0 as usize];
        state.pruned_memo_values += scope.declarations.len() as u32;
        self.traverse_pruned_scope(scope_block, state);
    }
}

fn count_memo_blocks<'a>(
    func: &ReactiveFunction<'a>,
    env: &Environment<'a>,
) -> (u32, u32, u32, u32) {
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

fn get_instruction_value<'x, 'a>(
    reactive_value: &'x ReactiveValue<'a>,
) -> Result<&'x InstructionValue<'a>, CompilerError> {
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
