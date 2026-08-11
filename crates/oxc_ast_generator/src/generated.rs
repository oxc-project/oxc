// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_generator.rs`.

//! Generated random AST implementations.

use rand::Rng;

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_str::{Ident, Str};

use crate::AstGenerator;
/// Generate an arena-backed AST value.
pub trait Generate<'a>: Sized {
    /// Generate a value.
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self;
}

impl<'a> Generate<'a> for oxc_ast::ast::Program<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Program::new(
            oxc_span::SPAN,
            generator.source_type(),
            "",
            oxc_allocator::Vec::new_in(generator.ast()),
            generator.generate::<Option<Hashbang<'a>>>(),
            oxc_allocator::Vec::new_in(generator.ast()),
            crate::custom::generate_program_body(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Expression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_expression(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::IdentifierName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::IdentifierName::new(
            oxc_span::SPAN,
            generator.generate::<Ident<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::IdentifierReference<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::IdentifierReference::new(
            oxc_span::SPAN,
            generator.generate::<Ident<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BindingIdentifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BindingIdentifier::new(
            oxc_span::SPAN,
            generator.generate::<Ident<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::LabelIdentifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::LabelIdentifier::new(
            oxc_span::SPAN,
            generator.generate::<Ident<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ThisExpression {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ThisExpression::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrayExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ArrayExpression::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, ArrayExpressionElement<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrayExpressionElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::Elision(generator.generate()),
                1u32 => generator.generate::<Expression<'a>>().into(),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::SpreadElement(generator.generate()),
                1u32 => Self::Elision(generator.generate()),
                2u32 => generator.generate::<Expression<'a>>().into(),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Elision {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Elision::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ObjectExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ObjectExpression::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, ObjectPropertyKind<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ObjectPropertyKind<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32]) {
                0u32 => Self::SpreadProperty(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::ObjectProperty(generator.generate()),
                1u32 => Self::SpreadProperty(generator.generate()),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ObjectProperty<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_object_property(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PropertyKey<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32]) {
            0u32 => Self::StaticIdentifier(generator.generate()),
            1u32 => Self::PrivateIdentifier(generator.generate()),
            2u32 => generator.generate::<Expression<'a>>().into(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PropertyKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[3u32, 1u32, 1u32]) {
            0u32 => Self::Init,
            1u32 => Self::Get,
            2u32 => Self::Set,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TemplateLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_template_literal(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TaggedTemplateExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::TaggedTemplateExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>()
            } else {
                None
            },
            generator.generate::<TemplateLiteral<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TemplateElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::TemplateElement::new(
            oxc_span::SPAN,
            generator.generate::<TemplateElementValue<'a>>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TemplateElementValue<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::TemplateElementValue {
            raw: generator.generate::<Str<'a>>(),
            cooked: generator.generate::<Option<Str<'a>>>(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::MemberExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32]) {
            0u32 => Self::ComputedMemberExpression(generator.generate()),
            1u32 => Self::StaticMemberExpression(generator.generate()),
            2u32 => Self::PrivateFieldExpression(generator.generate()),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ComputedMemberExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ComputedMemberExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::StaticMemberExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::StaticMemberExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<IdentifierName<'a>>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PrivateFieldExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::PrivateFieldExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<PrivateIdentifier<'a>>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CallExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_call_expression(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::NewExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_new_expression(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportMeta {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportMeta::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::NewTarget {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::NewTarget::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::SpreadElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::SpreadElement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Argument<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32]) {
                0u32 => generator.generate::<Expression<'a>>().into(),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::SpreadElement(generator.generate()),
                1u32 => generator.generate::<Expression<'a>>().into(),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::UpdateExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::UpdateExpression::new(
            oxc_span::SPAN,
            generator.generate::<UpdateOperator>(),
            generator.generate::<bool>(),
            generator.generate::<SimpleAssignmentTarget<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::UnaryExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::UnaryExpression::new(
            oxc_span::SPAN,
            generator.generate::<UnaryOperator>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BinaryExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BinaryExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<BinaryOperator>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PrivateInExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::PrivateInExpression::new(
            oxc_span::SPAN,
            generator.generate::<PrivateIdentifier<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::LogicalExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::LogicalExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<LogicalOperator>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ConditionalExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ConditionalExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentExpression::new(
            oxc_span::SPAN,
            generator.generate::<AssignmentOperator>(),
            generator.generate::<AssignmentTarget<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTarget<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    2u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator
                    .random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32])
                {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::TSAsExpression(generator.generate()),
                    2u32 => Self::TSSatisfiesExpression(generator.generate()),
                    3u32 => Self::TSNonNullExpression(generator.generate()),
                    4u32 => Self::TSTypeAssertion(generator.generate()),
                    5u32 => Self::ComputedMemberExpression(generator.generate()),
                    6u32 => Self::StaticMemberExpression(generator.generate()),
                    7u32 => Self::PrivateFieldExpression(generator.generate()),
                    8u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    9u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    2u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ComputedMemberExpression(generator.generate()),
                    2u32 => Self::StaticMemberExpression(generator.generate()),
                    3u32 => Self::PrivateFieldExpression(generator.generate()),
                    4u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    5u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::SimpleAssignmentTarget<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::TSAsExpression(generator.generate()),
                    2u32 => Self::TSSatisfiesExpression(generator.generate()),
                    3u32 => Self::TSNonNullExpression(generator.generate()),
                    4u32 => Self::TSTypeAssertion(generator.generate()),
                    5u32 => Self::ComputedMemberExpression(generator.generate()),
                    6u32 => Self::StaticMemberExpression(generator.generate()),
                    7u32 => Self::PrivateFieldExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ComputedMemberExpression(generator.generate()),
                    2u32 => Self::StaticMemberExpression(generator.generate()),
                    3u32 => Self::PrivateFieldExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::ArrayAssignmentTarget(generator.generate()),
            1u32 => Self::ObjectAssignmentTarget(generator.generate()),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrayAssignmentTarget<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ArrayAssignmentTarget::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>(),
            generator.generate::<Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ObjectAssignmentTarget<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ObjectAssignmentTarget::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, AssignmentTargetProperty<'a>>>(),
            generator.generate::<Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetRest<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentTargetRest::new(
            oxc_span::SPAN,
            generator.generate::<AssignmentTarget<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetMaybeDefault<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    2u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[
                    1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
                ]) {
                    0u32 => Self::AssignmentTargetWithDefault(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::TSAsExpression(generator.generate()),
                    3u32 => Self::TSSatisfiesExpression(generator.generate()),
                    4u32 => Self::TSNonNullExpression(generator.generate()),
                    5u32 => Self::TSTypeAssertion(generator.generate()),
                    6u32 => Self::ComputedMemberExpression(generator.generate()),
                    7u32 => Self::StaticMemberExpression(generator.generate()),
                    8u32 => Self::PrivateFieldExpression(generator.generate()),
                    9u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    10u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    1u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    2u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::AssignmentTargetWithDefault(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::ComputedMemberExpression(generator.generate()),
                    3u32 => Self::StaticMemberExpression(generator.generate()),
                    4u32 => Self::PrivateFieldExpression(generator.generate()),
                    5u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    6u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetWithDefault<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentTargetWithDefault::new(
            oxc_span::SPAN,
            generator.generate::<AssignmentTarget<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetProperty<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32]) {
                0u32 => Self::AssignmentTargetPropertyIdentifier(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::AssignmentTargetPropertyIdentifier(generator.generate()),
                1u32 => Self::AssignmentTargetPropertyProperty(generator.generate()),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentTargetPropertyIdentifier::new(
            oxc_span::SPAN,
            generator.generate::<IdentifierReference<'a>>(),
            generator.generate::<Option<Expression<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentTargetPropertyProperty<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentTargetPropertyProperty::new(
            oxc_span::SPAN,
            generator.generate::<PropertyKey<'a>>(),
            generator.generate::<AssignmentTargetMaybeDefault<'a>>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::SequenceExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::SequenceExpression::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, Expression<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Super {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Super::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AwaitExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AwaitExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ChainExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ChainExpression::new(
            oxc_span::SPAN,
            generator.generate::<ChainElement<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ChainElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32]) {
                    0u32 => Self::CallExpression(generator.generate()),
                    1u32 => Self::TSNonNullExpression(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::CallExpression(generator.generate()),
                    1u32 => Self::TSNonNullExpression(generator.generate()),
                    2u32 => Self::ComputedMemberExpression(generator.generate()),
                    3u32 => Self::StaticMemberExpression(generator.generate()),
                    4u32 => Self::PrivateFieldExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::CallExpression(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::CallExpression(generator.generate()),
                    1u32 => Self::ComputedMemberExpression(generator.generate()),
                    2u32 => Self::StaticMemberExpression(generator.generate()),
                    3u32 => Self::PrivateFieldExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ParenthesizedExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ParenthesizedExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Statement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_statement(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Directive<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Directive::new(
            oxc_span::SPAN,
            generator.generate::<StringLiteral<'a>>(),
            generator.generate::<Str<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Hashbang<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Hashbang::new(
            oxc_span::SPAN,
            generator.generate::<Str<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BlockStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BlockStatement::new(
            oxc_span::SPAN,
            crate::custom::generate_block_statements(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Declaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_declaration(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::VariableDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::VariableDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<VariableDeclarationKind>(),
            generator.generate::<ArenaVec<'a, VariableDeclarator<'a>>>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::VariableDeclarationKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
            0u32 => Self::Var,
            1u32 => Self::Let,
            2u32 => Self::Const,
            3u32 => Self::Using,
            4u32 => Self::AwaitUsing,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::VariableDeclarator<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::VariableDeclarator::new(
            oxc_span::SPAN,
            generator.generate::<BindingPattern<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.generate::<Option<Expression<'a>>>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::EmptyStatement {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::EmptyStatement::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExpressionStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExpressionStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::IfStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::IfStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<Statement<'a>>(),
            generator.generate::<Option<Statement<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::DoWhileStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::DoWhileStatement::new(
            oxc_span::SPAN,
            crate::custom::generate_loop_body(generator),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::WhileStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::WhileStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            crate::custom::generate_loop_body(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ForStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ForStatement::new(
            oxc_span::SPAN,
            generator.generate::<Option<ForStatementInit<'a>>>(),
            generator.generate::<Option<Expression<'a>>>(),
            generator.generate::<Option<Expression<'a>>>(),
            crate::custom::generate_loop_body(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ForStatementInit<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::VariableDeclaration(generator.generate()),
            1u32 => generator.generate::<Expression<'a>>().into(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ForInStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ForInStatement::new(
            oxc_span::SPAN,
            generator.generate::<ForStatementLeft<'a>>(),
            generator.generate::<Expression<'a>>(),
            crate::custom::generate_loop_body(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ForStatementLeft<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::VariableDeclaration(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    3u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[
                    1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
                ]) {
                    0u32 => Self::VariableDeclaration(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::TSAsExpression(generator.generate()),
                    3u32 => Self::TSSatisfiesExpression(generator.generate()),
                    4u32 => Self::TSNonNullExpression(generator.generate()),
                    5u32 => Self::TSTypeAssertion(generator.generate()),
                    6u32 => Self::ComputedMemberExpression(generator.generate()),
                    7u32 => Self::StaticMemberExpression(generator.generate()),
                    8u32 => Self::PrivateFieldExpression(generator.generate()),
                    9u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    10u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::VariableDeclaration(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    3u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::VariableDeclaration(generator.generate()),
                    1u32 => Self::AssignmentTargetIdentifier(generator.generate()),
                    2u32 => Self::ComputedMemberExpression(generator.generate()),
                    3u32 => Self::StaticMemberExpression(generator.generate()),
                    4u32 => Self::PrivateFieldExpression(generator.generate()),
                    5u32 => Self::ArrayAssignmentTarget(generator.generate()),
                    6u32 => Self::ObjectAssignmentTarget(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ForOfStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ForOfStatement::new(
            oxc_span::SPAN,
            generator.generate::<bool>(),
            generator.generate::<ForStatementLeft<'a>>(),
            generator.generate::<Expression<'a>>(),
            crate::custom::generate_loop_body(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ContinueStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ContinueStatement::new(
            oxc_span::SPAN,
            generator.generate::<Option<LabelIdentifier<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BreakStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BreakStatement::new(
            oxc_span::SPAN,
            generator.generate::<Option<LabelIdentifier<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ReturnStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ReturnStatement::new(
            oxc_span::SPAN,
            generator.generate::<Option<Expression<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::WithStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::WithStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<Statement<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::SwitchStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::SwitchStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            crate::custom::generate_switch_cases(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::SwitchCase<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::SwitchCase::new(
            oxc_span::SPAN,
            generator.generate::<Option<Expression<'a>>>(),
            crate::custom::generate_switch_consequent(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::LabeledStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::LabeledStatement::new(
            oxc_span::SPAN,
            generator.generate::<LabelIdentifier<'a>>(),
            generator.generate::<Statement<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ThrowStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ThrowStatement::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TryStatement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::TryStatement::new(
            oxc_span::SPAN,
            generator.generate::<ArenaBox<'a, BlockStatement<'a>>>(),
            generator.generate::<Option<ArenaBox<'a, CatchClause<'a>>>>(),
            generator.generate::<Option<ArenaBox<'a, BlockStatement<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CatchClause<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::CatchClause::new(
            oxc_span::SPAN,
            generator.generate::<Option<CatchParameter<'a>>>(),
            generator.generate::<ArenaBox<'a, BlockStatement<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CatchParameter<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::CatchParameter::new(
            oxc_span::SPAN,
            generator.generate::<BindingPattern<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::DebuggerStatement {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::DebuggerStatement::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BindingPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::BindingIdentifier(generator.generate()),
                1u32 => Self::ObjectPattern(generator.generate()),
                2u32 => Self::ArrayPattern(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                0u32 => Self::BindingIdentifier(generator.generate()),
                1u32 => Self::ObjectPattern(generator.generate()),
                2u32 => Self::ArrayPattern(generator.generate()),
                3u32 => Self::AssignmentPattern(generator.generate()),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AssignmentPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AssignmentPattern::new(
            oxc_span::SPAN,
            generator.generate::<BindingPattern<'a>>(),
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ObjectPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ObjectPattern::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, BindingProperty<'a>>>(),
            generator.generate::<Option<ArenaBox<'a, BindingRestElement<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BindingProperty<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BindingProperty::new(
            oxc_span::SPAN,
            generator.generate::<PropertyKey<'a>>(),
            generator.generate::<BindingPattern<'a>>(),
            generator.generate::<bool>(),
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrayPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ArrayPattern::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, Option<BindingPattern<'a>>>>(),
            generator.generate::<Option<ArenaBox<'a, BindingRestElement<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BindingRestElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BindingRestElement::new(
            oxc_span::SPAN,
            generator.generate::<BindingPattern<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Function<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_function(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FunctionType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_function_type(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FormalParameters<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::FormalParameters::new(
            oxc_span::SPAN,
            generator.generate::<FormalParameterKind>(),
            generator.generate::<ArenaVec<'a, FormalParameter<'a>>>(),
            generator.generate::<Option<ArenaBox<'a, FormalParameterRest<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FormalParameter<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::FormalParameter::new(
            oxc_span::SPAN,
            if generator.is_typescript() {
                generator.generate::<ArenaVec<'a, Decorator<'a>>>()
            } else {
                ArenaVec::<Decorator<'a>>::new_in(generator.ast())
            },
            generator.generate::<BindingPattern<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.generate::<Option<ArenaBox<'a, Expression<'a>>>>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() {
                generator.generate::<Option<TSAccessibility>>()
            } else {
                None
            },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FormalParameterKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_formal_parameter_kind(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FormalParameterRest<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::FormalParameterRest::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, Decorator<'a>>>(),
            generator.generate::<BindingRestElement<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::FunctionBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_function_body(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrowFunctionBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::FunctionBody(generator.generate()),
            1u32 => generator.generate::<Expression<'a>>().into(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ArrowFunctionExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_arrow_function(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::YieldExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_yield_expression(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Class<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Class::new(
            oxc_span::SPAN,
            generator.generate::<ClassType>(),
            generator.generate::<ArenaVec<'a, Decorator<'a>>>(),
            generator.generate::<Option<BindingIdentifier<'a>>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>()
            } else {
                None
            },
            generator.generate::<Option<ClassHeritage<'a>>>(),
            if generator.is_typescript() {
                generator.generate::<ArenaVec<'a, TSClassImplements<'a>>>()
            } else {
                ArenaVec::<TSClassImplements<'a>>::new_in(generator.ast())
            },
            generator.generate::<ArenaBox<'a, ClassBody<'a>>>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ClassHeritage<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ClassHeritage::new(
            generator.generate::<Expression<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ClassType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::ClassDeclaration,
            1u32 => Self::ClassExpression,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ClassBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ClassBody::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, ClassElement<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ClassElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::StaticBlock(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::StaticBlock(generator.generate()),
                    1u32 => Self::MethodDefinition(generator.generate()),
                    2u32 => Self::PropertyDefinition(generator.generate()),
                    3u32 => Self::AccessorProperty(generator.generate()),
                    4u32 => Self::TSIndexSignature(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::StaticBlock(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::StaticBlock(generator.generate()),
                    1u32 => Self::MethodDefinition(generator.generate()),
                    2u32 => Self::PropertyDefinition(generator.generate()),
                    3u32 => Self::AccessorProperty(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::MethodDefinition<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::MethodDefinition::new(
            oxc_span::SPAN,
            generator.generate::<MethodDefinitionType>(),
            generator.generate::<ArenaVec<'a, Decorator<'a>>>(),
            generator.generate::<PropertyKey<'a>>(),
            generator.generate::<ArenaBox<'a, Function<'a>>>(),
            generator.generate::<MethodDefinitionKind>(),
            generator.generate::<bool>(),
            generator.generate::<bool>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() {
                generator.generate::<Option<TSAccessibility>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::MethodDefinitionType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_method_definition_type(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PropertyDefinition<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::PropertyDefinition::new(
            oxc_span::SPAN,
            generator.generate::<PropertyDefinitionType>(),
            generator.generate::<ArenaVec<'a, Decorator<'a>>>(),
            generator.generate::<PropertyKey<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.generate::<Option<Expression<'a>>>(),
            generator.generate::<bool>(),
            generator.generate::<bool>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() {
                generator.generate::<Option<TSAccessibility>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PropertyDefinitionType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_property_definition_type(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::MethodDefinitionKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
            0u32 => Self::Constructor,
            1u32 => Self::Method,
            2u32 => Self::Get,
            3u32 => Self::Set,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::PrivateIdentifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::PrivateIdentifier::new(
            oxc_span::SPAN,
            generator.generate::<Ident<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::StaticBlock<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::StaticBlock::new(
            oxc_span::SPAN,
            crate::custom::generate_block_statements(generator),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ModuleDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::ExportNamedDeclaration(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::ImportDeclaration(generator.generate()),
                    1u32 => Self::ExportAllDeclaration(generator.generate()),
                    2u32 => Self::ExportDefaultDeclaration(generator.generate()),
                    3u32 => Self::ExportDeclaration(generator.generate()),
                    4u32 => Self::ExportNamedDeclaration(generator.generate()),
                    5u32 => Self::ExportFromDeclaration(generator.generate()),
                    6u32 => Self::TSExportAssignment(generator.generate()),
                    7u32 => Self::TSNamespaceExportDeclaration(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::ExportNamedDeclaration(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::ImportDeclaration(generator.generate()),
                    1u32 => Self::ExportAllDeclaration(generator.generate()),
                    2u32 => Self::ExportDefaultDeclaration(generator.generate()),
                    3u32 => Self::ExportDeclaration(generator.generate()),
                    4u32 => Self::ExportNamedDeclaration(generator.generate()),
                    5u32 => Self::ExportFromDeclaration(generator.generate()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AccessorPropertyType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_accessor_property_type(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::AccessorProperty<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::AccessorProperty::new(
            oxc_span::SPAN,
            generator.generate::<AccessorPropertyType>(),
            generator.generate::<ArenaVec<'a, Decorator<'a>>>(),
            generator.generate::<PropertyKey<'a>>(),
            if generator.is_typescript() {
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>()
            } else {
                None
            },
            generator.generate::<Option<Expression<'a>>>(),
            generator.generate::<bool>(),
            generator.generate::<bool>(),
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() { generator.generate::<bool>() } else { false },
            if generator.is_typescript() {
                generator.generate::<Option<TSAccessibility>>()
            } else {
                None
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportExpression::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.generate::<Option<Expression<'a>>>(),
            generator.generate::<Option<ImportPhase>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>>(),
            generator.generate::<StringLiteral<'a>>(),
            generator.generate::<Option<ImportPhase>>(),
            generator.generate::<Option<ArenaBox<'a, WithClause<'a>>>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportPhase {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::Source,
            1u32 => Self::Defer,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportDeclarationSpecifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.at_limit() {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::ImportDefaultSpecifier(generator.generate()),
                1u32 => Self::ImportNamespaceSpecifier(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::ImportSpecifier(generator.generate()),
                1u32 => Self::ImportDefaultSpecifier(generator.generate()),
                2u32 => Self::ImportNamespaceSpecifier(generator.generate()),
                _ => unreachable!(),
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportSpecifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportSpecifier::new(
            oxc_span::SPAN,
            generator.generate::<ModuleExportName<'a>>(),
            generator.generate::<BindingIdentifier<'a>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportDefaultSpecifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportDefaultSpecifier::new(
            oxc_span::SPAN,
            generator.generate::<BindingIdentifier<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportNamespaceSpecifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportNamespaceSpecifier::new(
            oxc_span::SPAN,
            generator.generate::<BindingIdentifier<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::WithClause<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::WithClause::new(
            oxc_span::SPAN,
            generator.generate::<WithClauseKeyword>(),
            generator.generate::<ArenaVec<'a, ImportAttribute<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::WithClauseKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::With,
            1u32 => Self::Assert,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportAttribute<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ImportAttribute::new(
            oxc_span::SPAN,
            generator.generate::<ImportAttributeKey<'a>>(),
            generator.generate::<StringLiteral<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportAttributeKey<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::Identifier(generator.generate()),
            1u32 => Self::StringLiteral(generator.generate()),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<Declaration<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportNamedDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportNamedDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, ExportSpecifier<'a>>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportFromDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportFromDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<ArenaVec<'a, ExportSpecifier<'a>>>(),
            generator.generate::<StringLiteral<'a>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.generate::<Option<ArenaBox<'a, WithClause<'a>>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportDefaultDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportDefaultDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<ExportDefaultDeclarationKind<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportAllDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportAllDeclaration::new(
            oxc_span::SPAN,
            generator.generate::<Option<ModuleExportName<'a>>>(),
            generator.generate::<StringLiteral<'a>>(),
            generator.generate::<Option<ArenaBox<'a, WithClause<'a>>>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportSpecifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::ExportSpecifier::new(
            oxc_span::SPAN,
            generator.generate::<ModuleExportName<'a>>(),
            generator.generate::<ModuleExportName<'a>>(),
            if generator.is_typescript() {
                generator.generate::<ImportOrExportKind>()
            } else {
                ImportOrExportKind::Value
            },
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ExportDefaultDeclarationKind<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => generator.generate::<Expression<'a>>().into(),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::FunctionDeclaration(generator.generate()),
                    1u32 => Self::ClassDeclaration(generator.generate()),
                    2u32 => Self::TSInterfaceDeclaration(generator.generate()),
                    3u32 => generator.generate::<Expression<'a>>().into(),
                    _ => unreachable!(),
                }
            }
        } else {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => generator.generate::<Expression<'a>>().into(),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::FunctionDeclaration(generator.generate()),
                    1u32 => Self::ClassDeclaration(generator.generate()),
                    2u32 => generator.generate::<Expression<'a>>().into(),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ModuleExportName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32]) {
            0u32 => Self::IdentifierName(generator.generate()),
            1u32 => Self::IdentifierReference(generator.generate()),
            2u32 => Self::StringLiteral(generator.generate()),
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::V8IntrinsicExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::V8IntrinsicExpression::new(
            oxc_span::SPAN,
            generator.generate::<IdentifierName<'a>>(),
            generator.generate::<ArenaVec<'a, Argument<'a>>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BooleanLiteral {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BooleanLiteral::new(
            oxc_span::SPAN,
            generator.generate::<bool>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::NullLiteral {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::NullLiteral::new(oxc_span::SPAN, generator.ast())
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::NumericLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::NumericLiteral::new(
            oxc_span::SPAN,
            generator.generate::<f64>(),
            None,
            generator.generate::<NumberBase>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::StringLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::StringLiteral::new(
            oxc_span::SPAN,
            generator.generate::<Str<'a>>(),
            None,
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::BigIntLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::BigIntLiteral::new(
            oxc_span::SPAN,
            generator.generate::<Str<'a>>(),
            None,
            generator.generate::<BigintBase>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::RegExpLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::RegExpLiteral::new(
            oxc_span::SPAN,
            generator.generate::<RegExp<'a>>(),
            None,
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::RegExp<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::RegExp {
            pattern: generator.generate::<RegExpPattern<'a>>(),
            flags: RegExpFlags::empty(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::RegExpPattern<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::RegExpPattern { text: generator.generate::<Str<'a>>(), pattern: None }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXElement generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXOpeningElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXOpeningElement generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXClosingElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXClosingElement generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXFragment<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXFragment generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXOpeningFragment {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXOpeningFragment generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXClosingFragment {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXClosingFragment generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXElementName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXElementName generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXNamespacedName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXNamespacedName generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXMemberExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXMemberExpression generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXMemberExpressionObject<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXMemberExpressionObject generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXExpressionContainer<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXExpressionContainer generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXExpression generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXEmptyExpression {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXEmptyExpression generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXAttributeItem<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXAttributeItem generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXAttribute<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXAttribute generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXSpreadAttribute<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXSpreadAttribute generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXAttributeName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXAttributeName generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXAttributeValue<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXAttributeValue generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXIdentifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXIdentifier generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXChild<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXChild generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXSpreadChild<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXSpreadChild generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSXText<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        panic!("JSXText generation is not implemented for JSX or TSX")
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSThisParameter<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSThisParameter::new(
                oxc_span::SPAN,
                oxc_span::SPAN,
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSThisParameter generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSEnumDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSEnumDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<TSEnumBody<'a>>(),
                generator.generate::<bool>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSEnumDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSEnumBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSEnumBody::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSEnumMember<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSEnumBody generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSEnumMember<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSEnumMember::new(
                oxc_span::SPAN,
                generator.generate::<TSEnumMemberName<'a>>(),
                generator.generate::<Option<Expression<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSEnumMember generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSEnumMemberName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                0u32 => Self::Identifier(generator.generate()),
                1u32 => Self::String(generator.generate()),
                2u32 => Self::ComputedString(generator.generate()),
                3u32 => Self::ComputedTemplateString(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            panic!("TSEnumMemberName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeAnnotation<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeAnnotation::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeAnnotation generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSLiteralType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSLiteralType::new(
                oxc_span::SPAN,
                generator.generate::<TSLiteral<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSLiteralType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::BooleanLiteral(generator.generate()),
                    1u32 => Self::NumericLiteral(generator.generate()),
                    2u32 => Self::BigIntLiteral(generator.generate()),
                    3u32 => Self::StringLiteral(generator.generate()),
                    4u32 => Self::TemplateLiteral(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::BooleanLiteral(generator.generate()),
                    1u32 => Self::NumericLiteral(generator.generate()),
                    2u32 => Self::BigIntLiteral(generator.generate()),
                    3u32 => Self::StringLiteral(generator.generate()),
                    4u32 => Self::TemplateLiteral(generator.generate()),
                    5u32 => Self::UnaryExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSLiteral generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            crate::custom::generate_ts_type(generator)
        } else {
            panic!("TSType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSConditionalType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSConditionalType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSConditionalType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSUnionType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSUnionType::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSType<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSUnionType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSIntersectionType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSIntersectionType::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSType<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSIntersectionType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSParenthesizedType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSParenthesizedType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSParenthesizedType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeOperator<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeOperator::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeOperatorOperator>(),
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeOperator generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeOperatorOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::Keyof,
                1u32 => Self::Unique,
                2u32 => Self::Readonly,
                _ => unreachable!(),
            }
        } else {
            panic!("TSTypeOperatorOperator generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSArrayType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSArrayType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSArrayType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSIndexedAccessType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSIndexedAccessType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSIndexedAccessType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTupleType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTupleType::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSTupleElement<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTupleType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNamedTupleMember<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNamedTupleMember::new(
                oxc_span::SPAN,
                generator.generate::<IdentifierName<'a>>(),
                generator.generate::<TSTupleElement<'a>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSNamedTupleMember generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSOptionalType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSOptionalType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSOptionalType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSRestType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSRestType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSRestType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTupleElement<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => generator.generate::<TSType<'a>>().into(),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::TSOptionalType(generator.generate()),
                    1u32 => Self::TSRestType(generator.generate()),
                    2u32 => generator.generate::<TSType<'a>>().into(),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSTupleElement generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSAnyKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSAnyKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSAnyKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSStringKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSStringKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSStringKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSBooleanKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSBooleanKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSBooleanKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNumberKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNumberKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSNumberKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNeverKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNeverKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSNeverKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSIntrinsicKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSIntrinsicKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSIntrinsicKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSUnknownKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSUnknownKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSUnknownKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNullKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNullKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSNullKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSUndefinedKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSUndefinedKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSUndefinedKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSVoidKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSVoidKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSVoidKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSSymbolKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSSymbolKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSSymbolKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSThisType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSThisType::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSThisType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSObjectKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSObjectKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSObjectKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSBigIntKeyword {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSBigIntKeyword::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("TSBigIntKeyword generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeReference<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeReference::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeName<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeReference generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32]) {
                    0u32 => Self::IdentifierReference(generator.generate()),
                    1u32 => Self::ThisExpression(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::IdentifierReference(generator.generate()),
                    1u32 => Self::QualifiedName(generator.generate()),
                    2u32 => Self::ThisExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSTypeName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSQualifiedName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSQualifiedName::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeName<'a>>(),
                generator.generate::<IdentifierName<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSQualifiedName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeParameterInstantiation<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            crate::custom::generate_ts_type_parameter_instantiation(generator)
        } else {
            panic!("TSTypeParameterInstantiation generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeParameter<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            crate::custom::generate_ts_type_parameter(generator)
        } else {
            panic!("TSTypeParameter generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeParameterDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            crate::custom::generate_ts_type_parameter_declaration(generator)
        } else {
            panic!("TSTypeParameterDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeAliasDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeAliasDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<TSType<'a>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeAliasDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSAccessibility {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::Private,
                1u32 => Self::Protected,
                2u32 => Self::Public,
                _ => unreachable!(),
            }
        } else {
            panic!("TSAccessibility generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSClassImplements<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSClassImplements::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeName<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSClassImplements generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSInterfaceDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSInterfaceDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<ArenaVec<'a, TSInterfaceHeritage<'a>>>(),
                generator.generate::<ArenaBox<'a, TSInterfaceBody<'a>>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSInterfaceDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSInterfaceBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSInterfaceBody::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSSignature<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSInterfaceBody generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSPropertySignature<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSPropertySignature::new(
                oxc_span::SPAN,
                generator.generate::<bool>(),
                generator.generate::<bool>(),
                generator.generate::<bool>(),
                generator.generate::<PropertyKey<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSPropertySignature generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSSignature<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::TSPropertySignature(generator.generate()),
                    1u32 => Self::TSCallSignatureDeclaration(generator.generate()),
                    2u32 => Self::TSConstructSignatureDeclaration(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::TSIndexSignature(generator.generate()),
                    1u32 => Self::TSPropertySignature(generator.generate()),
                    2u32 => Self::TSCallSignatureDeclaration(generator.generate()),
                    3u32 => Self::TSConstructSignatureDeclaration(generator.generate()),
                    4u32 => Self::TSMethodSignature(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSSignature generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSIndexSignature<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSIndexSignature::new(
                oxc_span::SPAN,
                generator.generate::<TSIndexSignatureName<'a>>(),
                generator.generate::<ArenaBox<'a, TSTypeAnnotation<'a>>>(),
                generator.generate::<bool>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSIndexSignature generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSCallSignatureDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSCallSignatureDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<Option<ArenaBox<'a, TSThisParameter<'a>>>>(),
                generator.generate::<ArenaBox<'a, FormalParameters<'a>>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSCallSignatureDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSMethodSignatureKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::Method,
                1u32 => Self::Get,
                2u32 => Self::Set,
                _ => unreachable!(),
            }
        } else {
            panic!("TSMethodSignatureKind generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSMethodSignature<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSMethodSignature::new(
                oxc_span::SPAN,
                generator.generate::<PropertyKey<'a>>(),
                generator.generate::<bool>(),
                generator.generate::<bool>(),
                generator.generate::<TSMethodSignatureKind>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<Option<ArenaBox<'a, TSThisParameter<'a>>>>(),
                generator.generate::<ArenaBox<'a, FormalParameters<'a>>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSMethodSignature generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSConstructSignatureDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSConstructSignatureDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<ArenaBox<'a, FormalParameters<'a>>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSConstructSignatureDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSIndexSignatureName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSIndexSignatureName::new(
                oxc_span::SPAN,
                generator.generate::<Ident<'a>>(),
                generator.generate::<ArenaBox<'a, TSTypeAnnotation<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSIndexSignatureName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSInterfaceHeritage<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSInterfaceHeritage::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeName<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSInterfaceHeritage generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypePredicate<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypePredicate::new(
                oxc_span::SPAN,
                generator.generate::<TSTypePredicateName<'a>>(),
                generator.generate::<bool>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypePredicate generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypePredicateName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::Identifier(generator.generate()),
                1u32 => Self::This(generator.generate()),
                _ => unreachable!(),
            }
        } else {
            panic!("TSTypePredicateName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSExternalModuleDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSExternalModuleDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<StringLiteral<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSModuleBlock<'a>>>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSExternalModuleDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNamespaceDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNamespaceDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<TSNamespaceDeclarationBody<'a>>(),
                generator.generate::<TSNamespaceDeclarationKind>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSNamespaceDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNamespaceDeclarationKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32]) {
                0u32 => Self::Module,
                1u32 => Self::Namespace,
                _ => unreachable!(),
            }
        } else {
            panic!("TSNamespaceDeclarationKind generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNamespaceDeclarationBody<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::TSModuleBlock(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32]) {
                    0u32 => Self::TSNamespaceDeclaration(generator.generate()),
                    1u32 => Self::TSModuleBlock(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSNamespaceDeclarationBody generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSGlobalDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSGlobalDeclaration::new(
                oxc_span::SPAN,
                oxc_span::SPAN,
                generator.generate::<TSModuleBlock<'a>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("TSGlobalDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSModuleBlock<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSModuleBlock::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, Directive<'a>>>(),
                crate::custom::generate_block_statements(generator),
                generator.ast(),
            )
        } else {
            panic!("TSModuleBlock generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeLiteral<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeLiteral::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TSSignature<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeLiteral generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSInferType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSInferType::new(
                oxc_span::SPAN,
                generator.generate::<ArenaBox<'a, TSTypeParameter<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSInferType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeQuery<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeQuery::new(
                oxc_span::SPAN,
                generator.generate::<TSTypeQueryExprName<'a>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeQuery generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeQueryExprName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32, 1u32]) {
                    0u32 => Self::IdentifierReference(generator.generate()),
                    1u32 => Self::ThisExpression(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
                    0u32 => Self::TSImportType(generator.generate()),
                    1u32 => Self::IdentifierReference(generator.generate()),
                    2u32 => Self::QualifiedName(generator.generate()),
                    3u32 => Self::ThisExpression(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSTypeQueryExprName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSImportType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSImportType::new(
                oxc_span::SPAN,
                generator.generate::<StringLiteral<'a>>(),
                generator.generate::<Option<ArenaBox<'a, ObjectExpression<'a>>>>(),
                generator.generate::<Option<TSImportTypeQualifier<'a>>>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSImportType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSImportTypeQualifier<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::Identifier(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32]) {
                    0u32 => Self::Identifier(generator.generate()),
                    1u32 => Self::QualifiedName(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSImportTypeQualifier generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSImportTypeQualifiedName<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSImportTypeQualifiedName::new(
                oxc_span::SPAN,
                generator.generate::<TSImportTypeQualifier<'a>>(),
                generator.generate::<IdentifierName<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSImportTypeQualifiedName generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSFunctionType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSFunctionType::new(
                oxc_span::SPAN,
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<Option<ArenaBox<'a, TSThisParameter<'a>>>>(),
                generator.generate::<ArenaBox<'a, FormalParameters<'a>>>(),
                generator.generate::<ArenaBox<'a, TSTypeAnnotation<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSFunctionType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSConstructorType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSConstructorType::new(
                oxc_span::SPAN,
                generator.generate::<bool>(),
                generator.generate::<Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>(),
                generator.generate::<ArenaBox<'a, FormalParameters<'a>>>(),
                generator.generate::<ArenaBox<'a, TSTypeAnnotation<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSConstructorType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSMappedType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSMappedType::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.generate::<Option<TSType<'a>>>(),
                generator.generate::<Option<TSType<'a>>>(),
                generator.generate::<Option<TSMappedTypeModifierOperator>>(),
                generator.generate::<Option<TSMappedTypeModifierOperator>>(),
                generator.ast(),
            )
        } else {
            panic!("TSMappedType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSMappedTypeModifierOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                0u32 => Self::True,
                1u32 => Self::Plus,
                2u32 => Self::Minus,
                _ => unreachable!(),
            }
        } else {
            panic!("TSMappedTypeModifierOperator generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTemplateLiteralType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTemplateLiteralType::new(
                oxc_span::SPAN,
                generator.generate::<ArenaVec<'a, TemplateElement<'a>>>(),
                generator.generate::<ArenaVec<'a, TSType<'a>>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTemplateLiteralType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSAsExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSAsExpression::new(
                oxc_span::SPAN,
                generator.generate::<Expression<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSAsExpression generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSSatisfiesExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSSatisfiesExpression::new(
                oxc_span::SPAN,
                generator.generate::<Expression<'a>>(),
                generator.generate::<TSType<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSSatisfiesExpression generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSTypeAssertion<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSTypeAssertion::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.generate::<Expression<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSTypeAssertion generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSImportEqualsDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSImportEqualsDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<BindingIdentifier<'a>>(),
                generator.generate::<TSModuleReference<'a>>(),
                generator.generate::<ImportOrExportKind>(),
                generator.ast(),
            )
        } else {
            panic!("TSImportEqualsDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSModuleReference<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            if generator.at_limit() {
                match generator.random_weighted(&[1u32]) {
                    0u32 => Self::IdentifierReference(generator.generate()),
                    _ => unreachable!(),
                }
            } else {
                match generator.random_weighted(&[1u32, 1u32, 1u32]) {
                    0u32 => Self::ExternalModuleReference(generator.generate()),
                    1u32 => Self::IdentifierReference(generator.generate()),
                    2u32 => Self::QualifiedName(generator.generate()),
                    _ => unreachable!(),
                }
            }
        } else {
            panic!("TSModuleReference generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSExternalModuleReference<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSExternalModuleReference::new(
                oxc_span::SPAN,
                generator.generate::<StringLiteral<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSExternalModuleReference generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNonNullExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNonNullExpression::new(
                oxc_span::SPAN,
                generator.generate::<Expression<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSNonNullExpression generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Decorator<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Decorator::new(
            oxc_span::SPAN,
            generator.generate::<Expression<'a>>(),
            generator.ast(),
        )
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSExportAssignment<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSExportAssignment::new(
                oxc_span::SPAN,
                generator.generate::<Expression<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSExportAssignment generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSNamespaceExportDeclaration<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::TSNamespaceExportDeclaration::new(
                oxc_span::SPAN,
                generator.generate::<IdentifierName<'a>>(),
                generator.ast(),
            )
        } else {
            panic!("TSNamespaceExportDeclaration generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::TSInstantiationExpression<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            crate::custom::generate_ts_instantiation_expression(generator)
        } else {
            panic!("TSInstantiationExpression generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::ImportOrExportKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        crate::custom::generate_import_or_export_kind(generator)
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSDocNullableType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::JSDocNullableType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("JSDocNullableType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSDocNonNullableType<'a> {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::JSDocNonNullableType::new(
                oxc_span::SPAN,
                generator.generate::<TSType<'a>>(),
                generator.generate::<bool>(),
                generator.ast(),
            )
        } else {
            panic!("JSDocNonNullableType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::JSDocUnknownType {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        if generator.is_typescript() {
            oxc_ast::ast::JSDocUnknownType::new(oxc_span::SPAN, generator.ast())
        } else {
            panic!("JSDocUnknownType generation requires a TypeScript source type")
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CommentKind {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32]) {
            0u32 => Self::Line,
            1u32 => Self::SingleLineBlock,
            2u32 => Self::MultiLineBlock,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CommentPosition {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::Leading,
            1u32 => Self::Trailing,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::CommentContent {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[
            1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
        ]) {
            0u32 => Self::None,
            1u32 => Self::Legal,
            2u32 => Self::Jsdoc,
            3u32 => Self::JsdocLegal,
            4u32 => Self::Pure,
            5u32 => Self::PureNotApplied,
            6u32 => Self::NoSideEffects,
            7u32 => Self::Webpack,
            8u32 => Self::Vite,
            9u32 => Self::CoverageIgnore,
            10u32 => Self::Turbopack,
            11u32 => Self::CoverageIgnoreFile,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_ast::ast::Comment {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        oxc_ast::ast::Comment {
            span: oxc_span::SPAN,
            attached_to: generator.generate::<u32>(),
            kind: generator.generate::<CommentKind>(),
            position: generator.generate::<CommentPosition>(),
            newlines: CommentNewlines::empty(),
            content: generator.generate::<CommentContent>(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::number::NumberBase {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32]) {
            0u32 => Self::Float,
            1u32 => Self::Decimal,
            2u32 => Self::Binary,
            3u32 => Self::Octal,
            4u32 => Self::Hex,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::number::BigintBase {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32]) {
            0u32 => Self::Decimal,
            1u32 => Self::Binary,
            2u32 => Self::Octal,
            3u32 => Self::Hex,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::operator::AssignmentOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[
            1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
            1u32, 1u32,
        ]) {
            0u32 => Self::Assign,
            1u32 => Self::Addition,
            2u32 => Self::Subtraction,
            3u32 => Self::Multiplication,
            4u32 => Self::Division,
            5u32 => Self::Remainder,
            6u32 => Self::Exponential,
            7u32 => Self::ShiftLeft,
            8u32 => Self::ShiftRight,
            9u32 => Self::ShiftRightZeroFill,
            10u32 => Self::BitwiseOR,
            11u32 => Self::BitwiseXOR,
            12u32 => Self::BitwiseAnd,
            13u32 => Self::LogicalOr,
            14u32 => Self::LogicalAnd,
            15u32 => Self::LogicalNullish,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::operator::BinaryOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[
            1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
            1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32,
        ]) {
            0u32 => Self::Equality,
            1u32 => Self::Inequality,
            2u32 => Self::StrictEquality,
            3u32 => Self::StrictInequality,
            4u32 => Self::LessThan,
            5u32 => Self::LessEqualThan,
            6u32 => Self::GreaterThan,
            7u32 => Self::GreaterEqualThan,
            8u32 => Self::Addition,
            9u32 => Self::Subtraction,
            10u32 => Self::Multiplication,
            11u32 => Self::Division,
            12u32 => Self::Remainder,
            13u32 => Self::Exponential,
            14u32 => Self::ShiftLeft,
            15u32 => Self::ShiftRight,
            16u32 => Self::ShiftRightZeroFill,
            17u32 => Self::BitwiseOR,
            18u32 => Self::BitwiseXOR,
            19u32 => Self::BitwiseAnd,
            20u32 => Self::In,
            21u32 => Self::Instanceof,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::operator::LogicalOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32]) {
            0u32 => Self::Or,
            1u32 => Self::And,
            2u32 => Self::Coalesce,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::operator::UnaryOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32]) {
            0u32 => Self::UnaryPlus,
            1u32 => Self::UnaryNegation,
            2u32 => Self::LogicalNot,
            3u32 => Self::BitwiseNot,
            4u32 => Self::Typeof,
            5u32 => Self::Void,
            6u32 => Self::Delete,
            _ => unreachable!(),
        }
    }
}

impl<'a> Generate<'a> for oxc_syntax::operator::UpdateOperator {
    fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
        let _ = &generator;
        match generator.random_weighted(&[1u32, 1u32]) {
            0u32 => Self::Increment,
            1u32 => Self::Decrement,
            _ => unreachable!(),
        }
    }
}
