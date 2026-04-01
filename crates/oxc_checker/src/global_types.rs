use std::sync::OnceLock;

use oxc_checker_host::IntrinsicIds;
use oxc_types::{IntrinsicType, ObjectFlags, TypeArena, TypeData, TypeFlags};

/// Cached lib.d.ts source text. The filesystem walk to find lib.es5.d.ts
/// is done at most once (even across threads). Each caller still parses
/// the source into their own arena.
static LIB_SOURCE_CACHE: OnceLock<Option<String>> = OnceLock::new();

/// Get the lib.d.ts source text, finding and caching it from disk if needed.
///
/// Used by `Project` (which checks lib.d.ts as a regular file).
pub fn find_lib_source() -> Option<String> {
    LIB_SOURCE_CACHE
        .get_or_init(find_and_read_lib)
        .clone()
}

/// Allocate all intrinsic types in the arena. Call once during Project
/// construction before any checkers are created.
pub fn allocate_intrinsics(arena: &TypeArena) -> IntrinsicIds {
    let new = |flags: TypeFlags, name: &'static str| {
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

/// Try to find lib.es5.d.ts on disk.
fn find_and_read_lib() -> Option<String> {
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
