// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/format.rs`.

use oxc_ast::ast::*;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter, trivia::FormatTrailingComments},
    generated::ast_nodes::{AstNode, SiblingNode},
    parentheses::NeedsParentheses,
    write::{FormatFunctionOptions, FormatJsArrowFunctionExpressionOptions, FormatWrite},
};

impl<'a> Format<'a> for AstNode<'a, Program<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let result = self.write(f);
        FormatTrailingComments::Comments(f.context().comments().unprinted_comments()).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Expression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, IdentifierName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ThisExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ArrayExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ArrayExpressionElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, Elision> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ObjectExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ObjectPropertyKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, PropertyKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TemplateElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, MemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, CallExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, NewExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, MetaProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, SpreadElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Argument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, UpdateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, UnaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BinaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, PrivateInExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, LogicalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ConditionalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetPropertyIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Super> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, AwaitExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ChainExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ChainElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Hashbang<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BlockStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Declaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, EmptyStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, IfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, WhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ForStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ForStatementInit<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ForInStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ForStatementLeft<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BreakStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, WithStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, SwitchCase<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TryStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, CatchClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, CatchParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, DebuggerStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BindingPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, BindingPatternKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BindingProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a, FormatFunctionOptions> for AstNode<'a, Function<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }

    fn fmt_with_options(
        &self,
        options: FormatFunctionOptions,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write_with_options(options, f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, FormalParameters<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, FormalParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, FunctionBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a, FormatJsArrowFunctionExpressionOptions>
    for AstNode<'a, ArrowFunctionExpression<'a>>
{
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }

    fn fmt_with_options(
        &self,
        options: FormatJsArrowFunctionExpressionOptions,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write_with_options(options, f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, YieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Class<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ClassBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ClassElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, StaticBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportDeclarationSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, WithClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ImportAttributeKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, ModuleExportName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, V8IntrinsicExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BooleanLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, NullLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, NumericLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, StringLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXFragment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXOpeningFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXClosingFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXElementName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXMemberExpressionObject<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXEmptyExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXAttributeItem<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXAttributeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXAttributeValue<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSXText<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSEnumMemberName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSUnionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSArrayType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTupleType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSRestType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTupleElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSAnyKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSStringKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSBooleanKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNumberKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNeverKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSUnknownKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNullKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSVoidKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSSymbolKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSThisType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSObjectKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSBigIntKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypePredicateName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSModuleDeclarationName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSModuleDeclarationBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSInferType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeQueryExprName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSImportType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSImportTypeQualifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSImportTypeQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSMappedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, Decorator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, TSInstantiationExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, JSDocUnknownType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        let result = self.write(f);
        self.format_trailing_comments(f)?;
        result
    }
}
