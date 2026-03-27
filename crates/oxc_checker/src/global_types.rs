use std::collections::HashMap;

use oxc_ast::ast::{Program, Statement, TSSignature, TSType};
use oxc_span::CompactStr;
use oxc_types::{InterfaceType, ObjectFlags, TypeArena, TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

/// Pre-parsed global type information extracted from lib.d.ts.
///
/// The lib.d.ts AST is not retained — only the extracted type
/// information (allocated in the checker's TypeArena) survives.
pub struct GlobalTypes {
    /// Map from global type name to TypeId.
    pub types: HashMap<CompactStr, TypeId>,
}

impl GlobalTypes {
    /// Build global types by parsing lib.es5.d.ts from disk.
    ///
    /// Allocates types into the provided arena (which becomes the
    /// checker's arena), so TypeIds are valid for the checker's lifetime.
    /// Returns empty globals if lib.d.ts cannot be found.
    pub fn from_lib(arena: &mut TypeArena) -> Self {
        // Try to find lib.es5.d.ts via common locations
        let source = Self::find_and_read_lib();
        let Some(source) = source else {
            return Self { types: HashMap::new() };
        };
        Self::from_source(&source, arena)
    }

    /// Build global types from a lib.d.ts source string.
    pub fn from_source(source: &str, arena: &mut TypeArena) -> Self {
        use oxc_allocator::Allocator;
        use oxc_parser::Parser;
        use oxc_span::SourceType;

        let allocator = Allocator::default();
        let source_type = SourceType::d_ts();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let mut types = HashMap::new();
        extract_declarations(&parsed.program, arena, &mut types);
        Self { types }
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

/// Extract type declarations from a parsed program into the arena.
fn extract_declarations(
    program: &Program<'_>,
    arena: &mut TypeArena,
    types: &mut HashMap<CompactStr, TypeId>,
) {
    for stmt in &program.body {
        match stmt {
            Statement::TSInterfaceDeclaration(decl) => {
                let name = CompactStr::new(decl.id.name.as_str());
                let mut properties = HashMap::new();

                for sig in &decl.body.body {
                    if let TSSignature::TSPropertySignature(prop) = sig {
                        if let Some(prop_name) = prop.key.static_name() {
                            let prop_type = if let Some(ann) = &prop.type_annotation {
                                resolve_simple_type(&ann.type_annotation, arena, types)
                            } else {
                                // Will be resolved later or stays as a placeholder
                                arena.new_type(
                                    TypeFlags::Any,
                                    ObjectFlags::None,
                                    TypeData::Intrinsic(oxc_types::IntrinsicType {
                                        intrinsic_name: "any",
                                    }),
                                    None,
                                )
                            };
                            properties
                                .insert(CompactStr::new(&prop_name), prop_type);
                        }
                    }
                    // Skip method signatures, index signatures, etc. for now
                }

                let type_id = arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Interface,
                    TypeData::Interface(InterfaceType {
                        target: None,
                        resolved_type_arguments: SmallVec::new(),
                        all_type_parameters: SmallVec::new(),
                        this_type: None,
                        resolved_base_types: SmallVec::new(),
                        properties,
                    }),
                    None,
                );
                types.insert(name, type_id);
            }
            Statement::TSTypeAliasDeclaration(decl) => {
                let name = CompactStr::new(decl.id.name.as_str());
                let type_id =
                    resolve_simple_type(&decl.type_annotation, arena, types);
                types.insert(name, type_id);
            }
            // Skip variable declarations, function declarations, etc.
            // They live in the value namespace.
            _ => {}
        }
    }
}

/// Resolve a simple type annotation to a TypeId during lib.d.ts extraction.
///
/// This is a limited resolver that handles keyword types and simple references.
/// Complex types (generics, mapped types, etc.) fall back to `any`.
fn resolve_simple_type(
    ts_type: &TSType<'_>,
    arena: &mut TypeArena,
    _types: &HashMap<CompactStr, TypeId>,
) -> TypeId {
    // For lib.d.ts extraction, we just need basic keyword types.
    // Properties with complex types get `any` for now.
    match ts_type {
        TSType::TSStringKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::String, "string"),
        TSType::TSNumberKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::Number, "number"),
        TSType::TSBooleanKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::Boolean, "boolean")
        }
        TSType::TSAnyKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::Any, "any"),
        TSType::TSVoidKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::Void, "void"),
        TSType::TSUndefinedKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::Undefined, "undefined")
        }
        TSType::TSNullKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::Null, "null"),
        TSType::TSNeverKeyword(_) => find_or_create_intrinsic(arena, TypeFlags::Never, "never"),
        TSType::TSUnknownKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::Unknown, "unknown")
        }
        TSType::TSBigIntKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::BigInt, "bigint")
        }
        TSType::TSSymbolKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::ESSymbol, "symbol")
        }
        TSType::TSObjectKeyword(_) => {
            find_or_create_intrinsic(arena, TypeFlags::NonPrimitive, "object")
        }
        _ => {
            // Complex type — return any for now
            find_or_create_intrinsic(arena, TypeFlags::Any, "any")
        }
    }
}

/// Create an intrinsic type in the arena.
/// Note: these won't be deduplicated with the checker's pre-allocated intrinsics,
/// but that's fine — they'll be matched by TypeFlags, not identity.
fn find_or_create_intrinsic(
    arena: &mut TypeArena,
    flags: TypeFlags,
    name: &'static str,
) -> TypeId {
    arena.new_type(
        flags,
        ObjectFlags::None,
        TypeData::Intrinsic(oxc_types::IntrinsicType {
            intrinsic_name: name,
        }),
        None,
    )
}
