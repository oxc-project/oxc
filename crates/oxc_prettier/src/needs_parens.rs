//! Direct port of needs-parens for adding or removing parentheses.
//!
//! See <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/needs-parens.js>

use oxc_ast::{
    ast::{
        match_member_expression, AssignmentTarget, ChainElement, ExportDefaultDeclarationKind,
        Expression, ForStatementInit, ForStatementLeft, MemberExpression, ObjectExpression,
        SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::{BinaryOperator, UnaryOperator, UpdateOperator},
    precedence::GetPrecedence,
};

use crate::{binaryish::BinaryishOperator, Prettier};

impl<'a> Prettier<'a> {
    // NOTE: Why this takes `mut`...?
    pub(crate) fn need_parens(&mut self, kind: AstKind<'a>) -> bool {
        if matches!(kind, AstKind::Program(_)) || kind.is_statement() || kind.is_declaration() {
            return false;
        }

        if matches!(kind, AstKind::ObjectExpression(e) if self.check_object_expression(e))
            || self.check_for_of_stmt_head_starts_with_async_or_let(kind)
            || self.check_for_of_stmt_head_starts_with_let(kind)
            || self.check_let_object(kind)
            || self.check_parent_kind(kind)
            || self.check_kind(kind)
        {
            return true;
        }

        false
    }

    fn check_kind(&self, kind: AstKind<'a>) -> bool {
        let parent_kind = self.parent_kind();
        let parent_parent_kind = self.parent_parent_kind();
        match kind {
            AstKind::NumericLiteral(literal) => {
                matches!(parent_kind, AstKind::MemberExpression(e) if e.object().span() == literal.span)
            }
            AstKind::SequenceExpression(e) => self.check_sequence_expression(e.span),
            AstKind::ObjectExpression(e) => self.check_object_function_class(e.span),
            AstKind::Function(f) if f.is_expression() => {
                if self.check_object_function_class(f.span) {
                    return true;
                }
                match parent_kind {
                    AstKind::CallExpression(call_expr) => call_expr.callee.span() == f.span,
                    AstKind::NewExpression(new_expr) => new_expr.callee.span() == f.span,
                    AstKind::TaggedTemplateExpression(_) => true,
                    _ => false,
                }
            }
            AstKind::Class(c) if c.is_expression() => {
                if self.check_object_function_class(c.span) {
                    return true;
                }
                if let AstKind::NewExpression(new_expr) = parent_kind {
                    return new_expr.callee.span() == c.span;
                }
                false
            }
            AstKind::AssignmentExpression(assign_expr) => match parent_kind {
                AstKind::ArrowFunctionExpression(arrow_expr)
                    if arrow_expr
                        .get_expression()
                        .is_some_and(|e| e.span() == assign_expr.span) =>
                {
                    true
                }
                AstKind::AssignmentExpression(_) => false,
                AstKind::ForStatement(stmt)
                    if stmt.init.as_ref().is_some_and(|e| e.span() == assign_expr.span)
                        || stmt.update.as_ref().is_some_and(|e| e.span() == assign_expr.span) =>
                {
                    false
                }
                AstKind::ExpressionStatement(_) => {
                    matches!(assign_expr.left, AssignmentTarget::ObjectAssignmentTarget(_))
                }
                AstKind::SequenceExpression(sequence_expr) => {
                    !matches!(parent_parent_kind, Some(AstKind::ForStatement(for_stat))
                        if for_stat
                            .init
                            .as_ref()
                            .is_some_and(|e| e.span() == sequence_expr.span)
                            || for_stat
                                .update
                                .as_ref()
                                .is_some_and(|e| e.span() == sequence_expr.span))
                }
                _ => true,
            },
            AstKind::UpdateExpression(update_expr) => match parent_kind {
                AstKind::UnaryExpression(unary_expr) => {
                    update_expr.prefix
                        && ((update_expr.operator == UpdateOperator::Increment
                            && unary_expr.operator == UnaryOperator::UnaryPlus)
                            || (update_expr.operator == UpdateOperator::Decrement
                                && unary_expr.operator == UnaryOperator::UnaryNegation))
                }
                _ => self.check_update_unary(update_expr.span),
            },
            AstKind::UnaryExpression(unary_expr) => match parent_kind {
                AstKind::UnaryExpression(parent_expr) => {
                    let u_op = unary_expr.operator;
                    u_op == parent_expr.operator
                        && (matches!(u_op, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation))
                }
                _ => self.check_update_unary(unary_expr.span),
            },
            AstKind::YieldExpression(e) => match parent_kind {
                AstKind::AwaitExpression(_) => true,
                _ => self.check_yield_await(e.span),
            },
            AstKind::AwaitExpression(e) => self.check_yield_await(e.span),
            AstKind::TSTypeAssertion(e) => self.check_binarish(e.span),
            AstKind::TSAsExpression(e) => self.check_binarish(e.span),
            AstKind::TSSatisfiesExpression(e) => self.check_binarish(e.span),
            AstKind::LogicalExpression(e) => self.check_binarish(e.span),
            AstKind::BinaryExpression(e) => match parent_kind {
                AstKind::UpdateExpression(_) => true,
                _ if e.operator.is_in() && self.is_path_in_for_statement_initializer(e.span) => {
                    true
                }
                _ => self.check_binarish(e.span),
            },
            AstKind::MemberExpression(e) => self.check_member_call(e.span()),
            AstKind::CallExpression(e) => self.check_member_call(e.span),
            AstKind::TaggedTemplateExpression(e) => {
                self.check_member_call_tagged_template_ts_non_null(e.span)
            }
            AstKind::TSNonNullExpression(e) => {
                self.check_member_call_tagged_template_ts_non_null(e.span)
            }
            AstKind::ConditionalExpression(e) => match parent_kind {
                AstKind::TaggedTemplateExpression(_)
                | AstKind::UnaryExpression(_)
                | AstKind::SpreadElement(_)
                | AstKind::BinaryExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::AwaitExpression(_)
                | AstKind::JSXSpreadAttribute(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_) => true,
                AstKind::CallExpression(call_expr) => call_expr.callee.span() == e.span,
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == e.span,
                AstKind::ConditionalExpression(cond_expr) => cond_expr.test.span() == e.span,
                AstKind::MemberExpression(member_expr) => member_expr.object().span() == e.span,
                _ => false,
            },
            AstKind::Function(e) if e.is_expression() => match parent_kind {
                AstKind::CallExpression(call_expr) => call_expr.callee.span() == e.span,
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == e.span,
                AstKind::TaggedTemplateExpression(_) => true,
                _ => false,
            },
            AstKind::ArrowFunctionExpression(e) => match parent_kind {
                AstKind::CallExpression(call_expr) => call_expr.callee.span() == e.span,
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == e.span,
                AstKind::MemberExpression(member_expr) => member_expr.object().span() == e.span,
                AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::UnaryExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::AwaitExpression(_)
                | AstKind::TSTypeAssertion(_) => true,
                AstKind::ConditionalExpression(cond_expr) => cond_expr.test.span() == e.span,
                _ => false,
            },
            AstKind::Class(class) if class.is_expression() => match parent_kind {
                AstKind::NewExpression(new_expr) => new_expr.callee.span() == class.span,
                _ => false,
            },
            _ => false,
        }
    }

    fn check_parent_kind(&mut self, kind: AstKind<'a>) -> bool {
        match self.parent_kind() {
            AstKind::Class(class) => {
                if let Some(h) = &class.super_class {
                    match kind {
                        AstKind::ArrowFunctionExpression(e) if e.span == h.span() => return true,
                        AstKind::AssignmentExpression(e) if e.span == h.span() => return true,
                        AstKind::AwaitExpression(e) if e.span == h.span() => return true,
                        AstKind::BinaryExpression(e) if e.span == h.span() => return true,
                        AstKind::ConditionalExpression(e) if e.span == h.span() => return true,
                        AstKind::LogicalExpression(e) if e.span == h.span() => return true,
                        AstKind::NewExpression(e) if e.span == h.span() => return true,
                        AstKind::ObjectExpression(e) if e.span == h.span() => return true,
                        AstKind::SequenceExpression(e) if e.span == h.span() => return true,
                        AstKind::TaggedTemplateExpression(e) if e.span == h.span() => return true,
                        AstKind::UnaryExpression(e) if e.span == h.span() => return true,
                        AstKind::UpdateExpression(e) if e.span == h.span() => return true,
                        AstKind::YieldExpression(e) if e.span == h.span() => return true,
                        AstKind::TSNonNullExpression(e) if e.span == h.span() => return true,
                        AstKind::Class(e)
                            if e.is_expression()
                                && !e.decorators.is_empty()
                                && e.span == h.span() =>
                        {
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            AstKind::ExportDefaultDeclaration(decl) => {
                return matches!(
                    decl.declaration,
                    ExportDefaultDeclarationKind::SequenceExpression(_)
                ) || (decl.declaration.is_expression()
                    && self.should_wrap_function_for_export_default());
            }
            AstKind::BinaryExpression(binary_expr) => {
                if binary_expr.operator.is_relational() {
                    if let AstKind::UnaryExpression(unary_expr) = kind {
                        if binary_expr.left.span() == unary_expr.span {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn check_sequence_expression(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::ReturnStatement(_) | AstKind::ForStatement(_) => false,
            AstKind::ExpressionStatement(expr) => expr.expression.span() != span,
            AstKind::ArrowFunctionExpression(expr) => expr.body.span != span,
            _ => true,
        }
    }

    fn check_object_expression(&self, obj_expr: &ObjectExpression<'a>) -> bool {
        let mut arrow_expr = None;
        for kind in self.stack.iter().rev() {
            if let AstKind::ArrowFunctionExpression(e) = kind {
                e.get_expression();
                arrow_expr = Some(e);
                break;
            }
        }
        if let Some(arrow_expr) = arrow_expr {
            if let Some(e) = arrow_expr.get_expression() {
                if !matches!(
                    e,
                    Expression::SequenceExpression(_) | Expression::AssignmentExpression(_)
                ) && Self::starts_with_no_lookahead_token(e, obj_expr.span)
                {
                    return true;
                }
            }
        }
        false
    }

    /// `for ((async) of []);` and `for ((let) of []);`
    fn check_for_of_stmt_head_starts_with_async_or_let(&self, kind: AstKind<'a>) -> bool {
        let AstKind::IdentifierReference(ident) = kind else { return false };
        let AstKind::ForOfStatement(stmt) = self.parent_kind() else { return false };
        if let ForStatementLeft::AssignmentTargetIdentifier(i) = &stmt.left {
            if (i.span == ident.span) && (i.name == "let" || (i.name == "async" && !stmt.r#await)) {
                return true;
            }
        }

        false
    }

    /// `for ((let.a) of []);`
    fn check_for_of_stmt_head_starts_with_let(&self, kind: AstKind<'a>) -> bool {
        let AstKind::IdentifierReference(ident) = kind else { return false };
        if ident.name != "let" {
            return false;
        }
        for kind in self.stack.iter().rev() {
            if let AstKind::ForOfStatement(stmt) = kind {
                if let Some(target) = stmt.left.as_assignment_target() {
                    if let Some(e) = target.as_member_expression() {
                        return Self::starts_with_no_lookahead_token(e.object(), ident.span);
                    }
                }
                break;
            }
        }
        false
    }

    /// `(let)[a] = 1`
    fn check_let_object(&self, kind: AstKind<'a>) -> bool {
        let AstKind::IdentifierReference(ident) = kind else { return false };
        if ident.name != "let" {
            return false;
        }
        let AstKind::MemberExpression(MemberExpression::ComputedMemberExpression(expr)) =
            self.parent_kind()
        else {
            return false;
        };
        if !matches!(&expr.object, Expression::Identifier(ident) if ident.name == "let") {
            return false;
        }
        let Some(statement) = self.stack.iter().rev().find(|node| {
            matches!(
                node,
                AstKind::ExpressionStatement(_)
                    | AstKind::ForStatement(_)
                    | AstKind::ForInStatement(_)
            )
        }) else {
            return false;
        };
        match statement {
            AstKind::ExpressionStatement(stmt) => {
                Self::starts_with_no_lookahead_token(&stmt.expression, ident.span)
            }
            AstKind::ForStatement(stmt) => stmt
                .init
                .as_ref()
                .and_then(ForStatementInit::as_expression)
                .is_some_and(|e| Self::starts_with_no_lookahead_token(e, ident.span)),
            AstKind::ForInStatement(stmt) => {
                if let Some(target) = stmt.left.as_assignment_target() {
                    if let Some(e) = target.as_member_expression() {
                        return Self::starts_with_no_lookahead_token(e.object(), ident.span);
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn check_object_function_class(&self, span: Span) -> bool {
        for ast_kind in self.stack.iter().rev() {
            if let AstKind::ExpressionStatement(e) = ast_kind {
                if Self::starts_with_no_lookahead_token(&e.expression, span) {
                    return true;
                }
            }
        }
        false
    }

    fn check_update_unary(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::MemberExpression(member_expr) => member_expr.object().span() == span,
            AstKind::CallExpression(call_expr) => call_expr.callee.span() == span,
            AstKind::NewExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::BinaryExpression(bin_expr) => {
                bin_expr.left.span() == span && bin_expr.operator == BinaryOperator::Exponential
            }
            AstKind::TaggedTemplateExpression(_) | AstKind::TSNonNullExpression(_) => true,
            _ => false,
        }
    }

    fn check_yield_await(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::TaggedTemplateExpression(_)
            | AstKind::UnaryExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::SpreadElement(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::BinaryExpression(_) => true,
            AstKind::MemberExpression(member_expr) => member_expr.object().span() == span,
            AstKind::NewExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::CallExpression(new_expr) => new_expr.callee.span() == span,
            AstKind::ConditionalExpression(con_expr) => con_expr.test.span() == span,
            _ => false,
        }
    }

    fn check_binarish(&self, span: Span) -> bool {
        let current_kind = self.current_kind();
        let parent_kind = self.parent_kind();
        match parent_kind {
            AstKind::TSAsExpression(_) | AstKind::TSSatisfiesExpression(_) => {
                return !self.is_binary_cast_expression(span);
            }
            AstKind::ConditionalExpression(_) => return self.is_binary_cast_expression(span),
            AstKind::NewExpression(new_expr) => return new_expr.callee.span() == span,
            AstKind::CallExpression(new_expr) => return new_expr.callee.span() == span,
            AstKind::Class(class) => {
                return class.super_class.as_ref().is_some_and(|e| e.span() == span);
            }
            AstKind::TSTypeAssertion(_)
            | AstKind::TaggedTemplateExpression(_)
            | AstKind::UnaryExpression(_)
            | AstKind::JSXSpreadAttribute(_)
            | AstKind::SpreadElement(_)
            | AstKind::AwaitExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::UpdateExpression(_) => return true,
            AstKind::MemberExpression(member_expr) => return member_expr.object().span() == span,
            AstKind::AssignmentExpression(assign_expr) => {
                return assign_expr.left.span() == span && self.is_binary_cast_expression(span);
            }
            AstKind::AssignmentPattern(assign_pat) => {
                return assign_pat.left.span() == span && self.is_binary_cast_expression(span);
            }
            AstKind::LogicalExpression(parent_logical_expr) => {
                if let AstKind::LogicalExpression(logical_expr) = current_kind {
                    return parent_logical_expr.operator != logical_expr.operator;
                }
            }
            _ => {}
        }

        let operator = match self.current_kind() {
            AstKind::LogicalExpression(e) => BinaryishOperator::from(e.operator),
            AstKind::BinaryExpression(e) => BinaryishOperator::from(e.operator),
            _ => return false,
        };

        let parent_operator = match parent_kind {
            AstKind::LogicalExpression(e) => BinaryishOperator::from(e.operator),
            AstKind::BinaryExpression(e) => BinaryishOperator::from(e.operator),
            _ => return false,
        };

        let precedence = operator.precedence();
        let parent_precedence = parent_operator.precedence();

        if parent_precedence > precedence {
            return true;
        }

        if parent_precedence == precedence && !parent_operator.should_flatten(operator) {
            return true;
        }

        if parent_precedence < precedence
            && matches!(operator, BinaryishOperator::BinaryOperator(op) if op == BinaryOperator::Remainder)
        {
            return matches!(parent_operator, BinaryishOperator::BinaryOperator(op) if matches!(op, BinaryOperator::Addition | BinaryOperator::Subtraction));
        }

        // Add parenthesis when working with bitwise operators
        // It's not strictly needed but helps with code understanding
        if matches!(parent_kind, AstKind::BinaryExpression(binary_expr) if binary_expr.operator.is_bitwise())
        {
            return true;
        }

        false
    }

    fn check_member_call(&self, span: Span) -> bool {
        // if (shouldAddParenthesesToChainElement(path)) {
        // return true;
        // }
        self.check_member_call_tagged_template_ts_non_null(span)
    }

    fn check_member_call_tagged_template_ts_non_null(&self, span: Span) -> bool {
        match self.parent_kind() {
            AstKind::NewExpression(new_expr) if new_expr.callee.span() == span => {
                let mut object = &new_expr.callee;
                loop {
                    object = match object {
                        Expression::CallExpression(_) => return true,
                        Expression::ComputedMemberExpression(e) => &e.object,
                        Expression::StaticMemberExpression(e) => &e.object,
                        Expression::PrivateFieldExpression(e) => &e.object,
                        Expression::TaggedTemplateExpression(e) => &e.tag,
                        Expression::TSNonNullExpression(e) => &e.expression,
                        _ => return false,
                    }
                }
            }
            _ => false,
        }
    }

    fn should_wrap_function_for_export_default(&mut self) -> bool {
        let kind = self.current_kind();
        let b = matches!(self.parent_kind(), AstKind::ExportDefaultDeclaration(_));
        if matches!(kind, AstKind::Function(f) if f.is_expression())
            || matches!(kind, AstKind::Class(c) if c.is_expression())
        {
            return b || !self.need_parens(self.current_kind());
        }

        if !Self::has_naked_left_side(kind) || (!b && self.need_parens(self.current_kind())) {
            return false;
        }

        let lhs = Self::get_left_side_path_name(kind);
        self.stack.push(lhs);
        let result = self.should_wrap_function_for_export_default();
        self.stack.pop();
        result
    }

    fn has_naked_left_side(kind: AstKind<'a>) -> bool {
        matches!(
            kind,
            AstKind::AssignmentExpression(_)
                | AstKind::BinaryExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::ConditionalExpression(_)
                | AstKind::CallExpression(_)
                | AstKind::MemberExpression(_)
                | AstKind::SequenceExpression(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::ChainExpression(_)
        ) || matches!(kind, AstKind::UpdateExpression(e) if !e.prefix)
    }

    fn get_left_side_path_name(kind: AstKind<'a>) -> AstKind<'a> {
        match kind {
            AstKind::CallExpression(e) => AstKind::from_expression(&e.callee),
            AstKind::ConditionalExpression(e) => AstKind::from_expression(&e.test),
            AstKind::TaggedTemplateExpression(e) => AstKind::from_expression(&e.tag),
            AstKind::AssignmentExpression(e) => AstKind::AssignmentTarget(&e.left),
            AstKind::MemberExpression(e) => AstKind::from_expression(e.object()),
            AstKind::BinaryExpression(e) => AstKind::from_expression(&e.left),
            AstKind::LogicalExpression(e) => AstKind::from_expression(&e.left),
            _ => panic!("need to handle {}", kind.debug_name()),
        }
    }

    fn is_binary_cast_expression(&self, _span: Span) -> bool {
        false
    }

    fn is_path_in_for_statement_initializer(&self, span: Span) -> bool {
        let mut node = Some(span);
        let mut parents = self.stack.iter().rev();
        while let Some(n) = node {
            let parent = parents.next();
            if let Some(AstKind::ForStatement(stmt)) = parent {
                if stmt.init.as_ref().is_some_and(|init| init.span() == n) {
                    return true;
                }
            }
            node = parent.map(GetSpan::span);
        }
        false
    }

    fn starts_with_no_lookahead_token(e: &Expression<'a>, span: Span) -> bool {
        match e {
            Expression::BinaryExpression(e) => Self::starts_with_no_lookahead_token(&e.left, span),
            Expression::LogicalExpression(e) => Self::starts_with_no_lookahead_token(&e.left, span),
            Expression::AssignmentExpression(e) => match &e.left {
                AssignmentTarget::AssignmentTargetIdentifier(_)
                | AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_) => false,
                AssignmentTarget::ComputedMemberExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
                AssignmentTarget::StaticMemberExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
                AssignmentTarget::PrivateFieldExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
                AssignmentTarget::TSAsExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
                AssignmentTarget::TSSatisfiesExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
                AssignmentTarget::TSNonNullExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
                AssignmentTarget::TSTypeAssertion(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
                AssignmentTarget::TSInstantiationExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
            },
            match_member_expression!(Expression) => {
                Self::starts_with_no_lookahead_token(e.to_member_expression().object(), span)
            }
            Expression::TaggedTemplateExpression(e) => {
                if matches!(e.tag, Expression::FunctionExpression(_)) {
                    return false;
                }
                Self::starts_with_no_lookahead_token(&e.tag, span)
            }
            Expression::CallExpression(e) => {
                if matches!(e.callee, Expression::FunctionExpression(_)) {
                    return false;
                }
                Self::starts_with_no_lookahead_token(&e.callee, span)
            }
            Expression::ConditionalExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.test, span)
            }
            Expression::UpdateExpression(e) => {
                !e.prefix
                    && match &e.argument {
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
                        SimpleAssignmentTarget::ComputedMemberExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.object, span)
                        }
                        SimpleAssignmentTarget::StaticMemberExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.object, span)
                        }
                        SimpleAssignmentTarget::PrivateFieldExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.object, span)
                        }
                        SimpleAssignmentTarget::TSAsExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSSatisfiesExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSNonNullExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSTypeAssertion(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                        SimpleAssignmentTarget::TSInstantiationExpression(e) => {
                            Self::starts_with_no_lookahead_token(&e.expression, span)
                        }
                    }
            }
            Expression::SequenceExpression(e) => {
                e.expressions.first().is_some_and(|e| Self::starts_with_no_lookahead_token(e, span))
            }
            Expression::ChainExpression(e) => match &e.expression {
                ChainElement::CallExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.callee, span)
                }
                ChainElement::TSNonNullExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.expression, span)
                }
                ChainElement::ComputedMemberExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
                ChainElement::StaticMemberExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
                ChainElement::PrivateFieldExpression(e) => {
                    Self::starts_with_no_lookahead_token(&e.object, span)
                }
            },
            Expression::TSSatisfiesExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            Expression::TSAsExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            Expression::TSNonNullExpression(e) => {
                Self::starts_with_no_lookahead_token(&e.expression, span)
            }
            _ => e.span() == span,
        }
    }
}
