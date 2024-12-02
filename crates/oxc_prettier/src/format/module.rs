use std::collections::VecDeque;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    group,
    ir::{array, if_break, indent, line, softline, space, text, Doc, DocBuilder, Separator},
    p_vec, Format, Prettier,
};

pub(super) fn print_export_declaration<'a>(
    p: &mut Prettier<'a>,
    decl: &ModuleDeclaration<'a>,
) -> Doc<'a> {
    debug_assert!(decl.is_export());

    let mut parts = p.vec();
    parts.push(text("export"));

    if decl.is_default_export() {
        parts.push(text(" default "));
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
        parts.push(text(" from "));
        parts.push(source.format(p));
    }

    if let Some(with_clause) = decl.with_clause() {
        parts.push(space());
        parts.push(with_clause.format(p));
    }

    if let Some(doc) = print_semicolon_after_export_declaration(p, decl) {
        parts.push(doc);
    }

    array(parts)
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
            match_expression!(ExportDefaultDeclarationKind) => Some(text(";")),
            _ => None,
        },
        ModuleDeclaration::ExportNamedDeclaration(decl) => {
            let Some(declaration) = &decl.declaration else {
                return Some(text(";"));
            };

            match declaration {
                Declaration::TSInterfaceDeclaration(_)
                | Declaration::VariableDeclaration(_)
                | Declaration::ClassDeclaration(_)
                | Declaration::TSModuleDeclaration(_) => None,
                _ => Some(text(";")),
            }
        }
        ModuleDeclaration::ExportAllDeclaration(_) | ModuleDeclaration::TSExportAssignment(_) => {
            Some(text(";"))
        }
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
        parts.push(text(" {}"));
    } else {
        parts.push(space());

        let mut specifiers_iter: VecDeque<_> = specifiers.iter().collect();
        if include_default {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(text(", "));
            }
        }

        if include_namespace {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(text(", "));
            }
        }

        if !specifiers_iter.is_empty() {
            let can_break = specifiers.len() > 1;

            if can_break {
                let docs =
                    specifiers_iter.iter().map(|s| s.format(p)).collect::<std::vec::Vec<_>>();
                parts.push(group![
                    p,
                    text("{"),
                    indent(p_vec!(
                        p,
                        if p.options.bracket_spacing { line() } else { softline() },
                        array(p.join(Separator::CommaLine, docs))
                    )),
                    if_break(
                        p.boxed(text(if p.should_print_es5_comma() { "," } else { "" })),
                        p.boxed(text("",)),
                        None,
                    ),
                    if p.options.bracket_spacing { line() } else { softline() },
                    text("}"),
                ]);
            } else {
                parts.push(text("{"));
                if p.options.bracket_spacing {
                    parts.push(space());
                }
                parts.extend(specifiers_iter.iter().map(|s| s.format(p)));
                if p.options.bracket_spacing {
                    parts.push(space());
                }
                parts.push(text("}"));
            }
        }
    }

    array(parts)
}
