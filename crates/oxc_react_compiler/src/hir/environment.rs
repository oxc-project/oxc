/// Compiler environment and configuration.
///
/// Port of `HIR/Environment.ts` from the React Compiler.
///
/// The `Environment` holds all compilation context and configuration,
/// including shape registries, global definitions, and feature flags.
/// `EnvironmentConfig` defines all the knobs for controlling compiler behavior.
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
    globals::{Global, GlobalRegistry},
    hir_types::{
        Effect, HIRFunction, IdentifierName, NonLocalBinding, ReactFunctionType, ValueKind,
    },
    object_shape::{
        BUILT_IN_DEFAULT_MUTATING_HOOK_ID, BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID,
        FunctionSignature, HookKind, ShapeRegistry,
    },
    types::{FunctionType, ObjectType, Type},
};

/// Configuration for an external function reference (source module + import specifier).
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub source: String,
    pub import_specifier_name: String,
}

/// Configuration for instrumentation.
#[derive(Debug, Clone)]
pub struct InstrumentationConfig {
    pub func: ExternalFunction,
    pub gating: Option<ExternalFunction>,
    pub global_gating: Option<String>,
}

/// Configuration for a custom hook.
#[derive(Debug, Clone)]
pub struct HookConfig {
    pub effect_kind: Effect,
    pub value_kind: ValueKind,
    pub no_alias: bool,
    pub transitive_mixed_data: bool,
}

/// The full environment configuration — all compiler knobs and settings.
///
/// This corresponds to the Zod-validated `EnvironmentConfig` in the TS version.
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub custom_hooks: FxHashMap<String, HookConfig>,

    /// A list of functions which the application compiles as macros.
    pub custom_macros: Option<Vec<String>>,

    /// Enable/disable the new type inference based on Flow types.
    pub enable_forest: bool,

    /// Enable function outlining.
    pub enable_function_outlining: bool,

    /// Enable JSX outlining.
    pub enable_jsx_outlining: bool,

    /// Enable naming anonymous functions.
    pub enable_name_anonymous_functions: bool,

    /// Whether to validate hooks usage.
    pub validate_hooks_usage: bool,

    /// Whether to validate no capitalized calls.
    pub validate_no_capitalized_calls: Option<Vec<String>>,

    /// Whether to validate ref access during render.
    pub validate_ref_access_during_render: bool,

    /// Whether to validate no setState in render.
    pub validate_no_set_state_in_render: bool,

    /// Whether to validate preserve existing memoization guarantees.
    pub validate_preserve_existing_memoization_guarantees: bool,

    /// Enable preserve existing memoization guarantees.
    pub enable_preserve_existing_memoization_guarantees: bool,

    /// Whether to validate exhaustive memoization dependencies.
    pub validate_exhaustive_memoization_dependencies: bool,

    /// Whether to validate exhaustive effect dependencies.
    pub validate_exhaustive_effect_dependencies: bool,

    /// Whether to validate no derived computations in effects.
    pub validate_no_derived_computations_in_effects: bool,

    /// Whether to validate no derived computations in effects (experimental).
    pub validate_no_derived_computations_in_effects_exp: bool,

    /// Whether to validate no setState in effects.
    pub validate_no_set_state_in_effects: bool,

    /// Whether to validate no JSX in try statements.
    pub validate_no_jsx_in_try_statements: bool,

    /// Whether to validate no impure functions in render.
    pub validate_no_impure_functions_in_render: bool,

    /// Whether to validate static components.
    pub validate_static_components: bool,

    /// Whether to validate source locations.
    pub validate_source_locations: bool,

    /// Whether to assert valid mutable ranges.
    pub assert_valid_mutable_ranges: bool,

    /// Enable emit instrument forget.
    pub enable_emit_instrument_forget: Option<InstrumentationConfig>,

    /// Enable emit hook guards.
    pub enable_emit_hook_guards: Option<ExternalFunction>,

    /// Whether to throw on unknown exceptions (test only).
    pub throw_unknown_exception_testonly: bool,

    /// Enable reset cache on source file changes (HMR support).
    pub enable_reset_cache_on_source_file_changes: Option<bool>,

    /// Enable custom type definitions for react-native-reanimated.
    ///
    /// When true, the compiler treats reanimated shared values as having
    /// specific type signatures to allow correct memoization behavior.
    pub enable_custom_type_definition_for_reanimated: bool,

    /// Enable the shared-runtime module type provider (test only).
    ///
    /// When true, the compiler registers type definitions for the `shared-runtime`
    /// test module, providing correct type information for hooks like `useFragment`
    /// (returns `MixedReadonly`, `noAlias: true`), `useNoAlias`, and typed functions.
    /// This matches the `sharedRuntimeTypeProvider` from the TS test harness.
    pub enable_shared_runtime_type_provider: bool,

    /// Whether to assume hooks follow the rules of React.
    ///
    /// When true (default), custom hooks are treated with `DefaultNonmutatingHook`:
    /// arguments are frozen and return values are frozen.
    /// When false, custom hooks are treated with `DefaultMutatingHook`:
    /// arguments may be conditionally mutated and return values are mutable.
    ///
    /// Corresponds to `enableAssumeHooksFollowRulesOfReact` in the TS version.
    pub enable_assume_hooks_follow_rules_of_react: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            custom_hooks: FxHashMap::default(),
            custom_macros: None,
            enable_forest: false,
            enable_function_outlining: true,
            enable_jsx_outlining: false,
            enable_name_anonymous_functions: false,
            validate_hooks_usage: true,
            validate_no_capitalized_calls: None,
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_preserve_existing_memoization_guarantees: true,
            enable_preserve_existing_memoization_guarantees: true,
            validate_exhaustive_memoization_dependencies: false,
            validate_exhaustive_effect_dependencies: false,
            validate_no_derived_computations_in_effects: false,
            validate_no_derived_computations_in_effects_exp: false,
            validate_no_set_state_in_effects: false,
            validate_no_jsx_in_try_statements: false,
            validate_no_impure_functions_in_render: false,
            validate_static_components: false,
            validate_source_locations: false,
            assert_valid_mutable_ranges: false,
            enable_emit_instrument_forget: None,
            enable_emit_hook_guards: None,
            throw_unknown_exception_testonly: false,
            enable_reset_cache_on_source_file_changes: None,
            enable_custom_type_definition_for_reanimated: false,
            enable_shared_runtime_type_provider: false,
            enable_assume_hooks_follow_rules_of_react: true,
        }
    }
}

/// Validate an environment config.
///
/// In the TS version, this uses Zod schema validation.
/// In Rust, we rely on the type system for basic validation
/// and add runtime checks as needed.
///
/// # Errors
/// Returns a `CompilerError` if the config is invalid.
pub fn validate_environment_config(
    config: EnvironmentConfig,
) -> Result<EnvironmentConfig, crate::compiler_error::CompilerError> {
    // Validate that enable_emit_instrument_forget has at least one gating method
    if let Some(ref instrumentation) = config.enable_emit_instrument_forget
        && instrumentation.gating.is_none()
        && instrumentation.global_gating.is_none()
    {
        return Err(crate::compiler_error::CompilerError::invalid_config(
            "Expected at least one of gating or globalGating in instrumentation config",
            None,
            None,
        ));
    }
    Ok(config)
}

/// The output mode of the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerOutputMode {
    /// Build optimized for SSR, with client features removed.
    Ssr,
    /// Build optimized for the client, with auto memoization.
    Client,
    /// Build optimized for the client without auto memo.
    ClientNoMemo,
    /// Lint mode: output is unused but validations run.
    Lint,
}

/// The compilation environment, holding all context needed during compilation.
///
/// This is the central context object threaded through all compiler passes.
#[derive(Debug, Clone)]
pub struct Environment {
    pub fn_type: ReactFunctionType,
    pub output_mode: CompilerOutputMode,
    pub config: EnvironmentConfig,
    pub shapes: ShapeRegistry,
    pub globals: GlobalRegistry,

    /// The source code of the file being compiled.
    /// Used for HMR/Fast Refresh cache invalidation.
    pub code: Option<String>,

    /// The filename of the file being compiled.
    /// Used for instrumentation (instrument forget) emission.
    pub filename: Option<String>,

    /// Whether validations should be enabled.
    pub enable_validations: bool,

    /// Whether memoization should be enabled.
    pub enable_memoization: bool,

    /// Whether manual memoization dropping is enabled.
    pub enable_drop_manual_memoization: bool,

    /// Collected diagnostics from lint-mode validation passes.
    diagnostics: Vec<crate::compiler_error::CompilerError>,

    /// Outlined functions extracted from this function.
    ///
    /// Each entry is a tuple of the outlined HIR function and an optional
    /// `ReactFunctionType` string (e.g., `Some("Component")` or `None`).
    outlined_functions: Vec<OutlinedFunctionEntry>,

    /// Known referenced names — used for generating globally unique identifiers.
    ///
    /// Corresponds to `ProgramContext.knownReferencedNames` in the TS version.
    known_referenced_names: FxHashSet<String>,

    /// Counter for generating globally unique identifier names.
    next_uid: u32,

    /// Module type registry — maps module names to their type definitions.
    ///
    /// Port of `#moduleTypes` from the TS `Environment` class.
    /// Used for resolving imports from modules with known type definitions
    /// (e.g., react-native-reanimated).
    module_types: FxHashMap<String, super::types::Type>,

    // ID counters
    next_block_id: u32,
    next_scope_id: u32,
    next_identifier_id: u32,
}

/// An entry in the outlined functions list.
#[derive(Debug, Clone)]
pub struct OutlinedFunctionEntry {
    pub func: HIRFunction,
    pub fn_type: Option<ReactFunctionType>,
}

impl Environment {
    /// Create a new environment with the given configuration.
    pub fn new(
        fn_type: ReactFunctionType,
        output_mode: CompilerOutputMode,
        config: EnvironmentConfig,
    ) -> Self {
        let enable_memoization =
            !matches!(output_mode, CompilerOutputMode::Lint | CompilerOutputMode::ClientNoMemo);
        let enable_validations = true;
        let enable_drop_manual_memoization = enable_memoization;

        // Initialize shapes and globals from the built-in definitions
        let mut shapes = super::globals::default_shapes();
        let globals = super::globals::default_globals(&mut shapes);

        // Register module types for configured type providers.
        let mut module_types = FxHashMap::default();
        if config.enable_custom_type_definition_for_reanimated {
            let reanimated_type = super::globals::get_reanimated_module_type(&mut shapes);
            module_types.insert("react-native-reanimated".to_string(), reanimated_type);
        }
        if config.enable_shared_runtime_type_provider {
            let shared_runtime_type = super::globals::get_shared_runtime_module_type(&mut shapes);
            module_types.insert("shared-runtime".to_string(), shared_runtime_type);
        }

        Self {
            fn_type,
            output_mode,
            config,
            shapes,
            globals,
            code: None,
            filename: None,
            enable_validations,
            enable_memoization,
            enable_drop_manual_memoization,
            diagnostics: Vec::new(),
            outlined_functions: Vec::new(),
            known_referenced_names: FxHashSet::default(),
            module_types,
            next_uid: 0,
            next_block_id: 0,
            next_scope_id: 0,
            next_identifier_id: 0,
        }
    }

    /// Get the next block ID value without incrementing.
    pub fn next_block_id_value(&self) -> u32 {
        self.next_block_id
    }

    /// Generate a fresh block ID.
    pub fn next_block_id(&mut self) -> super::hir_types::BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        super::hir_types::BlockId(id)
    }

    /// Generate a fresh scope ID.
    pub fn next_scope_id(&mut self) -> super::hir_types::ScopeId {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        super::hir_types::ScopeId(id)
    }

    /// Generate a fresh identifier ID.
    pub fn next_identifier_id(&mut self) -> super::hir_types::IdentifierId {
        let id = self.next_identifier_id;
        self.next_identifier_id += 1;
        super::hir_types::IdentifierId(id)
    }

    /// Advance all ID counters to be at least as high as the given environment's
    /// counters. This simulates the TS behavior where nested function lowering
    /// shares the same Environment object (by reference), so the outer function's
    /// counters automatically advance past all inner function's allocations.
    pub fn advance_counters_past(&mut self, other: &Environment) {
        if other.next_block_id > self.next_block_id {
            self.next_block_id = other.next_block_id;
        }
        if other.next_scope_id > self.next_scope_id {
            self.next_scope_id = other.next_scope_id;
        }
        if other.next_identifier_id > self.next_identifier_id {
            self.next_identifier_id = other.next_identifier_id;
        }
    }

    /// Log errors from a validation pass result. If the result is Err, the errors
    /// are collected into the environment's diagnostics list rather than propagated.
    ///
    /// This matches the TS `env.logErrors(result)` pattern used for lint-mode
    /// validation passes.
    pub fn log_errors(&mut self, result: Result<(), crate::compiler_error::CompilerError>) {
        if let Err(error) = result {
            self.diagnostics.push(error);
        }
    }

    /// Retrieve and clear all collected diagnostics.
    pub fn take_diagnostics(&mut self) -> Vec<crate::compiler_error::CompilerError> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Register a function to be outlined (extracted) from this function.
    ///
    /// Corresponds to `Environment.outlineFunction()` in the TS version.
    pub fn outline_function(&mut self, func: HIRFunction, fn_type: Option<ReactFunctionType>) {
        self.outlined_functions.push(OutlinedFunctionEntry { func, fn_type });
    }

    /// Get the list of outlined functions.
    ///
    /// Corresponds to `Environment.getOutlinedFunctions()` in the TS version.
    pub fn get_outlined_functions(&self) -> &[OutlinedFunctionEntry] {
        &self.outlined_functions
    }

    /// Generate a globally unique identifier name.
    ///
    /// Corresponds to `Environment.generateGloballyUniqueIdentifierName()` in
    /// the TS version. In the TS version this delegates to Babel's
    /// `scope.generateUidIdentifier()` which produces names like `_name`,
    /// `_name2`, etc.
    ///
    /// Babel's `generateUid` applies `toIdentifier()` to sanitize the hint
    /// into a valid JS identifier (replacing brackets, dots, angle brackets,
    /// spaces, etc. with camelCase), then strips leading underscores and
    /// trailing digits before prefixing with `_`.
    pub fn generate_globally_unique_identifier_name(
        &mut self,
        hint: Option<&str>,
    ) -> IdentifierName {
        let base = hint.unwrap_or("temp");
        // Mimic Babel's generateUid: toIdentifier(name).replace(/^_+/, "").replace(/\d+$/g, "")
        let sanitized = to_identifier(base);
        let stripped = sanitized.trim_start_matches('_');
        let stripped = stripped.trim_end_matches(|c: char| c.is_ascii_digit());
        let stripped = if stripped.is_empty() { "temp" } else { stripped };
        let prefix = format!("_{stripped}");
        let mut candidate = prefix.clone();
        loop {
            if !self.known_referenced_names.contains(&candidate) {
                break;
            }
            self.next_uid += 1;
            candidate = format!("{prefix}{}", self.next_uid);
        }
        self.known_referenced_names.insert(candidate.clone());
        IdentifierName::Named(candidate)
    }

    /// Register a name as known/referenced, preventing it from being
    /// used by `generate_globally_unique_identifier_name`.
    ///
    /// Corresponds to `ProgramContext.addNewReference()` in the TS version.
    pub fn add_new_reference(&mut self, name: &str) {
        self.known_referenced_names.insert(name.to_string());
    }

    // =========================================================================
    // Global and type lookup methods
    // =========================================================================

    /// Resolve a non-local binding to its global type.
    ///
    /// Port of `Environment.getGlobalDeclaration()` from `HIR/Environment.ts`.
    pub fn get_global_declaration(&self, binding: &NonLocalBinding) -> Option<Global> {
        match binding {
            NonLocalBinding::ModuleLocal { name } => {
                if is_hook_name(name) {
                    Some(self.get_custom_hook_type())
                } else {
                    None
                }
            }
            NonLocalBinding::Global { name } => {
                if let Some(g) = self.globals.get(name) {
                    Some(g.clone())
                } else if is_hook_name(name) {
                    Some(self.get_custom_hook_type())
                } else {
                    None
                }
            }
            NonLocalBinding::ImportSpecifier { name, module, imported } => {
                if is_known_react_module(module) {
                    // For React modules, look up by imported name
                    if let Some(g) = self.globals.get(imported) {
                        Some(g.clone())
                    } else if is_hook_name(imported) || is_hook_name(name) {
                        Some(self.get_custom_hook_type())
                    } else {
                        None
                    }
                } else {
                    // Check module type registry (e.g., react-native-reanimated)
                    if let Some(module_type) = self.resolve_module_type(module) {
                        if let Some(imported_type) = self.get_property_type(&module_type, imported)
                        {
                            return Some(Global::Typed(imported_type));
                        }
                    }

                    // Fall back to hook name pattern
                    if is_hook_name(imported) || is_hook_name(name) {
                        Some(self.get_custom_hook_type())
                    } else {
                        None
                    }
                }
            }
            NonLocalBinding::ImportDefault { name, module } => {
                if is_known_react_module(module) {
                    if let Some(g) = self.globals.get(name) {
                        Some(g.clone())
                    } else if is_hook_name(name) {
                        Some(self.get_custom_hook_type())
                    } else {
                        None
                    }
                } else {
                    // Check module type registry for default export
                    if let Some(module_type) = self.resolve_module_type(module) {
                        if let Some(default_type) = self.get_property_type(&module_type, "default")
                        {
                            return Some(Global::Typed(default_type));
                        }
                    }
                    if is_hook_name(name) { Some(self.get_custom_hook_type()) } else { None }
                }
            }
            NonLocalBinding::ImportNamespace { name, module } => {
                if is_known_react_module(module) {
                    if let Some(g) = self.globals.get(name) {
                        Some(g.clone())
                    } else if is_hook_name(name) {
                        Some(self.get_custom_hook_type())
                    } else {
                        None
                    }
                } else {
                    // Check module type registry for namespace import
                    if let Some(module_type) = self.resolve_module_type(module) {
                        return Some(Global::Typed(module_type));
                    }
                    if is_hook_name(name) { Some(self.get_custom_hook_type()) } else { None }
                }
            }
        }
    }

    /// Resolve a module name to its type definition, if one is registered.
    ///
    /// Port of `#resolveModuleType()` from `HIR/Environment.ts`.
    ///
    /// In the Rust port we only support pre-registered module types (e.g.,
    /// the reanimated module registered via `enable_custom_type_definition_for_reanimated`).
    /// The TS version also supports a dynamic `moduleTypeProvider` callback
    /// which is not yet ported.
    fn resolve_module_type(&self, module_name: &str) -> Option<Type> {
        self.module_types.get(module_name).cloned()
    }

    /// Look up a property type from the shape registry.
    ///
    /// Port of `Environment.getPropertyType()` from `HIR/Environment.ts`.
    ///
    /// Lookup order for string properties: exact name → wildcard ('*') → hook pattern.
    pub fn get_property_type(&self, receiver: &Type, property: &str) -> Option<Type> {
        let shape_id = match receiver {
            Type::Object(ObjectType { shape_id: Some(id) }) => Some(id.as_str()),
            Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id {
            if let Some(shape) = self.shapes.get(shape_id) {
                // Try exact property name, then wildcard, then hook pattern
                if let Some(t) = shape.properties.get(property) {
                    return Some(t.clone());
                }
                if let Some(t) = shape.properties.get("*") {
                    return Some(t.clone());
                }
                if is_hook_name(property) {
                    return Some(Global::to_type(&self.get_custom_hook_type()));
                }
                return None;
            }
        }

        // No shape: only check hook pattern for string properties
        if is_hook_name(property) {
            return Some(Global::to_type(&self.get_custom_hook_type()));
        }
        None
    }

    /// Get the fallthrough (wildcard) property type for a computed property access.
    ///
    /// Port of `Environment.getFallthroughPropertyType()` from `HIR/Environment.ts`.
    /// For computed property accesses like `obj[idx]`, only looks up the `*` wildcard
    /// property on the receiver's shape (ignoring the specific property value).
    pub fn get_fallthrough_property_type(&self, receiver: &Type) -> Option<Type> {
        let shape_id = match receiver {
            Type::Object(ObjectType { shape_id: Some(id) }) => Some(id.as_str()),
            Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id {
            if let Some(shape) = self.shapes.get(shape_id) {
                return shape.properties.get("*").cloned();
            }
        }
        None
    }

    /// Get the function signature from a function type's shape.
    ///
    /// Port of `Environment.getFunctionSignature()` from `HIR/Environment.ts`.
    pub fn get_function_signature(&self, type_: &Type) -> Option<&FunctionSignature> {
        let shape_id = match type_ {
            Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id {
            if let Some(shape) = self.shapes.get(shape_id) {
                return shape.function_type.as_ref();
            }
        }
        None
    }

    /// Get the default hook type for unrecognized hooks.
    ///
    /// Corresponds to `#getCustomHookType()` in the TS version.
    /// When `enableAssumeHooksFollowRulesOfReact` is true (default), returns
    /// `DefaultNonmutatingHook` (arguments frozen, return frozen).
    /// When false, returns `DefaultMutatingHook` (arguments conditionally mutated,
    /// return mutable).
    fn get_custom_hook_type(&self) -> Global {
        let shape_id = if self.config.enable_assume_hooks_follow_rules_of_react {
            BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID
        } else {
            BUILT_IN_DEFAULT_MUTATING_HOOK_ID
        };
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(shape_id.to_string()),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }))
    }
}

/// Check if a name matches the React hook naming convention: `use[A-Z0-9]`.
pub fn is_hook_name(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("use") {
        rest.starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit())
    } else {
        false
    }
}

/// Convert a string to a valid JavaScript identifier.
///
/// Port of Babel's `toIdentifier()` from `@babel/types/src/converters/toIdentifier.ts`.
///
/// 1. Replaces all non-identifier characters with `-`
/// 2. Removes leading dashes and digits
/// 3. CamelCases dash-separated segments (and collapses whitespace/dashes)
/// 4. Prefixes with `_` if result is not a valid identifier start
fn to_identifier(input: &str) -> String {
    // Step 1: Replace all non-identifier characters with `-`
    let mut dashed = String::with_capacity(input.len());
    for c in input.chars() {
        if is_identifier_char(c) {
            dashed.push(c);
        } else {
            dashed.push('-');
        }
    }

    // Step 2: Remove leading dashes and digits
    let trimmed = dashed.trim_start_matches(|c: char| c == '-' || c.is_ascii_digit());

    // Step 3: CamelCase — replace /[-\s]+(.)?/g with uppercased capture
    let mut result = String::with_capacity(trimmed.len());
    let mut capitalize_next = false;
    for c in trimmed.chars() {
        if c == '-' || c.is_whitespace() {
            capitalize_next = true;
        } else if capitalize_next {
            // Uppercase the first char after a dash/space sequence
            for uc in c.to_uppercase() {
                result.push(uc);
            }
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // Step 4: If still not valid identifier start, prefix with _
    if result.is_empty() {
        return "_".to_string();
    }
    let first = result.chars().next().unwrap();
    if !is_identifier_start(first) {
        return format!("_{result}");
    }

    result
}

/// Check if a character is a valid JS identifier continuation character.
///
/// Matches Unicode ID_Continue plus `$` and `_` (like Babel's `isIdentifierChar`).
fn is_identifier_char(c: char) -> bool {
    // Simple check: ASCII alphanumeric, _, $, or Unicode letter/digit
    c == '_' || c == '$' || c.is_alphanumeric()
}

/// Check if a character is a valid JS identifier start character.
fn is_identifier_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

/// Check if a module name is a known React module.
fn is_known_react_module(module: &str) -> bool {
    let lower = module.to_lowercase();
    lower == "react" || lower == "react-dom"
}

impl Global {
    /// Convert a Global to a Type (extracting the inner type if Typed).
    pub fn to_type(global: &Global) -> Type {
        match global {
            Global::Typed(t) => t.clone(),
            Global::Untyped => Type::Poly,
        }
    }
}

/// Get the hook kind for a given type, if it is a hook.
pub fn get_hook_kind_for_type(env: &Environment, type_: &Type) -> Option<HookKind> {
    env.get_function_signature(type_).and_then(|sig| sig.hook_kind)
}
