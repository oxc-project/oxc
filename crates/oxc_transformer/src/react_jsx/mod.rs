mod options;

use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};

pub use self::options::{ReactJsxOptions, ReactJsxRuntime};

/// Transform React JSX
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
pub struct ReactJsx<'a> {
    ast: Rc<AstBuilder<'a>>,
    options: ReactJsxOptions,

    has_jsx: bool,
}

impl<'a> ReactJsx<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: ReactJsxOptions) -> Self {
        Self { ast, options, has_jsx: false }
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        if let Expression::JSXElement(e) = expr {
            self.has_jsx = true;
            if let Some(e) = self.transform_jsx_element(e) {
                *expr = e;
            }
        }
    }

    pub fn add_react_jsx_runtime_import(&self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.options.runtime.is_classic() || !self.has_jsx {
            return;
        }

        let mut specifiers = self.ast.new_vec_with_capacity(1);
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
            span: SPAN,
            imported: ModuleExportName::Identifier(IdentifierName::new(SPAN, "jsx".into())),
            local: BindingIdentifier::new(SPAN, "_jsx".into()),
            import_kind: ImportOrExportKind::Value,
        }));
        let source = StringLiteral::new(SPAN, "react/jsx-runtime".into());
        let import_statement = self.ast.import_declaration(
            SPAN,
            Some(specifiers),
            source,
            None,
            ImportOrExportKind::Value,
        );
        let decl =
            self.ast.module_declaration(ModuleDeclaration::ImportDeclaration(import_statement));
        stmts.insert(0, decl);
    }

    fn transform_jsx_element(&self, e: &JSXElement<'a>) -> Option<Expression<'a>> {
        let callee = self.transform_create_element();

        let mut arguments = self.ast.new_vec_with_capacity(2 + e.children.len());
        arguments.push(Argument::Expression(self.transform_element_name(&e.opening_element.name)?));
        arguments.push(Argument::Expression(
            self.transform_jsx_attributes(&e.opening_element.attributes)?,
        ));
        arguments.extend(
            e.children
                .iter()
                .filter_map(|child| self.transform_jsx_child(child))
                .map(Argument::Expression),
        );

        Some(self.ast.call_expression(SPAN, callee, arguments, false, None))
    }

    fn transform_create_element(&self) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                // React
                let object = IdentifierReference::new(SPAN, "React".into());
                let object = self.ast.identifier_reference_expression(object);

                // React.createElement
                let property = IdentifierName::new(SPAN, "createElement".into());
                self.ast.static_member_expression(SPAN, object, property, false)
            }
            ReactJsxRuntime::Automatic => {
                let ident = IdentifierReference::new(SPAN, "_jsx".into());
                self.ast.identifier_reference_expression(ident)
            }
        }
    }

    fn transform_element_name(&self, name: &JSXElementName<'a>) -> Option<Expression<'a>> {
        match name {
            JSXElementName::Identifier(ident) => {
                let name = ident.name.clone();
                Some(if ident.name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                    self.ast.literal_string_expression(StringLiteral::new(SPAN, name))
                } else {
                    self.ast.identifier_reference_expression(IdentifierReference::new(SPAN, name))
                })
            }
            JSXElementName::MemberExpression(member_expr) => {
                Some(self.transform_jsx_member_expression(member_expr))
            }
            JSXElementName::NamespacedName(namespaced_name) => {
                if self.options.throw_if_namespace.is_some_and(|v| !v) {
                    // If the flag "throwIfNamespace" is false
                    // print XMLNamespace like string literal
                    let string_literal = StringLiteral::new(
                        SPAN,
                        Atom::from(format!(
                            "{}:{}",
                            namespaced_name.namespace.name, namespaced_name.property.name
                        )),
                    );

                    return Some(self.ast.literal_string_expression(string_literal));
                }
                None
            }
        }
    }

    fn transform_jsx_member_expression(&self, expr: &JSXMemberExpression<'a>) -> Expression<'a> {
        let object = match &expr.object {
            JSXMemberExpressionObject::Identifier(ident) => {
                self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    ident.name.clone(),
                ))
            }
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.transform_jsx_member_expression(expr)
            }
        };
        let property = IdentifierName::new(SPAN, expr.property.name.clone());
        self.ast.static_member_expression(SPAN, object, property, false)
    }

    fn transform_jsx_attributes(
        &self,
        attributes: &Vec<'a, JSXAttributeItem<'a>>,
    ) -> Option<Expression<'a>> {
        if attributes.is_empty() {
            return Some(self.ast.literal_null_expression(NullLiteral::new(SPAN)));
        }

        let mut properties = self.ast.new_vec_with_capacity(attributes.len());
        for attribute in attributes {
            let kind = PropertyKind::Init;
            let object_property = match attribute {
                JSXAttributeItem::Attribute(attr) => {
                    let key = match &attr.name {
                        JSXAttributeName::Identifier(ident) => PropertyKey::Identifier(
                            self.ast.alloc(IdentifierName::new(SPAN, ident.name.clone())),
                        ),
                        JSXAttributeName::NamespacedName(_ident) => {
                            /* TODO */
                            return None;
                        }
                    };
                    let value = match &attr.value {
                        Some(value) => {
                            match value {
                                JSXAttributeValue::StringLiteral(s) => {
                                    self.ast.literal_string_expression(s.clone())
                                }
                                JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_) => {
                                    /* TODO */
                                    return None;
                                }
                                JSXAttributeValue::ExpressionContainer(c) => {
                                    match &c.expression {
                                        JSXExpression::Expression(e) => self.ast.copy(e),
                                        JSXExpression::EmptyExpression(_e) =>
                                        /* TODO */
                                        {
                                            return None
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            self.ast.literal_boolean_expression(BooleanLiteral::new(SPAN, true))
                        }
                    };
                    let object_property =
                        self.ast.object_property(SPAN, kind, key, value, None, false, false, false);
                    ObjectPropertyKind::ObjectProperty(object_property)
                }
                JSXAttributeItem::SpreadAttribute(attr) => {
                    let argument = self.ast.copy(&attr.argument);
                    let spread_property = self.ast.spread_element(SPAN, argument);
                    ObjectPropertyKind::SpreadProperty(spread_property)
                }
            };
            properties.push(object_property);
        }

        let object_expression = self.ast.object_expression(SPAN, properties, None);
        Some(object_expression)
    }

    fn transform_jsx_child(&self, child: &JSXChild<'a>) -> Option<Expression<'a>> {
        match child {
            JSXChild::Text(text) => {
                let text = text.value.trim();
                (!text.trim().is_empty()).then(|| {
                    let text = text
                        .split(char::is_whitespace)
                        .map(str::trim)
                        .filter(|c| !c.is_empty())
                        .collect::<std::vec::Vec<_>>()
                        .join(" ");
                    let s = StringLiteral::new(SPAN, text.into());
                    self.ast.literal_string_expression(s)
                })
            }
            JSXChild::ExpressionContainer(e) => match &e.expression {
                JSXExpression::Expression(e) => Some(self.ast.copy(e)),
                JSXExpression::EmptyExpression(_) => None,
            },
            JSXChild::Element(e) => self.transform_jsx_element(e),
            _ => None,
        }
    }
}
