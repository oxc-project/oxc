use std::borrow::Cow;

use syn::parse_quote;

use super::{define_pass, AstType, Pass, Result};
use crate::{codegen::EarlyCtx, rust_ast::Inherit, util::NormalizeError};

pub trait Unresolved {
    fn unresolved(&self) -> bool;

    // TODO: remove me
    #[expect(dead_code)]
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

define_pass! {
    pub struct Linker;
}

impl Pass for Linker {
    /// # Panics
    /// On invalid inheritance.
    fn each(&mut self, ty: &mut AstType, ctx: &EarlyCtx) -> crate::Result<bool> {
        // Exit early if it isn't an enum, We only link to resolve enum inheritance!
        let AstType::Enum(ty) = ty else {
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
            .map(|it| match &it {
                Inherit::Unlinked(sup) => {
                    let linkee = ctx
                        .find(&Cow::Owned(sup.to_string()))
                        .normalize_with(format!("Unknown type {sup:?}"))?;
                    let linkee = linkee.borrow();
                    let inherit_value = format!(r#""{}""#, linkee.ident().unwrap());
                    let variants = match &*linkee {
                        AstType::Enum(enum_) => {
                            if enum_.meta.inherits.unresolved() {
                                return Ok(Err(it));
                            }
                            enum_.item.variants.clone().into_iter().map(|mut v| {
                                v.attrs = vec![parse_quote!(#[inherit = #inherit_value])];
                                v
                            })
                        }
                        _ => {
                            panic!(
                                "invalid inheritance, you can only inherit from enums and in enums."
                            )
                        }
                    };
                    ty.item.variants.extend(variants.clone());
                    Ok(Ok(Inherit::Linked {
                        super_: linkee.as_type().unwrap(),
                        variants: variants.collect(),
                    }))
                }
                Inherit::Linked { .. } => Ok(Ok(it)),
            })
            .collect::<Result<Vec<std::result::Result<Inherit, Inherit>>>>()?;
        let unresolved = inherits.iter().any(std::result::Result::is_err);

        ty.meta.inherits = inherits.into_iter().map(|it| it.unwrap_or_else(|it| it)).collect();

        Ok(!unresolved)
    }
}
