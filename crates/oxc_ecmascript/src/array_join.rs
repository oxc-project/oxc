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
        // If any element contains a lone surrogate, we cannot join them as strings.
        if strings.iter().any(|s| s.iter().any(|s| s.1)) {
            return None;
        }
        strings.map(|v| {
            v.iter()
                .map(|(s, _)| AsRef::as_ref(s))
                .collect::<Vec<_>>()
                .join(separator.unwrap_or(","))
        })
    }
}
