//! [`JSDoc`](https://github.com/microsoft/TypeScript/blob/54a554d8af2657630307cbfa8a3e4f3946e36507/src/compiler/types.ts#L393)

use serde::Serialize;

use crate::{Node, TSType};

#[derive(Debug, Serialize, PartialEq, Hash)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSDocNullableType<'a> {
    #[serde(flatten)]
    pub node: Node,
    pub type_annotation: TSType<'a>,
    pub postfix: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct JSDocUnknownType {
    #[serde(flatten)]
    pub node: Node,
}
