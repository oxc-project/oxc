use oxc_ast::ast::*;

use crate::{GlobalContext, ToJsString};

pub trait ArrayJoin<'a> {
    /// `Array.prototype.join ( separator )`
    /// <https://tc39.es/ecma262/#sec-array.prototype.join>
    fn array_join(&self, ctx: &impl GlobalContext<'a>, separator: Option<&str>) -> Option<String>;
}

impl<'a> ArrayJoin<'a> for ArrayExpression<'a> {
    fn array_join(&self, ctx: &impl GlobalContext<'a>, separator: Option<&str>) -> Option<String> {
        let strings = self.elements.iter().map(|e| e.to_js_string(ctx)).collect::<Option<Vec<_>>>();
        strings
            .map(|v| v.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(separator.unwrap_or(",")))
    }
}
