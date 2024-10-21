use oxc_allocator::{Address, GetAddress};

use crate::ast::Statement;

impl<'a> GetAddress for Statement<'a> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Statement::BlockStatement(s) => s.address(),
            Statement::BreakStatement(s) => s.address(),
            Statement::ContinueStatement(s) => s.address(),
            Statement::DebuggerStatement(s) => s.address(),
            Statement::DoWhileStatement(s) => s.address(),
            Statement::EmptyStatement(s) => s.address(),
            Statement::ExpressionStatement(s) => s.address(),
            Statement::ForInStatement(s) => s.address(),
            Statement::ForOfStatement(s) => s.address(),
            Statement::ForStatement(s) => s.address(),
            Statement::IfStatement(s) => s.address(),
            Statement::LabeledStatement(s) => s.address(),
            Statement::ReturnStatement(s) => s.address(),
            Statement::SwitchStatement(s) => s.address(),
            Statement::ThrowStatement(s) => s.address(),
            Statement::TryStatement(s) => s.address(),
            Statement::WhileStatement(s) => s.address(),
            Statement::WithStatement(s) => s.address(),
            Statement::VariableDeclaration(s) => s.address(),
            Statement::FunctionDeclaration(s) => s.address(),
            Statement::ClassDeclaration(s) => s.address(),
            Statement::TSTypeAliasDeclaration(s) => s.address(),
            Statement::TSInterfaceDeclaration(s) => s.address(),
            Statement::TSEnumDeclaration(s) => s.address(),
            Statement::TSModuleDeclaration(s) => s.address(),
            Statement::TSImportEqualsDeclaration(s) => s.address(),
            Statement::ImportDeclaration(s) => s.address(),
            Statement::ExportAllDeclaration(s) => s.address(),
            Statement::ExportDefaultDeclaration(s) => s.address(),
            Statement::ExportNamedDeclaration(s) => s.address(),
            Statement::TSExportAssignment(s) => s.address(),
            Statement::TSNamespaceExportDeclaration(s) => s.address(),
        }
    }
}
