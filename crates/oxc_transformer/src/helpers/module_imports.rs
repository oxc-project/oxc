use indexmap::IndexMap;
use std::cell::RefCell;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{CompactStr, SPAN};

pub struct NamedImport {
    imported: CompactStr,
    local: Option<CompactStr>, // Not used in `require`
}

impl NamedImport {
    pub fn new(imported: CompactStr, local: Option<CompactStr>) -> NamedImport {
        Self { imported, local }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum ImportKind {
    Import,
    Require,
}

#[derive(Hash, Eq, PartialEq)]
pub struct ImportType {
    kind: ImportKind,
    source: CompactStr,
}

impl ImportType {
    fn new(kind: ImportKind, source: CompactStr) -> Self {
        Self { kind, source }
    }
}

/// Manage import statement globally
/// <https://github.com/nicolo-ribaudo/babel/tree/main/packages/babel-helper-module-imports>
pub struct ModuleImports<'a> {
    ast: AstBuilder<'a>,

    imports: RefCell<IndexMap<ImportType, std::vec::Vec<NamedImport>>>,
}

impl<'a> ModuleImports<'a> {
    pub fn new(allocator: &'a Allocator) -> ModuleImports<'a> {
        let ast = AstBuilder::new(allocator);
        Self { ast, imports: RefCell::new(IndexMap::default()) }
    }

    /// Add `import { named_import } from 'source'`
    pub fn add_import(&self, source: CompactStr, import: NamedImport) {
        self.imports
            .borrow_mut()
            .entry(ImportType::new(ImportKind::Import, source))
            .or_default()
            .push(import);
    }

    /// Add `var named_import from 'source'`
    pub fn add_require(&self, source: CompactStr, import: NamedImport, front: bool) {
        let len = self.imports.borrow().len();
        self.imports
            .borrow_mut()
            .entry(ImportType::new(ImportKind::Require, source))
            .or_default()
            .push(import);
        if front {
            self.imports.borrow_mut().move_index(len, 0);
        }
    }

    pub fn get_import_statements(&self) -> Vec<'a, Statement<'a>> {
        self.ast.new_vec_from_iter(self.imports.borrow_mut().drain(..).map(
            |(import_type, names)| match import_type.kind {
                ImportKind::Import => self.get_named_import(&import_type.source, names),
                ImportKind::Require => self.get_require(&import_type.source, names),
            },
        ))
    }

    fn get_named_import(
        &self,
        source: &CompactStr,
        names: std::vec::Vec<NamedImport>,
    ) -> Statement<'a> {
        let specifiers = self.ast.new_vec_from_iter(names.into_iter().map(|name| {
            ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
                span: SPAN,
                imported: ModuleExportName::Identifier(IdentifierName::new(
                    SPAN,
                    self.ast.new_atom(name.imported.as_str()),
                )),
                local: BindingIdentifier::new(
                    SPAN,
                    self.ast.new_atom(name.local.unwrap_or(name.imported).as_str()),
                ),
                import_kind: ImportOrExportKind::Value,
            })
        }));
        let import_stmt = self.ast.import_declaration(
            SPAN,
            Some(specifiers),
            self.ast.string_literal(SPAN, source.as_str()),
            None,
            ImportOrExportKind::Value,
        );
        self.ast.module_declaration(ModuleDeclaration::ImportDeclaration(import_stmt))
    }

    fn get_require(&self, source: &CompactStr, names: std::vec::Vec<NamedImport>) -> Statement<'a> {
        let var_kind = VariableDeclarationKind::Var;
        let callee = {
            let ident = IdentifierReference::new(SPAN, "require".into());
            self.ast.identifier_reference_expression(ident)
        };
        let args = {
            let string = self.ast.string_literal(SPAN, source.as_str());
            let arg = Argument::Expression(self.ast.literal_string_expression(string));
            self.ast.new_vec_single(arg)
        };
        let name = names.into_iter().next().unwrap();
        let id = {
            let ident = BindingIdentifier::new(SPAN, self.ast.new_atom(&name.imported));
            self.ast.binding_pattern(self.ast.binding_pattern_identifier(ident), None, false)
        };
        let decl = {
            let init = self.ast.call_expression(SPAN, callee, args, false, None);
            let decl = self.ast.variable_declarator(SPAN, var_kind, id, Some(init), false);
            self.ast.new_vec_single(decl)
        };
        let var_decl = self.ast.variable_declaration(SPAN, var_kind, decl, Modifiers::empty());
        Statement::Declaration(Declaration::VariableDeclaration(var_decl))
    }
}
