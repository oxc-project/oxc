use crate::peephole::PeepholeOptimizations;
use oxc_allocator::TakeIn;
use oxc_ast::ast::{Argument, Expression, Function};

use crate::TraverseCtx;

impl<'a> PeepholeOptimizations {
    fn convert_function_expression_to_arrow_function_expression(
        func_expr: &mut Function<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(scope) = func_expr.scope_id.take() {
            ctx.ast.expression_arrow_function_with_scope_id_and_pure_and_pife(
                func_expr.span,
                false,
                func_expr.r#async,
                func_expr.type_parameters.take(),
                func_expr.params.take_in(ctx.ast),
                func_expr.return_type.take(),
                func_expr.body.take().unwrap(),
                scope,
                func_expr.pure,
                func_expr.pife,
            )
        } else {
            ctx.ast.expression_arrow_function(
                func_expr.span,
                false,
                func_expr.r#async,
                func_expr.type_parameters.take(),
                func_expr.params.take_in(ctx.ast),
                func_expr.return_type.take(),
                func_expr.body.take().unwrap(),
            )
        }
    }

    /// Transforms a function call with `this` as the first argument into an immediately invoked function expression.
    /// - (function () {}).call(this, a, b); -> (() => {})(a, b))
    pub fn substitute_function_call_this_for_arrow_function(
        e: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::CallExpression(call_expr) = e else { return };

        if !call_expr.arguments.is_empty()
            // check if the first argument is `this`
            && call_expr.arguments.first().is_some_and(|args| matches!(args, Argument::ThisExpression(_)))
            && let Expression::StaticMemberExpression(static_expr) = &mut call_expr.callee
            // only supports `call` as a result of bind is a BoundFunctionObject
            && static_expr.property.name == "call"
            && let Expression::FunctionExpression(func_expr) = &mut static_expr.object
            // do not process async and generator due to Object.getPrototypeOf
            && !func_expr.generator
            && !func_expr.r#async
            // we do not know if the name is used within a function
            && func_expr.name().is_none()
        {
            ctx.state.changed = true;
            call_expr.callee =
                Self::convert_function_expression_to_arrow_function_expression(func_expr, ctx);
            call_expr.arguments.remove(0);
        }
    }
}
