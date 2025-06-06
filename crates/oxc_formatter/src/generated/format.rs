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

impl<'a> Format<'a> for AstNode<'a, '_, Program<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Expression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, IdentifierName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ThisExpression> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ArrayExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ArrayExpressionElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Elision> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ObjectExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ObjectPropertyKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, PropertyKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TaggedTemplateExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TemplateElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, MemberExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, CallExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, NewExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, MetaProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SpreadElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Argument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, UpdateExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, UnaryExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BinaryExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PrivateInExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, LogicalExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ConditionalExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SimpleAssignmentTarget<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetMaybeDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPropertyIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SequenceExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Super> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AwaitExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ChainExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ChainElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ParenthesizedExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Hashbang<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BlockStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Declaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, EmptyStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, IfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, WhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForStatementInit<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForInStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForStatementLeft<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BreakStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, WithStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SwitchCase<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TryStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, CatchClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, CatchParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, DebuggerStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BindingPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BindingPatternKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BindingProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Function<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, FormalParameters<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, FormalParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, FunctionBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ArrowFunctionExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, YieldExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Class<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ClassBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ClassElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, StaticBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportDeclarationSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, WithClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportAttributeKey<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportDefaultDeclarationKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ModuleExportName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, V8IntrinsicExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BooleanLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, NullLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, NumericLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, StringLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXFragment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXOpeningFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXClosingFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXElementName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXMemberExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXMemberExpressionObject<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXEmptyExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeItem<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeValue<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXText<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumMemberName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSUnionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSArrayType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTupleType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSRestType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTupleElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSAnyKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSStringKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSBooleanKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNumberKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNeverKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSUnknownKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNullKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSVoidKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSSymbolKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSThisType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSObjectKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSBigIntKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypePredicateName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclarationName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclarationBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSInferType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeQueryExprName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSImportType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSMappedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSAsExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSSatisfiesExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAssertion<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNonNullExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Decorator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span().start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span().end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSInstantiationExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSDocUnknownType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}
