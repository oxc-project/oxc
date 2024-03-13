use oxc_allocator::Vec;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, ModuleDeclaration, ModuleExportName, Program, Statement};
use oxc_span::GetSpan;

/// Table imports context
///
/// Example:
/// ```js
/// // typ = true
/// // gap = 9   (.:.:.:.:.)
/// import      { useEffect } from 'react'
/// import type { FC }        from 'react'
/// ```
#[derive(Debug, Clone, Default)]
pub struct TableImports {
	/// At least one import is type import
	pub typ: bool,
	/// Alignment gap length
	///
	/// Gap is the length of the longest symbol.
	/// The number of spaces for alignment also
	/// depends on the type of specifier and
	/// the preference for spaces around curly braces.
	pub gap: usize,
}

impl TableImports {
	pub fn analyze(program: &Program) -> Self {
		let mut ti = TableImports::default();

		for import in imports_iter(program) {
			if let ImportOrExportKind::Type = &import.import_kind {
				ti.typ = true;
			}

			if let Some(specifiers) = &import.specifiers {
				for specifier in specifiers {
					if let ImportDeclarationSpecifier::ImportSpecifier(import) = specifier {
						if let ImportOrExportKind::Type = &import.import_kind {
							ti.typ = true;
						}
					}
					let sl = specifier_length(specifier);
					if sl > ti.gap {
						ti.gap = sl;
					}
				}
			}
		}

		ti
	}
}

fn imports_iter<'a, 'ast>(program: &'a Program<'ast>) -> impl Iterator<Item = &'a ImportDeclaration<'ast>> {
	program.body.iter().filter_map(|it| {
		if let Statement::ModuleDeclaration(module) = it {
			if let ModuleDeclaration::ImportDeclaration(import) = &**module {
				Some(&**import)
			} else {
				None
			}
		} else {
			None
		}
	})
}

pub fn specifiers_length(specifiers: &Vec<ImportDeclarationSpecifier>) -> usize {
	specifiers.iter().fold(0, |acc, b| acc + specifier_length(b))
}

pub fn specifier_length(specifier: &ImportDeclarationSpecifier) -> usize {
	match specifier {
		ImportDeclarationSpecifier::ImportSpecifier(it) => {
			if it.imported.span() == it.local.span {
				it.imported.name().len() + 2
			} else {
				match &it.imported {
					ModuleExportName::Identifier(id) => {
						id.name.len() + it.local.name.len() + 2 + 2 + 2  /* `as` keyword length + 2 spaces between + 2 braces */
					}
					ModuleExportName::StringLiteral(lit) => {
						lit.value.len() + it.local.name.len() + 2 /* quotes around */
					}
				}
			}
		}
		ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => {
			it.local.name.len()
		}
		ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => {
			it.local.name.len() + 2 + 2 + 1 /* `as` keyword length + spaces around + asterisk */
		}
	}
}
