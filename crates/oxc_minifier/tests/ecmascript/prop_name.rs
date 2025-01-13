use oxc_allocator::Allocator;
use oxc_ast::{ast::ObjectExpression, Visit};
use oxc_ecmascript::PropName;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn test_prop_name() {
    #[derive(Debug, Default)]
    struct TestVisitor;

    impl<'a> Visit<'a> for TestVisitor {
        fn visit_object_expression(&mut self, obj_expr: &ObjectExpression<'a>) {
            assert_eq!("a", obj_expr.properties[0].prop_name().unwrap().0);
            assert_eq!("b", obj_expr.properties[1].prop_name().unwrap().0);
            assert_eq!("c", obj_expr.properties[2].prop_name().unwrap().0);
            assert_eq!("d", obj_expr.properties[3].prop_name().unwrap().0);
            assert_eq!(None, obj_expr.properties[4].prop_name());
        }
    }

    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let source = r"
            const obj = {
                a() {},
                get b() {},
                set c(_) {},
                d: 1,
                [e]() {},
            }
        ";
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(!ret.program.is_empty());
    assert!(ret.errors.is_empty());

    let mut visitor = TestVisitor;
    visitor.visit_program(&ret.program);
}
