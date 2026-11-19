//! Cover grammars for destructuring: an expression that covers an assignment target or a
//! binding pattern.

use oxc_allocator::{ArenaBox, ArenaVec, Dummy, GetAllocator};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::cover::{Cover, CoverCheck, CoverElement};
use crate::{ParserConfig as Config, ParserImpl, diagnostics};

pub trait CoverGrammar<'a, T, C: Config>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a, C>) -> Self;
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::ArrayExpression(array_expr) => {
                let pat = ArrayAssignmentTarget::cover(array_expr.unbox(), p);
                AssignmentTarget::ArrayAssignmentTarget(p.alloc(pat))
            }
            Expression::ObjectExpression(object_expr) => {
                let pat = ObjectAssignmentTarget::cover(object_expr.unbox(), p);
                AssignmentTarget::ObjectAssignmentTarget(p.alloc(pat))
            }
            _ => AssignmentTarget::from(SimpleAssignmentTarget::cover(expr, p)),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for SimpleAssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::Identifier(ident) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)
            }
            match_member_expression!(Expression) => {
                let member_expr = expr.into_member_expression();
                SimpleAssignmentTarget::from(member_expr)
            }
            Expression::ParenthesizedExpression(expr) => {
                let span = expr.span;
                match expr.unbox().expression {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                        p.fatal_error(diagnostics::invalid_assignment(span))
                    }
                    expr => SimpleAssignmentTarget::cover(expr, p),
                }
            }
            Expression::TSAsExpression(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSAsExpression(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSSatisfiesExpression(expr) => {
                match expr.expression.get_inner_expression() {
                    Expression::Identifier(_)
                    | Expression::StaticMemberExpression(_)
                    | Expression::ComputedMemberExpression(_)
                    | Expression::PrivateFieldExpression(_) => {
                        SimpleAssignmentTarget::TSSatisfiesExpression(expr)
                    }
                    _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
                }
            }
            Expression::TSNonNullExpression(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSNonNullExpression(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSTypeAssertion(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSTypeAssertion(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSInstantiationExpression(expr) => {
                p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
            }
            expr => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, ArrayExpression<'a>, C> for ArrayAssignmentTarget<'a> {
    // Destructuring-target conversion is comparatively rare and large. Keeping it out of line
    // stops it being inlined into `AssignmentTarget::cover`, whose common arm (simple targets)
    // would otherwise carry this body's large stack frame + callee-saved spills on every call.
    #[inline(never)]
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let len = expr.elements.len();
        // A final spread becomes `rest`, so it needs no slot in `elements`.
        let capacity = len
            - usize::from(matches!(
                expr.elements.last(),
                Some(ArrayExpressionElement::SpreadElement(_))
            ));
        let mut elements = ArenaVec::with_capacity_in(capacity, p);
        let mut rest = None;

        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                match_expression!(ArrayExpressionElement) => {
                    let expr = elem.into_expression();
                    let target = AssignmentTargetMaybeDefault::cover(expr, p);
                    elements.push(Some(target));
                }
                ArrayExpressionElement::SpreadElement(elem) => {
                    if i == len - 1 {
                        let span = elem.span;
                        let argument = elem.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression(),
                            Expression::Identifier(_)
                                | Expression::ArrayExpression(_)
                                | Expression::ObjectExpression(_)
                                | Expression::StaticMemberExpression(_)
                                | Expression::ComputedMemberExpression(_)
                                | Expression::PrivateFieldExpression(_)
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
                        if let Some(span) = p.state.trailing_commas.remove(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(span));
                        }
                    } else {
                        let error = diagnostics::spread_last_element(elem.span);
                        return p.fatal_error(error);
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        ArrayAssignmentTarget::new(expr.span, elements, rest, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::AssignmentExpression(assignment_expr) => {
                if assignment_expr.operator != AssignmentOperator::Assign {
                    p.error(diagnostics::invalid_assignment_target_default_value_operator(
                        assignment_expr.span,
                    ));
                }
                let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p);
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.alloc(target))
            }
            expr => {
                let target = AssignmentTarget::cover(expr, p);
                AssignmentTargetMaybeDefault::from(target)
            }
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, AssignmentExpression<'a>, C>
    for AssignmentTargetWithDefault<'a>
{
    fn cover(expr: AssignmentExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        AssignmentTargetWithDefault::new(expr.span, expr.left, expr.right, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectExpression<'a>, C> for ObjectAssignmentTarget<'a> {
    // Kept out of line for the same reason as `ArrayAssignmentTarget::cover` above: avoid
    // inlining this large body into the hot `AssignmentTarget::cover` dispatcher.
    #[inline(never)]
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let len = expr.properties.len();
        // A final spread becomes `rest`, so it needs no slot in `properties`.
        let capacity = len
            - usize::from(matches!(
                expr.properties.last(),
                Some(ObjectPropertyKind::SpreadProperty(_))
            ));
        let mut properties = ArenaVec::with_capacity_in(capacity, p);
        let mut rest = None;

        for (i, elem) in expr.properties.into_iter().enumerate() {
            match elem {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let target = AssignmentTargetProperty::cover(property.unbox(), p);
                    properties.push(target);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if i == len - 1 {
                        let span = spread.span;
                        let argument = spread.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression(),
                            Expression::Identifier(_)
                                | Expression::StaticMemberExpression(_)
                                | Expression::ComputedMemberExpression(_)
                                | Expression::PrivateFieldExpression(_)
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        if let Some(span) = p.state.trailing_commas.remove(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(span));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        ObjectAssignmentTarget::new(expr.span, properties, rest, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectProperty<'a>, C> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if property.shorthand {
            let binding = match property.value {
                Expression::Identifier(ident) => ident.unbox(),
                _ => return p.unexpected(),
            };
            // convert `CoverInitializedName`
            let init = p
                .state
                .cover_initialized_name
                .remove(&property.span.start)
                .map(|cover| cover.initializer);
            AssignmentTargetProperty::new_assignment_target_property_identifier(
                property.span,
                binding,
                init,
                p,
            )
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p);
            AssignmentTargetProperty::new_assignment_target_property_property(
                property.span,
                property.key,
                binding,
                property.computed,
                p,
            )
        }
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    // `ObjectLiteral`, `ArrayLiteral` and `IdentifierReference` cover the matching binding
    // patterns. A conversion must only be given an expression that passed its check.

    /// Whether the elements of an undecided cover list can all be parameters.
    pub(super) fn is_cover_parameter_list(&self, cover: Cover) -> bool {
        let elements = &self.state.cover_elements[cover.base..];
        elements.iter().enumerate().all(|(index, element)| match element {
            CoverElement::Identifier { .. } | CoverElement::IdentifierWithInitializer { .. } => {
                true
            }
            CoverElement::LiteralWithInitializer { literal, .. } => {
                self.check_cover_binding(literal).is_ok()
            }
            // `(this): T => …` is reported as an arrow function with a `this` parameter.
            CoverElement::Expression(Expression::ThisExpression(_)) => self.is_ts && index == 0,
            CoverElement::Expression(expression) => {
                self.check_cover_binding_element(expression).is_ok()
            }
            CoverElement::Spread(spread) => {
                index == elements.len() - 1
                    && self.check_cover_binding_element(&spread.argument).is_ok()
            }
        })
    }

    /// Check that `expression` covers a `BindingElement`: a binding pattern that may have
    /// an initializer.
    pub(super) fn check_cover_binding_element(
        &self,
        expression: &Expression<'a>,
    ) -> CoverCheck<'a> {
        match expression {
            Expression::AssignmentExpression(assignment)
                if assignment.operator == AssignmentOperator::Assign
                    && !self.is_cover_parenthesized(assignment.span.start) =>
            {
                self.check_cover_binding_target(&assignment.left)
            }
            _ => self.check_cover_binding(expression),
        }
    }

    /// Check that `expression` covers a `BindingIdentifier` or a `BindingPattern`.
    pub(super) fn check_cover_binding(&self, expression: &Expression<'a>) -> CoverCheck<'a> {
        match expression {
            Expression::Identifier(ident) if !self.is_cover_parenthesized(ident.span.start) => {
                Ok(())
            }
            // `(yield) => …` in a generator. `yield` is not a valid `BindingIdentifier` there,
            // but that error is more useful than one about a `YieldExpression`.
            Expression::YieldExpression(expression)
                if expression.argument.is_none()
                    && !expression.delegate
                    && !self.is_cover_parenthesized(expression.span.start) =>
            {
                Ok(())
            }
            Expression::ObjectExpression(object)
                if !self.is_cover_parenthesized(object.span.start) =>
            {
                let last = object.properties.len().wrapping_sub(1);
                for (index, property) in object.properties.iter().enumerate() {
                    match property {
                        ObjectPropertyKind::ObjectProperty(property) => {
                            // Methods and accessors are not properties of a pattern.
                            if property.kind != PropertyKind::Init || property.method {
                                return Err(diagnostics::invalid_arrow_function_parameter(
                                    property.span,
                                ));
                            }
                            if !property.shorthand {
                                self.check_cover_binding_element(&property.value)?;
                            }
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            if index != last {
                                return Err(diagnostics::binding_rest_element_last(spread.span));
                            }
                            if !matches!(
                                &spread.argument,
                                Expression::Identifier(ident)
                                    if !self.is_cover_parenthesized(ident.span.start)
                            ) {
                                return Err(diagnostics::invalid_binding_rest_element(
                                    spread.argument.span(),
                                ));
                            }
                        }
                    }
                }
                Ok(())
            }
            Expression::ArrayExpression(array)
                if !self.is_cover_parenthesized(array.span.start) =>
            {
                let last = array.elements.len().wrapping_sub(1);
                for (index, element) in array.elements.iter().enumerate() {
                    match element {
                        ArrayExpressionElement::Elision(_) => {}
                        ArrayExpressionElement::SpreadElement(spread) => {
                            if index != last {
                                return Err(diagnostics::binding_rest_element_last(spread.span));
                            }
                            self.check_cover_binding_element(&spread.argument)?;
                        }
                        element => self.check_cover_binding_element(element.to_expression())?,
                    }
                }
                Ok(())
            }
            _ => Err(diagnostics::invalid_arrow_function_parameter(expression.span())),
        }
    }

    /// Check that an assignment target, which an expression was already refined to, covers
    /// a `BindingIdentifier` or a `BindingPattern`.
    fn check_cover_binding_target(&self, target: &AssignmentTarget<'a>) -> CoverCheck<'a> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident)
                if !self.is_cover_parenthesized(ident.span.start) =>
            {
                Ok(())
            }
            AssignmentTarget::ObjectAssignmentTarget(object)
                if !self.is_cover_parenthesized(object.span.start) =>
            {
                for property in &object.properties {
                    if let AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) =
                        property
                    {
                        self.check_cover_binding_target_maybe_default(&property.binding)?;
                    }
                }
                match &object.rest {
                    Some(rest)
                        if !matches!(
                            &rest.target,
                            AssignmentTarget::AssignmentTargetIdentifier(ident)
                                if !self.is_cover_parenthesized(ident.span.start)
                        ) =>
                    {
                        Err(diagnostics::invalid_binding_rest_element(rest.target.span()))
                    }
                    _ => Ok(()),
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(array)
                if !self.is_cover_parenthesized(array.span.start) =>
            {
                for element in array.elements.iter().flatten() {
                    self.check_cover_binding_target_maybe_default(element)?;
                }
                match &array.rest {
                    Some(rest) => self.check_cover_binding_target(&rest.target),
                    None => Ok(()),
                }
            }
            _ => Err(diagnostics::invalid_arrow_function_parameter(target.span())),
        }
    }

    fn check_cover_binding_target_maybe_default(
        &self,
        target: &AssignmentTargetMaybeDefault<'a>,
    ) -> CoverCheck<'a> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                self.check_cover_binding_target(&target.binding)
            }
            _ => self.check_cover_binding_target(target.to_assignment_target()),
        }
    }

    /// Convert an expression that covers a `BindingElement`.
    pub(super) fn cover_expression_to_binding_element(
        &mut self,
        expression: Expression<'a>,
    ) -> BindingPattern<'a> {
        match expression {
            Expression::AssignmentExpression(assignment) => {
                let AssignmentExpression { span, left, right, .. } = assignment.unbox();
                let left = self.cover_assignment_target_to_binding_pattern(left);
                BindingPattern::new_assignment_pattern(span, left, right, self)
            }
            expression => self.cover_expression_to_binding_pattern(expression),
        }
    }

    /// Convert an expression that covers a `BindingIdentifier` or a `BindingPattern`.
    pub(super) fn cover_expression_to_binding_pattern(
        &mut self,
        expression: Expression<'a>,
    ) -> BindingPattern<'a> {
        match expression {
            Expression::Identifier(ident) => {
                BindingPattern::new_binding_identifier(ident.span, ident.name, self)
            }
            Expression::YieldExpression(expression) => {
                let span = expression.span;
                self.error(diagnostics::identifier_generator("yield", span, false));
                BindingPattern::new_binding_identifier(span, self.ident("yield"), self)
            }
            Expression::ObjectExpression(object) => {
                let ObjectExpression { span, properties, .. } = object.unbox();
                let has_rest =
                    matches!(properties.last(), Some(ObjectPropertyKind::SpreadProperty(_)));
                let mut list =
                    ArenaVec::with_capacity_in(properties.len() - usize::from(has_rest), self);
                let mut rest = None;
                for property in properties {
                    match property {
                        ObjectPropertyKind::ObjectProperty(property) => {
                            let ObjectProperty { span, key, value, shorthand, computed, .. } =
                                property.unbox();
                            let value = if shorthand {
                                // `{ a }`, or the `CoverInitializedName` `{ a = b }`.
                                let binding = self.cover_expression_to_binding_pattern(value);
                                let cover_initialized_name = &mut self.state.cover_initialized_name;
                                if cover_initialized_name.is_empty() {
                                    binding
                                } else if let Some(cover) =
                                    cover_initialized_name.remove(&span.start)
                                {
                                    BindingPattern::new_assignment_pattern(
                                        span,
                                        binding,
                                        cover.initializer,
                                        self,
                                    )
                                } else {
                                    binding
                                }
                            } else {
                                self.cover_expression_to_binding_element(value)
                            };
                            list.push(BindingProperty::new(
                                span, key, value, shorthand, computed, self,
                            ));
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            rest = Some(self.cover_spread_to_binding_rest(spread, span.start));
                        }
                    }
                }
                BindingPattern::new_object_pattern(span, list, rest, self)
            }
            Expression::ArrayExpression(array) => {
                let ArrayExpression { span, elements, .. } = array.unbox();
                let has_rest =
                    matches!(elements.last(), Some(ArrayExpressionElement::SpreadElement(_)));
                let mut list =
                    ArenaVec::with_capacity_in(elements.len() - usize::from(has_rest), self);
                let mut rest = None;
                for element in elements {
                    match element {
                        ArrayExpressionElement::Elision(_) => list.push(None),
                        ArrayExpressionElement::SpreadElement(spread) => {
                            rest = Some(self.cover_spread_to_binding_rest(spread, span.start));
                        }
                        element => {
                            let element = element.into_expression();
                            list.push(Some(self.cover_expression_to_binding_element(element)));
                        }
                    }
                }
                BindingPattern::new_array_pattern(span, list, rest, self)
            }
            // The check rejects every other expression.
            _ => Dummy::dummy(self.allocator()),
        }
    }

    /// Convert the spread element that ends an object or array literal starting at
    /// `literal_start` to the rest element of a pattern.
    fn cover_spread_to_binding_rest(
        &mut self,
        spread: ArenaBox<'a, SpreadElement<'a>>,
        literal_start: u32,
    ) -> ArenaBox<'a, BindingRestElement<'a>> {
        let SpreadElement { span, argument, .. } = spread.unbox();
        let argument = self.cover_expression_to_binding_element(argument);
        if let BindingPattern::AssignmentPattern(pattern) = &argument {
            self.error(diagnostics::a_rest_element_cannot_have_an_initializer(pattern.span));
        }
        if !self.state.trailing_commas.is_empty()
            && let Some(comma_span) = self.state.trailing_commas.remove(&literal_start)
            && !self.ctx.has_ambient()
        {
            self.error(diagnostics::rest_element_trailing_comma(comma_span));
        }
        BindingRestElement::boxed(span, argument, self)
    }

    /// Convert an assignment target that covers a `BindingIdentifier` or a `BindingPattern`.
    pub(super) fn cover_assignment_target_to_binding_pattern(
        &mut self,
        target: AssignmentTarget<'a>,
    ) -> BindingPattern<'a> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                BindingPattern::new_binding_identifier(ident.span, ident.name, self)
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                let ObjectAssignmentTarget { span, properties, rest, .. } = object.unbox();
                let mut list = ArenaVec::with_capacity_in(properties.len(), self);
                for property in properties {
                    list.push(match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            let AssignmentTargetPropertyIdentifier { span, binding, init, .. } =
                                property.unbox();
                            let key = PropertyKey::StaticIdentifier(IdentifierName::boxed(
                                binding.span,
                                binding.name,
                                self,
                            ));
                            let value = BindingPattern::new_binding_identifier(
                                binding.span,
                                binding.name,
                                self,
                            );
                            let value = match init {
                                Some(init) => {
                                    BindingPattern::new_assignment_pattern(span, value, init, self)
                                }
                                None => value,
                            };
                            BindingProperty::new(span, key, value, true, false, self)
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            let AssignmentTargetPropertyProperty {
                                span,
                                name,
                                binding,
                                computed,
                                ..
                            } = property.unbox();
                            let value = self
                                .cover_assignment_target_maybe_default_to_binding_element(binding);
                            BindingProperty::new(span, name, value, false, computed, self)
                        }
                    });
                }
                let rest = rest.map(|rest| self.cover_assignment_target_rest_to_binding_rest(rest));
                BindingPattern::new_object_pattern(span, list, rest, self)
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                let ArrayAssignmentTarget { span, elements, rest, .. } = array.unbox();
                let mut list = ArenaVec::with_capacity_in(elements.len(), self);
                for element in elements {
                    list.push(element.map(|element| {
                        self.cover_assignment_target_maybe_default_to_binding_element(element)
                    }));
                }
                let rest = rest.map(|rest| self.cover_assignment_target_rest_to_binding_rest(rest));
                BindingPattern::new_array_pattern(span, list, rest, self)
            }
            // The check rejects every other target.
            _ => Dummy::dummy(self.allocator()),
        }
    }

    fn cover_assignment_target_maybe_default_to_binding_element(
        &mut self,
        target: AssignmentTargetMaybeDefault<'a>,
    ) -> BindingPattern<'a> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                let AssignmentTargetWithDefault { span, binding, init, .. } = target.unbox();
                let left = self.cover_assignment_target_to_binding_pattern(binding);
                BindingPattern::new_assignment_pattern(span, left, init, self)
            }
            target => {
                self.cover_assignment_target_to_binding_pattern(target.into_assignment_target())
            }
        }
    }

    fn cover_assignment_target_rest_to_binding_rest(
        &mut self,
        rest: ArenaBox<'a, AssignmentTargetRest<'a>>,
    ) -> ArenaBox<'a, BindingRestElement<'a>> {
        let AssignmentTargetRest { span, target, .. } = rest.unbox();
        let argument = self.cover_assignment_target_to_binding_pattern(target);
        BindingRestElement::boxed(span, argument, self)
    }
}
