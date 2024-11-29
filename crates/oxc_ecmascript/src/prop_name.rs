use oxc_ast::ast::{
    ClassElement, MethodDefinition, ObjectProperty, ObjectPropertyKind, PropertyDefinition,
    PropertyKey,
};
use oxc_span::Span;

/// [`PropName`](https://tc39.es/ecma262/#sec-static-semantics-propname)
pub trait PropName {
    fn prop_name(&self) -> Option<(&str, Span)>;
}

impl PropName for ObjectPropertyKind<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.prop_name(),
            ObjectPropertyKind::SpreadProperty(_) => None,
        }
    }
}

impl PropName for ObjectProperty<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.shorthand || self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl PropName for PropertyKey<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            PropertyKey::StaticIdentifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::Identifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::StringLiteral(lit) => Some((&lit.value, lit.span)),
            _ => None,
        }
    }
}

impl PropName for ClassElement<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ClassElement::MethodDefinition(def) => def.prop_name(),
            ClassElement::PropertyDefinition(def) => def.prop_name(),
            _ => None,
        }
    }
}

impl PropName for MethodDefinition<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl PropName for PropertyDefinition<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{ast::ObjectExpression, Visit};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::PropName;

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
}
