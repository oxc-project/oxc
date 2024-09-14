use super::{define_pass, AstType, Pass};
use crate::codegen::EarlyCtx;
use syn::{parse_quote, Fields};

define_pass! {
    pub struct FakeNodeId;
}

impl Pass for FakeNodeId {
    fn each(&mut self, ty: &mut AstType, _: &EarlyCtx) -> crate::Result<bool> {
        match ty {
            AstType::Struct(struct_) if struct_.meta.visitable => {
                if let Fields::Named(fields) = &mut struct_.item.fields {
                    fields.named.insert(0, parse_quote!(pub node_id: NodeId));
                }
            }
            _ => {}
        }
        Ok(true)
    }
}
