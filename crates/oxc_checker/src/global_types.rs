use rustc_hash::FxHashMap;
use std::sync::OnceLock;

use oxc_span::CompactStr;
use oxc_types::{IntrinsicType, ObjectFlags, TypeArena, TypeData, TypeFlags, TypeId};

/// Cached lib.d.ts source text. The filesystem walk to find lib.es5.d.ts
/// is done at most once (even across threads). Each caller still parses
/// the source into their own arena.
static LIB_SOURCE_CACHE: OnceLock<Option<String>> = OnceLock::new();

/// Pre-allocated intrinsic type IDs.
///
/// All 14 primitive/intrinsic types plus true/false literal types are
/// allocated once in the arena. Both the Checker and GlobalTypes extraction
/// share these IDs, eliminating duplicate intrinsic types.
#[derive(Clone, Copy)]
pub struct IntrinsicIds {
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub bigint_type: TypeId,
    pub boolean_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    pub non_primitive_type: TypeId,
    pub true_type: TypeId,
    pub false_type: TypeId,
}

/// Allocate all intrinsic types in the arena. Call once before both
/// GlobalTypes extraction and Checker construction.
pub fn allocate_intrinsics(arena: &TypeArena) -> IntrinsicIds {
    let new = |flags: TypeFlags, name: &'static str| -> TypeId {
        arena.new_type(
            flags,
            ObjectFlags::None,
            TypeData::Intrinsic(IntrinsicType { intrinsic_name: name }),
            None,
        )
    };

    let any_type = new(TypeFlags::Any, "any");
    let unknown_type = new(TypeFlags::Unknown, "unknown");
    let string_type = new(TypeFlags::String, "string");
    let number_type = new(TypeFlags::Number, "number");
    let bigint_type = new(TypeFlags::BigInt, "bigint");
    let boolean_type = new(TypeFlags::Boolean, "boolean");
    let es_symbol_type = new(TypeFlags::ESSymbol, "symbol");
    let void_type = new(TypeFlags::Void, "void");
    let undefined_type = new(TypeFlags::Undefined, "undefined");
    let null_type = new(TypeFlags::Null, "null");
    let never_type = new(TypeFlags::Never, "never");
    let non_primitive_type = new(TypeFlags::NonPrimitive, "object");

    let true_type = arena.new_type(
        TypeFlags::BooleanLiteral,
        ObjectFlags::None,
        TypeData::Literal(oxc_types::LiteralType::Boolean(true)),
        None,
    );
    let false_type = arena.new_type(
        TypeFlags::BooleanLiteral,
        ObjectFlags::None,
        TypeData::Literal(oxc_types::LiteralType::Boolean(false)),
        None,
    );

    IntrinsicIds {
        any_type,
        unknown_type,
        string_type,
        number_type,
        bigint_type,
        boolean_type,
        es_symbol_type,
        void_type,
        undefined_type,
        null_type,
        never_type,
        non_primitive_type,
        true_type,
        false_type,
    }
}

/// Pre-parsed global type information extracted from lib.d.ts.
///
/// Uses a bootstrap Checker to process lib.d.ts, so interfaces get
/// proper type parameters, method signatures, and complex types —
/// not just keyword types.
pub struct GlobalTypes {
    /// Map from global type name to TypeId.
    pub types: FxHashMap<CompactStr, TypeId>,
    /// Reverse mapping from TypeId to name for global types.
    /// Used for display since the SymbolIds from lib.d.ts are not valid
    /// in the user's Semantic.
    pub type_names: FxHashMap<TypeId, CompactStr>,
}

impl GlobalTypes {
    /// Build global types by parsing lib.es5.d.ts from disk.
    ///
    /// Allocates types into the provided arena, so TypeIds are valid
    /// for the arena's lifetime. Returns empty globals if lib.d.ts
    /// cannot be found.
    ///
    /// The filesystem walk to locate lib.es5.d.ts is cached globally
    /// (via `OnceLock`) so it only happens once, even across threads.
    /// The parsing + type extraction still happens per-call since types
    /// must be allocated into the caller's arena.
    pub fn from_lib(arena: &TypeArena, intrinsics: &IntrinsicIds) -> Self {
        let source = LIB_SOURCE_CACHE.get_or_init(Self::find_and_read_lib);
        let Some(source) = source else {
            return Self {
                types: FxHashMap::default(),
                type_names: FxHashMap::default(),
            };
        };
        Self::from_source(source, arena, intrinsics)
    }

    /// Build global types from a lib.d.ts source string.
    ///
    /// Creates a bootstrap Checker to process the source, using the full
    /// type resolution pipeline (type parameters, method signatures, etc.)
    /// instead of the old manual extraction that only handled keyword types.
    pub fn from_source(source: &str, arena: &TypeArena, intrinsics: &IntrinsicIds) -> Self {
        // Run bootstrap on a thread with a large stack.
        // In debug mode, the type resolution call chain through lib.d.ts
        // (get_type_from_type_node → get_declared_type_of_symbol →
        // resolve_declared_type → get_type_from_type_node) produces stack
        // frames that are ~10KB+ each (unoptimized), and lib.d.ts has
        // hundreds of interconnected declarations. This overflows the
        // default 8MB thread stack. Release mode optimizes frames and
        // works fine, but debug tests need more stack.
        //
        // Using std::thread::scope (Rust 1.63+) to borrow local references
        // across the thread boundary safely.
        Self::from_source_impl(source, arena, intrinsics)
    }

    fn from_source_impl(source: &str, arena: &TypeArena, intrinsics: &IntrinsicIds) -> Self {
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_semantic::SemanticBuilder;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source_type = SourceType::d_ts();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let semantic = SemanticBuilder::new().build(&parsed.program).semantic;

        // Create a bootstrap Checker with empty global_types.
        // Type references within lib.d.ts resolve via the symbol table
        // (from SemanticBuilder), not via get_global_type.
        let mut bootstrap = crate::Checker::new_inner(
            semantic,
            arena,
            FxHashMap::default(),
            FxHashMap::default(),
            None,
            *intrinsics,
            arena.len() as u32,
        );

        // Collect root scope symbols
        let root = bootstrap.semantic().scoping().root_scope_id();
        let symbols: Vec<oxc_syntax::symbol::SymbolId> = bootstrap
            .semantic()
            .scoping()
            .iter_bindings_in(root)
            .collect();

        let mut types = FxHashMap::default();
        for symbol_id in symbols {
            let name = bootstrap
                .semantic()
                .scoping()
                .symbol_name(symbol_id)
                .to_string();
            let type_id = bootstrap.get_declared_type_of_symbol(symbol_id);
            if type_id != bootstrap.any_type {
                types.insert(CompactStr::new(&name), type_id);
            }
        }

        // Create clean copies without stale bootstrap SymbolIds.
        // The bootstrap Semantic is dropped after this function returns,
        // so any SymbolIds from it would point into the wrong Semantic.
        // The ~50 extra type slots are allocated once at startup.
        let clean_types: FxHashMap<CompactStr, TypeId> = types
            .into_iter()
            .map(|(name, type_id)| {
                let flags = arena.get_flags(type_id);
                let obj_flags = arena.get_object_flags(type_id);
                let data = arena.get_data(type_id).clone();
                let clean_id = arena.new_type(flags, obj_flags, data, None);
                (name, clean_id)
            })
            .collect();
        let types = clean_types;

        // Build reverse name mapping for display purposes.
        let type_names: FxHashMap<TypeId, CompactStr> = types
            .iter()
            .map(|(name, &type_id)| (type_id, name.clone()))
            .collect();

        Self { types, type_names }
    }

    /// Try to find lib.es5.d.ts on disk.
    fn find_and_read_lib() -> Option<String> {
        // Walk up from current directory looking for node_modules/typescript
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let candidate = dir
                .join("node_modules")
                .join("typescript")
                .join("lib")
                .join("lib.es5.d.ts");
            if candidate.exists() {
                return std::fs::read_to_string(candidate).ok();
            }
            // Also check pnpm structure
            let pnpm_dir = dir.join("node_modules").join(".pnpm");
            if pnpm_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&pnpm_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("typescript@") {
                            let candidate = entry
                                .path()
                                .join("node_modules")
                                .join("typescript")
                                .join("lib")
                                .join("lib.es5.d.ts");
                            if candidate.exists() {
                                return std::fs::read_to_string(candidate).ok();
                            }
                        }
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }
}
