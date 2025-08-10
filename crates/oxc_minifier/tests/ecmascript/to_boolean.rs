use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{GlobalContext, ToBoolean};
use oxc_span::SPAN;

struct GlobalReferenceInformation {
    is_undefined_shadowed: bool,
}

impl<'a> GlobalContext<'a> for GlobalReferenceInformation {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> Option<bool> {
        if ident.name == "undefined" { Some(!self.is_undefined_shadowed) } else { None }
    }
}

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let undefined = ast.expression_identifier(SPAN, "undefined");
    let shadowed_undefined_bool =
        undefined.to_boolean(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_bool =
        undefined.to_boolean(&GlobalReferenceInformation { is_undefined_shadowed: false });

    assert_eq!(shadowed_undefined_bool, None);
    assert_eq!(global_undefined_bool, Some(false));
}
