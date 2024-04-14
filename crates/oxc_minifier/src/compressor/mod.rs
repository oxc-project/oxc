#![allow(clippy::unused_self)]

mod ast_util;
mod fold;
mod options;
mod prepass;
mod util;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::visit::walk_mut::{
    walk_binary_expression_mut, walk_expression_mut, walk_return_statement_mut, walk_statement_mut,
    walk_statements_mut,
};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, VisitMut};
use oxc_span::Span;
use oxc_syntax::{
    operator::{BinaryOperator, UnaryOperator},
    precedence::GetPrecedence,
    NumberBase,
};

pub use self::options::CompressOptions;
use self::prepass::Prepass;

pub struct Compressor<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,

    prepass: Prepass<'a>,
}

const SPAN: Span = Span::new(0, 0);

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator, options: CompressOptions) -> Self {
        Self { ast: AstBuilder::new(allocator), options, prepass: Prepass::new(allocator) }
    }

    pub fn build(mut self, program: &mut Program<'a>) {
        self.prepass.build(program);
        self.visit_program(program);
    }

    /* Utilities */

    /// `1/0`
    #[allow(unused)]
    fn create_one_div_zero(&mut self) -> Expression<'a> {
        let left = self.ast.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
        let left = self.ast.literal_number_expression(left);
        let right = self.ast.number_literal(SPAN, 0.0, "0", NumberBase::Decimal);
        let right = self.ast.literal_number_expression(right);
        self.ast.binary_expression(SPAN, left, BinaryOperator::Division, right)
    }

    /* Statements */

    /// Remove block from single line blocks
    /// `{ block } -> block`
    #[allow(clippy::only_used_in_recursion)] // `&self` is only used in recursion
    fn compress_block(&self, stmt: &mut Statement<'a>) {
        if let Statement::BlockStatement(block) = stmt {
            // Avoid compressing `if (x) { var x = 1 }` to `if (x) var x = 1` due to different
            // semantics according to AnnexB, which lead to different semantics.
            if block.body.len() == 1 && !matches!(&block.body[0], Statement::Declaration(_)) {
                *stmt = block.body.remove(0);
                self.compress_block(stmt);
            }
        }
    }

    /// Drop `drop_debugger` statement.
    /// Enabled by `compress.drop_debugger`
    fn drop_debugger(&mut self, stmt: &Statement<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && self.options.drop_debugger
    }

    /// Drop `console.*` expressions.
    /// Enabled by `compress.drop_console
    fn drop_console(&mut self, stmt: &Statement<'a>) -> bool {
        self.options.drop_console
            && matches!(stmt, Statement::ExpressionStatement(expr) if util::is_console(&expr.expression))
    }

    fn compress_console(&mut self, expr: &mut Expression<'a>) -> bool {
        if self.options.drop_console && util::is_console(expr) {
            *expr = self.ast.void_0();
            true
        } else {
            false
        }
    }

    /// Join consecutive var statements
    fn join_vars(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Collect all the consecutive ranges that contain joinable vars.
        // This is required because Rust prevents in-place vec mutation.
        let mut ranges = vec![];
        let mut range = 0..0;
        let mut i = 1usize;
        let mut capacity = 0usize;
        for window in stmts.windows(2) {
            let [prev, cur] = window else { unreachable!() };
            if let (
                Statement::Declaration(Declaration::VariableDeclaration(cur_decl)),
                Statement::Declaration(Declaration::VariableDeclaration(prev_decl)),
            ) = (cur, prev)
            {
                if cur_decl.kind == prev_decl.kind {
                    if i - 1 != range.end {
                        range.start = i - 1;
                    }
                    range.end = i + 1;
                }
            }
            if (range.end != i || i == stmts.len() - 1) && range.start < range.end {
                capacity += range.end - range.start - 1;
                ranges.push(range.clone());
                range = 0..0;
            }
            i += 1;
        }

        if ranges.is_empty() {
            return;
        }

        // Reconstruct the stmts array by joining consecutive ranges
        let mut new_stmts = self.ast.new_vec_with_capacity(stmts.len() - capacity);
        for (i, stmt) in stmts.drain(..).enumerate() {
            if i > 0 && ranges.iter().any(|range| range.contains(&(i - 1)) && range.contains(&i)) {
                if let Statement::Declaration(Declaration::VariableDeclaration(prev_decl)) =
                    new_stmts.last_mut().unwrap()
                {
                    if let Statement::Declaration(Declaration::VariableDeclaration(mut cur_decl)) =
                        stmt
                    {
                        prev_decl.declarations.append(&mut cur_decl.declarations);
                    }
                }
            } else {
                new_stmts.push(stmt);
            }
        }
        *stmts = new_stmts;
    }

    /// Transforms `while(expr)` to `for(;expr;)`
    fn compress_while(&mut self, stmt: &mut Statement<'a>) {
        let Statement::WhileStatement(while_stmt) = stmt else { return };
        if self.options.loops {
            let dummy_test = self.ast.this_expression(SPAN);
            let test = std::mem::replace(&mut while_stmt.test, dummy_test);
            let body = self.ast.move_statement(&mut while_stmt.body);
            *stmt = self.ast.for_statement(SPAN, None, Some(test), None, body);
        }
    }

    /* Expressions */

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

    /// Transforms `Infinity` => `1/0`
    #[allow(unused)]
    fn compress_infinity(&mut self, expr: &mut Expression<'a>) -> bool {
        let Expression::Identifier(ident) = expr else { return false };
        if ident.name == "Infinity" {
            // if let Some(reference_id) = ident.reference_id.get() {
            //&& self.semantic.symbols().is_global_reference(reference_id)
            *expr = self.create_one_div_zero();
            return true;
            // }
        }
        false
    }

    /// Transforms boolean expression `true` => `!0` `false` => `!1`
    /// Enabled by `compress.booleans`
    fn compress_boolean(&mut self, expr: &mut Expression<'a>) -> bool {
        let Expression::BooleanLiteral(lit) = expr else { return false };
        if self.options.booleans {
            let num = self.ast.number_literal(
                SPAN,
                if lit.value { 0.0 } else { 1.0 },
                if lit.value { "0" } else { "1" },
                NumberBase::Decimal,
            );
            let num = self.ast.literal_number_expression(num);
            *expr = self.ast.unary_expression(SPAN, UnaryOperator::LogicalNot, num);
            return true;
        }
        false
    }

    /// Transforms `typeof foo == "undefined"` into `foo === void 0`
    /// Enabled by `compress.typeofs`
    fn compress_typeof_undefined(&self, expr: &mut BinaryExpression<'a>) {
        if !self.options.typeofs {
            return;
        }
        match expr.operator {
            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                let pair = self.commutative_pair(
                    (&expr.left, &expr.right),
                    |a| {
                        if a.is_specific_string_literal("undefined") {
                            return Some(());
                        }
                        None
                    },
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
                if let Some((_void_exp, id_ref)) = pair {
                    let span = expr.span;
                    let left = self.ast.void_0();
                    let operator = BinaryOperator::StrictEquality;
                    let right = self.ast.identifier_reference_expression(id_ref);
                    let cmp = BinaryExpression { span, left, operator, right };
                    *expr = cmp;
                }
            }
            _ => {}
        };
    }

    fn commutative_pair<A, F, G, RetF: 'a, RetG: 'a>(
        &self,
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
    fn compress_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        if stmt.argument.as_ref().is_some_and(|expr| expr.is_undefined() || expr.is_void_0()) {
            stmt.argument = None;
        }
    }

    fn compress_variable_declarator(&mut self, decl: &mut VariableDeclarator<'a>) {
        if decl.kind.is_const() {
            return;
        }
        if decl.init.as_ref().is_some_and(|init| init.is_undefined() || init.is_void_0()) {
            decl.init = None;
        }
    }

    /// [Peephole Reorder Constant Expression](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeReorderConstantExpression.java)
    ///
    /// Reorder constant expression hoping for a better compression.
    /// ex. x === 0 -> 0 === x
    /// After reordering, expressions like 0 === x and 0 === y may have higher
    /// compression together than their original counterparts.
    #[allow(unused)]
    fn reorder_constant_expression(&self, expr: &mut BinaryExpression<'a>) {
        let operator = expr.operator;
        if operator.is_equality()
            || operator.is_compare()
            || operator == BinaryOperator::Multiplication
        {
            if expr.precedence() == expr.left.precedence() {
                return;
            }
            if !expr.left.is_immutable_value() && expr.right.is_immutable_value() {
                if let Some(inverse_operator) = operator.compare_inverse_operator() {
                    expr.operator = inverse_operator;
                }
                std::mem::swap(&mut expr.left, &mut expr.right);
            }
        }
    }
}

impl<'a> VisitMut<'a> for Compressor<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| {
            if self.drop_debugger(stmt) {
                return false;
            }
            if self.drop_console(stmt) {
                return false;
            }
            true
        });

        self.join_vars(stmts);

        walk_statements_mut(self, stmts);
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.compress_block(stmt);
        self.compress_while(stmt);
        self.fold_condition(stmt);
        walk_statement_mut(self, stmt);
    }

    fn visit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        walk_return_statement_mut(self, stmt);
        // We may fold `void 1` to `void 0`, so compress it after visiting
        self.compress_return_statement(stmt);
    }

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        for declarator in decl.declarations.iter_mut() {
            self.visit_variable_declarator(declarator);
            self.compress_variable_declarator(declarator);
        }
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        walk_expression_mut(self, expr);
        self.compress_console(expr);
        self.fold_expression(expr);
        if !self.compress_undefined(expr) {
            self.compress_boolean(expr);
        }
    }

    fn visit_binary_expression(&mut self, expr: &mut BinaryExpression<'a>) {
        walk_binary_expression_mut(self, expr);
        self.compress_typeof_undefined(expr);
    }
}
