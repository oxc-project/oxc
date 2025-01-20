use std::collections::VecDeque;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    array, group, if_break, indent,
    ir::{Doc, JoinSeparator},
    join, line, softline, text, Format, Prettier,
};

pub fn print_import_declaration<'a>(p: &mut Prettier<'a>, decl: &ImportDeclaration<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!("import"));

    if decl.import_kind.is_type() {
        parts.push(text!(" type"));
    }

    // TODO: Move these into `print_module_specifiers`...?
    if let Some(specifiers) = &decl.specifiers {
        let validate_namespace = |x: &ImportDeclarationSpecifier| {
            matches!(x, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
        };

        let is_default = specifiers
            .first()
            .is_some_and(|x| matches!(x, ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
        let is_namespace = specifiers.first().is_some_and(validate_namespace)
            || specifiers.get(1).is_some_and(validate_namespace);

        parts.push(print_module_specifiers(p, specifiers, is_default, is_namespace));
        parts.push(text!(" from"));
    }

    parts.push(text!(" "));
    parts.push(decl.source.format(p));

    if let Some(with_clause) = &decl.with_clause {
        parts.push(print_import_attributes(p, with_clause));
    }

    if let Some(semi) = p.semi() {
        parts.push(semi);
    }

    array!(p, parts)
}

fn print_import_attributes<'a>(p: &mut Prettier<'a>, with_clause: &WithClause<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!(" "));
    parts.push(with_clause.attributes_keyword.format(p));
    parts.push(text!(" {"));

    if !with_clause.with_entries.is_empty() {
        if p.options.bracket_spacing {
            parts.push(text!(" "));
        }

        let attributes_doc = with_clause
            .with_entries
            .iter()
            .map(|import_attr| import_attr.format(p))
            .collect::<std::vec::Vec<_>>();
        parts.push(join!(p, JoinSeparator::CommaSpace, attributes_doc));

        if p.options.bracket_spacing {
            parts.push(text!(" "));
        }
    }

    parts.push(text!("}"));

    array!(p, parts)
}

pub fn print_export_declaration<'a>(p: &mut Prettier<'a>, decl: &ModuleDeclaration<'a>) -> Doc<'a> {
    debug_assert!(decl.is_export());

    let mut parts = Vec::new_in(p.allocator);
    parts.push(text!("export"));

    if decl.is_default_export() {
        parts.push(text!(" default "));
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
        parts.push(text!(" from "));
        parts.push(source.format(p));
    }

    if let Some(with_clause) = decl.with_clause() {
        parts.push(print_import_attributes(p, with_clause));
    }

    if let Some(doc) = print_semicolon_after_export_declaration(p, decl) {
        parts.push(doc);
    }

    array!(p, parts)
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
            match_expression!(ExportDefaultDeclarationKind) => Some(text!(";")),
            _ => None,
        },
        ModuleDeclaration::ExportNamedDeclaration(decl) => {
            let Some(declaration) = &decl.declaration else {
                return Some(text!(";"));
            };

            match declaration {
                Declaration::TSInterfaceDeclaration(_)
                | Declaration::VariableDeclaration(_)
                | Declaration::ClassDeclaration(_)
                | Declaration::TSModuleDeclaration(_) => None,
                _ => Some(text!(";")),
            }
        }
        ModuleDeclaration::ExportAllDeclaration(_) | ModuleDeclaration::TSExportAssignment(_) => {
            Some(text!(";"))
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
    let mut parts = Vec::new_in(p.allocator);
    if specifiers.is_empty() {
        parts.push(text!(" {}"));
    } else {
        parts.push(text!(" "));

        let mut specifiers_iter: VecDeque<_> = specifiers.iter().collect();
        if include_default {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(text!(", "));
            }
        }

        if include_namespace {
            parts.push(specifiers_iter.pop_front().unwrap().format(p));
            if !specifiers_iter.is_empty() {
                parts.push(text!(", "));
            }
        }

        if !specifiers_iter.is_empty() {
            let can_break = specifiers.len() > 1;

            if can_break {
                let docs =
                    specifiers_iter.iter().map(|s| s.format(p)).collect::<std::vec::Vec<_>>();
                parts.push(group!(
                    p,
                    [
                        text!("{"),
                        indent!(
                            p,
                            [
                                if p.options.bracket_spacing { line!() } else { softline!() },
                                join!(p, JoinSeparator::CommaLine, docs)
                            ]
                        ),
                        if_break!(p, text!(if p.should_print_es5_comma() { "," } else { "" })),
                        if p.options.bracket_spacing { line!() } else { softline!() },
                        text!("}"),
                    ]
                ));
            } else {
                parts.push(text!("{"));
                if p.options.bracket_spacing {
                    parts.push(text!(" "));
                }
                parts.extend(specifiers_iter.iter().map(|s| s.format(p)));
                if p.options.bracket_spacing {
                    parts.push(text!(" "));
                }
                parts.push(text!("}"));
            }
        }
    }

    array!(p, parts)
}
