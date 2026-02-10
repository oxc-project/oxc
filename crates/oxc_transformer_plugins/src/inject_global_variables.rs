use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;

use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_ecmascript::BoundNames;
use oxc_semantic::ScopeFlags;
use oxc_span::{CompactStr, SPAN, format_compact_str};
use oxc_syntax::identifier;

use super::{
    DotDefineMemberExpression,
    replace_global_defines::{DotDefine, ReplaceGlobalDefines, ScopeBindingTracker},
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

    /// Lightweight scope binding tracker.
    scope_tracker: ScopeBindingTracker<'a>,

    /// Set of global identifier names that are referenced (used but not bound).
    /// Populated during the VisitMut walk.
    unresolved_references: rustc_hash::FxHashSet<&'a str>,

    /// Depth of non-arrow functions we're inside of. Used to compute scope flags for `this`
    /// replacement.
    non_arrow_function_depth: u32,
}

impl<'a> VisitMut<'a> for InjectGlobalVariables<'a> {
    fn enter_scope(
        &mut self,
        flags: ScopeFlags,
        _scope_id: &std::cell::Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        let is_function = flags.contains(ScopeFlags::Function);
        self.scope_tracker.enter_scope(is_function);
    }

    fn leave_scope(&mut self) {
        let current_idx = self.scope_tracker.scopes.len() - 1;
        let is_function =
            self.scope_tracker.function_scope_indices.last().is_some_and(|&i| i == current_idx);
        self.scope_tracker.leave_scope(is_function);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.replace_dot_defines(expr);
        walk_mut::walk_expression(self, expr);
    }

    fn visit_function(&mut self, func: &mut Function<'a>, flags: ScopeFlags) {
        if let Some(id) = &func.id {
            self.scope_tracker.add_binding(id.name.as_str());
        }
        self.non_arrow_function_depth += 1;
        walk_mut::walk_function(self, func, flags);
        self.non_arrow_function_depth -= 1;
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        if let Some(id) = &class.id {
            self.scope_tracker.add_binding(id.name.as_str());
        }
        walk_mut::walk_class(self, class);
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        param.pattern.bound_names(&mut |ident| {
            self.scope_tracker.add_binding(ident.name.as_str());
        });
        walk_mut::walk_formal_parameter(self, param);
    }

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        let is_var = decl.kind.is_var();
        decl.bound_names(&mut |ident| {
            let name = ident.name.as_str();
            if is_var {
                self.scope_tracker.add_var_binding(name);
            } else {
                self.scope_tracker.add_binding(name);
            }
        });
        walk_mut::walk_variable_declaration(self, decl);
    }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        // Track unresolved global references.
        let name = ident.name.as_str();
        if self.scope_tracker.is_global(name) {
            self.unresolved_references.insert(name);
        }
        walk_mut::walk_identifier_reference(self, ident);
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        decl.bound_names(&mut |ident| {
            self.scope_tracker.add_binding(ident.name.as_str());
        });
        walk_mut::walk_import_declaration(self, decl);
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
            scope_tracker: ScopeBindingTracker::new(),
            unresolved_references: rustc_hash::FxHashSet::default(),
            non_arrow_function_depth: 0,
        }
    }

    fn mark_as_changed(&mut self) {
        self.changed = true;
    }

    /// Compute the current scope flags based on function depth tracking.
    fn current_scope_flags(&self) -> ScopeFlags {
        if self.non_arrow_function_depth > 0 { ScopeFlags::Function } else { ScopeFlags::Top }
    }

    pub fn build(&mut self, program: &mut Program<'a>) -> InjectGlobalVariablesReturn {
        // Step 1: Walk the program to replace dot defines (if any) and collect scope bindings.
        let dot_defines = self
            .config
            .injects
            .iter()
            .filter(|i| i.replace_value.is_some())
            .map(DotDefineState::from)
            .collect::<Vec<_>>();

        if !dot_defines.is_empty() {
            self.dot_defines = dot_defines;
        }

        // Always walk the program to collect scope bindings for step 2
        // (and replace dot defines if any exist).
        self.visit_program(program);

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
                            // Check if the identifier was used as an unresolved global reference.
                            self.unresolved_references.contains(i.specifier.local().as_str())
                        }
                    }
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        if injects.is_empty() {
            return InjectGlobalVariablesReturn { changed: self.changed };
        }

        self.inject_imports(&injects, program);

        InjectGlobalVariablesReturn { changed: self.changed }
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

    fn replace_dot_defines(&mut self, expr: &mut Expression<'a>) {
        let scope_tracker = &self.scope_tracker;
        let scope_flags = self.current_scope_flags();
        match expr {
            Expression::StaticMemberExpression(member) => {
                for DotDefineState { dot_define, value_atom } in &mut self.dot_defines {
                    if ReplaceGlobalDefines::is_dot_define(
                        scope_tracker,
                        scope_flags,
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
