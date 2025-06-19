use oxc_ast::ast::*;

use crate::{ToJsString, is_global_reference::IsGlobalReference};

pub trait ArrayJoin<'a> {
    /// `Array.prototype.join ( separator )`
    /// <https://tc39.es/ecma262/#sec-array.prototype.join>
    fn array_join(
        &self,
        is_global_reference: &impl IsGlobalReference<'a>,
        separator: Option<&str>,
    ) -> Option<String>;
}

impl<'a> ArrayJoin<'a> for ArrayExpression<'a> {
    fn array_join(
        &self,
        is_global_reference: &impl IsGlobalReference<'a>,
        separator: Option<&str>,
    ) -> Option<String> {
        let strings = self
            .elements
            .iter()
            .map(|e| e.to_js_string(is_global_reference))
            .collect::<Option<Vec<_>>>();
        strings
            .map(|v| v.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(separator.unwrap_or(",")))
    }
}
