use oxc_ast::{ast::*, visit::walk_mut, AstBuilder, VisitMut};
use oxc_span::SPAN;
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::CompressOptions;

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
pub struct SubstituteAlternateSyntax<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,
}

impl<'a> VisitMut<'a> for SubstituteAlternateSyntax<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.compress_block(stmt);
        // self.compress_while(stmt);
        walk_mut::walk_statement(self, stmt);
    }

    fn visit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        walk_mut::walk_return_statement(self, stmt);
        // We may fold `void 1` to `void 0`, so compress it after visiting
        Self::compress_return_statement(stmt);
    }

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        for declarator in decl.declarations.iter_mut() {
            self.visit_variable_declarator(declarator);
            Self::compress_variable_declarator(declarator);
        }
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        // Bail cjs `Object.defineProperty(exports, ...)`
        if Self::is_object_define_property_exports(expr) {
            return;
        }
        walk_mut::walk_expression(self, expr);
        if !self.compress_undefined(expr) {
            self.compress_boolean(expr);
        }
    }

    fn visit_binary_expression(&mut self, expr: &mut BinaryExpression<'a>) {
        walk_mut::walk_binary_expression(self, expr);
        self.compress_typeof_undefined(expr);
    }
}

impl<'a> SubstituteAlternateSyntax<'a> {
    pub fn new(ast: AstBuilder<'a>, options: CompressOptions) -> Self {
        Self { ast, options }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    /* Utilities */

    /// Transforms `undefined` => `void 0`
    fn compress_undefined(&self, expr: &mut Expression<'a>) -> bool {
        let Expression::Identifier(ident) = expr else { return false };
        if ident.name == "undefined" {
            // if let Some(reference_id) = ident.reference_id.get() {
            // && self.semantic.symbols().is_global_reference(reference_id)
            *expr = self.ast.void_0();
            return true;
            // }
        }
        false
    }

    /// Test `Object.defineProperty(exports, ...)`
    fn is_object_define_property_exports(expr: &Expression<'a>) -> bool {
        let Expression::CallExpression(call_expr) = expr else { return false };
        let Some(Argument::Identifier(ident)) = call_expr.arguments.first() else { return false };
        if ident.name != "exports" {
            return false;
        }
        call_expr.callee.is_specific_member_access("Object", "defineProperty")
    }

    /* Statements */

    /// Remove block from single line blocks
    /// `{ block } -> block`
    #[allow(clippy::only_used_in_recursion)] // `&self` is only used in recursion
    fn compress_block(&self, stmt: &mut Statement<'a>) {
        if let Statement::BlockStatement(block) = stmt {
            // Avoid compressing `if (x) { var x = 1 }` to `if (x) var x = 1` due to different
            // semantics according to AnnexB, which lead to different semantics.
            if block.body.len() == 1 && !block.body[0].is_declaration() {
                *stmt = block.body.remove(0);
                self.compress_block(stmt);
            }
        }
    }

    // /// Transforms `while(expr)` to `for(;expr;)`
    // fn compress_while(&mut self, stmt: &mut Statement<'a>) {
    // let Statement::WhileStatement(while_stmt) = stmt else { return };
    // if self.options.loops {
    // let dummy_test = self.ast.expression_this(SPAN);
    // let test = std::mem::replace(&mut while_stmt.test, dummy_test);
    // let body = self.ast.move_statement(&mut while_stmt.body);
    // *stmt = self.ast.statement_for(SPAN, None, Some(test), None, body);
    // }
    // }

    /* Expressions */

    /// Transforms boolean expression `true` => `!0` `false` => `!1`
    /// Enabled by `compress.booleans`
    fn compress_boolean(&mut self, expr: &mut Expression<'a>) -> bool {
        let Expression::BooleanLiteral(lit) = expr else { return false };
        if self.options.booleans {
            let num = self.ast.expression_numeric_literal(
                SPAN,
                if lit.value { 0.0 } else { 1.0 },
                if lit.value { "0" } else { "1" },
                NumberBase::Decimal,
            );
            *expr = self.ast.expression_unary(SPAN, UnaryOperator::LogicalNot, num);
            return true;
        }
        false
    }

    /// Compress `typeof foo == "undefined"` into `typeof foo > "u"`
    /// Enabled by `compress.typeofs`
    fn compress_typeof_undefined(&self, expr: &mut BinaryExpression<'a>) {
        if !self.options.typeofs {
            return;
        }
        if !matches!(expr.operator, BinaryOperator::Equality | BinaryOperator::StrictEquality) {
            return;
        }
        let pair = Self::commutative_pair(
            (&expr.left, &expr.right),
            |a| a.is_specific_string_literal("undefined").then_some(()),
            |b| {
                if let Expression::UnaryExpression(op) = b {
                    if op.operator == UnaryOperator::Typeof {
                        if let Expression::Identifier(id) = &op.argument {
                            return Some((*id).clone());
                        }
                    }
                }
                None
            },
        );
        let Some((_void_exp, id_ref)) = pair else {
            return;
        };
        let argument = self.ast.expression_from_identifier_reference(id_ref);
        let left = self.ast.unary_expression(SPAN, UnaryOperator::Typeof, argument);
        let right = self.ast.string_literal(SPAN, "u");
        let binary_expr = self.ast.binary_expression(
            expr.span,
            self.ast.expression_from_unary(left),
            BinaryOperator::GreaterThan,
            self.ast.expression_from_string_literal(right),
        );
        *expr = binary_expr;
    }

    fn commutative_pair<A, F, G, RetF: 'a, RetG: 'a>(
        pair: (&A, &A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&A) -> Option<RetF>,
        G: Fn(&A) -> Option<RetG>,
    {
        if let Some(a) = check_a(pair.0) {
            if let Some(b) = check_b(pair.1) {
                return Some((a, b));
            }
        } else if let Some(a) = check_a(pair.1) {
            if let Some(b) = check_b(pair.0) {
                return Some((a, b));
            }
        }
        None
    }

    /// Removes redundant argument of `ReturnStatement`
    ///
    /// `return undefined` -> `return`
    /// `return void 0` -> `return`
    fn compress_return_statement(stmt: &mut ReturnStatement<'a>) {
        if stmt.argument.as_ref().is_some_and(|expr| expr.is_undefined() || expr.is_void_0()) {
            stmt.argument = None;
        }
    }

    fn compress_variable_declarator(decl: &mut VariableDeclarator<'a>) {
        if decl.kind.is_const() {
            return;
        }
        if decl.init.as_ref().is_some_and(|init| init.is_undefined() || init.is_void_0()) {
            decl.init = None;
        }
    }
}
