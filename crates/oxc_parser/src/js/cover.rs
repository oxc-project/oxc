//! Convert a shared expression/arrow-parameter prefix into binding patterns.

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{ParserConfig as Config, ParserImpl, diagnostics};

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Formal parameters store their initializer separately from their binding pattern.
    /// Nested defaults, in contrast, are represented by `AssignmentPattern`.
    pub(super) fn cover_arrow_parameter(
        &mut self,
        expression: Expression<'a>,
    ) -> (BindingPattern<'a>, Option<ArenaBox<'a, Expression<'a>>>) {
        if let Expression::AssignmentExpression(assignment) = expression {
            let assignment = assignment.unbox();
            if self.state.cover_parenthesized.contains(&assignment.span.start)
                || assignment.operator != AssignmentOperator::Assign
            {
                return (self.fatal_error(diagnostics::unexpected_token(assignment.span)), None);
            }
            let pattern = self.cover_binding_target(assignment.left);
            (pattern, Some(self.alloc(assignment.right)))
        } else {
            (self.cover_binding_pattern(expression), None)
        }
    }

    /// Only binding positions are converted. Computed property keys, initializers, and
    /// their nested function bodies retain their already-parsed expression ASTs.
    pub(super) fn cover_binding_pattern(
        &mut self,
        expression: Expression<'a>,
    ) -> BindingPattern<'a> {
        if self.state.cover_parenthesized.contains(&expression.span().start) {
            return self.fatal_error(diagnostics::unexpected_token(expression.span()));
        }
        match expression {
            Expression::Identifier(identifier) => self.cover_binding_identifier(&identifier),
            Expression::ArrayExpression(array) => self.cover_array_binding(array.unbox()),
            Expression::ObjectExpression(object) => self.cover_object_binding(object.unbox()),
            Expression::AssignmentExpression(assignment) => {
                let assignment = assignment.unbox();
                if assignment.operator != AssignmentOperator::Assign {
                    return self.fatal_error(diagnostics::unexpected_token(assignment.span));
                }
                let left = self.cover_binding_target(assignment.left);
                BindingPattern::new_assignment_pattern(
                    assignment.span,
                    left,
                    assignment.right,
                    self,
                )
            }
            // Parenthesized expressions, members, and TS expression wrappers cannot bind names.
            expression => self.fatal_error(diagnostics::unexpected_token(expression.span())),
        }
    }

    fn cover_binding_identifier(
        &mut self,
        identifier: &IdentifierReference<'a>,
    ) -> BindingPattern<'a> {
        // Expression parsing already checks the caller's context. An async-arrow commitment can
        // introduce an Await restriction; avoid repeating diagnostics from the original context.
        if identifier.name == "await" && self.ctx.has_await() {
            let reported = self.errors.iter().any(|error| {
                matches!(error, diagnostics::ParserDiagnostic::identifier_async { span, .. }
                    if *span == identifier.span)
            });
            if !reported {
                self.error(diagnostics::identifier_async("await", identifier.span));
            }
        } else if identifier.name == "yield" && self.ctx.has_yield() {
            let reported = self.errors.iter().any(|error| {
                matches!(error, diagnostics::ParserDiagnostic::identifier_generator { span, .. }
                    if *span == identifier.span)
            });
            if !reported {
                self.error(diagnostics::identifier_generator("yield", identifier.span, false));
            }
        }
        BindingPattern::new_binding_identifier(identifier.span, identifier.name, self)
    }

    fn cover_array_binding(&mut self, array: ArrayExpression<'a>) -> BindingPattern<'a> {
        let len = array.elements.len();
        let capacity = len
            - usize::from(matches!(
                array.elements.last(),
                Some(ArrayExpressionElement::SpreadElement(_))
            ));
        let mut elements = ArenaVec::with_capacity_in(capacity, self);
        let mut rest = None;

        for (index, element) in array.elements.into_iter().enumerate() {
            match element {
                ArrayExpressionElement::Elision(_) => elements.push(None),
                ArrayExpressionElement::SpreadElement(spread) => {
                    if index + 1 != len {
                        return self
                            .fatal_error(diagnostics::binding_rest_element_last(spread.span));
                    }
                    let spread = spread.unbox();
                    let argument = self.cover_binding_pattern(spread.argument);
                    if let BindingPattern::AssignmentPattern(assignment) = &argument {
                        self.error(diagnostics::a_rest_element_cannot_have_an_initializer(
                            assignment.span,
                        ));
                    }
                    rest = Some(BindingRestElement::boxed(spread.span, argument, self));
                    self.cover_binding_trailing_comma(array.span.start);
                }
                match_expression!(ArrayExpressionElement) => {
                    elements.push(Some(self.cover_binding_pattern(element.into_expression())));
                }
            }
        }

        BindingPattern::new_array_pattern(array.span, elements, rest, self)
    }

    fn cover_object_binding(&mut self, object: ObjectExpression<'a>) -> BindingPattern<'a> {
        let len = object.properties.len();
        let capacity = len
            - usize::from(matches!(
                object.properties.last(),
                Some(ObjectPropertyKind::SpreadProperty(_))
            ));
        let mut properties = ArenaVec::with_capacity_in(capacity, self);
        let mut rest = None;

        for (index, property) in object.properties.into_iter().enumerate() {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let property = property.unbox();
                    if property.method || property.kind != PropertyKind::Init {
                        return self.fatal_error(diagnostics::unexpected_token(property.span));
                    }
                    let mut value = self.cover_binding_pattern(property.value);
                    if property.shorthand
                        && let Some(cover) =
                            self.state.cover_initialized_name.remove(&property.span.start)
                    {
                        value = BindingPattern::new_assignment_pattern(
                            cover.span,
                            value,
                            cover.initializer,
                            self,
                        );
                    }
                    properties.push(BindingProperty::new(
                        property.span,
                        property.key,
                        value,
                        property.shorthand,
                        property.computed,
                        self,
                    ));
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if index + 1 != len {
                        return self
                            .fatal_error(diagnostics::binding_rest_element_last(spread.span));
                    }
                    let spread = spread.unbox();
                    let argument = self.cover_binding_pattern(spread.argument);
                    if let BindingPattern::AssignmentPattern(assignment) = &argument {
                        self.error(diagnostics::a_rest_element_cannot_have_an_initializer(
                            assignment.span,
                        ));
                    }
                    if !matches!(argument, BindingPattern::BindingIdentifier(_)) {
                        return self.fatal_error(diagnostics::invalid_binding_rest_element(
                            argument.span(),
                        ));
                    }
                    rest = Some(BindingRestElement::boxed(spread.span, argument, self));
                    self.cover_binding_trailing_comma(object.span.start);
                }
            }
        }

        BindingPattern::new_object_pattern(object.span, properties, rest, self)
    }

    fn cover_binding_trailing_comma(&mut self, start: u32) {
        if let Some(span) = self.state.trailing_commas.remove(&start)
            && !self.ctx.has_ambient()
        {
            self.error(diagnostics::rest_element_trailing_comma(span));
        }
    }

    /// Assignment parsing has already converted the left side of a default to an assignment
    /// target. Binding patterns are stricter: member targets and TS wrappers are not permitted.
    fn cover_binding_target(&mut self, target: AssignmentTarget<'a>) -> BindingPattern<'a> {
        if self.state.cover_parenthesized.contains(&target.span().start) {
            return self.fatal_error(diagnostics::unexpected_token(target.span()));
        }
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.cover_binding_identifier(&identifier)
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                let array = array.unbox();
                let mut elements = ArenaVec::with_capacity_in(array.elements.len(), self);
                for element in array.elements {
                    elements
                        .push(element.map(|element| self.cover_binding_target_default(element)));
                }
                let rest = array.rest.map(|rest| {
                    let rest = rest.unbox();
                    let argument = self.cover_binding_target(rest.target);
                    BindingRestElement::boxed(rest.span, argument, self)
                });
                BindingPattern::new_array_pattern(array.span, elements, rest, self)
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                let object = object.unbox();
                let mut properties = ArenaVec::with_capacity_in(object.properties.len(), self);
                for property in object.properties {
                    properties.push(self.cover_binding_target_property(property));
                }
                let rest = object.rest.map(|rest| {
                    let rest = rest.unbox();
                    let argument = self.cover_binding_target(rest.target);
                    if !matches!(argument, BindingPattern::BindingIdentifier(_)) {
                        return self.fatal_error(diagnostics::invalid_binding_rest_element(
                            argument.span(),
                        ));
                    }
                    BindingRestElement::boxed(rest.span, argument, self)
                });
                BindingPattern::new_object_pattern(object.span, properties, rest, self)
            }
            target => self.fatal_error(diagnostics::unexpected_token(target.span())),
        }
    }

    fn cover_binding_target_default(
        &mut self,
        target: AssignmentTargetMaybeDefault<'a>,
    ) -> BindingPattern<'a> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(assignment) => {
                let assignment = assignment.unbox();
                let left = self.cover_binding_target(assignment.binding);
                BindingPattern::new_assignment_pattern(assignment.span, left, assignment.init, self)
            }
            match_assignment_target!(AssignmentTargetMaybeDefault) => {
                self.cover_binding_target(target.into_assignment_target())
            }
        }
    }

    fn cover_binding_target_property(
        &mut self,
        property: AssignmentTargetProperty<'a>,
    ) -> BindingProperty<'a> {
        match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                let property = property.unbox();
                let key = IdentifierName::boxed(property.binding.span, property.binding.name, self);
                let mut value = self.cover_binding_identifier(&property.binding);
                if let Some(initializer) = property.init {
                    value = BindingPattern::new_assignment_pattern(
                        property.span,
                        value,
                        initializer,
                        self,
                    );
                }
                BindingProperty::new(
                    property.span,
                    PropertyKey::StaticIdentifier(key),
                    value,
                    true,
                    false,
                    self,
                )
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                let property = property.unbox();
                let value = self.cover_binding_target_default(property.binding);
                BindingProperty::new(
                    property.span,
                    property.name,
                    value,
                    false,
                    property.computed,
                    self,
                )
            }
        }
    }
}
