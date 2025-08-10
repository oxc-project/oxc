use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_ecmascript::ToBoolean;
use oxc_span::SPAN;

use super::GlobalReferenceInformation;

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
