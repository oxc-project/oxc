use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{ToNumber, is_global_reference::IsGlobalReference};
use oxc_span::SPAN;

struct GlobalReferenceInformation {
    is_undefined_shadowed: bool,
}

impl IsGlobalReference for GlobalReferenceInformation {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> Option<bool> {
        if ident.name == "undefined" { Some(!self.is_undefined_shadowed) } else { None }
    }
}

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let undefined = ast.expression_identifier(SPAN, "undefined");
    let shadowed_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: false });

    assert_eq!(shadowed_undefined_number, None);
    assert!(global_undefined_number.is_some_and(f64::is_nan));
}
