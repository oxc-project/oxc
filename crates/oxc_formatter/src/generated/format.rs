// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter.rs`.

#![allow(clippy::undocumented_unsafe_blocks)]
use oxc_ast::{AstKind, ast::*};

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

/// A hack for erasing the lifetime requirement.
pub fn hack<'ast, T>(t: &T) -> &'ast T {
    unsafe { std::mem::transmute(t) }
}

impl<'a> Format<'a> for Program<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Program(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Expression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for IdentifierName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::IdentifierName(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for IdentifierReference<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::IdentifierReference(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BindingIdentifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BindingIdentifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for LabelIdentifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::LabelIdentifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ThisExpression {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ThisExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ArrayExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ArrayExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ArrayExpressionElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ArrayExpressionElement(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Elision {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Elision(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ObjectExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ObjectExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ObjectPropertyKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ObjectProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ObjectProperty(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for PropertyKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::PropertyKey(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TemplateLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TemplateLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TaggedTemplateExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TaggedTemplateExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TemplateElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for MemberExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::MemberExpression(hack(self)));
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ComputedMemberExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for StaticMemberExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for PrivateFieldExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for CallExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::CallExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for NewExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::NewExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for MetaProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::MetaProperty(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for SpreadElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::SpreadElement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Argument<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Argument(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for UpdateExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::UpdateExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for UnaryExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::UnaryExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BinaryExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BinaryExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for PrivateInExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::PrivateInExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for LogicalExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::LogicalExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ConditionalExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ConditionalExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AssignmentExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AssignmentExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AssignmentTarget<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AssignmentTarget(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for SimpleAssignmentTarget<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::SimpleAssignmentTarget(hack(self)));
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AssignmentTargetPattern<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AssignmentTargetPattern(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ArrayAssignmentTarget<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ArrayAssignmentTarget(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ObjectAssignmentTarget<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ObjectAssignmentTarget(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AssignmentTargetRest<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AssignmentTargetMaybeDefault<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AssignmentTargetWithDefault<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AssignmentTargetWithDefault(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AssignmentTargetProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for SequenceExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::SequenceExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Super {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Super(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AwaitExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AwaitExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ChainExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ChainExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ChainElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ParenthesizedExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        result
    }
}

impl<'a> Format<'a> for Statement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for Directive<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Directive(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Hashbang<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Hashbang(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BlockStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BlockStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Declaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for VariableDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::VariableDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for VariableDeclarator<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::VariableDeclarator(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for EmptyStatement {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::EmptyStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ExpressionStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ExpressionStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for IfStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::IfStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for DoWhileStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::DoWhileStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for WhileStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::WhileStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ForStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ForStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ForStatementInit<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ForStatementInit(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ForInStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ForInStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ForStatementLeft<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ForOfStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ForOfStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ContinueStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ContinueStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BreakStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BreakStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ReturnStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ReturnStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for WithStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::WithStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for SwitchStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::SwitchStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for SwitchCase<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::SwitchCase(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for LabeledStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::LabeledStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ThrowStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ThrowStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TryStatement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TryStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for CatchClause<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::CatchClause(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for CatchParameter<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::CatchParameter(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for DebuggerStatement {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::DebuggerStatement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BindingPattern<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for BindingPatternKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for AssignmentPattern<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::AssignmentPattern(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ObjectPattern<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ObjectPattern(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BindingProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ArrayPattern<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ArrayPattern(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BindingRestElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BindingRestElement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Function<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Function(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for FormalParameters<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::FormalParameters(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for FormalParameter<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::FormalParameter(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for FunctionBody<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::FunctionBody(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ArrowFunctionExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ArrowFunctionExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for YieldExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::YieldExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Class<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Class(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ClassBody<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ClassBody(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ClassElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for MethodDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::MethodDefinition(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for PropertyDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::PropertyDefinition(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for PrivateIdentifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::PrivateIdentifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for StaticBlock<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::StaticBlock(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ModuleDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ModuleDeclaration(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for AccessorProperty<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ImportExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ImportExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ImportDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ImportDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ImportDeclarationSpecifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ImportSpecifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ImportSpecifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ImportDefaultSpecifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ImportDefaultSpecifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ImportNamespaceSpecifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ImportNamespaceSpecifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for WithClause<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ImportAttribute<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ImportAttributeKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ExportNamedDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ExportNamedDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ExportDefaultDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ExportDefaultDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ExportAllDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ExportAllDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ExportSpecifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::ExportSpecifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for ExportDefaultDeclarationKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for ModuleExportName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for V8IntrinsicExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::V8IntrinsicExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BooleanLiteral {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BooleanLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for NullLiteral {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::NullLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for NumericLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::NumericLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for StringLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::StringLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for BigIntLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::BigIntLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for RegExpLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::RegExpLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXElement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXOpeningElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXOpeningElement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXClosingElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXClosingElement(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXFragment<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXFragment(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXOpeningFragment {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXClosingFragment {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXElementName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXElementName(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXNamespacedName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXNamespacedName(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXMemberExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXMemberExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXMemberExpressionObject<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXMemberExpressionObject(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXExpressionContainer<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXExpressionContainer(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXEmptyExpression {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXAttributeItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXAttributeItem(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXAttribute<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXSpreadAttribute<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXSpreadAttribute(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXAttributeName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXAttributeValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXIdentifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXIdentifier(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSXChild<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXSpreadChild<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSXText<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::JSXText(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSThisParameter<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSThisParameter(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSEnumDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSEnumDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSEnumBody<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSEnumBody(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSEnumMember<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSEnumMember(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSEnumMemberName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSTypeAnnotation<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeAnnotation(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSLiteralType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSLiteralType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSConditionalType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSConditionalType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSUnionType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSUnionType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSIntersectionType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSIntersectionType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSParenthesizedType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSParenthesizedType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeOperator<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSArrayType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSIndexedAccessType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSIndexedAccessType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTupleType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSNamedTupleMember<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSNamedTupleMember(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSOptionalType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSRestType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSTupleElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSAnyKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSAnyKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSStringKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSStringKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSBooleanKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSBooleanKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSNumberKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSNumberKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSNeverKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSNeverKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSIntrinsicKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSIntrinsicKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSUnknownKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSUnknownKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSNullKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSNullKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSUndefinedKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSUndefinedKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSVoidKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSVoidKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSSymbolKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSSymbolKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSThisType {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSThisType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSObjectKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSObjectKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSBigIntKeyword {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSBigIntKeyword(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeReference<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeReference(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeName(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSQualifiedName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSQualifiedName(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeParameterInstantiation<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeParameterInstantiation(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeParameter<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeParameter(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeParameterDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeAliasDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSClassImplements<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSClassImplements(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSInterfaceDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSInterfaceDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSInterfaceBody<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSPropertySignature<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSPropertySignature(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSSignature<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSIndexSignature<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSCallSignatureDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSMethodSignature<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSMethodSignature(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSConstructSignatureDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSConstructSignatureDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSIndexSignatureName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSInterfaceHeritage<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSInterfaceHeritage(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypePredicate<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSTypePredicateName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSModuleDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSModuleDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSModuleDeclarationName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSModuleDeclarationBody<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSModuleBlock<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSModuleBlock(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeLiteral<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeLiteral(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSInferType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSInferType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeQuery<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeQuery(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeQueryExprName<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSImportType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSImportType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSFunctionType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSConstructorType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSMappedType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSMappedType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTemplateLiteralType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTemplateLiteralType(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSAsExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSAsExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSSatisfiesExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSSatisfiesExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSTypeAssertion<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSTypeAssertion(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSImportEqualsDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSImportEqualsDeclaration(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSModuleReference<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSModuleReference(hack(self)));
        let result = self.write(f);
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSExternalModuleReference<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSExternalModuleReference(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSNonNullExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSNonNullExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for Decorator<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Decorator(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSExportAssignment<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSExportAssignment(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let result = self.write(f);
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for TSNamespaceExportDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for TSInstantiationExpression<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::TSInstantiationExpression(hack(self)));
        format_leading_comments(self.span.start).fmt(f)?;
        let needs_parentheses = self.needs_parentheses(&f.state().stack);
        if needs_parentheses {
            "(".fmt(f)?;
        }
        let result = self.write(f);
        if needs_parentheses {
            ")".fmt(f)?;
        }
        format_trailing_comments(self.span.end).fmt(f)?;
        f.state_mut().stack.pop();
        result
    }
}

impl<'a> Format<'a> for JSDocNullableType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSDocNonNullableType<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}

impl<'a> Format<'a> for JSDocUnknownType {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.write(f)
    }
}
