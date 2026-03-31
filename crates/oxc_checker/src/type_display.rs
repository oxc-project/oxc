use oxc_types::{ObjectFlags, TypeData, TypeId};

use crate::Checker;

impl Checker<'_> {
    /// Safely look up a symbol name. Returns None if the SymbolId is out of
    /// bounds (e.g., from lib.d.ts bootstrap, not valid in user's semantic).
    fn try_symbol_name(&self, symbol_id: oxc_syntax::symbol::SymbolId) -> Option<&str> {
        if symbol_id.index() < self.semantic().scoping().symbols_len() {
            Some(self.semantic().scoping().symbol_name(symbol_id))
        } else {
            None
        }
    }

    /// Convert a `TypeId` to its string representation, matching tsc's output.
    ///
    /// For example: `"string"`, `"number"`, `"true"`, `"string | number"`.
    pub fn type_to_string(&self, type_id: TypeId) -> String {
        match self.type_arena().get_data(type_id) {
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
                        // Remove trailing ".0" to match tsc output (e.g., "42" not "42.0")
                        if s.ends_with(".0") {
                            s[..s.len() - 2].to_string()
                        } else {
                            s
                        }
                    }
                }
                oxc_types::LiteralType::BigInt(s) => format!("{s}n"),
                oxc_types::LiteralType::Boolean(b) => b.to_string(),
            },
            TypeData::Union(u) => {
                // Named unions (e.g., enums) display by name
                if let Some(name) = self.global_type_names.get(&type_id) {
                    return name.to_string();
                }
                if let Some(symbol_id) = self.type_arena().get_symbol(type_id) {
                    if let Some(name) = self.try_symbol_name(symbol_id) {
                        return name.to_string();
                    }
                }
                u.types
                    .iter()
                    .map(|&t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
            TypeData::Intersection(i) => {
                i.types
                    .iter()
                    .map(|&t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(" & ")
            }
            TypeData::Object(_) => {
                if let Some(name) = self.global_type_names.get(&type_id) {
                    return name.to_string();
                }
                if let Some(symbol_id) = self.type_arena().get_symbol(type_id) {
                    if let Some(name) = self.try_symbol_name(symbol_id) {
                        let obj_flags = self.type_arena().get_object_flags(type_id);
                        // Anonymous object types with a class/function/enum symbol display
                        // as "typeof X" — these represent the constructor/namespace value.
                        if obj_flags == ObjectFlags::Anonymous {
                            return format!("typeof {name}");
                        }
                        return name.to_string();
                    }
                }
                // Anonymous object — display structurally
                let TypeData::Object(obj) = self.type_arena().get_data(type_id) else {
                    unreachable!()
                };
                if obj.properties.is_empty() {
                    return "{}".to_string();
                }
                let props = obj
                    .properties
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.type_to_string(p.type_id)))
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("{{ {}; }}", props)
            }
            TypeData::Interface(_) => {
                // Named types (classes, interfaces) display by name
                if let Some(name) = self.global_type_names.get(&type_id) {
                    return name.to_string();
                }
                if let Some(symbol_id) = self.type_arena().get_symbol(type_id) {
                    if let Some(name) = self.try_symbol_name(symbol_id) {
                        return name.to_string();
                    }
                }
                // Anonymous interface — display structurally
                let TypeData::Interface(iface) = self.type_arena().get_data(type_id) else {
                    unreachable!()
                };
                if iface.properties.is_empty() {
                    return "{}".to_string();
                }
                let props = iface
                    .properties
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
                        // Check if this is Array<T> — display as T[]
                        let is_array = self.array_type != self.any_type
                            && target == self.array_type;
                        if is_array && tr.resolved_type_arguments.len() == 1 {
                            let elem_str =
                                self.type_to_string(tr.resolved_type_arguments[0]);
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
                    format!("({params}) => {ret}")
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
            // TODO: implement display for remaining type variants
            _ => "{...}".to_string(),
        }
    }
}
