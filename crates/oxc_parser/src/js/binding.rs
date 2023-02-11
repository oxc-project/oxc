use oxc_allocator::Box;
use oxc_ast::{ast::*, Node};
use oxc_diagnostics::{Diagnostic, Result};

use super::list::{ArrayPatternList, ObjectPatternProperties};
use crate::lexer::Kind;
use crate::list::SeparatedList;
use crate::Parser;

impl<'a> Parser<'a> {
    /// Destructuring Binding Patterns
    /// `LexicalBinding`
    ///     `BindingIdentifier` `Initializer_opt`
    ///     `BindingPattern` Initializer
    /// `BindingPattern`:
    ///     `ObjectBindingPattern`
    ///     `ArrayBindingPattern`
    pub fn parse_binding_pattern(&mut self) -> Result<(BindingPattern<'a>, bool)> {
        let kind = match self.cur_kind() {
            Kind::LCurly => self.parse_object_binding_pattern(),
            Kind::LBrack => self.parse_array_binding_pattern(),
            _ => self.parse_binding_pattern_identifier(),
        }?;
        if self.ts_enabled() {
            let optional = self.eat(Kind::Question);
            let (type_annotation, definite) = self.parse_ts_variable_annotation()?;
            Ok((self.ast.binding_pattern(kind, type_annotation, optional), definite))
        } else {
            Ok((self.ast.binding_pattern(kind, None, false), false))
        }
    }

    fn parse_binding_pattern_identifier(&mut self) -> Result<BindingPatternKind<'a>> {
        self.parse_binding_identifier().map(|ident| self.ast.binding_identifier(ident))
    }

    /// Section 14.3.3 Object Binding Pattern
    fn parse_object_binding_pattern(&mut self) -> Result<BindingPatternKind<'a>> {
        let node = self.start_node();
        let properties = ObjectPatternProperties::parse(self)?.elements;
        Ok(self.ast.object_pattern(self.end_node(node), properties))
    }

    /// Section 14.3.3 Array Binding Pattern
    fn parse_array_binding_pattern(&mut self) -> Result<BindingPatternKind<'a>> {
        let node = self.start_node();
        let elements = ArrayPatternList::parse(self)?.elements;
        Ok(self.ast.array_pattern(self.end_node(node), elements))
    }

    /// Section 14.3.3 Binding Rest Property
    pub fn parse_rest_element(&mut self) -> Result<Box<'a, RestElement<'a>>> {
        let node = self.start_node();
        self.bump_any(); // advance `...`
        let argument = self.parse_binding_element()?;
        let node = self.end_node(node);

        if self.at(Kind::Comma) {
            let error = if self.peek_at(Kind::RBrack) {
                Diagnostic::RestElementTraillingComma(self.cur_token().range())
            } else {
                Diagnostic::RestElement(node.range())
            };
            self.error(error);
        }

        Ok(self.ast.rest_element(node, argument))
    }

    /// `BindingElement`
    ///     `SingleNameBinding`
    ///     `BindingPattern` Initializer
    pub fn parse_binding_element(&mut self) -> Result<BindingPattern<'a>> {
        let node = self.start_node();
        let pattern = self.parse_binding_pattern()?.0;
        self.parse_initializer(node, pattern)
    }

    // object pattern property only has kind: init and method: false
    // https://github.com/oxc_ast/oxc_ast/blob/master/es2015.md#objectpattern
    pub fn parse_object_pattern_property(&mut self) -> Result<Property<'a>> {
        let node = self.start_node();

        let mut shorthand = false;
        let is_binding_identifier = self.cur_kind().is_binding_identifier();
        let (key, computed) = self.parse_property_name()?;

        let value = if is_binding_identifier && !self.at(Kind::Colon) {
            // let { a = b } = c
            // let { a } = b
            //       ^ BindingIdentifier
            if let PropertyKey::Identifier(ident) = &key {
                shorthand = true;
                let binding_identifier =
                    BindingIdentifier { node: ident.node, name: ident.name.clone() };
                let identifier = self.ast.binding_identifier(binding_identifier);
                let left = self.ast.binding_pattern(identifier, None, false);
                PropertyValue::Pattern(self.parse_initializer(node, left)?)
            } else {
                return self.unexpected();
            }
        } else {
            // let { a: b } = c
            //       ^ IdentifierReference
            self.expect(Kind::Colon)?;
            PropertyValue::Pattern(self.parse_binding_element()?)
        };

        Ok(Property {
            node: self.end_node(node),
            key,
            value,
            kind: PropertyKind::Init,
            method: false,
            shorthand,
            computed,
        })
    }

    /// Initializer[In, Yield, Await] :
    ///   = `AssignmentExpression`[?In, ?Yield, ?Await]
    fn parse_initializer(
        &mut self,
        node: Node,
        left: BindingPattern<'a>,
    ) -> Result<BindingPattern<'a>> {
        if self.eat(Kind::Eq) {
            let expr = self.parse_assignment_expression_base()?;
            Ok(self.ast.assignment_pattern(self.end_node(node), left, expr))
        } else {
            Ok(left)
        }
    }
}
