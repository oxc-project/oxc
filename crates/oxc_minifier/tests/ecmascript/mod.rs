mod array_join;
mod is_int32_or_uint32;
mod may_have_side_effects;
mod prop_name;
mod to_boolean;
mod to_number;
mod to_string;
mod value_type;

use oxc_ast::ast::IdentifierReference;
use oxc_ecmascript::GlobalContext;

struct GlobalReferenceInformation {
    is_undefined_shadowed: bool,
}

impl<'a> GlobalContext<'a> for GlobalReferenceInformation {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        if ident.name == "undefined" { !self.is_undefined_shadowed } else { false }
    }
}
