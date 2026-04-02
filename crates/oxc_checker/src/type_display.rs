use oxc_checker_host::CheckerHost;
use oxc_types::{ObjectFlags, StructuredTypeKind, TypeArena, TypeData, TypeId};

use crate::Checker;

/// Standalone type formatter that can work without a live Checker.
///
/// Used for post-check type display (diagnostics, LSP hover, conformance
/// harness) where only the arena and host are needed — no mutable Checker
/// state required.
///
/// Construct via `TypePrinter::new()` (from Project) or
/// `Checker::type_printer()` (during checking).
pub struct TypePrinter<'a> {
    arena: &'a TypeArena,
    host: &'a dyn CheckerHost,
    /// When checking a specific file, symbol lookups for the current file
    /// can go directly through the Semantic (faster than the host indirection).
    local_file: Option<(u16, &'a oxc_semantic::Semantic<'a>)>,
    array_type: TypeId,
    any_type: TypeId,
}

impl<'a> TypePrinter<'a> {
    /// Create a TypePrinter for post-check use (no local file optimization).
    /// All symbol lookups go through the host.
    pub fn new(
        arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        array_type: TypeId,
        any_type: TypeId,
    ) -> Self {
        Self { arena, host, local_file: None, array_type, any_type }
    }

    /// Create a TypePrinter with local file optimization for during-check use.
    pub(crate) fn with_local_file(
        arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        file_idx: u16,
        semantic: &'a oxc_semantic::Semantic<'a>,
        array_type: TypeId,
        any_type: TypeId,
    ) -> Self {
        Self { arena, host, local_file: Some((file_idx, semantic)), array_type, any_type }
    }

    fn symbol_name(
        &self,
        file_idx: u16,
        symbol_id: oxc_syntax::symbol::SymbolId,
    ) -> Option<String> {
        if let Some((local_idx, semantic)) = self.local_file {
            if file_idx == local_idx {
                return Some(semantic.scoping().symbol_name(symbol_id).to_string());
            }
        }
        self.host.get_symbol_name(file_idx, symbol_id).map(|s| s.to_string())
    }

    fn resolve_alias_name(&self, type_id: TypeId) -> Option<String> {
        let (file_idx, symbol_id) = self.arena.get_alias_symbol(type_id)?;
        self.symbol_name(file_idx, symbol_id)
    }

    fn resolve_symbol_name(&self, type_id: TypeId) -> Option<String> {
        let (file_idx, symbol_id) = self.arena.get_symbol(type_id)?;
        self.symbol_name(file_idx, symbol_id)
    }

    fn type_params_to_string(&self, type_params: &[TypeId]) -> String {
        if type_params.is_empty() {
            return String::new();
        }
        let params = type_params
            .iter()
            .map(|&tp_id| {
                let mut s = self.type_to_string(tp_id);
                if let TypeData::TypeParameter(tp) = self.arena.get_data(tp_id) {
                    if let Some(constraint) = tp.constraint {
                        s.push_str(" extends ");
                        s.push_str(&self.type_to_string(constraint));
                    }
                    if let Some(default) = tp.resolved_default_type {
                        s.push_str(" = ");
                        s.push_str(&self.type_to_string(default));
                    }
                }
                s
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("<{params}>")
    }

    /// Convert a `TypeId` to its string representation, matching tsc's output.
    pub fn type_to_string(&self, type_id: TypeId) -> String {
        match self.arena.get_data(type_id) {
            TypeData::Intrinsic(t) => t.intrinsic_name.to_string(),
            TypeData::Literal(lit) => match lit {
                oxc_types::LiteralType::String(s) => format!("\"{s}\""),
                oxc_types::LiteralType::Number(n) => {
                    if n == &f64::INFINITY {
                        "Infinity".to_string()
                    } else if n == &f64::NEG_INFINITY {
                        "-Infinity".to_string()
                    } else {
                        let s = n.to_string();
                        if s.ends_with(".0") { s[..s.len() - 2].to_string() } else { s }
                    }
                }
                oxc_types::LiteralType::BigInt(s) => format!("{s}n"),
                oxc_types::LiteralType::Boolean(b) => b.to_string(),
            },
            TypeData::Union(u) => {
                if let Some(name) =
                    self.resolve_alias_name(type_id).or_else(|| self.resolve_symbol_name(type_id))
                {
                    return name;
                }
                u.types.iter().map(|&t| self.type_to_string(t)).collect::<Vec<_>>().join(" | ")
            }
            TypeData::Intersection(i) => {
                if let Some(name) = self.resolve_alias_name(type_id) {
                    return name;
                }
                i.types.iter().map(|&t| self.type_to_string(t)).collect::<Vec<_>>().join(" & ")
            }
            TypeData::Structured(s) => {
                if let Some(name) = self.resolve_alias_name(type_id) {
                    return name;
                }
                if let Some(name) = self.resolve_symbol_name(type_id) {
                    if matches!(s.kind, StructuredTypeKind::Anonymous { .. }) {
                        let obj_flags = self.arena.get_object_flags(type_id);
                        if obj_flags == ObjectFlags::Anonymous {
                            return format!("typeof {name}");
                        }
                    }
                    return name;
                }
                if s.properties.is_empty() {
                    return "{}".to_string();
                }
                let props = s
                    .properties_in_decl_order()
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.type_to_string(p.type_id)))
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("{{ {}; }}", props)
            }
            TypeData::TypeReference(tr) => {
                if let Some(target) = tr.target {
                    let target_str = self.type_to_string(target);
                    if tr.resolved_type_arguments.is_empty() {
                        target_str
                    } else {
                        let is_array =
                            self.array_type != self.any_type && target == self.array_type;
                        if is_array && tr.resolved_type_arguments.len() == 1 {
                            let elem_str = self.type_to_string(tr.resolved_type_arguments[0]);
                            format!("{elem_str}[]")
                        } else {
                            let args = tr
                                .resolved_type_arguments
                                .iter()
                                .map(|&t| self.type_to_string(t))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{target_str}<{args}>")
                        }
                    }
                } else {
                    "{...}".to_string()
                }
            }
            TypeData::Tuple(tuple) => {
                let elements = tuple
                    .element_infos
                    .iter()
                    .map(|info| {
                        let type_str = self.type_to_string(info.element_type);
                        if let Some(label) = &info.label_name {
                            if info.flags.contains(oxc_types::ElementFlags::Optional) {
                                format!("{label}?: {type_str}")
                            } else if info.flags.contains(oxc_types::ElementFlags::Rest) {
                                format!("...{label}: {type_str}")
                            } else {
                                format!("{label}: {type_str}")
                            }
                        } else if info.flags.contains(oxc_types::ElementFlags::Optional) {
                            format!("{type_str}?")
                        } else if info.flags.contains(oxc_types::ElementFlags::Rest) {
                            format!("...{type_str}")
                        } else {
                            type_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{elements}]")
            }
            TypeData::Function(f) => {
                if let Some(sig) = f.signatures.first() {
                    let type_params = self.type_params_to_string(&sig.type_parameters);
                    let params = sig
                        .parameters
                        .iter()
                        .map(|p| {
                            let type_str = self.type_to_string(p.type_id);
                            if p.is_rest {
                                format!("...{}: {}", p.name, type_str)
                            } else if p.is_optional {
                                format!("{}?: {}", p.name, type_str)
                            } else {
                                format!("{}: {}", p.name, type_str)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret = self.type_to_string(sig.return_type);
                    format!("{type_params}({params}) => {ret}")
                } else {
                    "() => any".to_string()
                }
            }
            TypeData::TypeParameter(tp) => {
                if tp.is_this_type {
                    "this".to_string()
                } else if let Some(name) = &tp.name {
                    name.to_string()
                } else {
                    "{...}".to_string()
                }
            }
            _ => "{...}".to_string(),
        }
    }
}

// -- Checker delegation --

impl<'a> Checker<'a> {
    /// Create a TypePrinter for the current file.
    pub fn type_printer(&self) -> TypePrinter<'_> {
        TypePrinter::with_local_file(
            self.type_arena,
            self.host,
            self.file_idx,
            self.semantic,
            self.array_type,
            self.any_type,
        )
    }

    /// Convert a `TypeId` to its string representation.
    /// Delegates to `TypePrinter` for the actual formatting.
    pub fn type_to_string(&self, type_id: TypeId) -> String {
        self.type_printer().type_to_string(type_id)
    }
}
