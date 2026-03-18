use oxc_types::{TypeData, TypeId};

use crate::Checker;

impl Checker<'_> {
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
        }
    }
}
