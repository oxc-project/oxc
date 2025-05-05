//! Experiment into implementing AST traversal without recursion, using a stack instead.
//!
//! In this version, we use the `Ancestor` stack to double as the stack for visitation.
//!
//! But turns out the 2 don't map to each other exactly.
//! `walk_expression` here calls `walk_binary_expression` which calls `walk_expression`,
//! so it hasn't eradicated recursion.
//! Need to remove that, but `Ancestor`s stack doesn't include enums.
//!
//! Can we solve this with another enum which extends ancestor to also have exit variants?
//! The stack can be of `AncestorEnterOrExit` and `TraverseAncestry::parent` etc converts
//! to an `AncestorEnterOrExit` to an `Ancestor`?

#![expect(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::todo)]

use std::ptr::{self, addr_of_mut};

// use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Traverse, TraverseCtx,
    ancestor::{self, Ancestor, AncestorType},
};

pub unsafe fn walk_ast<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    program: &mut Program<'a>,
    ctx: &mut TraverseCtx<'a>,
) {
    let program_ptr = program as *mut Program;

    ctx.push_stack(Ancestor::ProgramBody(ancestor::ProgramWithoutBody::new(program_ptr)));

    traverser.enter_program(&mut *program, ctx);

    // TODO: Walk `Program` fields

    loop {
        let item = *ctx.ancestry.stack.last();
        if matches!(item, Ancestor::None) {
            break;
        }

        // These fire when *exiting* the field
        match item {
            Ancestor::BinaryExpressionLeft(bin_expr_ptr) => {
                let bin_expr_ptr = bin_expr_ptr.0;
                // Exit left
                let left_ptr = addr_of_mut!((*bin_expr_ptr).left);
                traverser.exit_expression(&mut *left_ptr, ctx);
                // Enter right
                ctx.retag_stack(AncestorType::BinaryExpressionRight);
                let right_ptr = addr_of_mut!((*bin_expr_ptr).right);
                walk_expression(traverser, right_ptr, ctx);
            }
            Ancestor::BinaryExpressionRight(bin_expr_ptr) => {
                ctx.pop_stack_unchecked();
                let bin_expr_ptr = bin_expr_ptr.0;
                let right_ptr = addr_of_mut!((*bin_expr_ptr).right);
                traverser.exit_expression(&mut *right_ptr, ctx);
                traverser.exit_binary_expression(&mut *bin_expr_ptr, ctx);
            }
            _ => todo!(),
        }
    }
}

unsafe fn walk_expression<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    expr_ptr: *mut Expression<'a>,
    ctx: &mut TraverseCtx<'a>,
) {
    let expr = &mut *expr_ptr;
    traverser.enter_expression(expr, ctx);

    match expr {
        Expression::BinaryExpression(bin_expr) => {
            let bin_expr_ptr = ptr::from_mut(bin_expr.as_mut());
            walk_binary_expression(traverser, bin_expr_ptr, ctx);
        }
        Expression::NullLiteral(null_lit) => {
            let null_lit_ptr = ptr::from_mut(null_lit.as_mut());
            walk_null_literal(traverser, null_lit_ptr, ctx);
        }
        _ => todo!(),
    }
}

unsafe fn walk_binary_expression<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    bin_expr_ptr: *mut BinaryExpression<'a>,
    ctx: &mut TraverseCtx<'a>,
) {
    ctx.push_stack(Ancestor::BinaryExpressionLeft(ancestor::BinaryExpressionWithoutLeft::new(
        bin_expr_ptr,
    )));
    // Enter left
    let left_ptr = addr_of_mut!((*bin_expr_ptr).left);
    walk_expression(traverser, left_ptr, ctx);
}

unsafe fn walk_null_literal<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    null_lit_ptr: *mut NullLiteral,
    ctx: &mut TraverseCtx<'a>,
) {
    let null_lit = &mut *null_lit_ptr;
    traverser.enter_null_literal(null_lit, ctx);
    traverser.exit_null_literal(null_lit, ctx);
}
