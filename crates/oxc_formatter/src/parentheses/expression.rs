use std::ptr;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{JsFormatter, JsFormatterExt as _},
    print::{BinaryLikeExpression, should_flatten},
    utils::expression::ExpressionLeftSide,
};

use super::NeedsParentheses;

impl NeedsParentheses<'_> for AstNode<'_, Expression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        match self.as_ast_nodes() {
            AstNodes::BooleanLiteral(it) => it.needs_parentheses(f),
            AstNodes::NullLiteral(it) => it.needs_parentheses(f),
            AstNodes::NumericLiteral(it) => it.needs_parentheses(f),
            AstNodes::BigIntLiteral(it) => it.needs_parentheses(f),
            AstNodes::RegExpLiteral(it) => it.needs_parentheses(f),
            AstNodes::StringLiteral(it) => it.needs_parentheses(f),
            AstNodes::TemplateLiteral(it) => it.needs_parentheses(f),
            AstNodes::IdentifierReference(it) => it.needs_parentheses(f),
            AstNodes::ImportMeta(it) => it.needs_parentheses(f),
            AstNodes::NewTarget(it) => it.needs_parentheses(f),
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

impl NeedsParentheses<'_> for AstNode<'_, IdentifierReference<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        // `(type) satisfies never;` and similar keyword-at-statement-start cast cases
        // <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/parentheses/identifier.js#L84-L107>
        let needs_parens_in_cast_chain = || {
            let mut parent = self.parent();
            while matches!(parent, AstNodes::TSSatisfiesExpression(_) | AstNodes::TSAsExpression(_))
            {
                parent = parent.parent();
            }

            // Needs at least one `satisfies`/`as` wrapper, with a statement right outside it
            !ptr::eq(self.parent(), parent)
                && matches!(
                    parent, AstNodes::ExpressionStatement(stmt) if !stmt.is_arrow_function_body()
                )
        };

        match self.name.as_str() {
            "async" => {
                matches!(self.parent(), AstNodes::ForOfStatement(stmt) if !stmt.r#await && stmt.left.span().contains_inclusive(self.span))
            }
            "let" => {
                // `(let) satisfies X;`, `(let) as X;` at statement start
                if needs_parens_in_cast_chain() {
                    return true;
                }

                // `let` needs parens when it is the leftmost token of:
                // - a computed member expression at statement start or in a `for(;;)` init,
                //   which would parse as a lexical declaration, e.g. `(let)[a] = 1`
                // - a `for-in`/`for-of` head, which cannot start with `let`,
                //   e.g. `for ((let).a in x)`, `for ((let) of x)`
                let is_computed_member_object = matches!(
                    self.parent(),
                    AstNodes::ComputedMemberExpression(m)
                        if m.object.span() == self.span() && !m.optional
                );

                // Check if `let` is at the leftmost position of the relevant construct
                let mut child_span = self.span;
                for parent in self.ancestors() {
                    let dominated = match parent {
                        AstNodes::ExpressionStatement(s) => {
                            return is_computed_member_object && !s.is_arrow_function_body();
                        }
                        AstNodes::ForStatement(s) => {
                            return is_computed_member_object
                                && matches!(&s.init, Some(init) if init.span() == child_span);
                        }
                        AstNodes::ForOfStatement(s) => {
                            return s.left.span().contains_inclusive(self.span);
                        }
                        AstNodes::ForInStatement(s) => {
                            return s.left.span().contains_inclusive(self.span);
                        }
                        AstNodes::ComputedMemberExpression(m) => m.object.span() == child_span,
                        AstNodes::StaticMemberExpression(m) => m.object.span() == child_span,
                        AstNodes::CallExpression(c) => c.callee.span() == child_span,
                        AstNodes::ChainExpression(c) => c.expression.span() == child_span,
                        AstNodes::AssignmentExpression(a) => a.left.span() == child_span,
                        AstNodes::BinaryExpression(b) => b.left.span() == child_span,
                        AstNodes::LogicalExpression(l) => l.left.span() == child_span,
                        AstNodes::ConditionalExpression(c) => c.test.span() == child_span,
                        AstNodes::SequenceExpression(s) => {
                            s.expressions.first().is_some_and(|e| e.span() == child_span)
                        }
                        AstNodes::TaggedTemplateExpression(t) => t.tag.span() == child_span,
                        AstNodes::UpdateExpression(u) => {
                            !u.prefix && u.argument.span() == child_span
                        }
                        AstNodes::TSAsExpression(e) => e.expression.span() == child_span,
                        AstNodes::TSSatisfiesExpression(e) => e.expression.span() == child_span,
                        AstNodes::TSNonNullExpression(e) => e.expression.span() == child_span,
                        _ => false,
                    };
                    if !dominated {
                        return false;
                    }
                    child_span = parent.span();
                }
                false
            }
            name => {
                matches!(
                    name,
                    "await"
                        | "interface"
                        | "module"
                        | "using"
                        | "yield"
                        | "component"
                        | "hook"
                        | "type"
                ) && needs_parens_in_cast_chain()
            }
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, BooleanLiteral> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, NullLiteral> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, BigIntLiteral<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, RegExpLiteral<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TemplateLiteral<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ImportMeta> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, NewTarget> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, Super> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, NumericLiteral<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        if let AstNodes::StaticMemberExpression(member) = self.parent() {
            return member.object.span() == self.span();
        }
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, StringLiteral<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        // To avoid becoming a directive, wrap in parens only when the expression
        // statement is directly inside a Program, FunctionBody, or BlockStatement.
        //
        // `label: "foo"` -> no parens needed (grandparent is LabeledStatement)
        // `"foo";` -> parens needed (grandparent is Program)
        // `() => "foo"` -> no parens needed (arrow function body)
        //
        // https://github.com/prettier/prettier/blob/00146ea15c30e16ad6526893c735e35683192efc/src/language-js/parentheses/needs-parentheses.js#L594-L609
        if let AstNodes::ExpressionStatement(stmt) = self.parent() {
            // `() => "foo"`
            !stmt.is_arrow_function_body()
                && matches!(
                    stmt.parent(),
                    AstNodes::Program(_) | AstNodes::FunctionBody(_) | AstNodes::BlockStatement(_)
                )
        } else {
            false
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ThisExpression> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ArrayExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        // Wrap array expressions in for-in initializers
        // e.g., `for (var a = ([b in c]) in {})`
        is_for_in_statement_init(self, self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ObjectExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap object expressions in for-in initializers
        // e.g., `for (var a = ({ b: b in c }) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

        is_class_extends(self.span, parent)
            || is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionStatementOrArrow,
            )
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TaggedTemplateExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        // `class A extends (tag`x`) {}`; keep in sync with the wrapped-in-`!` case
        // in `class_extends_needs_parens_through_non_null`.
        is_class_extends(self.span, self.parent())
            // `new (import("foo")`bar`)()`; a tag containing a call/import
            // cannot appear in a new callee without parens
            || (self.is_new_callee() && member_chain_callee_needs_parens(&self.tag))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, MemberExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ComputedMemberExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        self.is_new_callee() && (self.optional || member_chain_callee_needs_parens(&self.object))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, StaticMemberExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        self.is_new_callee() && member_chain_callee_needs_parens(&self.object)
    }
}

impl NeedsParentheses<'_> for AstNode<'_, PrivateFieldExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        self.is_new_callee() && (self.optional || member_chain_callee_needs_parens(&self.object))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, CallExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        match self.parent() {
            AstNodes::ExportDefaultDeclaration(_) => {
                let callee = &self.callee();
                let callee_span = callee.span();
                let leftmost = ExpressionLeftSide::leftmost(callee);
                // require parens for iife and
                // when the leftmost expression is not a class expression or a function expression
                callee_span != leftmost.span()
                    && matches!(
                        leftmost.as_ref(),
                        Expression::ClassExpression(_) | Expression::FunctionExpression(_)
                    )
            }
            _ => self.is_new_callee(),
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, NewExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        is_class_extends(self.span, self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, UpdateExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();
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

impl NeedsParentheses<'_> for AstNode<'_, UnaryExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();
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

impl NeedsParentheses<'_> for AstNode<'_, BinaryExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap binary expressions in for-in initializers
        // e.g., `for (var a = (1 in b) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

        // For `in` expressions in ForStatement: wrap to avoid ambiguity
        // e.g., `for (var a = (b in c);;)`
        if self.operator.is_in() && is_in_for_initializer(self) {
            return true;
        }

        binary_like_needs_parens(BinaryLikeExpression::BinaryExpression(self))
    }
}

/// Add parentheses if the `in` binary expression is inside of a `ForStatement` initializer.
///
/// Only checks ForStatement, NOT ForInStatement. ForInStatement init wrapping is handled
/// separately at the VariableDeclarator level (see `is_for_in_statement_init`).
///
/// <https://github.com/prettier/prettier/issues/907#issuecomment-284304321>
fn is_in_for_initializer(expr: &AstNode<'_, BinaryExpression<'_>>) -> bool {
    let mut ancestors = expr.ancestors();

    while let Some(parent) = ancestors.next() {
        match parent {
            AstNodes::ExpressionStatement(stmt) => {
                if stmt.is_arrow_function_body() {
                    // Expression body: `() => expr`
                    // Skip `FunctionBody` and `ArrowFunctionExpression`
                    let skipped = ancestors.by_ref().nth(1);
                    debug_assert!(matches!(skipped, Some(AstNodes::ArrowFunctionExpression(_))));
                    continue;
                }
                // Block body: `() => { expr; }` or `function() { expr; }` - continue checking
                // because for regular ForStatement, parens are still needed
                if matches!(stmt.parent(), AstNodes::FunctionBody(_)) {
                    continue;
                }

                return false;
            }
            AstNodes::ForStatement(stmt) => {
                return stmt
                    .init
                    .as_ref()
                    .is_some_and(|init| init.span().contains_inclusive(expr.span));
            }
            // ForInStatement is handled at VariableDeclarator level, not here
            AstNodes::ForInStatement(_) | AstNodes::Program(_) => return false,
            // Skip through function bodies - could be inside arrow/function in for init
            _ => {}
        }
    }

    false
}

/// Check if an expression is the init of a VariableDeclarator in a ForInStatement's LEFT.
///
/// Following Prettier's approach: wrap ANY expression that is the init of a
/// VariableDeclarator when that VariableDeclaration is the left side of a ForInStatement.
///
/// Legacy syntax: `for (var a = 1 in b);`
/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Invalid_for-in_initializer>
fn is_for_in_statement_init<T: GetSpan>(node: &T, parent: &AstNodes<'_>) -> bool {
    let AstNodes::VariableDeclarator(declarator) = parent else { return false };
    let Some(init) = &declarator.init else { return false };
    if init.span() != node.span() {
        return false;
    }
    let AstNodes::VariableDeclaration(decl) = declarator.parent() else { return false };
    // Check that this VariableDeclaration is the LEFT of a ForInStatement,
    // not just anywhere inside it (e.g., not in the body)
    matches!(decl.parent(), AstNodes::ForInStatement(stmt)
        if matches!(&stmt.left, ForStatementLeft::VariableDeclaration(d) if ptr::eq(d.as_ref(), decl.as_ref())))
}

impl NeedsParentheses<'_> for AstNode<'_, PrivateInExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        is_class_extends(self.span, self.parent())
            || matches!(self.parent(), AstNodes::UnaryExpression(_))
    }
}

impl NeedsParentheses<'_> for AstNode<'_, LogicalExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap logical expressions in for-in initializers
        // e.g., `for (var a = (1 || b in c) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

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

impl NeedsParentheses<'_> for AstNode<'_, ConditionalExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();
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
            e.test.span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, Function<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if self.r#type() != FunctionType::FunctionExpression {
            return false;
        }

        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap function expressions in for-in initializers
        // e.g., `for (var a = (function (x = b in c) {}) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

        matches!(parent, AstNodes::TaggedTemplateExpression(_))
            || self.is_call_like_callee()
            || is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionOrExportDefault,
            )
    }
}

impl NeedsParentheses<'_> for AstNode<'_, AssignmentExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        match self.parent() {
            // Expression statements, only object destructuring needs parens:
            // - `a = b` = no parens
            // - `{ x } = obj` -> `({ x } = obj)` = needed to prevent parsing as block statement
            // - `() => { x } = obj` -> `() => ({ x } = obj)` = needed in arrow function body
            // - `() => a = b` -> `() => (a = b)` = also parens needed
            AstNodes::ExpressionStatement(stmt) => {
                if stmt.is_arrow_function_body() {
                    return true;
                }

                matches!(self.left, AssignmentTarget::ObjectAssignmentTarget(_))
                    && is_first_in_statement(
                        self.span,
                        self.parent(),
                        FirstInStatementMode::ExpressionStatementOrArrow,
                    )
            }
            // Sequence expressions, need to traverse up to find if we're in a for statement context:
            // - `a = 1, b = 2` in for loops don't need parens
            // - `(a = 1, b = 2)` elsewhere usually need parens
            AstNodes::SequenceExpression(_) => {
                // Skip through SequenceExpression and ParenthesizedExpression ancestors
                if let Some(ancestor) = self.ancestors().find(|p| {
                    !matches!(
                        p,
                        AstNodes::SequenceExpression(_) | AstNodes::ParenthesizedExpression(_)
                    )
                }) && let AstNodes::ForStatement(for_stmt) = ancestor
                {
                    let is_initializer = for_stmt
                        .init
                        .as_ref()
                        .is_some_and(|init| init.span().contains_inclusive(self.span()));
                    let is_update = for_stmt
                        .update
                        .as_ref()
                        .is_some_and(|update| update.span().contains_inclusive(self.span()));
                    return !(is_initializer || is_update);
                }

                true
            }
            // `interface A { [a = 1]; }` not need parens
            AstNodes::TSPropertySignature(_) |
            // Never need parentheses in these contexts:
            // - `a = (b = c)` = nested assignments don't need extra parens
            AstNodes::AssignmentExpression(_) => false,
            // Computed member expressions: need parens when assignment is the object
            // - `obj[(a = b)]` parens needed for explicitness
            // - `(a = b)[obj]` = parens needed for object
            #[expect(clippy::match_same_arms)]
            AstNodes::ComputedMemberExpression(_) => true,
            // For statements, no parens needed in initializer or update sections:
            // - `for (a = 1; ...; a = 2) {}` = both assignments don't need parens
            AstNodes::ForStatement(stmt) => {
                let is_initializer =
                    stmt.init.as_ref().is_some_and(|init| init.span() == self.span());
                let is_update =
                    stmt.update.as_ref().is_some_and(|update| update.span() == self.span());
                !(is_initializer || is_update)
            }
            // Default: need parentheses in most other contexts
            // - `new (a = b)`
            // - `(a = b).prop`
            // - `await (a = b)`
            // - `class { [a = 1]; }`
            // - etc.
            _ => true,
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, SequenceExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        match self.parent() {
            AstNodes::ReturnStatement(_)
            | AstNodes::ThrowStatement(_)
            // There's a precedence for writing `x++, y++`
            | AstNodes::ForStatement(_) => false,
            AstNodes::ExpressionStatement(stmt) => !stmt.is_arrow_function_body(),
            _ => true,
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, AwaitExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        await_or_yield_needs_parens(self.span(), self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ChainExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }
        // Check if chain expression needs parens based on how it's being accessed
        chain_expression_needs_parens(self.span, self.parent())
    }
}

/// Check if a ChainExpression needs parentheses based on its parent context.
///
/// Parentheses are needed when the chain is:
/// - The expression of a non-null assertion (`(a?.b)!`, in any position);
///   this preserves the distinction from `a?.b!` (`ChainElement::TSNonNullExpression`)
/// - The callee of a non-optional call expression
/// - The callee of a new expression
/// - The object of a non-optional member expression
/// - The tag of a tagged template expression
fn chain_expression_needs_parens(span: Span, parent: &AstNodes<'_>) -> bool {
    match parent {
        AstNodes::TSNonNullExpression(_) | AstNodes::TaggedTemplateExpression(_) => true,
        AstNodes::NewExpression(new) => new.is_callee_span(span),
        AstNodes::CallExpression(call) => call.is_callee_span(span) && !call.optional,
        AstNodes::StaticMemberExpression(member) => !member.optional,
        AstNodes::ComputedMemberExpression(member) => {
            !member.optional && member.object.span() == span
        }
        _ => false,
    }
}

impl NeedsParentheses<'_> for AstNode<'_, Class<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if self.r#type() != ClassType::ClassExpression {
            return false;
        }

        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap class expressions in for-in initializers
        // e.g., `for (var a = (class extends (b in c) {}) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

        matches!(parent, AstNodes::TaggedTemplateExpression(_))
            || self.is_call_like_callee()
            || (is_class_extends(self.span, self.parent()) && !self.decorators.is_empty())
            || is_first_in_statement(
                self.span,
                parent,
                FirstInStatementMode::ExpressionOrExportDefault,
            )
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ParenthesizedExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        unreachable!("Already disabled `preserveParens` option in the parser")
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ArrowFunctionExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();

        // Wrap arrow functions in for-in initializers
        // e.g., `for (var a = (() => b in c) in {})`
        if is_for_in_statement_init(self, parent) {
            return true;
        }

        if matches!(
            parent,
            AstNodes::TSAsExpression(_)
                | AstNodes::TSSatisfiesExpression(_)
                | AstNodes::TSTypeAssertion(_)
                | AstNodes::TSInstantiationExpression(_)
                | AstNodes::UnaryExpression(_)
                | AstNodes::AwaitExpression(_)
                | AstNodes::LogicalExpression(_)
                | AstNodes::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstNodes::ConditionalExpression(e) = parent {
            e.test.span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl NeedsParentheses<'_> for AstNode<'_, YieldExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        let parent = self.parent();
        matches!(parent, AstNodes::AwaitExpression(_) | AstNodes::TSTypeAssertion(_))
            || await_or_yield_needs_parens(self.span(), parent)
    }
}

impl NeedsParentheses<'_> for AstNode<'_, ImportExpression<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        self.is_new_callee()
    }
}

impl NeedsParentheses<'_> for AstNode<'_, V8IntrinsicExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXMemberExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXExpression<'_>> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXEmptyExpression> {
    #[inline]
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        false
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSAsExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        ts_as_or_satisfies_needs_parens(self.span(), self.expression(), self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSSatisfiesExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        ts_as_or_satisfies_needs_parens(self.span(), self.expression(), self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSTypeAssertion<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        match self.parent() {
            AstNodes::TSAsExpression(_) | AstNodes::TSSatisfiesExpression(_) => true,
            AstNodes::BinaryExpression(binary) => {
                // The base of `**` must be an `UpdateExpression`; a type assertion `<T>x` is a
                // unary-like expression, so as the left operand it must be parenthesized
                // (`<T>x ** 2` is a syntax error, same restriction as `-2 ** 2`).
                matches!(binary.operator, BinaryOperator::ShiftLeft)
                    || (binary.operator == BinaryOperator::Exponential
                        && binary.left.span() == self.span())
            }
            _ => type_cast_like_needs_parens(self.span(), self.parent()),
        }
    }
}

fn type_cast_like_needs_parens(span: Span, parent: &AstNodes<'_>) -> bool {
    #[expect(clippy::match_same_arms)] // for better readability
    match parent {
        AstNodes::TSTypeAssertion(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::TSNonNullExpression(_)
        // template tag
        | AstNodes::TaggedTemplateExpression(_)
        // in spread
        | AstNodes::JSXSpreadChild(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::JSXSpreadAttribute(_)
        // static member
        | AstNodes::StaticMemberExpression(_)
        // private field member
        | AstNodes::PrivateFieldExpression(_) => true,
        AstNodes::ComputedMemberExpression(member) => {
            member.object.span() == span
        }
        // assignment left hand side
        AstNodes::UpdateExpression(_) | AstNodes::AssignmentTargetWithDefault(_) => true,
        AstNodes::AssignmentExpression(assignment) => {
            assignment.left.span() == span
        }
        _ => parent.is_call_like_callee_span(span) || is_class_extends(span, parent),
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSNonNullExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        (self.is_new_callee() && member_chain_callee_needs_parens(&self.expression))
            // `class A extends ({}!) {}` needs parens, `class A extends B! {}` does not:
            // like Prettier, judge by the expression under the `!`, not the `!` itself.
            || (is_class_extends(self.span, self.parent())
                && class_extends_needs_parens_through_non_null(&self.expression))
    }
}

/// The `superClass` expressions Prettier wraps in parentheses (`parent-needs-parentheses.js`),
/// applied to the expression under `!` wrappers.
/// Chain contents are call/member expressions, which never need them.
///
/// The grammar is `extends LeftHandSideExpression`.
/// So most variants below are REQUIRED, without parens the output is a syntax error (e.g. `extends a + b`).
fn class_extends_needs_parens_through_non_null(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::TSNonNullExpression(non_null) => {
            class_extends_needs_parens_through_non_null(&non_null.expression)
        }
        Expression::ClassExpression(class) => !class.decorators.is_empty(),
        Expression::ArrowFunctionExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::YieldExpression(_)
        // NOTE: These three are readability style only, since they are LeftHandSideExpressions and parse bare:
        // - `ObjectExpression` (`extends {} {}`, confusable with the class body)
        // - `NewExpression` (`extends new A() {}`, confusable with a constructor call)
        // - `TaggedTemplateExpression` (`extends tag`x` {}`, confusable with a template literal)
        | Expression::ObjectExpression(_)
        | Expression::NewExpression(_)
        | Expression::TaggedTemplateExpression(_) => true,
        _ => false,
    }
}

impl NeedsParentheses<'_> for AstNode<'_, TSInstantiationExpression<'_>> {
    fn needs_parentheses(&self, _f: &JsFormatter<'_, '_>) -> bool {
        let expr = match self.parent() {
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
        parent if parent.is_call_like_callee_span(binary_like.span()) => return true,
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
        Expression::PrivateFieldExpression(e) => Some(&e.object),
        Expression::TaggedTemplateExpression(e) => Some(&e.tag),
        Expression::TSNonNullExpression(e) => Some(&e.expression),
        _ => None,
    })
    .any(|object| matches!(object, Expression::CallExpression(_) | Expression::ImportExpression(_)))
}

#[derive(Clone, Copy)]
enum UnaryLike<'a, 'b> {
    UpdateExpression(&'b AstNode<'a, UpdateExpression<'a>>),
    UnaryExpression(&'b AstNode<'a, UnaryExpression<'a>>),
}

impl UnaryLike<'_, '_> {
    fn parent(&self) -> &AstNodes<'_> {
        match self {
            Self::UpdateExpression(e) => e.parent(),
            Self::UnaryExpression(e) => e.parent(),
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
        | AstNodes::PrivateFieldExpression(_)
        | AstNodes::TaggedTemplateExpression(_) => return true,
        _ if is_class_extends(span, parent) || parent.is_call_like_callee_span(span) => {
            return true;
        }
        _ => {}
    }
    if let AstNodes::ComputedMemberExpression(computed_member_expr) = parent {
        return computed_member_expr.object.span() == span;
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
    parent: &AstNodes<'_>,
    mode: FirstInStatementMode,
) -> bool {
    for (index, ancestor) in parent.ancestors().enumerate() {
        let is_not_first_iteration = index > 0;

        match ancestor {
            AstNodes::ExpressionStatement(stmt) => {
                if stmt.is_arrow_function_body() {
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
            | AstNodes::TaggedTemplateExpression(_)
            | AstNodes::ChainExpression(_)
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
            _ if ancestor.is_call_like_callee_span(current_span) => {}
            _ => break,
        }
        current_span = ancestor.span();
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
            | AstNodes::PrivateInExpression(_)
            | AstNodes::TSInstantiationExpression(_),
    ) {
        return true;
    }
    if let AstNodes::ConditionalExpression(e) = node {
        e.test.span() == span
    } else {
        update_or_lower_expression_needs_parens(span, node)
    }
}

fn ts_as_or_satisfies_needs_parens(
    span: Span,
    inner: &AstNode<'_, Expression<'_>>,
    parent: &AstNodes<'_>,
) -> bool {
    match parent {
        AstNodes::ConditionalExpression(_)
        // Binary-like
        | AstNodes::LogicalExpression(_)
        | AstNodes::BinaryExpression(_) => true,
        // `export default (function foo() {} as bar)` and `export default (class {} as bar)`.
        // Without parens, the leading `function`/`class` token makes `export default` parse the RHS as a declaration,
        // detaching the cast.
        // The leftmost expression is checked (not just `inner`) so chained casts and member chains are covered too,
        // e.g. `export default (function () {} as A as B)` and `export default (function () {}.bar as A)`.
        //
        // When the leftmost function/class already wraps itself,
        // the outer cast doesn't need its own parens, or we'd emit `((function () {})() as A)`.
        // Mirrors `Function::needs_parentheses`.
        AstNodes::ExportDefaultDeclaration(_) => {
            let leftmost = ExpressionLeftSide::leftmost(inner);
            matches!(
                leftmost.as_ref(),
                Expression::FunctionExpression(_) | Expression::ClassExpression(_)
            ) && !matches!(leftmost.parent(), AstNodes::TaggedTemplateExpression(_))
                && !leftmost.is_call_like_callee()
        }
        _ => {
            type_cast_like_needs_parens(span, parent)
        }
    }
}

fn is_class_extends(span: Span, parent: &AstNodes<'_>) -> bool {
    if let AstNodes::Class(c) = parent {
        return c.super_class.as_ref().is_some_and(|c| c.span() == span);
    }
    false
}

fn jsx_element_or_fragment_needs_paren(span: Span, parent: &AstNodes<'_>) -> bool {
    if is_class_extends(span, parent) {
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
        | AstNodes::TaggedTemplateExpression(_)
        | AstNodes::JSXSpreadAttribute(_)
        | AstNodes::JSXSpreadChild(_) => true,
        _ if parent.is_call_like_callee_span(span) => true,
        _ => false,
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXElement<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        jsx_element_or_fragment_needs_paren(self.span, self.parent())
    }
}

impl NeedsParentheses<'_> for AstNode<'_, JSXFragment<'_>> {
    fn needs_parentheses(&self, f: &JsFormatter<'_, '_>) -> bool {
        if f.comments().is_marked_as_type_cast_node(self) {
            return false;
        }

        jsx_element_or_fragment_needs_paren(self.span, self.parent())
    }
}
