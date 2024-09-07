use crate::context::Ctx;
use crate::es2017::utils::{
    async_generator_step, async_to_generator, function_apply, generate_caller_from_arrow,
    generate_caller_from_function,
};
use oxc_allocator::{Box, CloneIn};
use oxc_ast::ast::{
    ArrowFunctionExpression, AwaitExpression, BindingRestElement, Expression, FormalParameterKind,
    Function, FunctionType, Program, Statement, TSThisParameter, TSTypeAnnotation,
    TSTypeParameterDeclaration, TSTypeParameterInstantiation, VariableDeclarationKind,
};
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

/// ES2017: Async / Await
///
/// This plugin transforms async functions to generator functions.
///
/// Reference:
/// * <https://babeljs.io/docs/en/babel-plugin-transform-async-to-generator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-to-generator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-helper-remap-async-to-generator>
pub struct AsyncToGenerator<'a> {
    ctx: Ctx<'a>,

    inject_helpers: bool,
}

impl<'a> AsyncToGenerator<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { ctx, inject_helpers: false }
    }
}

impl<'a> AsyncToGenerator<'a> {
    pub fn transform_await_to_yield(&mut self, decl: &Box<AwaitExpression>) -> Expression<'a> {
        self.ctx.ast.expression_yield(
            SPAN,
            false,
            Some(decl.argument.clone_in(self.ctx.ast.allocator)),
        )
    }
}

impl<'a> Traverse<'a> for AsyncToGenerator<'a> {
    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        let mut stmts = self.ctx.ast.vec();

        if self.inject_helpers {
            stmts.push(async_generator_step(&self.ctx.ast).clone_in(self.ctx.ast.allocator));
            stmts.push(async_to_generator(&self.ctx.ast).clone_in(self.ctx.ast.allocator));
        }

        for stmt in self.ctx.ast.move_vec(&mut program.body) {
            match stmt {
                Statement::FunctionDeclaration(mut decl) => {
                    if !decl.r#async || decl.generator {
                        stmts.push(
                            self.ctx.ast.statement_declaration(
                                self.ctx.ast.declaration_from_function(
                                    decl.clone_in(self.ctx.ast.allocator),
                                ),
                            ),
                        );
                    } else {
                        let mut result = self.ctx.ast.vec();
                        decl.r#async = false;
                        decl.generator = true;
                        // TODO do not clone_in
                        let fn_name = decl
                            .id
                            .clone_in(self.ctx.ast.allocator)
                            .map_or("ref".to_owned(), |id| id.name.to_string());
                        let alias_name = "_".to_owned() + fn_name.as_str();
                        // generates the following code:
                        // function fn_name() {
                        //     return alias_name.apply(this, arguments);
                        // }
                        result.push(
                            self.ctx
                                .ast
                                .statement_declaration(self.ctx.ast.declaration_function(
                                    FunctionType::FunctionDeclaration,
                                    SPAN,
                                    Some(self.ctx.ast.binding_identifier(SPAN, fn_name.as_str())),
                                    false,
                                    false,
                                    false,
                                    decl.type_parameters.clone_in(self.ctx.ast.allocator),
                                    decl.this_param.clone_in(self.ctx.ast.allocator),
                                    decl.params.clone_in(self.ctx.ast.allocator),
                                    decl.return_type.clone_in(self.ctx.ast.allocator),
                                    Some(self.ctx.ast.function_body(
                                        SPAN,
                                        self.ctx.ast.vec(),
                                        self.ctx.ast.vec1(function_apply(
                                            alias_name.as_str(),
                                            &self.ctx.ast,
                                        )),
                                    )),
                                ))
                                .clone_in(self.ctx.ast.allocator),
                        );
                        result.push(self.ctx.ast.statement_declaration(self.ctx.ast.declaration_function(
                            FunctionType::FunctionDeclaration,
                            SPAN,
                            Some(self.ctx.ast.binding_identifier(SPAN, alias_name.as_str())),
                            false,
                            false,
                            false,
                            decl.type_parameters.clone_in(self.ctx.ast.allocator),
                            decl.this_param.clone_in(self.ctx.ast.allocator),
                            decl.params.clone_in(self.ctx.ast.allocator),
                            decl.return_type.clone_in(self.ctx.ast.allocator),
                            Some(self.ctx.ast.function_body(SPAN, self.ctx.ast.vec(), {
                                let mut result = self.ctx.ast.vec();
                                result.push(
                                    self.ctx.ast.statement_expression(
                                        SPAN,
                                        self.ctx.ast.expression_assignment(
                                            SPAN,
                                            AssignmentOperator::Assign,
                                            self.ctx.ast.assignment_target_simple(
                                                self.ctx.ast.simple_assignment_target_identifier_reference(
                                                    SPAN,
                                                    alias_name.as_str(),
                                                ),
                                            ),
                                            self.ctx.ast.expression_call(
                                                SPAN,
                                                self.ctx
                                                    .ast
                                                    .expression_identifier_reference(SPAN, "_asyncToGenerator"),
                                                None::<TSTypeParameterInstantiation>,
                                                self.ctx.ast.vec1(self.ctx.ast.argument_expression(
                                                    self.ctx.ast.expression_from_function(
                                                        decl.clone_in(self.ctx.ast.allocator),
                                                    ),
                                                )),
                                                false,
                                            ),
                                        ),
                                    ),
                                );
                                result.push(
                                    function_apply(fn_name.as_str(), &self.ctx.ast)
                                        .clone_in(self.ctx.ast.allocator),
                                );
                                result
                            })),
                        )));
                        stmts.extend(result);
                    }
                }
                _ => stmts.push(stmt),
            }
        }
        program.body = stmts;
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::AwaitExpression(decl) => {
                // Do not transform await in top-level
                let in_function = ctx.ancestry.ancestors().any(|ancestor| {
                    matches!(
                        ancestor,
                        Ancestor::FunctionBody(_) | Ancestor::ArrowFunctionExpressionBody(_)
                    )
                });
                if in_function {
                    *expr = self.transform_await_to_yield(decl).clone_in(self.ctx.ast.allocator);
                }
            }
            Expression::ArrowFunctionExpression(func) if func.r#async => {
                let func = func.clone_in(self.ctx.ast.allocator);
                if func.params.items.is_empty() {
                    *expr = generate_caller_from_arrow(&func, &self.ctx.ast)
                        .clone_in(self.ctx.ast.allocator);
                } else {
                    let mut statements = self.ctx.ast.vec();
                    statements.push(
                        self.ctx.ast.statement_declaration(
                            self.ctx.ast.declaration_variable(
                                SPAN,
                                VariableDeclarationKind::Var,
                                self.ctx.ast.vec1(
                                    self.ctx.ast.variable_declarator(
                                        SPAN,
                                        VariableDeclarationKind::Var,
                                        self.ctx.ast.binding_pattern(
                                            self.ctx.ast.binding_pattern_kind_binding_identifier(
                                                SPAN, "_ref",
                                            ),
                                            None::<TSTypeAnnotation>,
                                            false,
                                        ),
                                        Some(
                                            generate_caller_from_arrow(&func, &self.ctx.ast)
                                                .clone_in(self.ctx.ast.allocator),
                                        ),
                                        false,
                                    ),
                                ),
                                false,
                            ),
                        ),
                    );
                    statements.push(
                        function_apply("_ref", &self.ctx.ast).clone_in(self.ctx.ast.allocator),
                    );
                    *expr = self.ctx.ast.expression_parenthesized(
                        SPAN,
                        self.ctx.ast.expression_function(
                            FunctionType::FunctionExpression,
                            SPAN,
                            None,
                            false,
                            false,
                            false,
                            None::<TSTypeParameterDeclaration>,
                            None::<TSThisParameter>,
                            self.ctx.ast.formal_parameters(
                                SPAN,
                                FormalParameterKind::FormalParameter,
                                self.ctx.ast.vec(),
                                None::<BindingRestElement>,
                            ),
                            None::<TSTypeAnnotation>,
                            Some(self.ctx.ast.function_body(SPAN, self.ctx.ast.vec(), statements)),
                        ),
                    );
                }
            }
            Expression::FunctionExpression(func) if func.r#async => {
                let func = func.clone_in(self.ctx.ast.allocator);
                if func.params.items.is_empty() {
                    *expr = generate_caller_from_function(&func, &self.ctx.ast)
                        .clone_in(self.ctx.ast.allocator);
                } else {
                    let mut statements = self.ctx.ast.vec();
                    statements.push(
                        self.ctx.ast.statement_declaration(
                            self.ctx.ast.declaration_variable(
                                SPAN,
                                VariableDeclarationKind::Var,
                                self.ctx.ast.vec1(
                                    self.ctx.ast.variable_declarator(
                                        SPAN,
                                        VariableDeclarationKind::Var,
                                        self.ctx.ast.binding_pattern(
                                            self.ctx.ast.binding_pattern_kind_binding_identifier(
                                                SPAN, "_ref",
                                            ),
                                            None::<TSTypeAnnotation>,
                                            false,
                                        ),
                                        Some(
                                            generate_caller_from_function(&func, &self.ctx.ast)
                                                .clone_in(self.ctx.ast.allocator),
                                        ),
                                        false,
                                    ),
                                ),
                                false,
                            ),
                        ),
                    );
                    statements.push(
                        function_apply("_ref", &self.ctx.ast).clone_in(self.ctx.ast.allocator),
                    );
                    *expr = self.ctx.ast.expression_parenthesized(
                        SPAN,
                        self.ctx.ast.expression_function(
                            FunctionType::FunctionExpression,
                            SPAN,
                            None,
                            false,
                            false,
                            false,
                            None::<TSTypeParameterDeclaration>,
                            None::<TSThisParameter>,
                            self.ctx.ast.formal_parameters(
                                SPAN,
                                FormalParameterKind::FormalParameter,
                                self.ctx.ast.vec(),
                                None::<BindingRestElement>,
                            ),
                            None::<TSTypeAnnotation>,
                            Some(self.ctx.ast.function_body(SPAN, self.ctx.ast.vec(), statements)),
                        ),
                    );
                }
            }
            _ => {}
        }
    }

    fn enter_function(&mut self, function: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        if function.r#async && !function.generator {
            self.inject_helpers = true;
        }
    }

    fn enter_arrow_function_expression(
        &mut self,
        function: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if function.r#async {
            self.inject_helpers = true;
        }
    }
}
