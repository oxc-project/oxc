use std::fmt::Pointer;

use oxc_allocator::{Address, Vec};
use oxc_ast::{AstKind, ast::*};

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter, GroupId, prelude::*,
        separated::FormatSeparatedIter,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::{FormatTrailingCommas, TrailingSeparator},
    utils::call_expression::is_test_call_expression,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#const() {
            write!(f, ["const", space()])?;
        }
        if self.r#in() {
            write!(f, ["in", space()])?;
        }
        if self.out() {
            write!(f, ["out", space()])?;
        }
        write!(f, self.name())?;

        if let Some(constraint) = &self.constraint() {
            let group_id = f.group_id("constraint");

            write!(
                f,
                [
                    space(),
                    "extends",
                    group(&indent(&format_args!(
                        line_suffix_boundary(),
                        soft_line_break_or_space()
                    )))
                    .with_group_id(Some(group_id)),
                    indent_if_group_breaks(&constraint, group_id)
                ]
            )?;
        }
        if let Some(default) = &self.default() {
            write!(f, [space(), "=", space(), default])?;
        }
        Ok(())
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSTypeParameter<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Type parameter lists of arrow function expressions have to include at least one comma
        // to avoid any ambiguity with JSX elements.
        // Thus, we have to add a trailing comma when there is a single type parameter.
        // The comma can be omitted in the case where the single parameter has a constraint,
        // i.i. an `extends` clause.
        let trailing_separator = if self.len() == 1
        // This only concern sources that allow JSX or a restricted standard variant.
        && f.context().source_type().is_jsx()
        && matches!(self.parent.parent(), AstNodes::ArrowFunctionExpression(_))
        // Ignore Type parameter with an `extends` clause or a default type.
        && !self.first().is_some_and(|t| t.constraint().is_some() || t.default().is_some())
        {
            TrailingSeparator::Mandatory
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };

        f.join_with(soft_line_break_or_space())
            .entries_with_trailing_separator(self.iter(), ",", trailing_separator)
            .finish()
    }
}

#[derive(Default)]
pub struct FormatTSTypeParametersOptions {
    pub group_id: Option<GroupId>,
    pub is_type_or_interface_decl: bool,
}

pub struct FormatTSTypeParameters<'a, 'b> {
    decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
    options: FormatTSTypeParametersOptions,
}

impl<'a, 'b> FormatTSTypeParameters<'a, 'b> {
    pub fn new(
        decl: &'b AstNode<'a, TSTypeParameterDeclaration<'a>>,
        options: FormatTSTypeParametersOptions,
    ) -> Self {
        Self { decl, options }
    }
}

impl<'a> Format<'a> for FormatTSTypeParameters<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let params = self.decl.params();
        if params.is_empty() && self.options.is_type_or_interface_decl {
            write!(f, "<>")
        } else {
            write!(
                f,
                [group(&format_args!("<", format_once(|f| {
                    if matches!( self.decl.parent.parent().parent(), AstNodes::CallExpression(call) if is_test_call_expression(call))
                    {
                        f.join_nodes_with_space().entries_with_trailing_separator(params, ",", TrailingSeparator::Omit).finish()
                    } else {
                        soft_block_indent(&params).fmt(f)
                    }
                }), ">"))
                    .with_group_id(self.options.group_id)]
            )
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let params = self.params();

        if params.is_empty() {
            // This shouldn't happen in valid TypeScript code, but handle it gracefully
            return write!(
                f,
                [&group(&format_args!(
                    "<",
                    format_dangling_comments(self.span).with_soft_block_indent(),
                    ">"
                ))]
            );
        }

        // Check if this is in the context of an arrow function variable
        let is_arrow_function_vars = is_arrow_function_variable_type_argument(self);

        // Check if the first (and only) argument can be hugged
        let first_arg_can_be_hugged = if params.len() == 1 {
            if let Some(first_type) = params.first() {
                matches!(first_type.as_ref(), TSType::TSNullKeyword(_))
                    || should_hug_single_type(first_type.as_ref())
            } else {
                false
            }
        } else {
            false
        };

        let format_params = format_once(|f| {
            f.join_with(&soft_line_break_or_space())
                .entries_with_trailing_separator(params, ",", TrailingSeparator::Disallowed)
                .finish()
        });

        let should_inline =
            !is_arrow_function_vars && (params.is_empty() || first_arg_can_be_hugged);

        if should_inline {
            write!(f, ["<", format_params, ">"])
        } else {
            write!(f, [group(&format_args!("<", soft_block_indent(&format_params), ">"))])
        }
    }
}

/// Check if a TSType is a simple type (primitives, keywords, simple references)
fn is_simple_type(ty: &TSType) -> bool {
    match ty {
        TSType::TSAnyKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSThisType(_)
        | TSType::TSVoidKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSTemplateLiteralType(_)
        | TSType::TSLiteralType(_)
        | TSType::TSUnknownKeyword(_) => true,
        TSType::TSTypeReference(reference) => {
            // Simple reference without type arguments
            reference.type_arguments.is_none()
        }
        _ => false,
    }
}

/// Check if a TSType is object-like (object literal, mapped type, etc.)
fn is_object_like_type(ty: &TSType) -> bool {
    matches!(ty, TSType::TSTypeLiteral(_) | TSType::TSMappedType(_))
}

/// Check if a single type should be "hugged" (kept inline)
fn should_hug_single_type(ty: &TSType) -> bool {
    // Simple types and object-like types can be hugged
    if is_simple_type(ty) || is_object_like_type(ty) {
        return true;
    }

    // Check for union types with mostly void types and one object type
    // (e.g., `SomeType<ObjectType | null | undefined>`)
    if let TSType::TSUnionType(union_type) = ty {
        let types = &union_type.types;

        // Must have at least 2 types
        if types.len() < 2 {
            return types.len() == 1 && should_hug_single_type(&types[0]);
        }

        let has_object_type = types
            .iter()
            .any(|t| matches!(t, TSType::TSTypeLiteral(_) | TSType::TSTypeReference(_)));

        let void_count = types
            .iter()
            .filter(|t| {
                matches!(
                    t,
                    TSType::TSVoidKeyword(_)
                        | TSType::TSNullKeyword(_)
                        | TSType::TSUndefinedKeyword(_)
                )
            })
            .count();

        // Union is huggable if it's mostly void types with one object/reference type
        (types.len() - 1 == void_count && has_object_type) || types.len() == 1
    } else {
        false
    }
}

/// Check if this type parameter instantiation is in an arrow function variable context
///
/// This detects patterns like:
/// ```typescript
/// const foo: SomeThing<{ [P in "x" | "y"]: number }> = () => {};
/// ```
fn is_arrow_function_variable_type_argument<'a>(
    node: &AstNode<'a, TSTypeParameterInstantiation<'a>>,
) -> bool {
    let Some(first) = node.params().first() else { unreachable!() };

    // Skip check for single object-like types
    if node.params().len() == 1 && is_object_like_type(first.as_ref()) {
        return false;
    }

    matches!(
        &node.parent,
        AstNodes::TSTypeAnnotation(type_annotation)
            if matches!(
                &type_annotation.parent,
                AstNodes::VariableDeclarator(var_decl)
                    if matches!(&var_decl.init, Some(Expression::ArrowFunctionExpression(_)))
            )
    )
}
