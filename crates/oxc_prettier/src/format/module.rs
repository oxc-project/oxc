#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_export_declaration(&mut self, decl: &ModuleDeclaration<'a>) -> Doc<'a> {
        debug_assert!(decl.is_export());

        let mut parts = self.vec();
        parts.push(ss!("export"));

        if decl.is_default_export() {
            parts.push(ss!(" default "));
        }

        parts.push(match decl {
            ModuleDeclaration::ImportDeclaration(decl) => unreachable!(),
            ModuleDeclaration::ExportAllDeclaration(decl) => decl.format(self),
            ModuleDeclaration::ExportDefaultDeclaration(decl) => decl.format(self),
            ModuleDeclaration::ExportNamedDeclaration(decl) => decl.format(self),
            ModuleDeclaration::TSExportAssignment(decl) => decl.format(self),
            ModuleDeclaration::TSNamespaceExportDeclaration(decl) => decl.format(self),
        });

        Doc::Array(parts)
    }
}
