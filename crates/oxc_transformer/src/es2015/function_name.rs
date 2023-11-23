use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::GetSpan;

use crate::{
    context::TransformerCtx,
    options::{TransformOptions, TransformTarget},
};

/// ES2015: Function Name
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-function-name>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-function-name>
pub struct FunctionName<'a> {
    ast: Rc<AstBuilder<'a>>,
    //ctx: TransformerCtx<'a>,
}

impl<'a> FunctionName<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        // ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.function_name).then(|| Self { ast })
    }

    // https://github.com/babel/babel/blob/main/packages/babel-helper-function-name/src/index.ts
    pub fn transform_function<'b>(&mut self, func: &'b mut Function<'a>) {
        if func.r#type != FunctionType::FunctionExpression {
            return;
        }

        // Already has an id
        if func.id.is_some() {
            return;
        }
    }

    pub fn transform_variable_declarator<'b>(&mut self, decl: &'b mut VariableDeclarator<'a>) {
        let Some(init) = &mut decl.init else { return };

        // No name to infer from
        let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind else {
            return;
        };

        match init {
            // () => {}
            Expression::ArrowExpression(expr) => {
                let expr = self.ast.copy(&**expr);

                // Turn arrow into func, better way???
                let func = self.ast.function_expression(self.ast.function(
                    FunctionType::FunctionExpression,
                    expr.span,
                    Some((*id).clone()),
                    expr.expression,
                    expr.generator,
                    expr.r#async,
                    expr.params,
                    Some(expr.body),
                    expr.type_parameters,
                    expr.return_type,
                    Default::default(),
                ));

                decl.init = Some(func);
            }

            // function () {}
            // function name() {}
            Expression::FunctionExpression(expr) => {
                if expr.id.is_none() {
                    expr.id = Some((*id).clone());
                }
            }
            _ => {}
        };
    }
}
