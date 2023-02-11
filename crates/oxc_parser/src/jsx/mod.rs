//! [JSX](https://facebook.github.io/jsx)

#![allow(clippy::missing_errors_doc)]

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, Node};
use oxc_diagnostics::Result;

use crate::lexer::Kind;
use crate::Context;
use crate::Parser;

impl<'a> Parser<'a> {
    pub fn parse_jsx_expression(&mut self) -> Result<Expression<'a>> {
        if self.peek_at(Kind::RAngle) {
            self.parse_jsx_fragment().map(Expression::JSXFragment)
        } else {
            self.parse_jsx_element(false).map(Expression::JSXElement)
        }
    }

    /// `JSXFragment` :
    ///   < > `JSXChildren_opt` < / >
    fn parse_jsx_fragment(&mut self) -> Result<Box<'a, JSXFragment<'a>>> {
        let node = self.start_node();
        let opening_fragment = self.parse_jsx_opening_fragment(node)?;
        let children = self.parse_jsx_children()?;
        let closing_fragment = self.parse_jsx_closing_fragment()?;
        Ok(self.ast.jsx_fragment(self.end_node(node), opening_fragment, closing_fragment, children))
    }

    /// <>
    fn parse_jsx_opening_fragment(&mut self, node: Node) -> Result<JSXOpeningFragment> {
        self.expect(Kind::LAngle)?;
        self.expect_jsx_child(Kind::RAngle)?;
        Ok(self.ast.jsx_opening_fragment(self.end_node(node)))
    }

    /// </>
    fn parse_jsx_closing_fragment(&mut self) -> Result<JSXClosingFragment> {
        let node = self.start_node();
        self.expect(Kind::LAngle)?;
        self.expect(Kind::Slash)?;
        self.expect(Kind::RAngle)?;
        Ok(self.ast.jsx_closing_fragment(self.end_node(node)))
    }

    /// `JSXElement` :
    ///   `JSXSelfClosingElement`
    ///   `JSXOpeningElement` `JSXChildren_opt` `JSXClosingElement`
    /// `in_jsx_child`:
    ///     used for telling `JSXClosingElement` to parse the next jsx child or not
    ///     true when inside jsx element, false when at top level expression
    fn parse_jsx_element(&mut self, in_jsx_child: bool) -> Result<Box<'a, JSXElement<'a>>> {
        let node = self.start_node();
        let opening_element = self.parse_jsx_opening_element(node, in_jsx_child)?;
        let children = if opening_element.self_closing {
            self.ast.new_vec()
        } else {
            self.parse_jsx_children()?
        };
        let closing_element = (!opening_element.self_closing)
            .then(|| self.parse_jsx_closing_element(in_jsx_child))
            .transpose()?;
        Ok(self.ast.jsx_element(self.end_node(node), opening_element, closing_element, children))
    }

    /// `JSXOpeningElement` :
    /// < `JSXElementName` `JSXAttributes_opt` >
    fn parse_jsx_opening_element(
        &mut self,
        node: Node,
        in_jsx_child: bool,
    ) -> Result<Box<'a, JSXOpeningElement<'a>>> {
        self.expect(Kind::LAngle)?;
        let name = self.parse_jsx_element_name()?;
        // <Component<TsType> for tsx
        let type_parameters = if self.ts_enabled() {
            let ctx = self.ctx;
            self.ctx = Context::default();
            let args = self.parse_ts_type_arguments()?;
            self.ctx = ctx;
            args
        } else {
            None
        };
        let attributes = self.parse_jsx_attributes()?;
        let self_closing = self.eat(Kind::Slash);
        if !self_closing || in_jsx_child {
            self.expect_jsx_child(Kind::RAngle)?;
        } else {
            self.expect(Kind::RAngle)?;
        }
        Ok(self.ast.jsx_opening_element(
            self.end_node(node),
            self_closing,
            name,
            attributes,
            type_parameters,
        ))
    }

    fn parse_jsx_closing_element(
        &mut self,
        in_jsx_child: bool,
    ) -> Result<Box<'a, JSXClosingElement<'a>>> {
        let node = self.start_node();
        self.expect(Kind::LAngle)?;
        self.expect(Kind::Slash)?;
        let name = self.parse_jsx_element_name()?;
        if in_jsx_child {
            self.expect_jsx_child(Kind::RAngle)?;
        } else {
            self.expect(Kind::RAngle)?;
        }
        Ok(self.ast.jsx_closing_element(self.end_node(node), name))
    }

    /// `JSXElementName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    ///   `JSXMemberExpression`
    fn parse_jsx_element_name(&mut self) -> Result<JSXElementName<'a>> {
        let node = self.start_node();
        let identifier = self.parse_jsx_identifier()?;

        // <namespace:property />
        if self.eat(Kind::Colon) {
            let property = self.parse_jsx_identifier()?;
            return Ok(JSXElementName::NamespacedName(self.ast.jsx_namespaced_name(
                self.end_node(node),
                identifier,
                property,
            )));
        }

        // <member.foo.bar />
        if self.at(Kind::Dot) {
            return self
                .parse_jsx_member_expression(node, identifier)
                .map(JSXElementName::MemberExpression);
        }

        Ok(JSXElementName::Identifier(identifier))
    }

    /// `JSXMemberExpression` :
    /// `JSXIdentifier` . `JSXIdentifier`
    /// `JSXMemberExpression` . `JSXIdentifier`
    fn parse_jsx_member_expression(
        &mut self,
        node: Node,
        object: JSXIdentifier,
    ) -> Result<Box<'a, JSXMemberExpression<'a>>> {
        let mut node = node;
        let mut object = JSXMemberExpressionObject::Identifier(object);
        let mut property = None;

        while self.eat(Kind::Dot) && !self.at(Kind::Eof) {
            // <foo.bar.baz>
            if let Some(prop) = property {
                let obj = self.ast.jsx_member_expression(node, object, prop);
                object = JSXMemberExpressionObject::MemberExpression(obj);
            }

            // <foo.bar>
            property = Some(self.parse_jsx_identifier()?);
            node = self.end_node(node);
        }

        if let Some(property) = property {
            return Ok(self.ast.jsx_member_expression(self.end_node(node), object, property));
        }

        self.unexpected()
    }

    /// `JSXChildren` :
    ///   `JSXChild` `JSXChildren_opt`
    fn parse_jsx_children(&mut self) -> Result<Vec<'a, JSXChild<'a>>> {
        let mut children = self.ast.new_vec();
        while !self.at(Kind::Eof) {
            if let Some(child) = self.parse_jsx_child()? {
                children.push(child);
            } else {
                break;
            }
        }
        Ok(children)
    }

    /// `JSXChild` :
    ///   `JSXText`
    ///   `JSXElement`
    ///   `JSXFragment`
    ///   { `JSXChildExpression_opt` }
    fn parse_jsx_child(&mut self) -> Result<Option<JSXChild<'a>>> {
        match self.cur_kind() {
            // </ close fragment
            Kind::LAngle if self.peek_at(Kind::Slash) => Ok(None),
            // <> open fragment
            Kind::LAngle if self.peek_at(Kind::RAngle) => {
                self.parse_jsx_fragment().map(JSXChild::Fragment).map(Some)
            }
            // <ident open element
            Kind::LAngle if self.peek_at(Kind::Ident) || self.peek_kind().is_all_keyword() => {
                self.parse_jsx_element(true).map(JSXChild::Element).map(Some)
            }
            // {...expr}
            Kind::LCurly if self.peek_at(Kind::Dot3) => {
                self.parse_jsx_spread_child().map(JSXChild::Spread).map(Some)
            }
            // {expr}
            Kind::LCurly => self
                .parse_jsx_expression_container(true)
                .map(JSXChild::ExpressionContainer)
                .map(Some),
            // text
            Kind::JSXText => Ok(Some(JSXChild::Text(self.parse_jsx_text()))),
            _ => self.unexpected(),
        }
    }

    ///   { `JSXChildExpression_opt` }
    fn parse_jsx_expression_container(
        &mut self,
        in_jsx_child: bool,
    ) -> Result<JSXExpressionContainer<'a>> {
        let node = self.start_node();
        self.bump_any(); // bump `{`
        let expr = match self.cur_kind() {
            // {} empty
            Kind::RCurly => {
                let node = self.start_node();
                JSXExpression::EmptyExpression(self.ast.jsx_empty_expression(self.end_node(node)))
            }
            // {expr}
            _ => self.parse_jsx_assignment_expression().map(JSXExpression::Expression)?,
        };
        if in_jsx_child {
            self.expect_jsx_child(Kind::RCurly)?;
        } else {
            self.expect(Kind::RCurly)?;
        }
        Ok(self.ast.jsx_expression_container(self.end_node(node), expr))
    }

    fn parse_jsx_assignment_expression(&mut self) -> Result<Expression<'a>> {
        // TODO: check for SequenceExpression and warn
        // "JSX expressions may not use the comma operator. Did you meant to write an array?"
        let ctx = self.ctx;
        self.ctx = Context::default();
        let expr = self.parse_expression();
        self.ctx = ctx;
        expr
    }

    /// `JSXChildExpression` :
    ///   { ... `AssignmentExpression` }
    fn parse_jsx_spread_child(&mut self) -> Result<JSXSpreadChild<'a>> {
        let node = self.start_node();
        self.bump_any(); // bump `{`
        self.expect(Kind::Dot3)?;
        let expr = self.parse_jsx_assignment_expression()?;
        self.expect_jsx_child(Kind::RCurly)?;
        Ok(self.ast.jsx_spread_child(self.end_node(node), expr))
    }

    /// `JSXAttributes` :
    ///   `JSXSpreadAttribute` `JSXAttributes_opt`
    ///   `JSXAttribute` `JSXAttributes_opt`
    fn parse_jsx_attributes(&mut self) -> Result<Vec<'a, JSXAttributeItem<'a>>> {
        let mut attributes = self.ast.new_vec();
        while !matches!(self.cur_kind(), Kind::Eof | Kind::LAngle | Kind::RAngle | Kind::Slash) {
            let attribute = match self.cur_kind() {
                Kind::LCurly => {
                    self.parse_jsx_spread_attribute().map(JSXAttributeItem::SpreadAttribute)
                }
                _ => self.parse_jsx_attribute().map(JSXAttributeItem::Attribute),
            }?;
            attributes.push(attribute);
        }
        Ok(attributes)
    }

    /// `JSXAttribute` :
    ///   `JSXAttributeName` `JSXAttributeInitializer_opt`
    fn parse_jsx_attribute(&mut self) -> Result<Box<'a, JSXAttribute<'a>>> {
        let node = self.start_node();
        let name = self.parse_jsx_attribute_name()?;
        let value = if self.at(Kind::Eq) {
            self.expect_jsx_attribute_value(Kind::Eq)?;
            Some(self.parse_jsx_attribute_value()?)
        } else {
            None
        };
        Ok(self.ast.jsx_attribute(self.end_node(node), name, value))
    }

    /// `JSXSpreadAttribute` :
    ///   { ... `AssignmentExpression` }
    fn parse_jsx_spread_attribute(&mut self) -> Result<Box<'a, JSXSpreadAttribute<'a>>> {
        let node = self.start_node();
        self.bump_any(); // bump `{`
        self.expect(Kind::Dot3)?;
        let argument = self.parse_jsx_assignment_expression()?;
        self.expect(Kind::RCurly)?;
        Ok(self.ast.jsx_spread_attribute(self.end_node(node), argument))
    }

    /// `JSXAttributeName` :
    ///   `JSXIdentifier`
    ///   `JSXNamespacedName`
    fn parse_jsx_attribute_name(&mut self) -> Result<JSXAttributeName<'a>> {
        let node = self.start_node();
        let identifier = self.parse_jsx_identifier()?;

        if self.eat(Kind::Colon) {
            let property = self.parse_jsx_identifier()?;
            return Ok(JSXAttributeName::NamespacedName(self.ast.jsx_namespaced_name(
                self.end_node(node),
                identifier,
                property,
            )));
        }

        Ok(JSXAttributeName::Identifier(identifier))
    }

    fn parse_jsx_attribute_value(&mut self) -> Result<JSXAttributeValue<'a>> {
        match self.cur_kind() {
            Kind::Str => self.parse_literal_string().map(JSXAttributeValue::StringLiteral),
            Kind::LCurly => {
                let expr = self.parse_jsx_expression_container(false)?;
                Ok(JSXAttributeValue::ExpressionContainer(expr))
            }
            _ => self.unexpected(),
        }
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn parse_jsx_identifier(&mut self) -> Result<JSXIdentifier> {
        let node = self.start_node();
        if !self.at(Kind::Ident) && !self.cur_kind().is_all_keyword() {
            return self.unexpected();
        }
        // we are at a valid normal Ident or Keyword, let's keep on lexing for `-`
        self.re_lex_jsx_identifier();
        let name = self.cur_atom().unwrap().clone();
        self.bump_any();
        Ok(self.ast.jsx_identifier(self.end_node(node), name))
    }

    fn parse_jsx_text(&mut self) -> JSXText {
        let node = self.start_node();
        let value = self.cur_atom().unwrap().clone();
        self.bump_any();
        self.ast.jsx_text(self.end_node(node), value)
    }
}
