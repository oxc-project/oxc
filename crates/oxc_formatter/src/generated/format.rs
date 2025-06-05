// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter.rs`.

#![allow(clippy::undocumented_unsafe_blocks)]
use oxc_ast::{AstKind, ast::*};

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    generated::ast_nodes::AstNode,
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

/// A hack for erasing the lifetime requirement.
pub fn hack<'ast, T>(t: &T) -> &'ast T {
    unsafe { std::mem::transmute(t) }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Program<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Expression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, IdentifierName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ThisExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ArrayExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ArrayExpressionElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Elision> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ObjectExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ObjectPropertyKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, PropertyKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TaggedTemplateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TemplateElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, MemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, CallExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, NewExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, MetaProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, SpreadElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Argument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, UpdateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, UnaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BinaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, PrivateInExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, LogicalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ConditionalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetMaybeDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetPropertyIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, SequenceExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Super> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AwaitExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ChainExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ChainElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ParenthesizedExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Hashbang<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BlockStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Declaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, EmptyStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, IfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, WhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ForStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ForStatementInit<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ForInStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ForStatementLeft<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BreakStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, WithStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, SwitchCase<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TryStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, CatchClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, CatchParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, DebuggerStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BindingPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BindingPatternKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BindingProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Function<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, FormalParameters<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, FormalParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, FunctionBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ArrowFunctionExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, YieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Class<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ClassBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ClassElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, StaticBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportDeclarationSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, WithClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ImportAttributeKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ExportDefaultDeclarationKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, ModuleExportName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, V8IntrinsicExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BooleanLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, NullLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, NumericLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, StringLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXFragment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXOpeningFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXClosingFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXElementName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXMemberExpressionObject<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXEmptyExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXAttributeItem<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXAttributeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXAttributeValue<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSXText<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSEnumMemberName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSUnionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSArrayType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTupleType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSRestType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTupleElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSAnyKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSStringKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSBooleanKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNumberKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNeverKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSUnknownKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNullKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSVoidKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSSymbolKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSThisType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSObjectKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSBigIntKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypePredicateName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSModuleDeclarationName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSModuleDeclarationBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSInferType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeQueryExprName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSImportType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSMappedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSAsExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSSatisfiesExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSTypeAssertion<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNonNullExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, Decorator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, TSInstantiationExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a, 'b> Format<'a> for AstNode<'a, 'b, JSDocUnknownType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}
