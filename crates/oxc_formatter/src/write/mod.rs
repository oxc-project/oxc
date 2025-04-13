mod array_element_list;
mod block_statement;
mod directive;
mod function;
mod if_statement;
mod object_like;
mod semicolon;
mod utils;
mod variable_declaration;

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter, group_id, prelude::*,
        separated::FormatSeparatedIter, trivia::format_trailing_comments, write,
    },
    options::FormatTrailingCommas,
    write,
};

use self::{
    array_element_list::ArrayElementList, object_like::ObjectLike, semicolon::OptionalSemicolon,
    utils::FormatStatementBody,
};

impl<'a, T: Format<'a>> Format<'a> for Box<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.as_ref().fmt(f)
    }
}

pub trait FormatWrite<'ast> {
    fn write(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()>;
}

impl<'a> FormatWrite<'a> for Program<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.hashbang,
                format_leading_comments(self.span.start),
                self.directives,
                self.body,
                format_leading_comments(self.span.end), // comments before the EOF token
                hard_line_break()
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for Expression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Expression::BooleanLiteral(it) => it.fmt(f),
            Expression::NullLiteral(it) => it.fmt(f),
            Expression::NumericLiteral(it) => it.fmt(f),
            Expression::BigIntLiteral(it) => it.fmt(f),
            Expression::RegExpLiteral(it) => it.fmt(f),
            Expression::StringLiteral(it) => it.fmt(f),
            Expression::TemplateLiteral(it) => it.fmt(f),
            Expression::Identifier(it) => it.fmt(f),
            Expression::MetaProperty(it) => it.fmt(f),
            Expression::Super(it) => it.fmt(f),
            Expression::ArrayExpression(it) => it.fmt(f),
            Expression::ArrowFunctionExpression(it) => it.fmt(f),
            Expression::AssignmentExpression(it) => it.fmt(f),
            Expression::AwaitExpression(it) => it.fmt(f),
            Expression::BinaryExpression(it) => it.fmt(f),
            Expression::CallExpression(it) => it.fmt(f),
            Expression::ChainExpression(it) => it.fmt(f),
            Expression::ClassExpression(it) => it.fmt(f),
            Expression::ConditionalExpression(it) => it.fmt(f),
            Expression::FunctionExpression(it) => it.fmt(f),
            Expression::ImportExpression(it) => it.fmt(f),
            Expression::LogicalExpression(it) => it.fmt(f),
            Expression::NewExpression(it) => it.fmt(f),
            Expression::ObjectExpression(it) => it.fmt(f),
            Expression::ParenthesizedExpression(it) => it.fmt(f),
            Expression::SequenceExpression(it) => it.fmt(f),
            Expression::TaggedTemplateExpression(it) => it.fmt(f),
            Expression::ThisExpression(it) => it.fmt(f),
            Expression::UnaryExpression(it) => it.fmt(f),
            Expression::UpdateExpression(it) => it.fmt(f),
            Expression::YieldExpression(it) => it.fmt(f),
            Expression::PrivateInExpression(it) => it.fmt(f),
            Expression::JSXElement(it) => it.fmt(f),
            Expression::JSXFragment(it) => it.fmt(f),
            Expression::TSAsExpression(it) => it.fmt(f),
            Expression::TSSatisfiesExpression(it) => it.fmt(f),
            Expression::TSTypeAssertion(it) => it.fmt(f),
            Expression::TSNonNullExpression(it) => it.fmt(f),
            Expression::TSInstantiationExpression(it) => it.fmt(f),
            Expression::V8IntrinsicExpression(it) => it.fmt(f),
            match_member_expression!(Expression) => self.to_member_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for IdentifierName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for IdentifierReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for BindingIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for LabelIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_ref(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for ThisExpression {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for ArrayExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;

        if self.elements.is_empty() {
            write!(f, format_dangling_comments(self.span).with_block_indent());
        } else {
            let group_id = f.group_id("array");
            let should_expand = false; // TODO
            let elements = ArrayElementList::new(&self.elements, group_id);

            write!(
                f,
                group(&soft_block_indent(&elements))
                    .with_group_id(Some(group_id))
                    .should_expand(should_expand)
            )?;
        }

        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for ArrayExpressionElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::SpreadElement(elem) => elem.fmt(f),
            Self::Elision(_span) => Ok(()),
            _ => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for Elision {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ObjectExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::ObjectExpression(self).fmt(f)
    }
}

impl<'a> Format<'a> for Vec<'a, ObjectPropertyKind<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_soft_line();
        for (element, formatted) in self.iter().zip(
            FormatSeparatedIter::new(self.iter(), ",").with_trailing_separator(trailing_separator),
        ) {
            join.entry(element.span(), source_text, &formatted);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for ObjectPropertyKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ObjectProperty(p) => p.fmt(f),
            Self::SpreadProperty(p) => p.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ObjectProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.key, ":", space(), self.value])
    }
}

impl<'a> FormatWrite<'a> for PropertyKey<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::StaticIdentifier(key) => key.fmt(f),
            Self::PrivateIdentifier(key) => key.fmt(f),
            match_expression!(Self) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TemplateLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TaggedTemplateExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TemplateElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for MemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ComputedMemberExpression(e) => e.fmt(f),
            Self::StaticMemberExpression(e) => e.fmt(f),
            Self::PrivateFieldExpression(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ComputedMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, "[", self.expression, "]"])
    }
}

impl<'a> FormatWrite<'a> for StaticMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for PrivateFieldExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, ".", self.field])
    }
}

impl<'a> FormatWrite<'a> for CallExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.callee, "("])?;
        write!(f, [self.arguments, ")"])
    }
}

impl<'a> Format<'a> for Vec<'a, Argument<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        for arg in self {
            write!(f, arg)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for NewExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["new", space(), self.callee, "(", self.arguments, ")"])
    }
}

impl<'a> FormatWrite<'a> for MetaProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.meta, ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for SpreadElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument])
    }
}

impl<'a> FormatWrite<'a> for Argument<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::SpreadElement(e) => e.fmt(f),
            match_expression!(Argument) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for UpdateExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.prefix {
            write!(f, self.operator.as_str())?;
        }
        write!(f, self.argument)?;
        if !self.prefix {
            write!(f, self.operator.as_str())?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for UnaryExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.operator.as_str());
        if self.operator.is_keyword() {
            write!(f, hard_space());
        }
        write!(f, self.argument)
    }
}

impl<'a> FormatWrite<'a> for BinaryExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), self.operator.as_str(), space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for PrivateInExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), "in", space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for LogicalExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), self.operator.as_str(), space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for ConditionalExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.test,
                space(),
                "?",
                space(),
                self.consequent,
                space(),
                ":",
                space(),
                self.alternate
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AssignmentExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), self.operator.as_str(), space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            match_simple_assignment_target!(Self) => self.to_simple_assignment_target().fmt(f),
            match_assignment_target_pattern!(Self) => self.to_assignment_target_pattern().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for SimpleAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.fmt(f),
            Self::ComputedMemberExpression(e) => e.fmt(f),
            Self::StaticMemberExpression(e) => e.fmt(f),
            Self::PrivateFieldExpression(e) => e.fmt(f),
            Self::TSAsExpression(e) => e.fmt(f),
            Self::TSSatisfiesExpression(e) => e.fmt(f),
            Self::TSNonNullExpression(e) => e.fmt(f),
            Self::TSTypeAssertion(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ArrayAssignmentTarget(target) => target.fmt(f),
            Self::ObjectAssignmentTarget(target) => target.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ArrayAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ObjectAssignmentTarget<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetRest<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.target])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetMaybeDefault<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetWithDefault(target) => target.fmt(f),
            match_assignment_target!(Self) => self.to_assignment_target().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetWithDefault<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.binding, space(), "=", space(), self.init])
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::AssignmentTargetPropertyIdentifier(ident) => ident.fmt(f),
            Self::AssignmentTargetPropertyProperty(prop) => prop.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPropertyIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.binding)?;
        if let Some(expr) = &self.init {
            write!(f, [space(), "=", space(), expr])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AssignmentTargetPropertyProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for SequenceExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        for (i, expr) in self.expressions.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, expr)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for Super {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "super")
    }
}

impl<'a> FormatWrite<'a> for AwaitExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["await ", self.argument])
    }
}

impl<'a> FormatWrite<'a> for ChainExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.expression.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ChainElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::CallExpression(expr) => expr.fmt(f),
            Self::TSNonNullExpression(expr) => expr.fmt(f),
            Self::ComputedMemberExpression(expr) => expr.fmt(f),
            Self::StaticMemberExpression(expr) => expr.fmt(f),
            Self::PrivateFieldExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ParenthesizedExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.expression, ")"])
    }
}

impl<'a> Format<'a> for Vec<'a, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'a> FormatWrite<'a> for Statement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BlockStatement(s) => s.fmt(f),
            Self::BreakStatement(s) => s.fmt(f),
            Self::ContinueStatement(s) => s.fmt(f),
            Self::DebuggerStatement(s) => s.fmt(f),
            Self::DoWhileStatement(s) => s.fmt(f),
            Self::EmptyStatement(s) => s.fmt(f),
            Self::ExpressionStatement(s) => s.fmt(f),
            Self::ForInStatement(s) => s.fmt(f),
            Self::ForOfStatement(s) => s.fmt(f),
            Self::ForStatement(s) => s.fmt(f),
            Self::IfStatement(s) => s.fmt(f),
            Self::LabeledStatement(s) => s.fmt(f),
            Self::ReturnStatement(s) => s.fmt(f),
            Self::SwitchStatement(s) => s.fmt(f),
            Self::ThrowStatement(s) => s.fmt(f),
            Self::TryStatement(s) => s.fmt(f),
            Self::WhileStatement(s) => s.fmt(f),
            Self::WithStatement(s) => s.fmt(f),
            Self::VariableDeclaration(s) => write!(f, [s, OptionalSemicolon]),
            Self::FunctionDeclaration(s) => s.fmt(f),
            Self::ClassDeclaration(s) => s.fmt(f),
            Self::TSTypeAliasDeclaration(s) => s.fmt(f),
            Self::TSInterfaceDeclaration(s) => s.fmt(f),
            Self::TSEnumDeclaration(s) => s.fmt(f),
            Self::TSModuleDeclaration(s) => s.fmt(f),
            Self::TSImportEqualsDeclaration(s) => s.fmt(f),
            match_module_declaration!(Statement) => self.to_module_declaration().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for Hashbang<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#!", dynamic_text(self.value.as_str(), self.span.start), hard_line_break()])
    }
}

impl<'a> FormatWrite<'a> for Declaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Declaration::VariableDeclaration(d) => d.fmt(f),
            Declaration::FunctionDeclaration(d) => d.fmt(f),
            Declaration::ClassDeclaration(d) => d.fmt(f),
            Declaration::TSTypeAliasDeclaration(d) => d.fmt(f),
            Declaration::TSInterfaceDeclaration(d) => d.fmt(f),
            Declaration::TSEnumDeclaration(d) => d.fmt(f),
            Declaration::TSModuleDeclaration(d) => d.fmt(f),
            Declaration::TSImportEqualsDeclaration(d) => d.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for EmptyStatement {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [";", hard_line_break()])
    }
}

impl<'a> FormatWrite<'a> for ExpressionStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for DoWhileStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["do", self.body, "while(", self.test, ")", OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for WhileStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["while(", self.test, ")", self.body, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for ForStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ForStatementInit<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ForInStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ForStatementLeft<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ForOfStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ContinueStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "continue")?;
        if let Some(label) = &self.label {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for BreakStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "break")?;
        if let Some(label) = &self.label {
            write!(f, [space(), label])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for ReturnStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "return")?;
        if let Some(argument) = &self.argument {
            write!(f, [space(), argument])?;
        }
        write!(f, OptionalSemicolon)
    }
}

impl<'a> FormatWrite<'a> for WithStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            group(&format_args!(
                "with",
                space(),
                "(",
                self.object,
                ")",
                FormatStatementBody::new(&self.body)
            ))
        )
    }
}

impl<'a> FormatWrite<'a> for SwitchStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for SwitchCase<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for LabeledStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.label, ": ", self.body, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for ThrowStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["throw ", self.argument, OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for TryStatement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                "try ",
                self.block,
                "catch",
                self.handler,
                "finally",
                self.finalizer,
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for CatchClause<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.param, self.body])
    }
}

impl<'a> FormatWrite<'a> for CatchParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.pattern, ")"])
    }
}

impl<'a> FormatWrite<'a> for DebuggerStatement {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["debugger", OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for BindingPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.kind)
    }
}

impl<'a> FormatWrite<'a> for BindingPatternKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BindingIdentifier(p) => p.fmt(f),
            Self::ObjectPattern(p) => p.fmt(f),
            Self::ArrayPattern(p) => p.fmt(f),
            Self::AssignmentPattern(p) => p.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AssignmentPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, space(), "=", space(), self.right])
    }
}

impl<'a> FormatWrite<'a> for ObjectPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for BindingProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ArrayPattern<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for BindingRestElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.argument])
    }
}

impl<'a> FormatWrite<'a> for FormalParameters<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "(");
        for param in &self.items {
            write!(f, param)?;
        }
        if let Some(param) = &self.rest {
            write!(f, param)?;
        }
        write!(f, ")");
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for FormalParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ArrowFunctionExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for YieldExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "yield")?;
        if self.delegate {
            write!(f, "*")?;
        }
        write!(f, self.argument)
    }
}

impl<'a> FormatWrite<'a> for Class<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "class ")?;
        if self.r#abstract {
            write!(f, "abstract ")?;
        }
        write!(f, [self.id, space(), self.body])
    }
}

impl<'a> FormatWrite<'a> for ClassBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", block_indent(&self.body), "}"])
    }
}

impl<'a> Format<'a> for Vec<'a, ClassElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        for e in self {
            write!(f, e)?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ClassElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::StaticBlock(e) => e.fmt(f),
            Self::MethodDefinition(e) => e.fmt(f),
            Self::PropertyDefinition(e) => e.fmt(f),
            Self::AccessorProperty(e) => e.fmt(f),
            Self::TSIndexSignature(e) => e.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for MethodDefinition<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.key])
    }
}

impl<'a> FormatWrite<'a> for PropertyDefinition<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for PrivateIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#", dynamic_text(self.name.as_str(), self.span.start)])
    }
}

impl<'a> FormatWrite<'a> for StaticBlock<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["static", space(), "{"])?;
        for stmt in &self.body {
            write!(f, stmt);
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for ModuleDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ImportDeclaration(decl) => decl.fmt(f),
            Self::ExportAllDeclaration(decl) => decl.fmt(f),
            Self::ExportDefaultDeclaration(decl) => decl.fmt(f),
            Self::ExportNamedDeclaration(decl) => decl.fmt(f),
            Self::TSExportAssignment(decl) => decl.fmt(f),
            Self::TSNamespaceExportDeclaration(decl) => decl.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AccessorProperty<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import(", self.source, ")"])
    }
}

impl<'a> FormatWrite<'a> for ImportDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportDeclarationSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportDefaultSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportNamespaceSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for WithClause<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ImportAttributeKey<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ExportNamedDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "export ")?;

        if self.export_kind.is_type()
            && !self.declaration.as_ref().is_some_and(oxc_ast::ast::Declaration::is_type)
        {
            write!(f, "type ")?;
        }

        self.declaration.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for ExportDefaultDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export", space(), "default", space(), self.declaration])
    }
}

impl<'a> FormatWrite<'a> for ExportAllDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ExportSpecifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for ExportDefaultDeclarationKind<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::FunctionDeclaration(decl) => decl.fmt(f),
            Self::ClassDeclaration(decl) => decl.fmt(f),
            Self::TSInterfaceDeclaration(decl) => decl.fmt(f),
            match_expression!(Self) => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for ModuleExportName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for V8IntrinsicExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["%", self.name, "(", self.arguments, ")"])
    }
}

impl<'a> FormatWrite<'a> for BooleanLiteral {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, if self.value { "true" } else { "false" })
    }
}

impl<'a> FormatWrite<'a> for NullLiteral {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for NumericLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.raw.unwrap().as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for StringLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["\"", dynamic_text(self.value.as_str(), self.span.start), "\""])
    }
}

impl<'a> FormatWrite<'a> for BigIntLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.raw.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for RegExpLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let raw = self.raw.unwrap().as_str();
        let (pattern, flags) = raw.rsplit_once('/').unwrap();
        // TODO: print the flags without allocation.
        let mut flags = flags.chars().collect::<std::vec::Vec<_>>();
        flags.sort_unstable();
        let flags = flags.into_iter().collect::<String>();
        let s = format!("{pattern}/{flags}");
        write!(f, dynamic_text(&s, self.span.start))
    }
}

impl<'a> FormatWrite<'a> for JSXElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["<", self.opening_element.name])?;
        for attr in &self.opening_element.attributes {
            match attr {
                JSXAttributeItem::Attribute(_) => {
                    write!(f, hard_space())?;
                }
                JSXAttributeItem::SpreadAttribute(_) => {
                    write!(f, space())?;
                }
            }
            write!(f, attr)?;
        }
        if self.closing_element.is_none() {
            write!(f, [space(), "/"])?;
        }
        write!(f, ">")?;

        for child in &self.children {
            write!(f, child)?;
        }
        write!(f, self.closing_element)
    }
}

impl<'a> FormatWrite<'a> for JSXOpeningElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // Implemented in JSXElement above due to no access to
        // no `self_closing`.
        unreachable!()
    }
}

impl<'a> FormatWrite<'a> for JSXClosingElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["</", self.name, ">"])
    }
}

impl<'a> FormatWrite<'a> for JSXFragment<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.opening_fragment)?;
        for child in &self.children {
            write!(f, child)?;
        }
        write!(f, self.closing_fragment)
    }
}

impl<'a> FormatWrite<'a> for JSXOpeningFragment {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<>")
    }
}

impl<'a> FormatWrite<'a> for JSXClosingFragment {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "</>")
    }
}

impl<'a> FormatWrite<'a> for JSXElementName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(identifier) => identifier.fmt(f),
            Self::IdentifierReference(identifier) => identifier.fmt(f),
            Self::NamespacedName(namespaced_name) => namespaced_name.fmt(f),
            Self::MemberExpression(member_expr) => member_expr.fmt(f),
            Self::ThisExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXNamespacedName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.namespace, ":", self.name])
    }
}

impl<'a> FormatWrite<'a> for JSXMemberExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object, ".", self.property])
    }
}

impl<'a> FormatWrite<'a> for JSXMemberExpressionObject<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::MemberExpression(member_expr) => member_expr.fmt(f),
            Self::ThisExpression(expr) => expr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXExpressionContainer<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{", self.expression, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::EmptyExpression(expr) => expr.fmt(f),
            _ => self.to_expression().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXEmptyExpression {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeItem<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Attribute(attr) => attr.fmt(f),
            Self::SpreadAttribute(spread_attr) => spread_attr.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.name)?;
        if let Some(value) = &self.value {
            write!(f, ["=", value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for JSXSpreadAttribute<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{...", self.argument, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::NamespacedName(namespaced_name) => namespaced_name.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXAttributeValue<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Fragment(fragment) => fragment.fmt(f),
            Self::Element(el) => el.fmt(f),
            Self::StringLiteral(lit) => {
                let quote = if lit.value.contains('"') { "'" } else { "\"" };
                write!(f, [quote, dynamic_text(lit.value.as_str(), lit.span.start), quote])
            }
            Self::ExpressionContainer(expr_container) => expr_container.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXIdentifier<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for JSXChild<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Fragment(fragment) => fragment.fmt(f),
            Self::Element(el) => el.fmt(f),
            Self::Spread(spread) => spread.fmt(f),
            Self::ExpressionContainer(expr_container) => expr_container.fmt(f),
            Self::Text(text) => text.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for JSXSpreadChild<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["{...", self.expression, "}"])
    }
}

impl<'a> FormatWrite<'a> for JSXText<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value.as_str(), self.span.start))
    }
}

impl<'a> FormatWrite<'a> for TSThisParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")?;
        if let Some(type_annotation) = &self.type_annotation {
            write!(f, ": ")?;
            type_annotation.fmt(f);
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumMember<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSEnumMemberName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeAnnotation<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.type_annotation.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for TSLiteralType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.literal.fmt(f)
    }
}

impl<'a> FormatWrite<'a> for TSLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::BooleanLiteral(decl) => decl.fmt(f),
            Self::NumericLiteral(decl) => decl.fmt(f),
            Self::BigIntLiteral(decl) => decl.fmt(f),
            Self::StringLiteral(decl) => decl.fmt(f),
            Self::TemplateLiteral(decl) => decl.fmt(f),
            Self::UnaryExpression(decl) => decl.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSFunctionType(ty) => ty.fmt(f),
            Self::TSConstructorType(ty) => ty.fmt(f),
            Self::TSArrayType(ty) => ty.fmt(f),
            Self::TSTupleType(ty) => ty.fmt(f),
            Self::TSUnionType(ty) => ty.fmt(f),
            Self::TSParenthesizedType(ty) => ty.fmt(f),
            Self::TSIntersectionType(ty) => ty.fmt(f),
            Self::TSConditionalType(ty) => ty.fmt(f),
            Self::TSInferType(ty) => ty.fmt(f),
            Self::TSIndexedAccessType(ty) => ty.fmt(f),
            Self::TSMappedType(ty) => ty.fmt(f),
            Self::TSNamedTupleMember(ty) => ty.fmt(f),
            Self::TSLiteralType(ty) => ty.fmt(f),
            Self::TSImportType(ty) => ty.fmt(f),
            Self::TSAnyKeyword(ty) => ty.fmt(f),
            Self::TSBigIntKeyword(ty) => ty.fmt(f),
            Self::TSBooleanKeyword(ty) => ty.fmt(f),
            Self::TSIntrinsicKeyword(ty) => ty.fmt(f),
            Self::TSNeverKeyword(ty) => ty.fmt(f),
            Self::TSNullKeyword(ty) => ty.fmt(f),
            Self::TSNumberKeyword(ty) => ty.fmt(f),
            Self::TSObjectKeyword(ty) => ty.fmt(f),
            Self::TSStringKeyword(ty) => ty.fmt(f),
            Self::TSSymbolKeyword(ty) => ty.fmt(f),
            Self::TSThisType(ty) => ty.fmt(f),
            Self::TSUndefinedKeyword(ty) => ty.fmt(f),
            Self::TSUnknownKeyword(ty) => ty.fmt(f),
            Self::TSVoidKeyword(ty) => ty.fmt(f),
            Self::TSTemplateLiteralType(ty) => ty.fmt(f),
            Self::TSTypeLiteral(ty) => ty.fmt(f),
            Self::TSTypeOperatorType(ty) => ty.fmt(f),
            Self::TSTypePredicate(ty) => ty.fmt(f),
            Self::TSTypeQuery(ty) => ty.fmt(f),
            Self::TSTypeReference(ty) => ty.fmt(f),
            Self::JSDocNullableType(ty) => ty.fmt(f),
            Self::JSDocNonNullableType(ty) => ty.fmt(f),
            Self::JSDocUnknownType(ty) => ty.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSConditionalType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.check_type,
                " extends ",
                self.extends_type,
                " ? ",
                self.true_type,
                " : ",
                self.false_type
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for TSUnionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some((first, rest)) = self.types.split_first() else {
            return Ok(());
        };
        write!(f, first);
        for item in rest {
            write!(f, [" | ", item])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSIntersectionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some((first, rest)) = self.types.split_first() else {
            return Ok(());
        };
        write!(f, first);
        for item in rest {
            write!(f, [" & ", item])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSParenthesizedType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["(", self.type_annotation, ")"])
    }
}

impl<'a> FormatWrite<'a> for TSTypeOperator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.operator.to_str(), hard_space(), self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSArrayType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.element_type, "[]"])
    }
}

impl<'a> FormatWrite<'a> for TSIndexedAccessType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object_type, "[", self.index_type, "]"])
    }
}

impl<'a> FormatWrite<'a> for TSTupleType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "[")?;
        for (i, ty) in self.element_types.iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, ty)?;
        }
        write!(f, "]")
    }
}

impl<'a> FormatWrite<'a> for TSNamedTupleMember<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.label)?;
        if self.optional {
            write!(f, "?")?;
        }
        write!(f, [":", space(), self.element_type])
    }
}

impl<'a> FormatWrite<'a> for TSOptionalType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_annotation, "?"])
    }
}

impl<'a> FormatWrite<'a> for TSRestType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["...", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSTupleElement<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSOptionalType(ty) => ty.fmt(f),
            Self::TSRestType(ty) => ty.fmt(f),
            match_ts_type!(TSTupleElement) => self.to_ts_type().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSAnyKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "any")
    }
}

impl<'a> FormatWrite<'a> for TSStringKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "string")
    }
}

impl<'a> FormatWrite<'a> for TSBooleanKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "boolean")
    }
}

impl<'a> FormatWrite<'a> for TSNumberKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "number")
    }
}

impl<'a> FormatWrite<'a> for TSNeverKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "never")
    }
}

impl<'a> FormatWrite<'a> for TSIntrinsicKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "intrinsic")
    }
}

impl<'a> FormatWrite<'a> for TSUnknownKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "unknown")
    }
}

impl<'a> FormatWrite<'a> for TSNullKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "null")
    }
}

impl<'a> FormatWrite<'a> for TSUndefinedKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "undefined")
    }
}

impl<'a> FormatWrite<'a> for TSVoidKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "void")
    }
}

impl<'a> FormatWrite<'a> for TSSymbolKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "symbol")
    }
}

impl<'a> FormatWrite<'a> for TSThisType {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "this")
    }
}

impl<'a> FormatWrite<'a> for TSObjectKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "object")
    }
}

impl<'a> FormatWrite<'a> for TSBigIntKeyword {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "bigint")
    }
}

impl<'a> FormatWrite<'a> for TSTypeReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.type_name, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypeName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::QualifiedName(name) => name.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSQualifiedName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.left, ".", self.right])
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameterInstantiation<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        for (i, param) in self.params.iter().enumerate() {
            if i != 0 {
                write!(f, [",", space()])?;
            }
            write!(f, param)?;
        }
        write!(f, ">")
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameter<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#const {
            write!(f, "const ")?;
        }
        write!(f, self.name)?;
        if let Some(constraint) = &self.constraint {
            write!(f, [" extends ", constraint])?;
        }
        if let Some(default) = &self.default {
            write!(f, [" = ", default])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeParameterDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeAliasDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                self.declare.then_some("declare "),
                "type",
                space(),
                self.id,
                self.type_parameters,
                space(),
                "=",
                space(),
                self.type_annotation
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for TSClassImplements<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSPropertySignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::TSIndexSignature(o) => o.fmt(f),
            Self::TSPropertySignature(o) => o.fmt(f),
            Self::TSCallSignatureDeclaration(o) => o.fmt(f),
            Self::TSConstructSignatureDeclaration(o) => o.fmt(f),
            Self::TSMethodSignature(o) => o.fmt(f),
        }
    }
}

impl<'a> Format<'a> for Vec<'a, TSSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSIndexSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSCallSignatureDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSMethodSignature<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSConstructSignatureDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSIndexSignatureName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSInterfaceHeritage<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypePredicate<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypePredicateName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclarationName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::StringLiteral(s) => s.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSModuleDeclarationBody<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSModuleBlock<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTypeLiteral<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        ObjectLike::TSTypeLiteral(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for TSInferType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["infer ", self.type_parameter])
    }
}

impl<'a> FormatWrite<'a> for TSTypeQuery<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["typeof ", self.expr_name, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for TSTypeQueryExprName<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            match_ts_type_name!(Self) => self.to_ts_type_name().fmt(f),
            Self::TSImportType(decl) => decl.fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSImportType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSFunctionType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSConstructorType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSMappedType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for TSTemplateLiteralType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "`")?;
        for (index, item) in self.quasis.iter().enumerate() {
            if index != 0 {
                if let Some(types) = self.types.get(index - 1) {
                    write!(f, ["${", types, "}"])?;
                }
            }
            write!(f, dynamic_text(item.value.raw.as_str(), item.span.start));
        }
        write!(f, "`")
    }
}

impl<'a> FormatWrite<'a> for TSAsExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, " as ", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSSatisfiesExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, " satisfies ", self.type_annotation])
    }
}

impl<'a> FormatWrite<'a> for TSTypeAssertion<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "<")?;
        // var r = < <T>(x: T) => T > ((x) => { return null; });
        //          ^ make sure space is printed here.
        if matches!(self.type_annotation, TSType::TSFunctionType(_)) {
            write!(f, space())?;
        }
        write!(f, [self.type_annotation, ">", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSImportEqualsDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import ", self.id, " = ", self.module_reference])
    }
}

impl<'a> FormatWrite<'a> for TSModuleReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::ExternalModuleReference(decl) => decl.fmt(f),
            match_ts_type_name!(Self) => self.to_ts_type_name().fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for TSExternalModuleReference<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["require(", self.expression, ")"])
    }
}

impl<'a> FormatWrite<'a> for TSNonNullExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, "!"])
    }
}

impl<'a> FormatWrite<'a> for Decorator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["@", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSExportAssignment<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export = ", self.expression])
    }
}

impl<'a> FormatWrite<'a> for TSNamespaceExportDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export as namespace ", self.id])
    }
}

impl<'a> FormatWrite<'a> for TSInstantiationExpression<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.expression, self.type_arguments])
    }
}

impl<'a> FormatWrite<'a> for JSDocNullableType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix {
            write!(f, [self.type_annotation, "?"])
        } else {
            write!(f, ["?", self.type_annotation])
        }
    }
}

impl<'a> FormatWrite<'a> for JSDocNonNullableType<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.postfix {
            write!(f, [self.type_annotation, "!"])
        } else {
            write!(f, ["!", self.type_annotation])
        }
    }
}

impl<'a> FormatWrite<'a> for JSDocUnknownType {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}
