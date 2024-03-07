use std::collections::VecDeque;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    doc::{Doc, DocBuilder, Separator},
    group, if_break, indent, line, softline, space, ss, Format, Prettier,
};

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

    if let Some(source) = decl.source() {
        parts.push(ss!(" from "));
        parts.push(source.format(p));
    }

    if let Some(with_clause) = decl.with_clause() {
        parts.push(space!());
        parts.push(with_clause.format(p));
    }

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

    match decl {
        ModuleDeclaration::ExportDefaultDeclaration(decl) => match decl.declaration {
            ExportDefaultDeclarationKind::Expression(_) => Some(ss!(";")),
            ExportDefaultDeclarationKind::FunctionDeclaration(_)
            | ExportDefaultDeclarationKind::ClassDeclaration(_)
            | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
            | ExportDefaultDeclarationKind::TSEnumDeclaration(_) => None,
        },
        ModuleDeclaration::ExportAllDeclaration(_)
        | ModuleDeclaration::ExportNamedDeclaration(_)
        | ModuleDeclaration::TSExportAssignment(_) => Some(ss!(";")),
        _ => None,
    }
}

pub fn print_module_specifiers<'a, T: Format<'a>>(
    p: &mut Prettier<'a>,
    specifiers: &Vec<'a, T>,
    include_default: bool,
    include_namespace: bool,
) -> Doc<'a> {
    let mut parts = p.vec();
    if specifiers.is_empty() {
        parts.push(ss!(" {}"));
    } else {
        parts.push(space!());

        let mut specifiers_iter: VecDeque<_> = specifiers.iter().collect();
        if include_default {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(p.str(", "));
            }
        }

        if include_namespace {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(p.str(", "));
            }
        }

        if !specifiers_iter.is_empty() {
            let can_break = specifiers.len() > 1;

            if can_break {
                let docs =
                    specifiers_iter.iter().map(|s| s.format(p)).collect::<std::vec::Vec<_>>();
                parts.push(group![
                    p,
                    ss!("{"),
                    indent![
                        p,
                        if p.options.bracket_spacing { line!() } else { softline!() },
                        Doc::Array(p.join(Separator::CommaLine, docs))
                    ],
                    if_break!(p, if p.should_print_es5_comma() { "," } else { "" }, "", None),
                    if p.options.bracket_spacing { line!() } else { softline!() },
                    ss!("}"),
                ]);
            } else {
                parts.push(ss!("{"));
                if p.options.bracket_spacing {
                    parts.push(space!());
                }
                parts.extend(specifiers_iter.iter().map(|s| s.format(p)));
                if p.options.bracket_spacing {
                    parts.push(space!());
                }
                parts.push(ss!("}"));
            }
        }
    }

    Doc::Array(parts)
}
