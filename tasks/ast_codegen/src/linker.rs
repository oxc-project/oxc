use std::collections::VecDeque;

use syn::parse_quote;

use super::{CodegenCtx, Cow, Inherit, Itertools, RType, Result};

pub trait Linker: Sized {
    fn link(self, linker: impl FnMut(&mut RType, &Self) -> Result<bool>) -> Result<Self>;
}

pub trait Unresolved {
    fn unresolved(&self) -> bool;

    // TODO: remove me
    #[allow(dead_code)]
    fn resolved(&self) -> bool {
        !self.unresolved()
    }
}

impl Unresolved for Inherit {
    fn unresolved(&self) -> bool {
        matches!(self, Self::Unlinked(_))
    }
}

impl Unresolved for Vec<Inherit> {
    fn unresolved(&self) -> bool {
        self.iter().any(Unresolved::unresolved)
    }
}

impl Linker for CodegenCtx {
    fn link(self, mut linker: impl FnMut(&mut RType, &Self) -> Result<bool>) -> Result<Self> {
        // we sort by `TypeId` so we always have the same ordering as how it is written in the rust.
        let mut unresolved = self
            .ident_table
            .iter()
            .sorted_by_key(|it| it.1)
            .map(|it| it.0)
            .collect::<VecDeque<_>>();

        while let Some(next) = unresolved.pop_back() {
            let next_id = *self.type_id(next).unwrap();

            let val = &mut self.ty_table[next_id].borrow_mut();

            if !linker(val, &self)? {
                // for now we don't have entangled dependencies so we just add unresolved item back
                // to the list so we revisit it again at the end.
                unresolved.push_front(next);
            }
        }
        Ok(self)
    }
}

/// Returns false if can't resolve at the moment
/// # Panics
/// On invalid inheritance.
#[allow(clippy::unnecessary_wraps)]
pub fn linker(ty: &mut RType, ctx: &CodegenCtx) -> Result<bool> {
    // Exit early if it isn't an enum, We only link to resolve enum inheritance!
    let RType::Enum(ty) = ty else {
        return Ok(true);
    };

    // Exit early if there is this enum doesn't use enum inheritance
    if ty.meta.inherits.is_empty() {
        return Ok(true);
    }

    let inherits = ty
        .meta
        .inherits
        .drain(..)
        .map(|it| match it {
            Inherit::Unlinked(ref sup) => {
                let linkee = ctx.find(&Cow::Owned(sup.to_string())).unwrap();
                let linkee = linkee.borrow();
                let inherit_value = format!(r#""{}""#, linkee.ident().unwrap());
                let variants = match &*linkee {
                    RType::Enum(enum_) => {
                        if enum_.meta.inherits.unresolved() {
                            return Err(it);
                        }
                        enum_.item.variants.clone().into_iter().map(|mut v| {
                            v.attrs = vec![parse_quote!(#[inherit = #inherit_value])];
                            v
                        })
                    }
                    _ => {
                        panic!("invalid inheritance, you can only inherit from enums and in enums.")
                    }
                };
                ty.item.variants.extend(variants.clone());
                Ok(Inherit::Linked {
                    super_: linkee.as_type().unwrap(),
                    variants: variants.collect(),
                })
            }
            Inherit::Linked { .. } => Ok(it),
        })
        .collect::<Vec<std::result::Result<Inherit, Inherit>>>();
    let unresolved = inherits.iter().any(std::result::Result::is_err);

    ty.meta.inherits = inherits.into_iter().map(|it| it.unwrap_or_else(|it| it)).collect();

    Ok(!unresolved)
}
