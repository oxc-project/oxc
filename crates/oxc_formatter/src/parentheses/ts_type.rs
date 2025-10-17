use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::NeedsParentheses;
use crate::{
    Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::Formatter,
    write::{BinaryLikeExpression, should_flatten},
};

impl NeedsParentheses<'_> for AstNode<'_, TSType<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.as_ast_nodes() {
            AstNodes::TSFunctionType(it) => it.needs_parentheses(f),
            AstNodes::TSInferType(it) => it.needs_parentheses(f),
            AstNodes::TSConstructorType(it) => it.needs_parentheses(f),
            AstNodes::TSUnionType(it) => it.needs_parentheses(f),
            AstNodes::TSIntersectionType(it) => it.needs_parentheses(f),
            AstNodes::TSConditionalType(it) => it.needs_parentheses(f),
            AstNodes::TSTypeOperator(it) => it.needs_parentheses(f),
            AstNodes::TSTypeQuery(it) => it.needs_parentheses(f),
            _ => {
                // TODO: incomplete
                false
            }
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSFunctionType<'_>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        function_like_type_needs_parentheses(self.span(), self.parent, Some(&self.return_type))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSInferType<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.parent {
            AstNodes::TSIntersectionType(_) | AstNodes::TSUnionType(_) => true,
            AstNodes::TSRestType(_) => false,
            _ => operator_type_or_higher_needs_parens(self.span, self.parent),
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSConstructorType<'_>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        function_like_type_needs_parentheses(self.span(), self.parent, Some(&self.return_type))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSUnionType<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.parent {
            AstNodes::TSUnionType(union) => self.types.len() > 1 && union.types.len() > 1,
            AstNodes::TSIntersectionType(intersection) => {
                self.types.len() > 1 && intersection.types.len() > 1
            }
            parent => operator_type_or_higher_needs_parens(self.span(), parent),
        }
    }
}

/// Returns `true` if a TS primary type needs parentheses
/// Common logic for determining if function-like types (TSFunctionType, TSConstructorType)
/// need parentheses based on their parent context.
///
/// Ported from Biome's function_like_type_needs_parentheses
fn function_like_type_needs_parentheses<'a>(
    span: Span,
    parent: &'a AstNodes<'a>,
    return_type: Option<&'a TSTypeAnnotation<'a>>,
) -> bool {
    match parent {
        // Arrow function return types need parens
        AstNodes::TSTypeAnnotation(type_annotation) => {
            matches!(type_annotation.parent, AstNodes::ArrowFunctionExpression(_))
        }
        // In conditional types
        AstNodes::TSConditionalType(conditional) => {
            let is_check_type = conditional.check_type().span() == span;
            if is_check_type {
                return true;
            }

            let is_extends_type = conditional.extends_type().span() == span;
            if is_extends_type {
                // Need parentheses if return type is TSInferType with constraint
                // or TSTypePredicate with type annotation
                if let Some(return_type) = return_type {
                    match &return_type.type_annotation {
                        TSType::TSInferType(infer_type) => {
                            return infer_type.type_parameter.constraint.is_some();
                        }
                        TSType::TSTypePredicate(predicate) => {
                            return predicate.type_annotation.is_some();
                        }
                        _ => {}
                    }
                }
            }
            false
        }
        AstNodes::TSUnionType(union) => union.types.len() > 1,
        AstNodes::TSIntersectionType(intersection) => intersection.types.len() > 1,
        _ => operator_type_or_higher_needs_parens(span, parent),
    }
}

/// Returns `true` if a TS primary type needs parentheses
/// This is for types that have higher precedence operators as parents
fn operator_type_or_higher_needs_parens(span: Span, parent: &AstNodes) -> bool {
    match parent {
        // These parent types always require parentheses for their operands
        AstNodes::TSArrayType(_)
        | AstNodes::TSTypeOperator(_)
        | AstNodes::TSRestType(_)
        | AstNodes::TSOptionalType(_) => true,
        // Indexed access requires parens if this is the object type
        AstNodes::TSIndexedAccessType(indexed) => indexed.object_type.span() == span,
        _ => false,
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSIntersectionType<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.parent {
            AstNodes::TSUnionType(union) => self.types.len() > 1 && union.types.len() > 1,
            AstNodes::TSIntersectionType(intersection) => {
                self.types.len() > 1 && intersection.types.len() > 1
            }
            parent => operator_type_or_higher_needs_parens(self.span(), parent),
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSConditionalType<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.parent {
            AstNodes::TSConditionalType(ty) => {
                ty.extends_type().span() == self.span() || ty.check_type().span() == self.span()
            }
            AstNodes::TSUnionType(union) => union.types.len() > 1,
            AstNodes::TSIntersectionType(intersection) => intersection.types.len() > 1,
            _ => operator_type_or_higher_needs_parens(self.span, self.parent),
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSTypeOperator<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        operator_type_or_higher_needs_parens(self.span(), self.parent)
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSTypeQuery<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        match self.parent {
            AstNodes::TSArrayType(_) => true,
            // Typeof operators are parenthesized when used as an object type in an indexed access
            // to avoid ambiguity of precedence, as it's higher than the JS equivalent:
            // ```typescript
            // const array = [1, 2, 3]
            // type T = typeof array[0]; // => number
            // type T2 = (typeof array)[0]; // => number
            // const J1 = typeof array[0]; // => 'number'
            // const J2 = (typeof array)[0]; // => 'o', because `typeof array` is 'object'
            // ```
            AstNodes::TSIndexedAccessType(indexed) => {
                // The typeof operator only needs parens if it's the object of the indexed access.
                // If it's the index_type, then the braces already act as the visual precedence.
                indexed.object_type().span() == self.span()
            }
            _ => false,
        }
    }
}
