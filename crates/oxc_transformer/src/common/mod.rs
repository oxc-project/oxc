//! Utility transforms which are in common between other transforms.

use arrow_function_converter::ArrowFunctionConverter;
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_traverse::{Ancestor, Traverse};

use crate::{EnvOptions, context::TraverseCtx, state::TransformState};
use module_imports::ModuleImportsStore;

pub mod arrow_function_converter;
pub mod computed_key;
pub mod duplicate;
pub mod helper_loader;
pub mod module_imports;
pub mod statement_injector;
pub mod top_level_statements;
pub mod var_declarations;

pub struct Common<'a> {
    arrow_function_converter: ArrowFunctionConverter<'a>,
}

impl Common<'_> {
    pub fn new(options: &EnvOptions) -> Self {
        Self { arrow_function_converter: ArrowFunctionConverter::new(options) }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Common<'a> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Module imports: insert import/require statements.
        // Drain imports from store into a local Vec first, then build statements.
        // This avoids split-borrow issues with `ctx.state`.
        let source_type = ctx.state.source_type;
        let imports: Vec<_> = ctx.state.module_imports.imports.drain(..).collect();
        if !imports.is_empty() {
            if source_type.is_module() {
                let stmts: Vec<_> = imports
                    .into_iter()
                    .map(|(source, names)| ModuleImportsStore::get_import(source, names, ctx))
                    .collect();
                ctx.state.top_level_statements.insert_statements(stmts);
            } else {
                let require_symbol_id = ctx.scoping().get_root_binding(ctx.ast.ident("require"));
                let stmts: Vec<_> = imports
                    .into_iter()
                    .map(|(source, names)| {
                        ModuleImportsStore::get_require(source, names, require_symbol_id, ctx)
                    })
                    .collect();
                ctx.state.top_level_statements.insert_statements(stmts);
            }
        }

        // Var declarations: insert var/let statements at program level.
        // Pop from the stack into a local, then build statements.
        {
            let var_stmt = ctx.state.var_declarations.get_var_statement(ctx.ast);
            if let Some((var_statement, let_statement)) = var_stmt {
                let stmts: Vec<Statement<'a>> =
                    var_statement.into_iter().chain(let_statement).collect();
                ctx.state.top_level_statements.insert_statements(stmts);
            }

            // Check stack is exhausted
            ctx.state.var_declarations.assert_stack_exhausted();
        }

        // Top level statements
        ctx.state.top_level_statements.insert_into_program(program);
        self.arrow_function_converter.exit_program(program, ctx);
        ctx.state.statement_injector.assert_no_insertions_remaining();
    }

    fn enter_statements(
        &mut self,
        _stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        ctx.state.var_declarations.record_entering_statements();
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let is_program_body = matches!(ctx.parent(), Ancestor::ProgramBody(_));
        ctx.state.var_declarations.insert_into_statements(stmts, is_program_body, ctx.ast);
        ctx.state.statement_injector.insert_into_statements(stmts, ctx.ast);
    }

    fn enter_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.enter_function(func, ctx);
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.exit_function(func, ctx);
    }

    fn enter_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.enter_arrow_function_expression(arrow, ctx);
    }

    fn exit_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.exit_arrow_function_expression(arrow, ctx);
    }

    fn enter_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.enter_function_body(body, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.exit_function_body(body, ctx);
    }

    fn enter_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.enter_static_block(block, ctx);
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.exit_static_block(block, ctx);
    }

    fn enter_jsx_element_name(
        &mut self,
        element_name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.enter_jsx_element_name(element_name, ctx);
    }

    fn enter_jsx_member_expression_object(
        &mut self,
        object: &mut JSXMemberExpressionObject<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.enter_jsx_member_expression_object(object, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.enter_expression(expr, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.arrow_function_converter.exit_expression(expr, ctx);
    }

    fn enter_binding_identifier(
        &mut self,
        node: &mut BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.enter_binding_identifier(node, ctx);
    }

    fn enter_identifier_reference(
        &mut self,
        node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.arrow_function_converter.enter_identifier_reference(node, ctx);
    }
}
