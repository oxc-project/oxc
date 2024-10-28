//! Utility transforms which are in common between other transforms.

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

pub mod helper_loader;
pub mod module_imports;
pub mod statement_injector;
pub mod top_level_statements;
pub mod var_declarations;

use module_imports::ModuleImports;
use statement_injector::StatementInjector;
use top_level_statements::TopLevelStatements;
use var_declarations::VarDeclarations;

pub struct Common<'a, 'ctx> {
    module_imports: ModuleImports<'a, 'ctx>,
    var_declarations: VarDeclarations<'a, 'ctx>,
    statement_injector: StatementInjector<'a, 'ctx>,
    top_level_statements: TopLevelStatements<'a, 'ctx>,
}

impl<'a, 'ctx> Common<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            module_imports: ModuleImports::new(ctx),
            var_declarations: VarDeclarations::new(ctx),
            statement_injector: StatementInjector::new(ctx),
            top_level_statements: TopLevelStatements::new(ctx),
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for Common<'a, 'ctx> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.module_imports.exit_program(program, ctx);
        self.var_declarations.exit_program(program, ctx);
        self.top_level_statements.exit_program(program, ctx);
    }

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.enter_statements(stmts, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.exit_statements(stmts, ctx);
        self.statement_injector.exit_statements(stmts, ctx);
    }
}
