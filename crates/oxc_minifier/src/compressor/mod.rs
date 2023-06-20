#![allow(clippy::unused_self)]

mod fold;

use oxc_allocator::{Allocator, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_hir::{hir::*, HirBuilder, VisitMut};
use oxc_semantic::Semantic;
use oxc_span::Span;
use oxc_syntax::{
    operator::{BinaryOperator, UnaryOperator},
    precedence::GetPrecedence,
    NumberBase,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct CompressOptions {
    /// Various optimizations for boolean context, for example `!!a ? b : c` → `a ? b : c`
    /// Default true
    pub booleans: bool,

    /// Remove `debugger;` statements
    /// Default true
    pub drop_debugger: bool,

    /// Join consecutive var statements
    /// Default true
    pub join_vars: bool,

    /// Optimizations for do, while and for loops when we can statically determine the condition
    /// Default: true
    pub loops: bool,

    /// Transforms `typeof foo == "undefined" into `foo === void 0`
    /// Default true
    pub typeofs: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self { booleans: true, drop_debugger: true, join_vars: true, loops: true, typeofs: true }
    }
}

pub struct Compressor<'a> {
    hir: HirBuilder<'a>,
    semantic: Semantic<'a>,
    options: CompressOptions,
}

const SPAN: Span = Span::new(0, 0);

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator, semantic: Semantic<'a>, options: CompressOptions) -> Self {
        Self { hir: HirBuilder::new(allocator), semantic, options }
    }

    pub fn build<'b>(mut self, program: &'b mut Program<'a>) -> Semantic<'a> {
        self.visit_program(program);
        self.semantic
    }

    /* Utilities */

    /// `void 0`
    fn create_void_0(&mut self) -> Expression<'a> {
        let left = self.hir.number_literal(SPAN, 0.0, "0", NumberBase::Decimal);
        let num = self.hir.literal_number_expression(left);
        self.hir.unary_expression(SPAN, UnaryOperator::Void, num)
    }

    /// `1/0`
    fn create_one_div_zero(&mut self) -> Expression<'a> {
        let left = self.hir.number_literal(SPAN, 1.0, "1", NumberBase::Decimal);
        let left = self.hir.literal_number_expression(left);
        let right = self.hir.number_literal(SPAN, 0.0, "0", NumberBase::Decimal);
        let right = self.hir.literal_number_expression(right);
        self.hir.binary_expression(SPAN, left, BinaryOperator::Division, right)
    }

    /* Statements */

    /// Remove block from single line blocks
    /// `{ block } -> block`
    #[allow(clippy::only_used_in_recursion)] // `&self` is only used in recursion
    fn compress_block<'b>(&self, stmt: &'b mut Statement<'a>) {
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
    fn drop_debugger<'b>(&mut self, stmt: &'b Statement<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && self.options.drop_debugger
    }

    /// Join consecutive var statements
    fn join_vars<'b>(&mut self, stmts: &'b mut Vec<'a, Statement<'a>>) {
        // Collect all the consecutive ranges that contain joinable vars.
        // This is required because Rust prevents in-place vec mutation.
        let mut ranges = vec![];
        let mut range = 0..0;
        let mut i = 1usize;
        let mut capacity = 0usize;
        for window in stmts.windows(2) {
            let [prev, cur] = window else { unreachable!() };
            if let Statement::Declaration(Declaration::VariableDeclaration(cur_decl)) = cur
                && let Statement::Declaration(Declaration::VariableDeclaration(prev_decl)) = prev
                && cur_decl.kind == prev_decl.kind {
                if i - 1 != range.end  {
                    range.start = i - 1;
                }
                range.end = i + 1;
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
        let mut new_stmts = self.hir.new_vec_with_capacity(stmts.len() - capacity);
        for (i, stmt) in stmts.drain(..).enumerate() {
            if i > 0
                && ranges.iter().any(|range| range.contains(&(i-1)) && range.contains(&i))
                && let Statement::Declaration(Declaration::VariableDeclaration(prev_decl)) = new_stmts.last_mut().unwrap()
                && let Statement::Declaration(Declaration::VariableDeclaration(mut cur_decl)) = stmt {
                prev_decl.declarations.append(&mut cur_decl.declarations);
            } else {
                new_stmts.push(stmt);
            }
        }
        *stmts = new_stmts;
    }

    /// Transforms `while(expr)` to `for(;expr;)`
    fn compress_while<'b>(&mut self, stmt: &'b mut Statement<'a>) {
        if let Statement::WhileStatement(while_stmt) = stmt
            && self.options.loops {
            let dummy_test = self.hir.this_expression(SPAN);
            let test = std::mem::replace(&mut while_stmt.test, dummy_test);
            let body = while_stmt.body.take();
            *stmt = self.hir.for_statement(SPAN, None, Some(test), None, body);
        }
    }

    /* Expressions */

    /// Transforms `undefined` => `void 0`
    fn compress_undefined<'b>(&mut self, expr: &'b mut Expression<'a>) -> bool {
        if let Expression::Identifier(ident) = expr
        && ident.name == "undefined"
        && self.semantic.symbols().is_global_reference(ident.reference_id.clone().into_inner()) {
            *expr = self.create_void_0();
            return true;
        }
        false
    }

    /// Transforms `Infinity` => `1/0`
    #[allow(unused)]
    fn compress_infinity<'b>(&mut self, expr: &'b mut Expression<'a>) -> bool {
        if let Expression::Identifier(ident) = expr
        && ident.name == "Infinity"
        && self.semantic.symbols().is_global_reference(ident.reference_id.clone().into_inner()) {
            *expr = self.create_one_div_zero();
            return true;
        }
        false
    }

    /// Transforms boolean expression `true` => `!0` `false` => `!1`
    /// Enabled by `compress.booleans`
    fn compress_boolean<'b>(&mut self, expr: &'b mut Expression<'a>) -> bool {
        if let Expression::BooleanLiteral(lit) = expr
        && self.options.booleans {
            let num = self.hir.number_literal(
                SPAN,
                if lit.value { 0.0 } else { 1.0 },
                if lit.value { "0" } else { "1" },
                NumberBase::Decimal,
            );
            let num = self.hir.literal_number_expression(num);
            *expr = self.hir.unary_expression(SPAN, UnaryOperator::LogicalNot, num);
            return true;
        }
        false
    }

    /// Transforms `typeof foo == "undefined"` into `foo === void 0`
    /// Enabled by `compress.typeofs`
    fn compress_typeof_undefined<'b>(&mut self, expr: &'b mut BinaryExpression<'a>) {
        if expr.operator.is_equality()
            && self.options.typeofs
            && let Expression::UnaryExpression(unary_expr) = &expr.left
            && unary_expr.operator == UnaryOperator::Typeof
            && let Expression::Identifier(ident) = &unary_expr.argument
            && let Expression::StringLiteral(s) = &expr.right
            && s.value == "undefined" {
            let left = self.hir.identifier_reference_expression((*ident).clone());
            let right = self.create_void_0();
            let operator = BinaryOperator::StrictEquality;
            *expr = BinaryExpression {span: SPAN, left, operator, right};
        }
    }

    /// Removes redundant argument of `ReturnStatement`
    ///
    /// `return undefined` -> `return`
    /// `return void 0` -> `return`
    fn compress_return_statement<'b>(&mut self, stmt: &'b mut ReturnStatement<'a>) {
        if let Some(expr) = &stmt.argument
            && (expr.is_undefined() || expr.is_void_0()) {
            stmt.argument = None;
        }
    }

    /// [Peephole Reorder Constant Expression](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeReorderConstantExpression.java)
    ///
    /// Reorder constant expression hoping for a better compression.
    /// ex. x === 0 -> 0 === x
    /// After reordering, expressions like 0 === x and 0 === y may have higher
    /// compression together than their original counterparts.
    #[allow(unused)]
    fn reorder_constant_expression<'b>(&self, expr: &'b mut BinaryExpression<'a>) {
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

impl<'a, 'b> VisitMut<'a, 'b> for Compressor<'a> {
    fn visit_statements(&mut self, stmts: &'b mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| !self.drop_debugger(stmt));

        self.join_vars(stmts);

        for stmt in stmts.iter_mut() {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &'b mut Statement<'a>) {
        self.compress_block(stmt);
        self.compress_while(stmt);
        self.visit_statement_match(stmt);
    }

    fn visit_return_statement(&mut self, stmt: &'b mut ReturnStatement<'a>) {
        self.compress_return_statement(stmt);
        if let Some(arg) = &mut stmt.argument {
            self.visit_expression(arg);
        }
    }

    fn visit_expression(&mut self, expr: &'b mut Expression<'a>) {
        self.visit_expression_match(expr);
        self.fold_expression(expr);
        if !self.compress_undefined(expr) {
            self.compress_boolean(expr);
        }
    }

    fn visit_binary_expression(&mut self, expr: &'b mut BinaryExpression<'a>) {
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);

        self.compress_typeof_undefined(expr);
    }
}
