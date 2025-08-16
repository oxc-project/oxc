use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    Format,
    formatter::Formatter,
    generated::ast_nodes::{AstNode, AstNodes},
    utils::is_expression_used_as_call_argument,
    write::{BinaryLikeExpression, ExpressionLeftSide, should_flatten},
};

use super::NeedsParentheses;

// Helper function to check if a MemberExpression has a CallExpression in its object chain
fn member_has_call_object(member: &MemberExpression) -> bool {
    match member {
        MemberExpression::ComputedMemberExpression(m) => expression_is_or_contains_call(&m.object),
        MemberExpression::StaticMemberExpression(m) => expression_is_or_contains_call(&m.object),
        MemberExpression::PrivateFieldExpression(m) => expression_is_or_contains_call(&m.object),
    }
}

// Helper function to check if an Expression is or contains a CallExpression
fn expression_is_or_contains_call(expr: &Expression) -> bool {
    match expr {
        Expression::CallExpression(_) => true,
        Expression::TaggedTemplateExpression(t) => {
            // Tagged templates like x()`` where the tag is a call expression
            expression_is_or_contains_call(&t.tag)
        }
        Expression::ComputedMemberExpression(m) => expression_is_or_contains_call(&m.object),
        Expression::StaticMemberExpression(m) => expression_is_or_contains_call(&m.object),
        Expression::PrivateFieldExpression(m) => expression_is_or_contains_call(&m.object),
        _ => false,
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Expression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.as_ast_nodes() {
            // AstNodes::BooleanLiteral(it) => it.needs_parentheses(f),
            // AstNodes::NullLiteral(it) => it.needs_parentheses(f),
            AstNodes::NumericLiteral(it) => it.needs_parentheses(f),
            // AstNodes::BigIntLiteral(it) => it.needs_parentheses(f),
            // AstNodes::RegExpLiteral(it) => it.needs_parentheses(f),
            AstNodes::StringLiteral(it) => it.needs_parentheses(f),
            // AstNodes::TemplateLiteral(it) => it.needs_parentheses(f),
            AstNodes::IdentifierReference(it) => it.needs_parentheses(f),
            // AstNodes::MetaProperty(it) => it.needs_parentheses(f),
            // AstNodes::Super(it) => it.needs_parentheses(f),
            AstNodes::ArrayExpression(it) => it.needs_parentheses(f),
            AstNodes::ArrowFunctionExpression(it) => it.needs_parentheses(f),
            AstNodes::AssignmentExpression(it) => it.needs_parentheses(f),
            AstNodes::AwaitExpression(it) => it.needs_parentheses(f),
            AstNodes::BinaryExpression(it) => it.needs_parentheses(f),
            AstNodes::CallExpression(it) => it.needs_parentheses(f),
            AstNodes::ChainExpression(it) => it.needs_parentheses(f),
            AstNodes::Class(it) => it.needs_parentheses(f),
            AstNodes::ConditionalExpression(it) => it.needs_parentheses(f),
            AstNodes::Function(it) => it.needs_parentheses(f),
            AstNodes::ImportExpression(it) => it.needs_parentheses(f),
            AstNodes::LogicalExpression(it) => it.needs_parentheses(f),
            AstNodes::NewExpression(it) => it.needs_parentheses(f),
            AstNodes::ObjectExpression(it) => it.needs_parentheses(f),
            AstNodes::ParenthesizedExpression(it) => it.needs_parentheses(f),
            AstNodes::SequenceExpression(it) => it.needs_parentheses(f),
            AstNodes::TaggedTemplateExpression(it) => it.needs_parentheses(f),
            AstNodes::ThisExpression(it) => it.needs_parentheses(f),
            AstNodes::UnaryExpression(it) => it.needs_parentheses(f),
            AstNodes::UpdateExpression(it) => it.needs_parentheses(f),
            AstNodes::YieldExpression(it) => it.needs_parentheses(f),
            AstNodes::PrivateInExpression(it) => it.needs_parentheses(f),
            // AstNodes::JSXElement(it) => it.needs_parentheses(f),
            // AstNodes::JSXFragment(it) => it.needs_parentheses(f),
            AstNodes::TSAsExpression(it) => it.needs_parentheses(f),
            AstNodes::TSSatisfiesExpression(it) => it.needs_parentheses(f),
            AstNodes::TSTypeAssertion(it) => it.needs_parentheses(f),
            AstNodes::TSNonNullExpression(it) => it.needs_parentheses(f),
            AstNodes::TSInstantiationExpression(it) => it.needs_parentheses(f),
            AstNodes::V8IntrinsicExpression(it) => it.needs_parentheses(f),
            AstNodes::StaticMemberExpression(it) => it.needs_parentheses(f),
            AstNodes::ComputedMemberExpression(it) => it.needs_parentheses(f),
            AstNodes::PrivateFieldExpression(it) => it.needs_parentheses(f),
            _ => {
                // TODO: incomplete
                false
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, NumericLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstNodes::StaticMemberExpression(member) = self.parent {
            return member.object.without_parentheses().span() == self.span();
        }
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, StringLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstNodes::ExpressionStatement(stmt) = self.parent {
            // `() => "foo"`
            if let AstNodes::FunctionBody(arrow) = stmt.parent {
                if let AstNodes::ArrowFunctionExpression(arrow) = arrow.parent {
                    !arrow.expression()
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ThisExpression> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ArrayExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ObjectExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;

        // Object expressions don't need parentheses when used as function arguments
        if is_expression_used_as_call_argument(self.span, parent) {
            return false;
        }

        is_class_extends(parent, self.span())
            || is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionStatementOrArrow,
            )
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, MemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Member expressions with call expression or another member expression with call as object
        // need parentheses when used as the callee of a new expression: new (a().b)()
        if let AstNodes::NewExpression(new_expr) = self.parent {
            if new_expr.callee.span() == self.span() {
                // Check if the object of this member expression needs parens
                return member_has_call_object(self);
            }
        }
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ComputedMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Computed member expressions with call expression objects need parentheses
        // when used as the callee of a new expression: new (a()[0])()
        if let AstNodes::NewExpression(new_expr) = self.parent {
            if new_expr.callee.span() == self.span() {
                // Check if the object is or contains a call expression
                return expression_is_or_contains_call(&self.object);
            }
        }
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Static member expressions with call expression objects need parentheses
        // when used as the callee of a new expression: new (a().b)()
        if let AstNodes::NewExpression(new_expr) = self.parent {
            if new_expr.callee.span() == self.span() {
                // Check if the object is or contains a call expression
                return expression_is_or_contains_call(&self.object);
            }
        }
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, CallExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Call expressions used directly as the callee of a new expression need parentheses
        // Example: new (factory())()
        if let AstNodes::NewExpression(new_expr) = self.parent {
            return new_expr.callee.span() == self.span();
        }

        matches!(self.parent, AstNodes::ExportDefaultDeclaration(_)) && {
            let callee = &self.callee;
            let callee_span = callee.span();
            let leftmost = ExpressionLeftSide::leftmost(callee);
            // require parens for iife and
            // when the leftmost expression is not a class expression or a function expression
            callee_span != leftmost.span()
                && matches!(
                    leftmost,
                    ExpressionLeftSide::Expression(
                        Expression::ClassExpression(_) | Expression::FunctionExpression(_)
                    )
                )
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, NewExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;

        // New expressions with call expressions as callees need parentheses when being called
        if let AstNodes::CallExpression(call) = parent {
            if call.callee.span() == self.span() {
                // Only need parens if the new expression's callee is a call expression
                if let Expression::CallExpression(_) = self.callee {
                    return true;
                }
            }
        }

        is_class_extends(parent, self.span())
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, UpdateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        if self.prefix() {
            if let AstNodes::UnaryExpression(unary) = parent {
                let parent_operator = unary.operator();
                let operator = self.operator();
                return (parent_operator == UnaryOperator::UnaryPlus
                    && operator == UpdateOperator::Increment)
                    || (parent_operator == UnaryOperator::UnaryNegation
                        && operator == UpdateOperator::Decrement);
            }
        }
        unary_like_expression_needs_parens(UnaryLike::UpdateExpression(self))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, UnaryExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        match parent {
            AstNodes::UnaryExpression(parent_unary) => {
                let parent_operator = parent_unary.operator();
                let operator = self.operator();
                matches!(operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
                    && parent_operator == operator
            }
            // A user typing `!foo instanceof Bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) instance Bar` to what is really happening
            // A user typing `!foo in bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) in bar` to what is really happening
            AstNodes::BinaryExpression(e) if e.operator().is_relational() => true,
            _ => unary_like_expression_needs_parens(UnaryLike::UnaryExpression(self)),
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, BinaryExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        (self.operator.is_in() && is_in_for_initializer(self))
            || binary_like_needs_parens(BinaryLikeExpression::BinaryExpression(self))
    }
}

/// Add parentheses if the `in` is inside of a `for` initializer (see tests).
fn is_in_for_initializer(expr: &AstNode<'_, BinaryExpression<'_>>) -> bool {
    let mut parent = expr.parent;
    loop {
        match parent {
            AstNodes::ExpressionStatement(stmt) => {
                let grand_parent = parent.parent();

                if matches!(grand_parent, AstNodes::FunctionBody(_)) {
                    let grand_grand_parent = grand_parent.parent();
                    if matches!(
                        grand_grand_parent,
                        AstNodes::ArrowFunctionExpression(arrow) if arrow.expression()
                    ) {
                        parent = grand_grand_parent;
                        continue;
                    }
                }

                return false;
            }
            AstNodes::ForStatement(stmt) => {
                return stmt
                    .init
                    .as_ref()
                    .is_some_and(|init| init.span().contains_inclusive(expr.span));
            }
            AstNodes::ForInStatement(stmt) => {
                return stmt.left.span().contains_inclusive(expr.span);
            }
            AstNodes::Program(_) => {
                return false;
            }
            _ => {
                parent = parent.parent();
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, PrivateInExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, LogicalExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        if let AstNodes::LogicalExpression(parent) = parent {
            parent.operator() != self.operator()
        } else if self.operator().is_coalesce()
            && matches!(parent, AstNodes::ConditionalExpression(_))
        {
            true
        } else {
            binary_like_needs_parens(BinaryLikeExpression::LogicalExpression(self))
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ConditionalExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        if matches!(
            parent,
            AstNodes::UnaryExpression(_)
                | AstNodes::AwaitExpression(_)
                | AstNodes::TSTypeAssertion(_)
                | AstNodes::TSAsExpression(_)
                | AstNodes::TSSatisfiesExpression(_)
                | AstNodes::SpreadElement(_)
                | AstNodes::LogicalExpression(_)
                | AstNodes::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstNodes::ConditionalExpression(e) = parent {
            e.test.without_parentheses().span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Function<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if self.r#type() != FunctionType::FunctionExpression {
            return false;
        }
        let parent = self.parent;

        // Check if this function is an argument in a call/new expression
        // If so, it doesn't need parentheses
        match parent {
            AstNodes::CallExpression(call) => {
                // Check if this function is in the arguments array
                !call.arguments.iter().any(|arg| match arg {
                    Argument::FunctionExpression(func) => func.span() == self.span(),
                    _ => false,
                })
            }
            AstNodes::NewExpression(new_expr) => {
                // Check if this function is in the arguments array
                !new_expr.arguments.iter().any(|arg| match arg {
                    Argument::FunctionExpression(func) => func.span() == self.span(),
                    _ => false,
                })
            }
            AstNodes::TemplateLiteral(_) => true,
            _ => is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionOrExportDefault,
            ),
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, AssignmentExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // TODO
        match self.parent {
            AstNodes::ExpressionStatement(parent) => {
                let parent_parent = parent.parent;
                if let AstNodes::FunctionBody(body) = parent_parent {
                    let parent_parent_parent = body.parent;
                    matches!(parent_parent_parent, AstNodes::ArrowFunctionExpression(arrow) if arrow.expression())
                } else {
                    is_first_in_statement(
                        self.span,
                        self.parent,
                        FirstInStatementMode::ExpressionStatementOrArrow,
                    ) && matches!(self.left, AssignmentTarget::ObjectAssignmentTarget(_))
                }
            }
            AstNodes::AssignmentExpression(_) | AstNodes::ComputedMemberExpression(_) => false,
            AstNodes::ForStatement(stmt)
                if stmt.init.as_ref().is_some_and(|init| init.span() == self.span()) =>
            {
                false
            }
            _ => true,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        !matches!(
            self.parent,
            AstNodes::ReturnStatement(_)
                | AstNodes::ForStatement(_)
                | AstNodes::ExpressionStatement(_)
                | AstNodes::SequenceExpression(_)
        )
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, AwaitExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        await_or_yield_needs_parens(self.span(), self.parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ChainExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            AstNodes::CallExpression(call) => {
                // Only add parentheses if this chain expression is the callee, not an argument
                call.callee.span() == self.span()
            }
            AstNodes::NewExpression(new) => {
                // Only add parentheses if this chain expression is the callee, not an argument
                new.callee.span() == self.span()
            }
            AstNodes::StaticMemberExpression(member) => {
                // Only add parentheses if this chain expression is the object, not the property
                member.object.span() == self.span()
            }
            AstNodes::ComputedMemberExpression(member) => member.object.span() == self.span(),
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Class<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if self.r#type() != ClassType::ClassExpression {
            return false;
        }
        let parent = self.parent;

        // Class expressions don't need parentheses when used as function arguments
        if is_expression_used_as_call_argument(self.span, parent) {
            return false;
        }

        match parent {
            AstNodes::ExportDefaultDeclaration(_) => true,
            _ => is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionOrExportDefault,
            ),
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // ParenthesizedExpression nodes are only created when preserve_parens is true,
        // which is not the case in our conformance tests. This implementation is kept
        // for when preserve_parens is enabled.
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ArrowFunctionExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        if matches!(
            parent,
            AstNodes::TSAsExpression(_)
                | AstNodes::TSSatisfiesExpression(_)
                | AstNodes::TSTypeAssertion(_)
                | AstNodes::UnaryExpression(_)
                | AstNodes::AwaitExpression(_)
                | AstNodes::LogicalExpression(_)
                | AstNodes::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstNodes::ConditionalExpression(e) = parent {
            e.test.without_parentheses().span() == self.span()
        } else if let AstNodes::CallExpression(call) = parent {
            // Only add parentheses if this arrow function is the callee, not an argument
            call.callee.span() == self.span()
        } else if let AstNodes::NewExpression(new_expr) = parent {
            // Only add parentheses if this arrow function is the callee, not an argument
            new_expr.callee.span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, YieldExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        matches!(parent, AstNodes::AwaitExpression(_) | AstNodes::TSTypeAssertion(_))
            || await_or_yield_needs_parens(self.span(), parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ImportExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::NewExpression(_))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, V8IntrinsicExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXEmptyExpression> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(
            self.parent,
            AstNodes::ComputedMemberExpression(_)
                | AstNodes::StaticMemberExpression(_)
                | AstNodes::PrivateFieldExpression(_)
                | AstNodes::IdentifierReference(_)
                | AstNodes::AssignmentExpression(_)
                | AstNodes::AssignmentTargetWithDefault(_)
        )
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        is_class_extends(parent, self.span())
            || (matches!(parent, AstNodes::NewExpression(_))
                && member_chain_callee_needs_parens(self.expression()))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSInstantiationExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let expr = match self.parent {
            AstNodes::StaticMemberExpression(expr) => &expr.object,
            AstNodes::ComputedMemberExpression(expr) => &expr.object,
            AstNodes::PrivateFieldExpression(expr) => &expr.object,
            _ => return false,
        };

        self.span == expr.span()
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, IdentifierReference<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Keywords like 'let' should not get parentheses in member expressions
        // This prevents cases like `(let)[a]` when it should be `let[a]`
        if self.name == "let" {
            // Check if this identifier is in a member expression context
            match self.parent {
                AstNodes::ComputedMemberExpression(member) => {
                    // If 'let' is the object of a computed member expression, don't add parentheses
                    member.object.span() == self.span()
                }
                AstNodes::StaticMemberExpression(member) => {
                    // If 'let' is the object of a static member expression, don't add parentheses
                    member.object.span() == self.span()
                }
                AstNodes::AssignmentExpression(assignment) => {
                    // If 'let' is the left side of an assignment, don't add parentheses
                    assignment.left.span() == self.span()
                }
                AstNodes::VariableDeclarator(declarator) => {
                    // If 'let' is the binding in a variable declarator, don't add parentheses
                    declarator.id.span() == self.span()
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

fn binary_like_needs_parens(binary_like: BinaryLikeExpression<'_, '_>) -> bool {
    let parent = match binary_like.parent() {
        AstNodes::TSAsExpression(_)
        | AstNodes::TSSatisfiesExpression(_)
        | AstNodes::TSTypeAssertion(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::TSNonNullExpression(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::StaticMemberExpression(_)
        | AstNodes::TaggedTemplateExpression(_) => return true,
        AstNodes::CallExpression(call) => {
            // Only add parentheses if this expression is the callee, not an argument
            return call.callee.span() == binary_like.span();
        }
        AstNodes::NewExpression(new_expr) => {
            // Only add parentheses if this expression is the callee, not an argument
            return new_expr.callee.span() == binary_like.span();
        }
        AstNodes::Class(class) => {
            return class.super_class.as_ref().is_some_and(|super_class| {
                super_class.span().contains_inclusive(binary_like.span())
            });
        }
        AstNodes::BinaryExpression(binary) => BinaryLikeExpression::BinaryExpression(binary),
        AstNodes::LogicalExpression(logical) => BinaryLikeExpression::LogicalExpression(logical),
        _ => return false,
    };

    let parent_operator = parent.operator();
    let operator = binary_like.operator();
    let parent_precedence = parent_operator.precedence();
    let precedence = operator.precedence();

    // If the parent has a higher precedence than parentheses are necessary to not change the semantic meaning
    // when re-parsing.
    if parent_precedence > precedence {
        return true;
    }

    let is_right = parent.right().span() == binary_like.span();

    // `a ** b ** c`
    if is_right && parent_precedence == precedence {
        return true;
    }

    // Add parentheses around bitwise and bit shift operators
    // `a * 3 >> 5` -> `(a * 3) >> 5`
    if parent_precedence.is_bitwise() || parent_precedence.is_shift() {
        return true;
    }

    // `a % 4 + 4` -> `(a % 4) + 4)`
    if parent_precedence < precedence && operator.is_remainder() {
        return parent_precedence.is_additive();
    }

    parent_precedence == precedence && !should_flatten(parent_operator, operator)
}

fn member_chain_callee_needs_parens(e: &Expression) -> bool {
    std::iter::successors(Some(e), |e| match e {
        Expression::ComputedMemberExpression(e) => Some(&e.object),
        Expression::StaticMemberExpression(e) => Some(&e.object),
        Expression::TSNonNullExpression(e) => Some(&e.expression),
        _ => None,
    })
    .any(|object| matches!(object, Expression::CallExpression(_)))
}

#[derive(Clone, Copy)]
enum UnaryLike<'a, 'b> {
    UpdateExpression(&'b AstNode<'a, UpdateExpression<'a>>),
    UnaryExpression(&'b AstNode<'a, UnaryExpression<'a>>),
}

impl UnaryLike<'_, '_> {
    fn parent(&self) -> &AstNodes<'_> {
        match self {
            Self::UpdateExpression(e) => e.parent,
            Self::UnaryExpression(e) => e.parent,
        }
    }
}

impl GetSpan for UnaryLike<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::UpdateExpression(e) => e.span(),
            Self::UnaryExpression(e) => e.span(),
        }
    }
}

fn unary_like_expression_needs_parens(node: UnaryLike<'_, '_>) -> bool {
    match node.parent() {
        AstNodes::BinaryExpression(e) => {
            e.operator() == BinaryOperator::Exponential && e.left().span() == node.span()
        }
        parent => update_or_lower_expression_needs_parens(node.span(), parent),
    }
}

/// Returns `true` if an expression with lower precedence than an update expression needs parentheses.
///
/// This is generally the case if the expression is used in a left hand side, or primary expression context.
fn update_or_lower_expression_needs_parens(span: Span, parent: &AstNodes<'_>) -> bool {
    match parent {
        AstNodes::TSNonNullExpression(_)
        | AstNodes::StaticMemberExpression(_)
        | AstNodes::TemplateLiteral(_)
        | AstNodes::TaggedTemplateExpression(_) => return true,

        // For call expressions, only add parentheses if this expression is the callee, not an argument
        AstNodes::CallExpression(call) => {
            return call.callee.span() == span;
        }

        // For new expressions, only add parentheses if this expression is the callee, not an argument
        AstNodes::NewExpression(new_expr) => {
            return new_expr.callee.span() == span;
        }

        AstNodes::ComputedMemberExpression(computed_member_expr) => {
            return computed_member_expr.object.span() == span;
        }

        _ => {}
    }

    if is_class_extends(parent, span) {
        return true;
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstInStatementMode {
    /// Considers [ExpressionStatement] and the body of [ArrowFunctionExpression] as the first statement.
    ExpressionStatementOrArrow,
    /// Considers [ExpressionStatement] and [ExportDefaultDeclaration] as the first statement.
    ExpressionOrExportDefault,
}

/// Returns `true` if this node is at the start of an expression (depends on the passed `mode`).
///
/// Traverses upwards the tree for as long as the `node` is the left most expression until the node isn't
/// the left most node or reached a statement.
fn is_first_in_statement(
    mut current_span: Span,
    mut parent: &AstNodes<'_>,
    mode: FirstInStatementMode,
) -> bool {
    let mut is_not_first_iteration = false;
    loop {
        match parent {
            AstNodes::ExpressionStatement(stmt) => {
                if matches!(stmt.parent.parent(), AstNodes::ArrowFunctionExpression(arrow) if arrow.expression)
                {
                    if mode == FirstInStatementMode::ExpressionStatementOrArrow {
                        if is_not_first_iteration
                            && matches!(
                                stmt.expression,
                                Expression::SequenceExpression(_)
                                    | Expression::AssignmentExpression(_)
                            )
                        {
                            // The original node doesn't need parens,
                            // because an ancestor requires parens.
                            break;
                        }
                    } else {
                        return false;
                    }
                }

                return true;
            }
            AstNodes::StaticMemberExpression(_)
            | AstNodes::TemplateLiteral(_)
            | AstNodes::TaggedTemplateExpression(_)
            | AstNodes::CallExpression(_)
            | AstNodes::NewExpression(_)
            | AstNodes::TSAsExpression(_)
            | AstNodes::TSSatisfiesExpression(_)
            | AstNodes::TSNonNullExpression(_) => {}
            AstNodes::SequenceExpression(sequence) => {
                if sequence.expressions.first().unwrap().span() != current_span {
                    break;
                }
            }
            AstNodes::ComputedMemberExpression(member) => {
                if member.object.span() != current_span {
                    break;
                }
            }
            AstNodes::AssignmentExpression(assignment) => {
                if assignment.left.span() != current_span {
                    break;
                }
            }
            AstNodes::ConditionalExpression(conditional) => {
                if conditional.test.span() != current_span {
                    break;
                }
            }
            AstNodes::BinaryExpression(binary) => {
                if binary.left.span() != current_span {
                    break;
                }
            }
            AstNodes::LogicalExpression(logical) => {
                if logical.left.span() != current_span {
                    break;
                }
            }
            AstNodes::ExportDefaultDeclaration(_)
                if mode == FirstInStatementMode::ExpressionOrExportDefault =>
            {
                return !is_not_first_iteration;
            }
            _ => break,
        }
        current_span = parent.span();
        parent = parent.parent();
        is_not_first_iteration = true;
    }

    false
}

fn await_or_yield_needs_parens(span: Span, node: &AstNodes<'_>) -> bool {
    if matches!(
        node,
        AstNodes::UnaryExpression(_)
            | AstNodes::TSAsExpression(_)
            | AstNodes::TSSatisfiesExpression(_)
            | AstNodes::SpreadElement(_)
            | AstNodes::LogicalExpression(_)
            | AstNodes::BinaryExpression(_),
    ) {
        return true;
    }
    if let AstNodes::ConditionalExpression(e) = node {
        e.test.without_parentheses().span() == span
    } else {
        update_or_lower_expression_needs_parens(span, node)
    }
}

fn ts_as_or_satisfies_needs_parens(parent: &AstNodes<'_>) -> bool {
    matches!(
        parent,
        AstNodes::ComputedMemberExpression(_)
            | AstNodes::StaticMemberExpression(_)
            | AstNodes::PrivateFieldExpression(_)
            | AstNodes::AssignmentExpression(_)
            | AstNodes::AssignmentTargetWithDefault(_)
    )
}

fn is_class_extends(parent: &AstNodes<'_>, span: Span) -> bool {
    if let AstNodes::Class(c) = parent {
        return c.super_class.as_ref().is_some_and(|c| c.without_parentheses().span() == span);
    }
    false
}
