use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use crate::{
    Format,
    formatter::{FormatResult, Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
    parentheses::NeedsParentheses,
    utils::typecast::format_type_cast_comment_node,
    write,
    write::FormatWrite,
};

pub struct FormatExpressionWithoutTrailingComments<'a, 'b>(pub &'b AstNode<'a, Expression<'a>>);

impl<'a> Format<'a> for FormatExpressionWithoutTrailingComments<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_object_or_array_expression = matches!(
            self.0.as_ast_nodes(),
            AstNodes::ObjectExpression(_) | AstNodes::ArrayExpression(_)
        );
        if format_type_cast_comment_node(self.0, is_object_or_array_expression, f)? {
            return Ok(());
        }

        let needs_parentheses = self.0.needs_parentheses(f);
        let print_left_paren =
            |f: &mut Formatter<'_, 'a>| write!(f, needs_parentheses.then_some("("));

        match self.0.as_ast_nodes() {
            AstNodes::BooleanLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::NullLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::NumericLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::BigIntLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::RegExpLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::StringLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TemplateLiteral(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::IdentifierReference(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::MetaProperty(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::Super(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ArrayExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ArrowFunctionExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::AssignmentExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::AwaitExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::BinaryExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::CallExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ChainExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::Class(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ConditionalExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::Function(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ImportExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::LogicalExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::NewExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ObjectExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ParenthesizedExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::SequenceExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TaggedTemplateExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ThisExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::UnaryExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::UpdateExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::YieldExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::PrivateInExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::JSXElement(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::JSXFragment(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TSAsExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TSSatisfiesExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TSTypeAssertion(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TSNonNullExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::TSInstantiationExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::V8IntrinsicExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::StaticMemberExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::ComputedMemberExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            AstNodes::PrivateFieldExpression(n) => {
                n.format_leading_comments(f)?;
                print_left_paren(f)?;
                n.write(f)
            }
            _ => unreachable!(),
        }?;

        if needs_parentheses {
            write!(f, [")"])?;
        }

        Ok(())
    }
}
