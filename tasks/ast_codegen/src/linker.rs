use std::collections::VecDeque;

use super::{CodegenCtx, Cow, Inherit, Itertools, RType, Result};

pub trait Linker<'a> {
    fn link(&'a self, linker: impl FnMut(&mut RType, &'a Self) -> Result<bool>) -> Result<&'a ()>;
}

impl<'a> Linker<'a> for CodegenCtx {
    fn link(
        &'a self,
        mut linker: impl FnMut(&mut RType, &'a Self) -> Result<bool>,
    ) -> Result<&'a ()> {
        let mut unresolved = self.ident_table.keys().collect::<VecDeque<_>>();
        while let Some(next) = unresolved.pop_back() {
            let next_id = *self.type_id(next).unwrap();

            let val = &mut self.ty_table[next_id].borrow_mut();

            if !linker(val, self)? {
                // for now we don't have entangled dependencies so we just add unresolved item back
                // to the list so we revisit it again at the end.
                unresolved.push_front(next);
            }
        }
        Ok(&())
    }
}

/// Returns false if can't resolve
/// TODO: right now we don't resolve nested inherits, return is always true for now.
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

    ty.meta.inherits = ty
        .meta
        .inherits
        .drain(..)
        .map(|it| match it {
            Inherit::Unlinked(it) => {
                let linkee = ctx.find(&Cow::Owned(it.to_string())).unwrap();
                let variants = match &*linkee.borrow() {
                    RType::Enum(enum_) => enum_.item.variants.clone(),
                    _ => {
                        panic!("invalid inheritance, you can only inherit from enums and in enums.")
                    }
                };
                ty.item.variants.extend(variants.clone());
                Inherit::Linked { super_: it.clone(), variants }
            }
            Inherit::Linked { .. } => it,
        })
        .collect_vec();

    Ok(true)
}
