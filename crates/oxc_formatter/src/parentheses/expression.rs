use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    keyword::is_reserved_keyword,
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

// Helper function to check if an expression can be used unparenthesized in a decorator
// Based on Prettier's isDecoratorMemberExpression
fn is_decorator_member_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(_) => true,
        Expression::StaticMemberExpression(m) if !m.optional => {
            // Non-optional static member access like a.b.c
            is_decorator_member_expression(&m.object)
        }
        Expression::ComputedMemberExpression(m) if !m.optional => {
            // Non-optional computed member access like a[0] or a["prop"]
            // Note: Prettier allows this without parentheses
            is_decorator_member_expression(&m.object)
        }
        _ => false,
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Expression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.as_ast_nodes() {
            AstNodes::BooleanLiteral(it) => it.needs_parentheses(f),
            AstNodes::NullLiteral(it) => it.needs_parentheses(f),
            AstNodes::NumericLiteral(it) => it.needs_parentheses(f),
            AstNodes::BigIntLiteral(it) => it.needs_parentheses(f),
            AstNodes::RegExpLiteral(it) => it.needs_parentheses(f),
            AstNodes::StringLiteral(it) => it.needs_parentheses(f),
            AstNodes::TemplateLiteral(it) => it.needs_parentheses(f),
            AstNodes::IdentifierReference(it) => it.needs_parentheses(f),
            AstNodes::MetaProperty(it) => it.needs_parentheses(f),
            AstNodes::Super(it) => it.needs_parentheses(f),
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
            AstNodes::JSXElement(it) => it.needs_parentheses(f),
            AstNodes::JSXFragment(it) => it.needs_parentheses(f),
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

impl<'a> NeedsParentheses<'a> for AstNode<'a, IdentifierReference<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.name.as_str() {
            "async" => {
                matches!(self.parent, AstNodes::ForOfStatement(stmt) if !stmt.r#await && stmt.left.span().contains_inclusive(self.span))
            }
            "let" => {
                // Check if 'let' is used as the object of a computed member expression
                if let AstNodes::ComputedMemberExpression(member) = self.parent {
                    if member.object.span() == self.span() {
                        // Check various contexts where 'let' needs or doesn't need parentheses
                        let mut parent = member.parent;

                        // Walk up to find the context
                        loop {
                            match parent {
                                // In call arguments, don't add extra parentheses
                                AstNodes::CallExpression(_) | AstNodes::NewExpression(_) => {
                                    return false;
                                }
                                // In for-statement contexts, need parentheses
                                AstNodes::ForStatement(_)
                                | AstNodes::ForInStatement(_)
                                | AstNodes::ForOfStatement(_) => {
                                    return true;
                                }
                                // At the start of a statement, need parentheses
                                AstNodes::ExpressionStatement(_) | AstNodes::Program(_) => {
                                    return is_first_in_statement(
                                        member.span(),
                                        member.parent,
                                        FirstInStatementMode::ExpressionStatementOrArrow,
                                    );
                                }
                                _ => parent = parent.parent(),
                            }
                        }
                    }
                }

                // Check for-statement contexts where 'let' needs parentheses
                let mut parent = self.parent;
                loop {
                    match parent {
                        AstNodes::Program(_) | AstNodes::ExpressionStatement(_) => return false,
                        AstNodes::ForOfStatement(stmt) => {
                            return stmt.left.span().contains_inclusive(self.span);
                        }
                        AstNodes::TSSatisfiesExpression(expr) => {
                            return expr.expression.span() == self.span();
                        }
                        AstNodes::ForInStatement(stmt) => {
                            return stmt.left.span().contains_inclusive(self.span);
                        }
                        AstNodes::ForStatement(stmt) => {
                            return stmt
                                .init
                                .as_ref()
                                .is_some_and(|init| init.span().contains_inclusive(self.span));
                        }
                        _ => parent = parent.parent(),
                    }
                }
            }
            name => {
                // <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/needs-parens.js#L123-L133>
                matches!(
                    self.parent,
                    AstNodes::TSSatisfiesExpression(_) | AstNodes::TSAsExpression(_)
                ) && matches!(
                    name,
                    "await"
                        | "interface"
                        | "module"
                        | "using"
                        | "yield"
                        | "component"
                        | "hook"
                        | "type"
                )
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, BooleanLiteral> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, NullLiteral> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, BigIntLiteral<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, RegExpLiteral<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TemplateLiteral<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, MetaProperty<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Super> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
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
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ArrayExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ObjectExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let span = self.span();
        let parent = self.parent;

        // Object expressions don't need parentheses when used as function arguments
        if is_expression_used_as_call_argument(span, parent) {
            return false;
        }

        // Object expressions don't need parentheses when used as the expression of a cast
        // that is itself used as an argument
        if let AstNodes::TSAsExpression(as_expr) = parent
            && is_expression_used_as_call_argument(as_expr.span, as_expr.parent)
        {
            return false;
        }
        if let AstNodes::TSSatisfiesExpression(satisfies_expr) = parent
            && is_expression_used_as_call_argument(satisfies_expr.span, satisfies_expr.parent)
        {
            return false;
        }

        is_class_extends(parent, span)
            || is_first_in_statement(span, parent, FirstInStatementMode::ExpressionStatementOrArrow)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TaggedTemplateExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, MemberExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // Member expressions with call expression or another member expression with call as object
        // need parentheses when used as the callee of a new expression: new (a().b)()
        if let AstNodes::NewExpression(new_expr) = self.parent {
            let span = self.span();
            if new_expr.callee.span() == span {
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
            let span = self.span();
            if new_expr.callee.span() == span {
                // Check if the object is or contains a call expression
                return expression_is_or_contains_call(&self.object);
            }
        }

        // Computed member expressions need parentheses in decorators
        // Example: @(decorators[0]) and @(decorators?.[0])
        if let AstNodes::Decorator(_) = self.parent {
            return true;
        }

        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::NewExpression(_)) && {
            ExpressionLeftSide::Expression(self.object()).iter().any(|expr| {
                matches!(expr, ExpressionLeftSide::Expression(e) if
                    matches!(e.as_ref(), Expression::CallExpression(_))
                )
            })
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, CallExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            AstNodes::NewExpression(_) => true,
            AstNodes::Decorator(_) => !is_decorator_member_expression(&self.callee),
            AstNodes::ExportDefaultDeclaration(_) => {
                let callee = &self.callee();
                let callee_span = callee.span();
                let leftmost = ExpressionLeftSide::leftmost(callee);
                // require parens for iife and
                // when the leftmost expression is not a class expression or a function expression
                callee_span != leftmost.span()
                    && matches!(
                        leftmost,
                        ExpressionLeftSide::Expression(e)
                         if matches!(e.as_ref(), Expression::ClassExpression(_) | Expression::FunctionExpression(_))
                    )
            }
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, NewExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let span = self.span();
        let parent = self.parent;

        // New expressions with call expressions as callees need parentheses when being called
        if let AstNodes::CallExpression(call) = parent
            && call.callee.span() == span
        {
            // Only need parens if the new expression's callee is a call expression
            if let Expression::CallExpression(_) = self.callee {
                return true;
            }
        }

        is_class_extends(parent, span)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, UpdateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent;
        if self.prefix()
            && let AstNodes::UnaryExpression(unary) = parent
        {
            let parent_operator = unary.operator();
            let operator = self.operator();
            return (parent_operator == UnaryOperator::UnaryPlus
                && operator == UpdateOperator::Increment)
                || (parent_operator == UnaryOperator::UnaryNegation
                    && operator == UpdateOperator::Decrement);
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
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        is_class_extends(self.parent, self.span)
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
                | AstNodes::JSXSpreadAttribute(_)
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

        // Function expressions in call/new arguments don't need parentheses
        // unless they are the callee
        if let AstNodes::CallExpression(call) = parent {
            // Only add parentheses if this function is the callee, not an argument
            return call.callee.span() == self.span;
        }

        matches!(parent, AstNodes::NewExpression(_) | AstNodes::TaggedTemplateExpression(_))
            || is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionOrExportDefault,
            )
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, AssignmentExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            // Expression statements, only object destructuring needs parens:
            // - `a = b` = no parens
            // - `{ x } = obj` -> `({ x } = obj)` = needed to prevent parsing as block statement
            // - `() => { x } = obj` -> `() => ({ x } = obj)` = needed in arrow function body
            // - `() => a = b` -> `() => (a = b)` = also parens needed
            AstNodes::ExpressionStatement(parent) => {
                let parent_parent = parent.parent;
                if let AstNodes::FunctionBody(body) = parent_parent {
                    let parent_parent_parent = body.parent;
                    if matches!(parent_parent_parent, AstNodes::ArrowFunctionExpression(arrow) if arrow.expression())
                    {
                        return true;
                    }
                }
                matches!(self.left, AssignmentTarget::ObjectAssignmentTarget(_))
                    && is_first_in_statement(
                        self.span,
                        self.parent,
                        FirstInStatementMode::ExpressionStatementOrArrow,
                    )
            }
            // Sequence expressions, need to traverse up to find if we're in a for statement context:
            // - `a = 1, b = 2` in for loops don't need parens
            // - `(a = 1, b = 2)` elsewhere usually need parens
            AstNodes::SequenceExpression(sequence) => {
                let mut current_parent = self.parent;
                loop {
                    match current_parent {
                        AstNodes::SequenceExpression(_) | AstNodes::ParenthesizedExpression(_) => {
                            current_parent = current_parent.parent();
                        }
                        AstNodes::ForStatement(for_stmt) => {
                            let is_initializer = for_stmt
                                .init
                                .as_ref()
                                .is_some_and(|init| init.span().contains_inclusive(self.span()));
                            let is_update = for_stmt.update.as_ref().is_some_and(|update| {
                                update.span().contains_inclusive(self.span())
                            });
                            return !(is_initializer || is_update);
                        }
                        _ => break,
                    }
                }
                true
            }
            // `interface { [a = 1]; }` and `class { [a = 1]; }` not need parens
            AstNodes::TSPropertySignature(_) | AstNodes::PropertyDefinition(_) |
            // Never need parentheses in these contexts:
            // - `a = (b = c)` = nested assignments don't need extra parens
            AstNodes::AssignmentExpression(_) => false,
            // Computed member expressions: need parens when assignment is the object
            // - `obj[a = b]` = no parens needed for property
            // - `(a = b)[obj]` = parens needed for object
            AstNodes::ComputedMemberExpression(member) => member.object.span() == self.span(),
            // For statements, no parens needed in initializer or update sections:
            // - `for (a = 1; ...; a = 2) {}` = both assignments don't need parens
            AstNodes::ForStatement(stmt) => {
                let is_initializer =
                    stmt.init.as_ref().is_some_and(|init| init.span() == self.span());
                let is_update =
                    stmt.update.as_ref().is_some_and(|update| update.span() == self.span());
                !(is_initializer || is_update)
            }
            // Arrow functions, only need parens if assignment is the direct body:
            // - `() => a = b` -> `() => (a = b)` = needed
            // - `() => someFunc(a = b)` = no extra parens needed
            AstNodes::ArrowFunctionExpression(arrow) => {
                arrow.expression() && arrow.body.span() == self.span()
            }
            // Default: need parentheses in most other contexts
            // - `new (a = b)`
            // - `(a = b).prop`
            // - `await (a = b)`
            // - etc.
            _ => true,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, SequenceExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        !matches!(
            self.parent,
            AstNodes::ReturnStatement(_)
            // There's a precedence for writing `x++, y++`
            | AstNodes::ForStatement(_)
            | AstNodes::ExpressionStatement(_)
            | AstNodes::SequenceExpression(_)
            // Handled as part of the arrow function formatting
            | AstNodes::ArrowFunctionExpression(_)
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
        let span = self.span();

        // Chain expressions don't need parentheses when used as call arguments
        if is_expression_used_as_call_argument(span, &self.parent) {
            return false;
        }

        match self.parent {
            AstNodes::NewExpression(_) => true,
            AstNodes::CallExpression(call) => !call.optional,
            AstNodes::StaticMemberExpression(member) => !member.optional,
            AstNodes::ComputedMemberExpression(member) => {
                !member.optional && member.object.span() == self.span()
            }
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, Class<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if self.r#type() != ClassType::ClassExpression {
            return false;
        }
        let span = self.span();
        let parent = self.parent;

        // Decorated class expressions need parentheses when used in extends clause
        if !self.decorators.is_empty() && is_class_extends(parent, span) {
            return true;
        }

        // Class expressions don't need parentheses when used as function arguments
        if is_expression_used_as_call_argument(span, parent) {
            return false;
        }

        match parent {
            AstNodes::ExportDefaultDeclaration(_) => true,
            _ => {
                is_first_in_statement(span, parent, FirstInStatementMode::ExpressionOrExportDefault)
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ParenthesizedExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        unreachable!("Already disabled `preserveParens` option in the parser")
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, ArrowFunctionExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let span = self.span();
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
            e.test.without_parentheses().span() == span
        } else if let AstNodes::CallExpression(call) = parent {
            // Only add parentheses if this arrow function is the callee, not an argument
            call.callee.span() == span
        } else if let AstNodes::NewExpression(new_expr) = parent {
            // Only add parentheses if this arrow function is the callee, not an argument
            new_expr.callee.span() == span
        } else {
            update_or_lower_expression_needs_parens(span, parent)
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
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXExpression<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, JSXEmptyExpression> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.span(), &self.expression, self.parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.span(), &self.expression, self.parent)
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSTypeAssertion<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            AstNodes::TSAsExpression(_) | AstNodes::TSSatisfiesExpression(_) => true,
            AstNodes::BinaryExpression(binary) => {
                matches!(binary.operator, BinaryOperator::ShiftLeft)
            }
            _ => type_cast_like_needs_parens(self.span(), self.parent),
        }
    }
}

fn type_cast_like_needs_parens(span: Span, parent: &AstNodes<'_>) -> bool {
    #[expect(clippy::match_same_arms)] // for better readability
    match parent {
        AstNodes::ExportDefaultDeclaration(_)
        | AstNodes::TSTypeAssertion(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::TSNonNullExpression(_)
        // Callee
        | AstNodes::CallExpression(_)
        | AstNodes::NewExpression(_)
        // template tag
        | AstNodes::TaggedTemplateExpression(_)
        // in spread
        | AstNodes::JSXSpreadChild(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::JSXSpreadAttribute(_)
        // static member
        | AstNodes::StaticMemberExpression(_) => true,
        AstNodes::ComputedMemberExpression(member) => {
            member.object.span() == span
        }
        // assignment left hand side
        AstNodes::UpdateExpression(_) | AstNodes::AssignmentTargetWithDefault(_) => true,
        AstNodes::AssignmentExpression(assignment) => {
            assignment.left.span() == span
        }
        _ => is_class_extends(parent, span),
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSNonNullExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let span = self.span();
        let parent = self.parent;
        is_class_extends(parent, span)
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

fn binary_like_needs_parens(binary_like: BinaryLikeExpression<'_, '_>) -> bool {
    let parent = match binary_like.parent() {
        AstNodes::TSAsExpression(_)
        | AstNodes::TSSatisfiesExpression(_)
        | AstNodes::TSTypeAssertion(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::TSNonNullExpression(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::JSXSpreadAttribute(_)
        | AstNodes::CallExpression(_)
        | AstNodes::NewExpression(_)
        | AstNodes::ChainExpression(_)
        | AstNodes::StaticMemberExpression(_)
        | AstNodes::TaggedTemplateExpression(_) => return true,
        AstNodes::ComputedMemberExpression(computed) => {
            return computed.object.span() == binary_like.span();
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

    // Only cache span calculation for multiple uses

    let parent_precedence = parent_operator.precedence();
    let precedence = operator.precedence();

    // If the parent has a higher precedence than parentheses are necessary to not change the semantic meaning
    // when re-parsing.
    if parent_precedence > precedence {
        return true;
    }

    // Cache span for multiple comparisons to avoid recalculation
    let binary_span = binary_like.span();
    let is_right = parent.right().span() == binary_span;

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
        Expression::TaggedTemplateExpression(e) => Some(&e.tag),
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
    if matches!(
        parent,
        AstNodes::TSNonNullExpression(_)
            | AstNodes::CallExpression(_)
            | AstNodes::NewExpression(_)
            | AstNodes::StaticMemberExpression(_)
            | AstNodes::TaggedTemplateExpression(_)
            | AstNodes::ComputedMemberExpression(_)
    ) || is_class_extends(parent, span)
    {
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
            | AstNodes::ChainExpression(_)
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
            | AstNodes::BinaryExpression(_)
            | AstNodes::PrivateInExpression(_),
    ) {
        return true;
    }
    if let AstNodes::ConditionalExpression(e) = node {
        e.test.without_parentheses().span() == span
    } else {
        update_or_lower_expression_needs_parens(span, node)
    }
}

fn ts_as_or_satisfies_needs_parens(
    span: Span,
    inner: &Expression<'_>,
    parent: &AstNodes<'_>,
) -> bool {
    match parent {
        AstNodes::ConditionalExpression(_)
        // Binary-like
        | AstNodes::LogicalExpression(_)
        | AstNodes::BinaryExpression(_) => true,
        // `export default (function foo() {} as bar)` and `export default (class {} as bar)`
        AstNodes::ExportDefaultDeclaration(_) =>
            matches!(inner, Expression::FunctionExpression(_) | Expression::ClassExpression(_)),
        _ => {
            type_cast_like_needs_parens(span, parent)
        }
    }
}

fn is_class_extends(parent: &AstNodes<'_>, span: Span) -> bool {
    if let AstNodes::Class(c) = parent {
        return c.super_class.as_ref().is_some_and(|c| c.without_parentheses().span() == span);
    }
    false
}

fn jsx_element_or_fragment_needs_paren(span: Span, parent: &AstNodes<'_>) -> bool {
    if is_class_extends(parent, span) {
        return true;
    }

    match parent {
        AstNodes::BinaryExpression(binary) => {
            let is_left = binary.left.span() == span;
            binary.operator == BinaryOperator::LessThan && is_left
        }
        AstNodes::TSAsExpression(_)
        | AstNodes::TSSatisfiesExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::StaticMemberExpression(_)
        | AstNodes::ComputedMemberExpression(_)
        | AstNodes::SequenceExpression(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::TSNonNullExpression(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::CallExpression(_)
        | AstNodes::NewExpression(_)
        | AstNodes::TaggedTemplateExpression(_)
        | AstNodes::JSXSpreadAttribute(_)
        | AstNodes::JSXSpreadChild(_) => true,
        _ => false,
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXElement<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        jsx_element_or_fragment_needs_paren(self.span, self.parent)
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXFragment<'_>> {
    fn needs_parentheses(&self, f: &Formatter<'_, '_>) -> bool {
        jsx_element_or_fragment_needs_paren(self.span, self.parent)
    }
}
