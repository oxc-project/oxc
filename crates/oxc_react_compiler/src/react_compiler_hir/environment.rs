use std::mem::take;

use cow_utils::CowUtils;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, GetAllocator};
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_index::IndexVec;
use oxc_span::Span;
use oxc_str::{Ident, IdentHashMap, IdentHashSet, format_ident};
use oxc_syntax::reference::ReferenceId;
use oxc_syntax::symbol::SymbolId;

use crate::diagnostics::ErrorCategory;

use crate::react_compiler_hir::default_module_type_provider::default_module_type_provider;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
use crate::react_compiler_hir::globals;
use crate::react_compiler_hir::globals::Global;
use crate::react_compiler_hir::globals::GlobalRegistry;
use crate::react_compiler_hir::object_shape::BUILT_IN_MIXED_READONLY_ID;
use crate::react_compiler_hir::object_shape::FunctionSignature;
use crate::react_compiler_hir::object_shape::HookKind;
use crate::react_compiler_hir::object_shape::HookSignatureBuilder;
use crate::react_compiler_hir::object_shape::ShapeRegistry;
use crate::react_compiler_hir::object_shape::add_hook;
use crate::react_compiler_hir::object_shape::default_mutating_hook;
use crate::react_compiler_hir::object_shape::default_nonmutating_hook;
use crate::react_compiler_hir::*;

/// Output mode for the compiler, mirrored from the entrypoint's CompilerOutputMode.
/// Stored on Environment so pipeline passes can access it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Ssr,
    Client,
    Lint,
}

pub struct Environment<'a> {
    // Arena allocator for the compilation unit. HIR strings (identifier names,
    // shape ids, property keys) live here so they can be borrowed from the
    // source AST at lowering and emitted into the output AST without copies.
    pub allocator: &'a Allocator,

    // Counters
    pub next_block_id_counter: u32,
    pub next_scope_id_counter: u32,
    next_mutable_range_id_counter: u32,

    // Arenas (use direct field access for sliced borrows)
    pub identifiers: IndexVec<IdentifierId, Identifier<'a>>,
    pub types: IndexVec<TypeId, Type<'a>>,
    pub scopes: IndexVec<ScopeId, ReactiveScope<'a>>,
    pub functions: IndexVec<FunctionId, HirFunction<'a>>,

    // Error accumulation
    pub errors: Diagnostics,

    // Set during lowering when the function uses syntax the compiler can't handle
    // yet (currently `using`/`await using`, whose disposal semantics aren't
    // preserved). Signals the pipeline to skip compiling this function silently —
    // no diagnostic — while other functions in the file still compile.
    pub skip_compilation: bool,

    // Function type classification (Component, Hook, Other)
    pub fn_type: ReactFunctionType,

    // Output mode (Client, Ssr, Lint)
    pub output_mode: OutputMode,

    // Pre-resolved import local names for instrumentation/hook guards/memo cache.
    // Set by the program-level code before compilation.
    pub instrument_fn_name: Option<Ident<'a>>,
    pub instrument_gating_name: Option<Ident<'a>>,
    pub hook_guard_name: Option<Ident<'a>>,
    pub memo_cache_name: Option<Ident<'a>>,

    // Renames from lowering (collision suffixes like `name_0`), recorded per
    // resolved reference so codegen can rename identifiers inside preserved TS
    // type annotations by reference identity.
    pub renames: FxHashMap<ReferenceId, Ident<'a>>,

    // Hoisted identifiers: tracks which bindings have already been hoisted
    // via DeclareContext to avoid duplicate hoisting.
    hoisted_identifiers: FxHashSet<SymbolId>,

    // Config flags for validation passes (kept for backwards compat with existing pipeline code)
    pub validate_preserve_existing_memoization_guarantees: bool,
    pub validate_no_set_state_in_render: bool,
    pub enable_preserve_existing_memoization_guarantees: bool,

    // Type system registries
    globals: GlobalRegistry<'a>,
    pub shapes: ShapeRegistry<'a>,
    module_types: IdentHashMap<'a, Option<Global<'a>>>,
    module_type_errors: IdentHashMap<'a, Vec<String>>,

    // Environment configuration (feature flags, custom hooks, etc.)
    pub config: EnvironmentConfig,

    // Cached default hook types (lazily initialized)
    default_nonmutating_hook: Option<Global<'a>>,
    default_mutating_hook: Option<Global<'a>>,

    // Outlined functions: functions extracted from the component during outlining passes
    outlined_functions: Vec<OutlinedFunctionEntry<'a>>,

    // Known names for collision-aware UID generation. Lazily populated from
    // identifiers on first use, then updated with each generated name.
    // Matches Babel's generateUid behavior of checking hasBinding/hasReference.
    uid_known_names: Option<IdentHashSet<'a>>,
}

/// An outlined function entry, stored on Environment during compilation.
/// Corresponds to TS `{ fn: HIRFunction, type: ReactFunctionType | null }`.
#[derive(Debug, Clone)]
pub struct OutlinedFunctionEntry<'a> {
    pub func: HirFunction<'a>,
    pub fn_type: Option<ReactFunctionType>,
}

impl<'a> GetAllocator<'a> for Environment<'a> {
    fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}

impl<'a> Environment<'a> {
    /// Create a new Environment with the given configuration.
    ///
    /// Initializes the shape and global registries, registers custom hooks,
    /// and sets up the module type cache.
    pub fn with_config(allocator: &'a Allocator, config: EnvironmentConfig) -> Self {
        let mut shapes = ShapeRegistry::with_base(globals::base_shapes(), allocator);
        let mut global_registry = GlobalRegistry::with_base(globals::base_globals());

        // Register custom hooks from config
        for (hook_name, hook) in &config.custom_hooks {
            // Don't overwrite existing globals (matches TS invariant)
            if global_registry.contains_key(hook_name) {
                continue;
            }
            let return_type = if hook.transitive_mixed_data {
                Type::Object { shape_id: Some(BUILT_IN_MIXED_READONLY_ID) }
            } else {
                Type::Poly
            };
            let hook_type = add_hook(
                &mut shapes,
                HookSignatureBuilder {
                    rest_param: Some(hook.effect_kind),
                    return_type,
                    return_value_kind: hook.value_kind,
                    hook_kind: HookKind::Custom,
                    no_alias: hook.no_alias,
                    ..Default::default()
                },
                None,
            );
            global_registry.insert(Ident::from_str_in(hook_name, &allocator), hook_type);
        }

        // Register reanimated module type when enabled
        let mut module_types: IdentHashMap<'a, Option<Global<'a>>> = IdentHashMap::default();
        if config.enable_custom_type_definition_for_reanimated {
            let reanimated_module_type = globals::get_reanimated_module_type(&mut shapes);
            module_types
                .insert(Ident::from("react-native-reanimated"), Some(reanimated_module_type));
        }

        Self {
            allocator,
            next_block_id_counter: 0,
            next_scope_id_counter: 0,
            next_mutable_range_id_counter: 0,
            identifiers: IndexVec::new(),
            types: IndexVec::new(),
            scopes: IndexVec::new(),
            functions: IndexVec::new(),
            errors: Diagnostics::new(),
            skip_compilation: false,
            fn_type: ReactFunctionType::Other,
            output_mode: OutputMode::Client,
            instrument_fn_name: None,
            instrument_gating_name: None,
            hook_guard_name: None,
            memo_cache_name: None,
            renames: FxHashMap::default(),
            hoisted_identifiers: FxHashSet::default(),
            validate_preserve_existing_memoization_guarantees: config
                .validate_preserve_existing_memoization_guarantees,
            validate_no_set_state_in_render: config.validate_no_set_state_in_render,
            enable_preserve_existing_memoization_guarantees: config
                .enable_preserve_existing_memoization_guarantees,
            globals: global_registry,
            shapes,
            module_types,
            module_type_errors: IdentHashMap::default(),
            default_nonmutating_hook: None,
            default_mutating_hook: None,
            outlined_functions: Vec::new(),
            uid_known_names: None,
            config,
        }
    }

    pub fn next_block_id(&mut self) -> BlockId {
        let id = BlockId::from_usize(self.next_block_id_counter as usize);
        self.next_block_id_counter += 1;
        id
    }

    /// Create a new MutableRange with a unique ID.
    /// Use this when creating a logically new range (not copying an existing one).
    /// To copy a range preserving its identity, use `.clone()` instead.
    pub fn new_mutable_range(
        &mut self,
        start: EvaluationOrder,
        end: EvaluationOrder,
    ) -> MutableRange {
        let id = MutableRangeId::from_usize(self.next_mutable_range_id_counter as usize);
        self.next_mutable_range_id_counter += 1;
        MutableRange { id, start, end }
    }

    /// Allocate a new Identifier in the arena with default values,
    /// returns its IdentifierId.
    pub fn next_identifier_id(&mut self) -> IdentifierId {
        let id = self.identifiers.next_idx();
        let type_id = self.make_type();
        let mutable_range = self.new_mutable_range(EvaluationOrder::UNSET, EvaluationOrder::UNSET);
        self.identifiers.push(Identifier {
            id,
            declaration_id: DeclarationId::from_usize(id.index()),
            name: None,
            mutable_range,
            scope: None,
            type_: type_id,
            span: None,
        });
        id
    }

    /// Allocate a new ReactiveScope in the arena, returns its ScopeId.
    pub fn next_scope_id(&mut self) -> ScopeId {
        let id = ScopeId::from_usize(self.next_scope_id_counter as usize);
        self.next_scope_id_counter += 1;
        let range = self.new_mutable_range(EvaluationOrder::UNSET, EvaluationOrder::UNSET);
        self.scopes.push(ReactiveScope {
            id,
            range,
            dependencies: Vec::new(),
            declarations: Vec::new(),
            reassignments: Vec::new(),
            early_return_value: None,
            merged: Vec::new(),
            span: None,
        });
        id
    }

    /// Allocate a new Type in the arena, returns its TypeId.
    pub fn next_type_id(&mut self) -> TypeId {
        let id = self.types.next_idx();
        self.types.push(Type::Var { id });
        id
    }

    /// Allocate a new Type (TypeVar) in the arena, returns its TypeId.
    pub fn make_type(&mut self) -> TypeId {
        self.next_type_id()
    }

    pub fn add_function(&mut self, func: HirFunction<'a>) -> FunctionId {
        let id = self.functions.next_idx();
        self.functions.push(func);
        id
    }

    pub fn record_error(&mut self, diagnostic: OxcDiagnostic) -> Result<(), OxcDiagnostic> {
        if ErrorCategory::Invariant.matches(&diagnostic) {
            self.errors.push(diagnostic.clone());
            return Err(diagnostic);
        }
        self.errors.push(diagnostic);
        Ok(())
    }

    pub fn record_diagnostic(&mut self, diagnostic: OxcDiagnostic) {
        self.errors.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if any recorded errors have Invariant category.
    /// In TS, Invariant errors throw immediately from recordError(),
    /// which aborts the current operation.
    pub fn has_invariant_errors(&self) -> bool {
        self.errors.iter().any(|d| ErrorCategory::Invariant.matches(d))
    }

    /// Take the accumulated errors, to be returned at the end of the pipeline.
    pub fn take_errors(&mut self) -> Diagnostics {
        take(&mut self.errors)
    }

    /// Take only the Invariant errors, leaving non-Invariant errors in place.
    /// In TS, Invariant errors throw as a separate error, so only the
    /// Invariant error is surfaced.
    pub fn take_invariant_errors(&mut self) -> Diagnostics {
        let (invariant, remaining): (Vec<_>, Vec<_>) =
            take(&mut self.errors).into_iter().partition(|d| ErrorCategory::Invariant.matches(d));
        self.errors = remaining.into();
        invariant.into()
    }

    /// Check if a binding has been hoisted (via DeclareContext) already.
    pub fn is_hoisted_identifier(&self, binding_id: SymbolId) -> bool {
        self.hoisted_identifiers.contains(&binding_id)
    }

    /// Mark a binding as hoisted.
    pub fn add_hoisted_identifier(&mut self, binding_id: SymbolId) {
        self.hoisted_identifiers.insert(binding_id);
    }

    // =========================================================================
    // Type resolution methods (ported from Environment.ts)
    // =========================================================================

    /// Resolve a non-local binding to its type. Ported from TS `getGlobalDeclaration`.
    ///
    /// The `span` parameter is used for error diagnostics when validating module type
    /// configurations. Pass `None` if no source location is available.
    pub fn get_global_declaration(
        &mut self,
        binding: &NonLocalBinding,
        span: Option<Span>,
    ) -> Result<Option<Global<'a>>, OxcDiagnostic> {
        match binding {
            NonLocalBinding::ModuleLocal { name, .. } => {
                if is_hook_name(name) {
                    Ok(Some(self.get_custom_hook_type()))
                } else {
                    Ok(None)
                }
            }
            NonLocalBinding::Global { name, .. } => {
                if let Some(ty) = self.globals.get(name) {
                    return Ok(Some(ty.clone()));
                }
                if is_hook_name(name) { Ok(Some(self.get_custom_hook_type())) } else { Ok(None) }
            }
            NonLocalBinding::ImportSpecifier { name, module, imported } => {
                if self.is_known_react_module(module) {
                    if let Some(ty) = self.globals.get(imported) {
                        return Ok(Some(ty.clone()));
                    }
                    if is_hook_name(imported) || is_hook_name(name) {
                        return Ok(Some(self.get_custom_hook_type()));
                    }
                    return Ok(None);
                }

                // Try module type provider. We resolve first, then do property
                // lookup on the cloned result to avoid double-borrow of self.
                let module_type = self.resolve_module_type(module);

                // Check for module type validation errors (hook-name vs hook-type mismatches)
                if let Some(errors) = self.module_type_errors.remove(module.as_str())
                    && let Some(first_error) = errors.into_iter().next()
                {
                    self.record_error(
                        ErrorCategory::Config
                            .diagnostic("Invalid type configuration for module")
                            .with_help(first_error)
                            .with_labels(span),
                    )?;
                }

                if let Some(module_type) = module_type
                    && let Some(imported_type) =
                        Self::get_property_type_from_shapes(&self.shapes, &module_type, imported)
                {
                    return Ok(Some(imported_type));
                }

                if is_hook_name(imported) || is_hook_name(name) {
                    Ok(Some(self.get_custom_hook_type()))
                } else {
                    Ok(None)
                }
            }
            NonLocalBinding::ImportDefault { name, module }
            | NonLocalBinding::ImportNamespace { name, module } => {
                let is_default = matches!(binding, NonLocalBinding::ImportDefault { .. });

                if self.is_known_react_module(module) {
                    if let Some(ty) = self.globals.get(name) {
                        return Ok(Some(ty.clone()));
                    }
                    if is_hook_name(name) {
                        return Ok(Some(self.get_custom_hook_type()));
                    }
                    return Ok(None);
                }

                let module_type = self.resolve_module_type(module);

                // Check for module type validation errors (hook-name vs hook-type mismatches)
                if let Some(errors) = self.module_type_errors.remove(module.as_str())
                    && let Some(first_error) = errors.into_iter().next()
                {
                    self.record_error(
                        ErrorCategory::Config
                            .diagnostic("Invalid type configuration for module")
                            .with_help(first_error)
                            .with_labels(span),
                    )?;
                }

                if let Some(module_type) = module_type {
                    let imported_type = if is_default {
                        Self::get_property_type_from_shapes(&self.shapes, &module_type, "default")
                    } else {
                        Some(module_type)
                    };
                    if let Some(imported_type) = imported_type {
                        // Validate hook-name vs hook-type consistency for module name
                        let expect_hook = is_hook_name(module);
                        let is_hook =
                            self.get_hook_kind_for_type(&imported_type).ok().flatten().is_some();
                        if expect_hook != is_hook {
                            self.record_error(
                                ErrorCategory::Config
                                    .diagnostic("Invalid type configuration for module")
                                    .with_help(format!(
                                        "Expected type for `import ... from '{}'` {} based on the module name",
                                        module,
                                        if expect_hook { "to be a hook" } else { "not to be a hook" }
                                    ))
                                    .with_labels(span),
                            )?;
                        }
                        return Ok(Some(imported_type));
                    }
                }

                if is_hook_name(name) { Ok(Some(self.get_custom_hook_type())) } else { Ok(None) }
            }
        }
    }

    /// Static helper: resolve a property type using only the shapes registry.
    /// Used internally to avoid double-borrow of `self`. Includes hook-name
    /// fallback matching TS `getPropertyType`.
    fn get_property_type_from_shapes(
        shapes: &ShapeRegistry<'a>,
        receiver: &Type<'a>,
        property: &str,
    ) -> Option<Type<'a>> {
        let shape_id = match receiver {
            Type::Object { shape_id } | Type::Function { shape_id, .. } => shape_id.as_deref(),
            _ => None,
        };
        if let Some(shape_id) = shape_id {
            let shape = shapes.get(shape_id)?;
            if let Some(ty) = shape.properties.get(property) {
                return Some(ty.clone());
            }
            if let Some(ty) = shape.properties.get("*") {
                return Some(ty.clone());
            }
            // Hook-name fallback: callers that need the custom hook type
            // check is_hook_name after this returns None, which produces
            // the same result as the TS getPropertyType hook-name fallback.
        }
        None
    }

    /// Get the function signature for a function type.
    /// Ported from TS `getFunctionSignature`.
    pub fn get_function_signature(
        &self,
        ty: &Type,
    ) -> Result<Option<&FunctionSignature>, OxcDiagnostic> {
        let shape_id = match ty {
            Type::Function { shape_id, .. } => shape_id.as_deref(),
            _ => return Ok(None),
        };
        if let Some(shape_id) = shape_id {
            let shape = self.shapes.get(shape_id).ok_or_else(|| {
                ErrorCategory::Invariant.diagnostic(format!(
                    "[HIR] Forget internal error: cannot resolve shape {shape_id}"
                ))
            })?;
            return Ok(shape.function_type.as_ref());
        }
        Ok(None)
    }

    /// Get the hook kind for a type, if it represents a hook.
    /// Ported from TS `getHookKindForType` in HIR.ts.
    pub fn get_hook_kind_for_type(&self, ty: &Type) -> Result<Option<&HookKind>, OxcDiagnostic> {
        Ok(self.get_function_signature(ty)?.and_then(|sig| sig.hook_kind.as_ref()))
    }

    /// Resolve the module type provider for a given module name.
    /// Caches results. Checks pre-resolved provider results first, then falls
    /// back to `defaultModuleTypeProvider` (hardcoded).
    fn resolve_module_type(&mut self, module_name: &str) -> Option<Global<'a>> {
        if let Some(cached) = self.module_types.get(module_name) {
            return cached.clone();
        }

        // Check pre-resolved provider results first, then fall back to default
        let module_config = self
            .config
            .module_type_provider
            .as_ref()
            .and_then(|map| map.get(module_name).cloned())
            .or_else(|| default_module_type_provider(module_name));

        let module_type = module_config.map(|config| {
            let mut type_errors: Vec<String> = Vec::new();
            let ty = globals::install_type_config(
                &mut self.shapes,
                &config,
                module_name,
                &mut type_errors,
            );
            // Store errors for later reporting when the import is actually used
            for err in type_errors {
                self.module_type_errors
                    .entry(Ident::from_str_in(module_name, &self.allocator))
                    .or_default()
                    .push(err);
            }
            ty
        });
        self.module_types
            .insert(Ident::from_str_in(module_name, &self.allocator), module_type.clone());
        module_type
    }

    fn is_known_react_module(&self, module_name: &str) -> bool {
        let lower = module_name.cow_to_lowercase();
        lower == "react" || lower == "react-dom"
    }

    fn get_custom_hook_type(&mut self) -> Global<'a> {
        if self.config.enable_assume_hooks_follow_rules_of_react {
            if self.default_nonmutating_hook.is_none() {
                self.default_nonmutating_hook = Some(default_nonmutating_hook(&mut self.shapes));
            }
            self.default_nonmutating_hook.clone().unwrap()
        } else {
            if self.default_mutating_hook.is_none() {
                self.default_mutating_hook = Some(default_mutating_hook(&mut self.shapes));
            }
            self.default_mutating_hook.clone().unwrap()
        }
    }

    /// Public accessor for the custom hook type, used by InferTypes for
    /// property resolution fallback when a property name looks like a hook.
    pub fn get_custom_hook_type_opt(&mut self) -> Option<Global<'a>> {
        Some(self.get_custom_hook_type())
    }

    /// Get a reference to the globals registry.
    pub fn globals(&self) -> &GlobalRegistry<'a> {
        &self.globals
    }

    /// Generate a globally unique identifier name, analogous to TS
    /// `generateGloballyUniqueIdentifierName` which delegates to Babel's
    /// `scope.generateUidIdentifier`. Matches Babel's naming convention:
    /// first name is `_<name>`, subsequent are `_<name>2`, `_<name>3`, etc.
    /// Also applies Babel's `toIdentifier` sanitization on the input name.
    ///
    /// Like Babel's `generateUid`, checks for collisions against existing
    /// bindings (source-level identifier names) and previously generated UIDs,
    /// rather than using a blind counter.
    pub fn generate_globally_unique_identifier_name(&mut self, name: Option<&str>) -> Ident<'a> {
        let base = name.unwrap_or("temp");
        // Apply Babel's toIdentifier sanitization:
        // 1. Replace non-identifier chars with '-'
        // 2. Strip leading '-' and digits
        // 3. CamelCase: replace '-' sequences + optional following char with uppercase of that char
        let mut dashed = String::new();
        for c in base.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                dashed.push(c);
            } else {
                dashed.push('-');
            }
        }
        // Strip leading dashes and digits
        let trimmed = dashed.trim_start_matches(|c: char| c == '-' || c.is_ascii_digit());
        // CamelCase conversion: replace sequences of '-' followed by optional char with uppercase
        let mut camel = String::new();
        let mut chars = trimmed.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '-' {
                while chars.peek() == Some(&'-') {
                    chars.next();
                }
                if let Some(next) = chars.next() {
                    for uc in next.to_uppercase() {
                        camel.push(uc);
                    }
                }
            } else {
                camel.push(c);
            }
        }
        if camel.is_empty() {
            camel = "temp".to_string();
        }
        // Strip leading '_' and trailing digits (Babel's generateUid behavior)
        let stripped = camel.trim_start_matches('_');
        let stripped = stripped.trim_end_matches(|c: char| c.is_ascii_digit());
        let uid_base = if stripped.is_empty() { "temp" } else { stripped };

        // Lazily build the set of known names from existing identifiers.
        // This approximates Babel's hasBinding/hasGlobal/hasReference checks.
        if self.uid_known_names.is_none() {
            let mut known = IdentHashSet::default();
            for id in &self.identifiers {
                if let Some(name) = &id.name {
                    known.insert(Ident::from(name.value()));
                }
            }
            self.uid_known_names = Some(known);
        }

        // Find a name that doesn't collide, matching Babel's generateUid loop
        let mut i = 1u32;
        let uid = loop {
            let candidate = if i == 1 {
                format_ident!(self.allocator, "_{uid_base}")
            } else {
                format_ident!(self.allocator, "_{uid_base}{i}")
            };
            i += 1;
            if !self.uid_known_names.as_ref().unwrap().contains(&candidate) {
                break candidate;
            }
        };

        // Register the generated name so subsequent calls see it
        self.uid_known_names.as_mut().unwrap().insert(uid);

        uid
    }

    /// Seed the UID known names set with external names (e.g. from ProgramContext).
    /// This ensures UID generation avoids names generated by previous function compilations,
    /// matching Babel's behavior where the program scope accumulates all generated UIDs.
    pub fn seed_uid_known_names(&mut self, names: &IdentHashSet<'a>) {
        match &mut self.uid_known_names {
            Some(existing) => existing.extend(names.iter().copied()),
            None => self.uid_known_names = Some(names.clone()),
        }
    }

    /// Return the UID known names accumulated during this compilation.
    pub fn take_uid_known_names(&mut self) -> Option<IdentHashSet<'a>> {
        self.uid_known_names.take()
    }

    /// Record an outlined function (extracted during outlineFunctions or outlineJSX).
    /// Corresponds to TS `env.outlineFunction(fn, type)`.
    pub fn outline_function(&mut self, func: HirFunction<'a>, fn_type: Option<ReactFunctionType>) {
        self.outlined_functions.push(OutlinedFunctionEntry { func, fn_type });
    }

    /// Take the outlined functions, leaving the vec empty.
    pub fn take_outlined_functions(&mut self) -> Vec<OutlinedFunctionEntry<'a>> {
        take(&mut self.outlined_functions)
    }

    /// Whether memoization is enabled for this compilation.
    /// Ported from TS `get enableMemoization()` in Environment.ts.
    /// Returns true for client/lint modes, false for SSR.
    pub fn enable_memoization(&self) -> bool {
        match self.output_mode {
            OutputMode::Client | OutputMode::Lint => true,
            OutputMode::Ssr => false,
        }
    }

    /// Whether validations are enabled for this compilation.
    /// Ported from TS `get enableValidations()` in Environment.ts.
    pub fn enable_validations(&self) -> bool {
        match self.output_mode {
            OutputMode::Client | OutputMode::Lint | OutputMode::Ssr => true,
        }
    }

    // =========================================================================
    // ID-based type helper methods
    // =========================================================================

    /// Check whether the function type for an identifier has a noAlias signature.
    /// Looks up the identifier's type and checks its function signature.
    pub fn has_no_alias_signature(&self, identifier_id: IdentifierId) -> bool {
        let ty = &self.types[self.identifiers[identifier_id].type_];
        self.get_function_signature(ty).ok().flatten().is_some_and(|sig| sig.no_alias)
    }

    /// Get the hook kind for an identifier, if its type represents a hook.
    /// Looks up the identifier's type and delegates to `get_hook_kind_for_type`.
    pub fn get_hook_kind_for_id(
        &self,
        identifier_id: IdentifierId,
    ) -> Result<Option<&HookKind>, OxcDiagnostic> {
        let ty = &self.types[self.identifiers[identifier_id].type_];
        self.get_hook_kind_for_type(ty)
    }
}

/// Check if a name matches the React hook naming convention: `use[A-Z0-9]`.
/// Ported from TS `isHookName` in Environment.ts.
pub fn is_hook_name(name: &str) -> bool {
    if name.len() < 4 {
        return false;
    }
    if !name.starts_with("use") {
        return false;
    }
    let fourth_char = name.as_bytes()[3];
    fourth_char.is_ascii_uppercase() || fourth_char.is_ascii_digit()
}
