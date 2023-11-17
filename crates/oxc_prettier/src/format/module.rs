#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_export_declaration<'a>(
    p: &mut Prettier<'a>,
    decl: &ModuleDeclaration<'a>,
) -> Doc<'a> {
    debug_assert!(decl.is_export());

    let mut parts = p.vec();
    parts.push(ss!("export"));

    if decl.is_default_export() {
        parts.push(ss!(" default "));
    }

    parts.push(match decl {
        ModuleDeclaration::ImportDeclaration(decl) => unreachable!(),
        ModuleDeclaration::ExportAllDeclaration(decl) => decl.format(p),
        ModuleDeclaration::ExportDefaultDeclaration(decl) => decl.format(p),
        ModuleDeclaration::ExportNamedDeclaration(decl) => decl.format(p),
        ModuleDeclaration::TSExportAssignment(decl) => decl.format(p),
        ModuleDeclaration::TSNamespaceExportDeclaration(decl) => decl.format(p),
    });

    if let Some(doc) = print_semicolon_after_export_declaration(p, decl) {
        parts.push(doc);
    }

    Doc::Array(parts)
}

fn print_semicolon_after_export_declaration<'a>(
    p: &Prettier<'a>,
    decl: &ModuleDeclaration<'a>,
) -> Option<Doc<'a>> {
    if !p.options.semi {
        return None;
    }

    let ModuleDeclaration::ExportDefaultDeclaration(decl) = decl else { return None };

    match decl.declaration {
        ExportDefaultDeclarationKind::Expression(_) => Some(ss!(";")),
        ExportDefaultDeclarationKind::FunctionDeclaration(_)
        | ExportDefaultDeclarationKind::ClassDeclaration(_)
        | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
        | ExportDefaultDeclarationKind::TSEnumDeclaration(_) => None,
    }
}
