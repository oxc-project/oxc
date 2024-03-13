use std::collections::VecDeque;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    doc::{Doc, DocBuilder, Separator},
    group, if_break, indent, line, softline, space, ss, Format, Prettier,
    analyzer::table_imports::{specifier_length, specifiers_length}
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

static SPACES: &str = "                                                                                                                                ";

fn print_import_kw<'a>(p: &mut Prettier<'a>, import_kind: ImportOrExportKind, specifier_kind: Option<ImportOrExportKind>) -> Doc<'a> {
		let mut parts = p.vec();
    parts.push(ss!("import"));
    if import_kind.is_type() || specifier_kind.is_some_and(|it| it.is_type()) {
        parts.push(ss!(" type"));
    } else if p.options.table_imports && p.ctx.ti.typ {
        parts.push(ss!("     "));
    }

		Doc::Array(parts)
}

fn is_braceless_specifier(specifier: &ImportDeclarationSpecifier) -> bool {
	matches!(specifier, ImportDeclarationSpecifier::ImportDefaultSpecifier(_) | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
}

fn print_rest_import_parts<'a>(
	p: &mut Prettier<'a>,
	source: &StringLiteral<'a>,
	with_clause: &Option<WithClause<'a>>,
) -> Doc<'a> {
	let mut parts = p.vec();
	parts.push(space!());
	parts.push(source.format(p));

	if let Some(with_clause) = &with_clause {
		parts.push(space!());
		parts.push(with_clause.format(p));
	}

	if let Some(semi) = p.semi() {
		parts.push(semi);
	}

	Doc::Array(parts)
}

pub fn print_import_declaration_with_single_specifier<'a>(
    p: &mut Prettier<'a>,
    import_kind: ImportOrExportKind,
    specifier: &ImportDeclarationSpecifier<'a>,
    source: &StringLiteral<'a>,
    with_clause: &Option<WithClause<'a>>,
) -> Doc<'a> {
	let mut parts = p.vec();
	let specifier_kind = if let ImportDeclarationSpecifier::ImportSpecifier(it) = specifier {
		Some(it.import_kind)
	} else {
		None
	};
	parts.push(print_import_kw(p, import_kind, specifier_kind));
	parts.push(space!());

	let braceless = is_braceless_specifier(specifier);

	if !braceless {
		parts.push(ss!("{"));
		if p.options.bracket_spacing {
			parts.push(space!());
		}
	}
	parts.push(specifier.format(p));
	if !braceless {
		if p.options.bracket_spacing {
			parts.push(space!());
		}
		parts.push(ss!("}"));
	}

	if p.options.table_imports {
		let mut gap = p.ctx.ti.gap - specifier_length(specifier);

		if is_braceless_specifier(specifier) && p.options.bracket_spacing {
			gap += 2;
		}

		parts.push(ss!(&SPACES[..gap]));
	}

	parts.push(ss!(" from"));
	parts.push(print_rest_import_parts(p, source, with_clause));

	Doc::Array(parts)
}

pub fn print_import_declaration<'a>(p: &mut Prettier<'a>, decl: &ImportDeclaration<'a>) -> Doc<'a> {
    let mut parts = p.vec();
		parts.push(print_import_kw(p, decl.import_kind, None));
    if let Some(specifiers) = &decl.specifiers {
        let is_default = specifiers.first().is_some_and(|x| {
            matches!(x, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
        });

        let validate_namespace = |x: &ImportDeclarationSpecifier| {
            matches!(x, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
        };

        let is_namespace = specifiers.first().is_some_and(validate_namespace)
          || specifiers.get(1).is_some_and(validate_namespace);

        parts.push(print_module_specifiers(p, specifiers, is_default, is_namespace));
        if p.options.table_imports {
            let mut gap = p.ctx.ti.gap.saturating_sub(specifiers_length(specifiers));

            for (idx, specifier) in specifiers.iter().enumerate() {
	            if is_braceless_specifier(specifier) && p.options.bracket_spacing {
		            gap += 2;
	            }
            }

            parts.push(ss!(&SPACES[..gap]));
        }
        parts.push(ss!(" from"));
    } else if p.options.table_imports {
      parts.push(ss!(&SPACES[..p.ctx.ti.gap + 4 + 4  /* `from` keyword length + spaces */]));
    }

    parts.push(print_rest_import_parts(p, &decl.source, &decl.with_clause));
    Doc::Array(parts)
}
