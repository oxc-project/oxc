use smallvec::SmallVec;

use crate::TypeId;

/// A type mapper: maps type parameters to concrete types.
///
/// Mirrors tsgo's `TypeMapper` (mapper.go). We use a Rust enum
/// instead of Go's interface dispatch (see checker_architecture.md §7).
///
/// All variants are flat (no Box/heap indirection). Clone is a cheap
/// SmallVec stack copy for ≤4 type parameters (the common case).
#[derive(Debug, Clone)]
pub enum TypeMapper {
    /// Single substitution: one type parameter → one type argument.
    /// Covers: `Array<string>`, `Promise<number>`.
    Simple { source: TypeId, target: TypeId },

    /// Multiple substitutions: N type parameters → N type arguments.
    /// Covers: `Map<string, number>`, `Record<K, V>`, and composed mappers.
    Array { sources: SmallVec<[TypeId; 4]>, targets: SmallVec<[TypeId; 4]> },
}

impl TypeMapper {
    /// Create a mapper from type parameter list and type argument list.
    pub fn from_type_parameters(type_params: &[TypeId], type_args: &[TypeId]) -> Option<Self> {
        if type_params.is_empty() || type_params.len() != type_args.len() {
            return None;
        }

        if type_params.len() == 1 {
            Some(Self::Simple { source: type_params[0], target: type_args[0] })
        } else {
            Some(Self::Array {
                sources: SmallVec::from_slice(type_params),
                targets: SmallVec::from_slice(type_args),
            })
        }
    }

    /// Map a type through this mapper. Returns `None` if the type
    /// is not a type parameter in this mapper (meaning "leave it alone").
    pub fn map(&self, t: TypeId) -> Option<TypeId> {
        match self {
            Self::Simple { source, target } => {
                if t == *source {
                    Some(*target)
                } else {
                    None
                }
            }
            Self::Array { sources, targets } => {
                sources.iter().position(|s| *s == t).map(|i| targets[i])
            }
        }
    }

    /// Return a new mapper with an additional or overridden mapping.
    /// If `source` already exists in the mapper, its target is replaced.
    /// Otherwise the mapping is appended.
    ///
    /// Clone is cheap (SmallVec stack copy for ≤4 params). No heap allocation.
    pub fn with_mapping(self, source: TypeId, target: TypeId) -> Self {
        match self {
            Self::Simple { source: s, target: t } => {
                if s == source {
                    Self::Simple { source, target }
                } else {
                    Self::Array {
                        sources: smallvec::smallvec![s, source],
                        targets: smallvec::smallvec![t, target],
                    }
                }
            }
            Self::Array { mut sources, mut targets } => {
                if let Some(pos) = sources.iter().position(|&s| s == source) {
                    targets[pos] = target;
                } else {
                    sources.push(source);
                    targets.push(target);
                }
                Self::Array { sources, targets }
            }
        }
    }
}
