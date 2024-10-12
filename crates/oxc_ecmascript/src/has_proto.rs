use oxc_ast::ast::ObjectExpression;

use crate::PropName;

pub trait HasProto {
    /// Returns `true` if this object has a property named `__proto__`
    fn has_proto(&self) -> bool;
}

impl<'a> HasProto for ObjectExpression<'a> {
    fn has_proto(&self) -> bool {
        self.properties.iter().any(|p| p.prop_name().is_some_and(|name| name.0 == "__proto__"))
    }
}
