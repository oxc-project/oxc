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

    imports: Vec<'a, Statement<'a>>,
    import_jsx: bool,
    import_fragment: bool,
    import_create_element: bool,
}

enum JSXElementOrFragment<'a, 'b> {
    Element(&'b JSXElement<'a>),
    Fragment(&'b JSXFragment<'a>),
}

impl<'a, 'b> JSXElementOrFragment<'a, 'b> {
    fn attributes(&self) -> Option<&'b Vec<'a, JSXAttributeItem<'a>>> {
        match self {
            Self::Element(e) if !e.opening_element.attributes.is_empty() => {
                Some(&e.opening_element.attributes)
            }
            _ => None,
        }
    }

    fn children(&self) -> &'b Vec<'a, JSXChild<'a>> {
        match self {
            Self::Element(e) => &e.children,
            Self::Fragment(e) => &e.children,
        }
    }

    /// The react jsx/jsxs transform falls back to `createElement` when an explicit `key` argument comes after a spread
    /// <https://github.com/microsoft/TypeScript/blob/6134091642f57c32f50e7b5604635e4d37dd19e8/src/compiler/transformers/jsx.ts#L264-L278>
    fn has_key_after_props_spread(&self) -> bool {
        let Self::Element(e) = self else { return false };
        let mut spread = false;
        for attr in &e.opening_element.attributes {
            if matches!(attr, JSXAttributeItem::SpreadAttribute(_)) {
                spread = true;
            } else if spread {
                if let JSXAttributeItem::Attribute(a) = attr {
                    if let JSXAttributeName::Identifier(ident) = &a.name {
                        if ident.name == "key" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'a> ReactJsx<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: ReactJsxOptions) -> Self {
        let imports = ast.new_vec();
        Self {
            ast,
            options,
            imports,
            import_jsx: false,
            import_fragment: false,
            import_create_element: false,
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::JSXElement(e) => {
                if let Some(e) = self.transform_jsx(&JSXElementOrFragment::Element(e)) {
                    *expr = e;
                }
            }
            Expression::JSXFragment(e) => {
                if let Some(e) = self.transform_jsx(&JSXElementOrFragment::Fragment(e)) {
                    *expr = e;
                }
            }
            _ => {}
        }
    }

    pub fn add_react_jsx_runtime_import(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.options.runtime.is_classic() {
            return;
        }
        self.imports.extend(stmts.drain(..));
        *stmts = self.ast.move_statement_vec(&mut self.imports);
    }

    fn add_import<'b>(
        &mut self,
        e: &JSXElementOrFragment<'a, 'b>,
        has_key_after_props_spread: bool,
    ) {
        if self.options.runtime.is_classic() {
            return;
        }
        match e {
            JSXElementOrFragment::Element(_) if has_key_after_props_spread => {
                self.add_import_create_element();
            }
            JSXElementOrFragment::Element(_) => self.add_import_jsx(),
            JSXElementOrFragment::Fragment(_) => self.add_import_fragment(),
        }
    }

    fn add_import_jsx(&mut self) {
        if !self.import_jsx {
            self.import_jsx = true;
            self.add_import_statement("jsx", "_jsx", "react/jsx-runtime");
        }
    }

    fn add_import_fragment(&mut self) {
        if !self.import_fragment {
            self.import_fragment = true;
            self.add_import_statement("Fragment", "_Fragment", "react/jsx-runtime");
            self.add_import_jsx();
        }
    }

    fn add_import_create_element(&mut self) {
        if !self.import_create_element {
            self.import_create_element = true;
            self.add_import_statement("createElement", "_createElement", "react");
        }
    }

    fn add_import_statement(&mut self, imported: &str, local: &str, source: &str) {
        let mut specifiers = self.ast.new_vec_with_capacity(1);
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
            span: SPAN,
            imported: ModuleExportName::Identifier(IdentifierName::new(SPAN, imported.into())),
            local: BindingIdentifier::new(SPAN, local.into()),
            import_kind: ImportOrExportKind::Value,
        }));
        let source = StringLiteral::new(SPAN, source.into());
        let import_statement = self.ast.import_declaration(
            SPAN,
            Some(specifiers),
            source,
            None,
            ImportOrExportKind::Value,
        );
        let decl =
            self.ast.module_declaration(ModuleDeclaration::ImportDeclaration(import_statement));
        self.imports.push(decl);
    }

    fn transform_jsx<'b>(&mut self, e: &JSXElementOrFragment<'a, 'b>) -> Option<Expression<'a>> {
        let has_key_after_props_spread = e.has_key_after_props_spread();
        let callee = self.get_create_element(has_key_after_props_spread);
        let children = e.children();

        // TODO: compute the correct capacity for both runtimes
        let mut arguments = self.ast.new_vec_with_capacity(1);

        arguments.push(Argument::Expression(match e {
            JSXElementOrFragment::Element(e) => {
                self.transform_element_name(&e.opening_element.name)?
            }
            JSXElementOrFragment::Fragment(_) => self.get_fragment(),
        }));

        // TODO: compute the correct capacity for both runtimes
        let mut properties = self.ast.new_vec_with_capacity(0);
        if let Some(attributes) = e.attributes() {
            for attribute in attributes {
                let kind = PropertyKind::Init;
                match attribute {
                    JSXAttributeItem::Attribute(attr) => {
                        let key = self.get_attribute_name(&attr.name);
                        let value = match &attr.value {
                            Some(value) => {
                                match value {
                                    JSXAttributeValue::StringLiteral(s) => {
                                        self.ast.literal_string_expression(s.clone())
                                    }
                                    JSXAttributeValue::Element(_)
                                    | JSXAttributeValue::Fragment(_) => {
                                        /* TODO */
                                        continue;
                                    }
                                    JSXAttributeValue::ExpressionContainer(c) => {
                                        match &c.expression {
                                            JSXExpression::Expression(e) => self.ast.copy(e),
                                            JSXExpression::EmptyExpression(_e) =>
                                            /* TODO */
                                            {
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                self.ast.literal_boolean_expression(BooleanLiteral::new(SPAN, true))
                            }
                        };
                        let object_property = self
                            .ast
                            .object_property(SPAN, kind, key, value, None, false, false, false);
                        let object_property = ObjectPropertyKind::ObjectProperty(object_property);
                        properties.push(object_property);
                    }
                    JSXAttributeItem::SpreadAttribute(attr) => match &attr.argument {
                        Expression::ObjectExpression(expr) => {
                            for object_property in &expr.properties {
                                properties.push(self.ast.copy(object_property));
                            }
                        }
                        expr => {
                            let argument = self.ast.copy(expr);
                            let spread_property = self.ast.spread_element(SPAN, argument);
                            let object_property =
                                ObjectPropertyKind::SpreadProperty(spread_property);
                            properties.push(object_property);
                        }
                    },
                }
            }
        } else if self.options.runtime.is_classic() {
            let null_expr = self.ast.literal_null_expression(NullLiteral::new(SPAN));
            arguments.push(Argument::Expression(null_expr));
        }

        if self.options.runtime.is_automatic() && !children.is_empty() {
            let key =
                self.ast.property_key_identifier(IdentifierName::new(SPAN, "children".into()));
            let value = if children.len() == 1 {
                self.transform_jsx_child(&children[0])?
            } else {
                let mut elements = self.ast.new_vec_with_capacity(children.len());
                for child in children {
                    if let Some(e) = self.transform_jsx_child(child) {
                        elements.push(ArrayExpressionElement::Expression(e));
                    }
                }
                self.ast.array_expression(SPAN, elements, None)
            };
            let object_property = self.ast.object_property(
                SPAN,
                PropertyKind::Init,
                key,
                value,
                None,
                false,
                false,
                false,
            );
            properties.push(ObjectPropertyKind::ObjectProperty(object_property));
        }

        if !properties.is_empty() || self.options.runtime.is_automatic() {
            let object_expression = self.ast.object_expression(SPAN, properties, None);
            arguments.push(Argument::Expression(object_expression));
        }

        if self.options.runtime.is_classic() && !children.is_empty() {
            arguments.extend(
                children
                    .iter()
                    .filter_map(|child| self.transform_jsx_child(child))
                    .map(Argument::Expression),
            );
        }

        self.add_import(e, has_key_after_props_spread);

        Some(self.ast.call_expression(SPAN, callee, arguments, false, None))
    }

    fn get_react_references(&mut self) -> Expression<'a> {
        let ident = IdentifierReference::new(SPAN, "React".into());
        self.ast.identifier_reference_expression(ident)
    }

    fn get_create_element(&mut self, has_key_after_props_spread: bool) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                let object = self.get_react_references();
                let property = IdentifierName::new(SPAN, "createElement".into());
                self.ast.static_member_expression(SPAN, object, property, false)
            }
            ReactJsxRuntime::Automatic => {
                let name = if has_key_after_props_spread { "_createElement" } else { "_jsx" };
                let ident = IdentifierReference::new(SPAN, name.into());
                self.ast.identifier_reference_expression(ident)
            }
        }
    }

    fn get_fragment(&mut self) -> Expression<'a> {
        match self.options.runtime {
            ReactJsxRuntime::Classic => {
                let object = self.get_react_references();
                let property = IdentifierName::new(SPAN, "Fragment".into());
                self.ast.static_member_expression(SPAN, object, property, false)
            }
            ReactJsxRuntime::Automatic => {
                let ident = IdentifierReference::new(SPAN, "_Fragment".into());
                self.ast.identifier_reference_expression(ident)
            }
        }
    }

    fn get_attribute_name(&self, name: &JSXAttributeName<'a>) -> PropertyKey<'a> {
        match name {
            JSXAttributeName::Identifier(ident) => {
                let name = ident.name.clone();
                if ident.name.contains('-') {
                    let expr = self.ast.literal_string_expression(StringLiteral::new(SPAN, name));
                    self.ast.property_key_expression(expr)
                } else {
                    self.ast.property_key_identifier(IdentifierName::new(SPAN, name))
                }
            }
            JSXAttributeName::NamespacedName(name) => {
                let name = format!("{}:{}", name.namespace.name, name.property.name);
                let expr =
                    self.ast.literal_string_expression(StringLiteral::new(SPAN, name.into()));
                self.ast.property_key_expression(expr)
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

    fn transform_jsx_child(&mut self, child: &JSXChild<'a>) -> Option<Expression<'a>> {
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
            JSXChild::Element(e) => self.transform_jsx(&JSXElementOrFragment::Element(e)),
            JSXChild::Fragment(e) => self.transform_jsx(&JSXElementOrFragment::Fragment(e)),
            JSXChild::Spread(_) => None,
        }
    }
}
