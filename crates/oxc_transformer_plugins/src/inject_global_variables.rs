use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;

use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_semantic::Scoping;
use oxc_span::{CompactStr, SPAN, format_compact_str};
use oxc_syntax::identifier;
use oxc_traverse::{Traverse, traverse_mut};

use super::{
    DotDefineMemberExpression, TraverseCtx,
    replace_global_defines::{DotDefine, ReplaceGlobalDefines},
};

#[derive(Debug, Clone)]
pub struct InjectGlobalVariablesConfig {
    injects: Arc<[InjectImport]>,
}

impl InjectGlobalVariablesConfig {
    pub fn new(injects: Vec<InjectImport>) -> Self {
        Self { injects: Arc::from(injects) }
    }
}

#[derive(Debug, Clone)]
pub struct InjectImport {
    /// `import _ from `source`
    source: CompactStr,
    specifier: InjectImportSpecifier,
    /// value to be replaced for `specifier.local` if it's a `StaticMemberExpression` in the form of `foo.bar.baz`.
    replace_value: Option<CompactStr>,
}

impl InjectImport {
    pub fn named_specifier(source: &str, imported: Option<&str>, local: &str) -> InjectImport {
        InjectImport {
            source: CompactStr::from(source),
            specifier: InjectImportSpecifier::Specifier {
                imported: imported.map(CompactStr::from),
                local: CompactStr::from(local),
            },
            replace_value: Self::replace_name(local),
        }
    }

    pub fn namespace_specifier(source: &str, local: &str) -> InjectImport {
        InjectImport {
            source: CompactStr::from(source),
            specifier: InjectImportSpecifier::NamespaceSpecifier { local: CompactStr::from(local) },
            replace_value: Self::replace_name(local),
        }
    }

    pub fn default_specifier(source: &str, local: &str) -> InjectImport {
        InjectImport {
            source: CompactStr::from(source),
            specifier: InjectImportSpecifier::DefaultSpecifier { local: CompactStr::from(local) },
            replace_value: Self::replace_name(local),
        }
    }

    fn replace_name(local: &str) -> Option<CompactStr> {
        match local.cow_replace('.', "_") {
            Cow::Owned(local) => Some(format_compact_str!("$inject_{local}")),
            Cow::Borrowed(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InjectImportSpecifier {
    /// `import { local } from "source"`
    /// `import { default as local } from "source"` when `imported` is `None`
    Specifier { imported: Option<CompactStr>, local: CompactStr },
    /// import * as local from "source"
    NamespaceSpecifier { local: CompactStr },
    /// import local from "source"
    DefaultSpecifier { local: CompactStr },
}

impl InjectImportSpecifier {
    fn local(&self) -> &CompactStr {
        match self {
            Self::Specifier { local, .. }
            | Self::NamespaceSpecifier { local, .. }
            | Self::DefaultSpecifier { local, .. } => local,
        }
    }
}

/// Wrapper around `DotDefine` which caches the `Atom` to replace with.
/// `value_atom` is populated lazily when first replacement happens.
/// If no replacement is made, `value_atom` remains `None`.
struct DotDefineState<'a> {
    dot_define: DotDefine,
    value_atom: Option<Atom<'a>>,
}

impl From<&InjectImport> for DotDefineState<'_> {
    fn from(inject: &InjectImport) -> Self {
        let parts = inject.specifier.local().split('.').map(CompactStr::from).collect::<Vec<_>>();
        let value = inject.replace_value.clone().unwrap();
        let dot_define = DotDefine { parts, value };
        Self { dot_define, value_atom: None }
    }
}

#[must_use]
pub struct InjectGlobalVariablesReturn {
    pub scoping: Scoping,
    pub changed: bool,
}

/// Injects import statements for global variables.
///
/// References:
///
/// * <https://www.npmjs.com/package/@rollup/plugin-inject>
pub struct InjectGlobalVariables<'a> {
    ast: AstBuilder<'a>,
    config: InjectGlobalVariablesConfig,

    // states
    /// Dot defines derived from the config.
    dot_defines: Vec<DotDefineState<'a>>,

    /// Identifiers for which dot define replaced a member expression.
    replaced_dot_defines:
        Vec<(/* identifier of member expression */ CompactStr, /* local */ CompactStr)>,

    changed: bool,
}

impl<'a> Traverse<'a, ()> for InjectGlobalVariables<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.replace_dot_defines(expr, ctx);
    }
}

impl<'a> InjectGlobalVariables<'a> {
    pub fn new(allocator: &'a Allocator, config: InjectGlobalVariablesConfig) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            config,
            dot_defines: vec![],
            replaced_dot_defines: vec![],
            changed: false,
        }
    }

    fn mark_as_changed(&mut self) {
        self.changed = true;
    }

    pub fn build(
        &mut self,
        scoping: Scoping,
        program: &mut Program<'a>,
    ) -> InjectGlobalVariablesReturn {
        let mut scoping = scoping;
        // Step 1: slow path where visiting the AST is required to replace dot defines.
        let dot_defines = self
            .config
            .injects
            .iter()
            .filter(|i| i.replace_value.is_some())
            .map(DotDefineState::from)
            .collect::<Vec<_>>();

        if !dot_defines.is_empty() {
            self.dot_defines = dot_defines;
            scoping = traverse_mut(self, self.ast.allocator, program, scoping, ());
        }

        // Step 2: find all the injects that are referenced.
        let injects = self
            .config
            .injects
            .iter()
            .filter(|i| {
                // remove replaced `Buffer` for `Buffer` + Buffer.isBuffer` combo.
                match &i.replace_value {
                    Some(replace_value) => {
                        self.replaced_dot_defines.iter().any(|d| d.1 == replace_value)
                    }
                    _ => {
                        if self.replaced_dot_defines.iter().any(|d| d.0 == i.specifier.local()) {
                            false
                        } else {
                            scoping
                                .root_unresolved_references()
                                .contains_key(i.specifier.local().as_str())
                        }
                    }
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        if injects.is_empty() {
            return InjectGlobalVariablesReturn { scoping, changed: self.changed };
        }

        self.inject_imports(&injects, program);

        InjectGlobalVariablesReturn { scoping, changed: self.changed }
    }

    fn inject_imports(&mut self, injects: &[InjectImport], program: &mut Program<'a>) {
        let imports = injects.iter().map(|inject| {
            let specifiers = Some(self.ast.vec1(self.inject_import_to_specifier(inject)));
            let source = self.ast.string_literal(SPAN, self.ast.atom(&inject.source), None);
            let kind = ImportOrExportKind::Value;
            let import_decl = self
                .ast
                .module_declaration_import_declaration(SPAN, specifiers, source, None, NONE, kind);
            Statement::from(import_decl)
        });
        program.body.splice(0..0, imports);
        self.mark_as_changed();
    }

    fn inject_import_to_specifier(&self, inject: &InjectImport) -> ImportDeclarationSpecifier<'a> {
        match &inject.specifier {
            InjectImportSpecifier::Specifier { imported, local } => {
                let imported = match imported {
                    Some(imported_name) => {
                        let imported_name = self.ast.atom(imported_name);
                        if identifier::is_identifier_name(&imported_name) {
                            self.ast.module_export_name_identifier_name(SPAN, imported_name)
                        } else {
                            self.ast.module_export_name_string_literal(SPAN, imported_name, None)
                        }
                    }
                    None => self.ast.module_export_name_identifier_name(SPAN, "default"),
                };

                let local = inject.replace_value.as_ref().unwrap_or(local).as_str();

                self.ast.import_declaration_specifier_import_specifier(
                    SPAN,
                    imported,
                    self.ast.binding_identifier(SPAN, self.ast.atom(local)),
                    ImportOrExportKind::Value,
                )
            }
            InjectImportSpecifier::DefaultSpecifier { local } => {
                let local = inject.replace_value.as_ref().unwrap_or(local).as_str();
                let local = self.ast.binding_identifier(SPAN, self.ast.atom(local));
                self.ast.import_declaration_specifier_import_default_specifier(SPAN, local)
            }
            InjectImportSpecifier::NamespaceSpecifier { local } => {
                let local = inject.replace_value.as_ref().unwrap_or(local).as_str();
                let local = self.ast.binding_identifier(SPAN, self.ast.atom(local));
                self.ast.import_declaration_specifier_import_namespace_specifier(SPAN, local)
            }
        }
    }

    fn replace_dot_defines(&mut self, expr: &mut Expression<'a>, ctx: &TraverseCtx<'a>) {
        match expr {
            Expression::StaticMemberExpression(member) => {
                for DotDefineState { dot_define, value_atom } in &mut self.dot_defines {
                    if ReplaceGlobalDefines::is_dot_define(
                        ctx,
                        dot_define,
                        DotDefineMemberExpression::StaticMemberExpression(member),
                    ) {
                        // If this is first replacement made for this dot define,
                        // create `Atom` for replacement, and record in `replaced_dot_defines`
                        let value_atom = *value_atom.get_or_insert_with(|| {
                            self.replaced_dot_defines
                                .push((dot_define.parts[0].clone(), dot_define.value.clone()));
                            self.ast.atom(dot_define.value.as_str())
                        });

                        let value = self.ast.expression_identifier(SPAN, value_atom);
                        *expr = value;
                        self.mark_as_changed();
                        break;
                    }
                }
            }
            Expression::MetaProperty(meta_property) => {
                // Check if this is import.meta and if it should be replaced
                if meta_property.meta.name == "import" && meta_property.property.name == "meta" {
                    for DotDefineState { dot_define, value_atom } in &mut self.dot_defines {
                        // Check if dot_define is exactly ["import", "meta"]
                        if dot_define.parts.len() == 2
                            && dot_define.parts[0].as_str() == "import"
                            && dot_define.parts[1].as_str() == "meta"
                        {
                            // If this is first replacement made for this dot define,
                            // create `Atom` for replacement, and record in `replaced_dot_defines`
                            let value_atom = *value_atom.get_or_insert_with(|| {
                                self.replaced_dot_defines
                                    .push((dot_define.parts[0].clone(), dot_define.value.clone()));
                                self.ast.atom(dot_define.value.as_str())
                            });

                            let value = self.ast.expression_identifier(SPAN, value_atom);
                            *expr = value;
                            self.mark_as_changed();
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
