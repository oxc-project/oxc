use std::fmt::Write;

use cow_utils::CowUtils;
use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, ExportNamedDeclaration, ImportDeclaration, ImportDeclarationSpecifier,
        ImportOrExportKind, Statement, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

fn prefer_export_from_diagnostic(import_span: Span, export_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer re-exporting directly from the source module.")
        .with_labels([import_span.label("Imported here."), export_span.label("Re-exported here.")])
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferExportFrom {
    /// When true, if an import is used in other places than just a re-export, all variables in the import declaration will be ignored.
    ignore_used_variables: bool,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// When re-exporting from a module, it's unnecessary to import and then export.
    /// It can be done in a single export…from declaration.
    /// This rule encourages using direct re-export syntax (export ... from) instead of importing and then exporting.
    /// It helps reduce boilerplate code and keeps the module scope clean by avoiding unnecessary local bindings.
    ///
    /// ### Why is this bad?
    ///
    /// Separating re-exports into import and export statements is discouraged because it
    /// unnecessarily pollutes the current module's scope and adds redundant boilerplate code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import defaultExport from './foo.js';
    /// export default defaultExport;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export { default } from './foo.js';
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { named } from './foo.js';
    /// export { named };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export { named } from './foo.js';
    /// ```
    PreferExportFrom,
    unicorn,
    style,
    suggestion,
    config = PreferExportFrom,
    version = "next",
);

// Define SpecifierSpec structure
#[derive(Debug)]
struct SpecifierSpec<'a> {
    specifier: &'a ImportDeclarationSpecifier<'a>,
    name: String,
    decl_type: bool,
}

/// Represents a violation where an import is unnecessarily imported and then re-exported
#[derive(Debug)]
struct Violation {
    /// The name to use in the export statement
    export_name: String,
    /// The AST node ID of the export declaration containing this violation
    export_node_id: NodeId,
    /// The AST node ID of the import specifier that is being re-exported
    import_specifier_id: NodeId,
    /// Whether this represents a namespace export (export * as name)
    is_namespace_export: bool,
    /// Whether this is a TypeScript type import/export
    is_typescript_type: bool,
    /// Whether the export needs to include source info (for mixed type/value exports)
    needs_source: bool,
    /// The span of the original export statement that should be replaced
    original_export_span: Option<Span>,
}

impl Rule for PreferExportFrom {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ImportDeclaration(import_decl) = node.kind() {
            if import_decl.specifiers.is_none() {
                return;
            }

            // Check for duplicate TypeAliases, and skip the check if one exists.
            if has_matching_type_alias(import_decl, ctx) {
                return;
            }

            let corresponding_export: Option<&ExportNamedDeclaration> =
                find_corresponding_export(ctx, import_decl);

            let symbol_to_specifier_specs = Self::get_symbol_to_specifier(import_decl);
            self.check_re_export(
                ctx,
                &symbol_to_specifier_specs,
                import_decl,
                corresponding_export,
            );
        }
    }
}

fn has_matching_type_alias<'a>(
    import_decl: &'a ImportDeclaration<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    if !matches!(import_decl.import_kind, ImportOrExportKind::Value) {
        return false;
    }

    let Some(specifiers) = &import_decl.specifiers else { return false };

    // collect import specifier identifiers
    let import_names: FxHashSet<&str> = specifiers
        .iter()
        .map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(import_spec) => {
                import_spec.local.name.as_str()
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default_spec) => {
                default_spec.local.name.as_str()
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_spec) => {
                namespace_spec.local.name.as_str()
            }
        })
        .collect();

    ctx.semantic()
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind(), AstKind::TSTypeAliasDeclaration(_)))
        .any(|node| {
            if let AstKind::TSTypeAliasDeclaration(decl) = node.kind() {
                import_names.contains(decl.id.name.as_str())
            } else {
                false
            }
        })
}

fn find_corresponding_export<'a>(
    ctx: &LintContext<'a>,
    import_decl: &'a ImportDeclaration<'a>,
) -> Option<&'a ExportNamedDeclaration<'a>> {
    ctx.semantic().nodes().iter().find_map(|n| {
        if let AstKind::ExportNamedDeclaration(export_decl) = n.kind()
            && let Some(ref src) = export_decl.source
            && src.value == import_decl.source.value
        {
            if import_decl.import_kind == export_decl.export_kind {
                return Some(export_decl);
            } else if matches!(import_decl.import_kind, ImportOrExportKind::Type) {
                let specifiers = &export_decl.specifiers;
                let is_all_type = specifiers
                    .iter()
                    .all(|specifier| matches!(specifier.export_kind, ImportOrExportKind::Type));
                if is_all_type {
                    return Some(export_decl);
                }
            }
        }
        None
    })
}

// Helper function to check if a reference is not in an export statement
fn is_not_in_export_statement(parent_node: &AstNode) -> bool {
    !matches!(
        parent_node.kind(),
        AstKind::ExportSpecifier(_) | AstKind::ExportDefaultDeclaration(_)
    )
}

impl PreferExportFrom {
    fn check_re_export<'a>(
        &self,
        ctx: &LintContext<'a>,
        symbol_to_specifier: &FxIndexMap<SymbolId, SpecifierSpec<'a>>,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        re_export_decl: Option<&'a ExportNamedDeclaration<'a>>,
    ) {
        let import_node_id = import_decl.node_id();
        let (used_specifiers, grouped_violations) =
            self.analyze_import_usage(ctx, symbol_to_specifier, import_node_id, import_decl);

        let source = import_decl.source.value.as_str();
        let with_clause = if let Some(with_clause) = import_decl.with_clause.as_ref() {
            let assert_type: &str =
                if matches!(with_clause.keyword, oxc_ast::ast::WithClauseKeyword::Assert) {
                    "assert"
                } else {
                    "with"
                };
            let with_clause_str = with_clause
                .with_entries
                .iter()
                .map(|attribute| {
                    let key = match &attribute.key {
                        oxc_ast::ast::ImportAttributeKey::Identifier(ident_name) => {
                            ident_name.name.as_str()
                        }
                        oxc_ast::ast::ImportAttributeKey::StringLiteral(string_literal) => {
                            string_literal.value.as_str()
                        }
                    };
                    let value = &attribute.value.raw.unwrap();
                    format!("{key}: {value}")
                })
                .format(", ");

            Some(&format!("{assert_type} {{ {with_clause_str} }}"))
        } else {
            None
        };

        let replace_span = Self::get_replace_span(ctx, import_decl.node_id());

        let mut namespace_grouped_violations: FxHashMap<NodeId, Vec<Violation>> =
            FxHashMap::default();
        let mut regular_violations: FxHashMap<NodeId, Vec<Violation>> = FxHashMap::default();

        for (import_node_id, violations) in grouped_violations {
            let (namespace_violations, regular_violations_part): (Vec<Violation>, Vec<Violation>) =
                violations.into_iter().partition(|v| v.is_namespace_export);

            if !namespace_violations.is_empty() {
                namespace_grouped_violations.insert(import_node_id, namespace_violations);
            }

            if !regular_violations_part.is_empty() {
                regular_violations.insert(import_node_id, regular_violations_part);
            }
        }

        // Process namespace_grouped_violations according to template
        if !namespace_grouped_violations.is_empty() {
            for (_, violations) in namespace_grouped_violations {
                Self::process_violations(
                    ctx,
                    &violations,
                    &used_specifiers,
                    symbol_to_specifier,
                    import_decl,
                    re_export_decl,
                    source,
                    with_clause,
                    replace_span,
                    true,
                );
            }
        }

        // Process regular exports (keep original logic)
        for (_, violations) in regular_violations {
            if violations.is_empty() {
                continue;
            }

            Self::process_violations(
                ctx,
                &violations,
                &used_specifiers,
                symbol_to_specifier,
                import_decl,
                re_export_decl,
                source,
                with_clause,
                replace_span,
                false,
            );
        }
    }

    fn analyze_import_usage<'a>(
        &self,
        ctx: &LintContext<'a>,
        symbol_to_specifier: &FxIndexMap<SymbolId, SpecifierSpec<'a>>,
        import_node_id: NodeId,
        import_decl: &ImportDeclaration<'a>,
    ) -> (Vec<SymbolId>, FxHashMap<NodeId, Vec<Violation>>) {
        let mut used_specifiers: Vec<SymbolId> = Vec::new();
        let mut grouped_violations: FxHashMap<NodeId, Vec<Violation>> = FxHashMap::default();

        for (symbol_id, specifier_spec) in symbol_to_specifier {
            // Check if we should ignore this symbol because it's used outside export statements
            let mut references = ctx.symbol_references(*symbol_id);
            if self.ignore_used_variables {
                let has_non_export_usage = references.any(|reference| {
                    let ref_node = ctx.nodes().get_node(reference.node_id());
                    let parent_node = ctx.nodes().parent_node(ref_node.id());
                    is_not_in_export_statement(parent_node)
                });

                if has_non_export_usage {
                    break;
                }
            }

            // let references = ctx.symbol_references(*symbol_id);
            for reference in references {
                let ref_node = ctx.nodes().get_node(reference.node_id());
                let parent_node = ctx.nodes().parent_node(ref_node.id());

                let result = match parent_node.kind() {
                    AstKind::ExportDefaultDeclaration(_)
                        if matches!(ref_node.kind(), AstKind::IdentifierReference(_)) =>
                    {
                        Self::analyze_default_export_usage(
                            specifier_spec,
                            *symbol_id,
                            reference.node_id(),
                            parent_node.id(),
                            parent_node.span(),
                        )
                    }

                    AstKind::ExportSpecifier(export_specifier)
                        if matches!(ref_node.kind(), AstKind::IdentifierReference(_)) =>
                    {
                        Self::analyze_named_export_usage(
                            ctx,
                            specifier_spec,
                            *symbol_id,
                            reference.node_id(),
                            export_specifier,
                            import_decl,
                        )
                    }
                    AstKind::VariableDeclarator(var_decl) if var_decl.type_annotation.is_none() => {
                        Self::analyze_variable_declaration_usage(
                            ctx,
                            specifier_spec,
                            *symbol_id,
                            reference.node_id(),
                            var_decl,
                        )
                    }

                    _ => None,
                };

                if let Some((sym_id, violation)) = result {
                    if !used_specifiers.contains(&sym_id) {
                        used_specifiers.push(sym_id);
                    }

                    grouped_violations.entry(import_node_id).or_default().push(violation);
                }
            }
        }
        (used_specifiers, grouped_violations)
    }

    fn analyze_default_export_usage(
        specifier_spec: &SpecifierSpec<'_>,
        symbol_id: SymbolId,
        reference_node_id: NodeId,
        parent_node_id: NodeId,
        parent_span: Span,
    ) -> Option<(SymbolId, Violation)> {
        //  Skip if namespace（import * as namespace from 'foo');
        if matches!(
            specifier_spec.specifier,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
        ) {
            return None;
        }

        let mut target_name = Self::get_target_name_for_default_export(specifier_spec);

        if specifier_spec.decl_type
            && matches!(
                specifier_spec.specifier,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
            )
        {
            target_name = format!("type {target_name}");
        }

        let violation = Violation {
            export_name: target_name,
            export_node_id: parent_node_id,
            import_specifier_id: reference_node_id,
            is_namespace_export: false,
            is_typescript_type: false,
            needs_source: false,
            original_export_span: Some(parent_span),
        };

        Some((symbol_id, violation))
    }

    fn analyze_named_export_usage<'a>(
        ctx: &LintContext<'a>,
        specifier_spec: &SpecifierSpec<'a>,
        symbol_id: SymbolId,
        reference_node_id: NodeId,
        export_specifier: &oxc_ast::ast::ExportSpecifier,
        import_decl: &ImportDeclaration<'a>,
    ) -> Option<(SymbolId, Violation)> {
        let export_parent_decl = ctx.nodes().parent_node(export_specifier.node_id());
        if let AstKind::ExportNamedDeclaration(export_decl) = export_parent_decl.kind() {
            let is_export_default = Self::is_export_as_default(&export_decl.specifiers);
            let export_ts_kind = export_decl.export_kind;

            //Skip if namespaces and default export
            if matches!(
                specifier_spec.specifier,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
            ) && is_export_default
            {
                return None;
            }

            let export_name = Self::get_export_name(specifier_spec, export_specifier);

            if matches!(
                specifier_spec.specifier,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
            ) {
                let name = format!("* as {export_name}");

                let import_ts_kind = import_decl.import_kind;
                let ts_kind = import_ts_kind == ImportOrExportKind::Type
                    || export_ts_kind == ImportOrExportKind::Type;

                let violation = Violation {
                    export_name: name,
                    export_node_id: export_decl.node_id(),
                    import_specifier_id: reference_node_id,
                    is_namespace_export: true,
                    is_typescript_type: ts_kind,
                    needs_source: (import_ts_kind == ImportOrExportKind::Value
                        && export_ts_kind == ImportOrExportKind::Type),
                    original_export_span: Some(export_decl.span()),
                };

                return Some((symbol_id, violation));
            } else if matches!(
                specifier_spec.specifier,
                ImportDeclarationSpecifier::ImportSpecifier(_)
                    | ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
            ) {
                let ts_kind = if matches!(&import_decl.import_kind, ImportOrExportKind::Type) {
                    true
                } else if let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) =
                    specifier_spec.specifier
                {
                    import_specifier.import_kind == ImportOrExportKind::Type
                } else {
                    false
                };

                let needs_source = !ts_kind
                    && matches!(export_decl.export_kind, ImportOrExportKind::Type)
                    && export_decl.source.is_none();

                let violation = Violation {
                    export_name,
                    export_node_id: export_decl.node_id(),
                    import_specifier_id: reference_node_id,
                    is_namespace_export: false,
                    is_typescript_type: ts_kind,
                    needs_source,
                    original_export_span: Some(export_decl.span()),
                };

                return Some((symbol_id, violation));
            }
        }

        None
    }

    fn analyze_variable_declaration_usage<'a>(
        ctx: &LintContext<'a>,
        specifier_spec: &SpecifierSpec<'a>,
        symbol_id: SymbolId,
        reference_node_id: NodeId,
        var_decl: &oxc_ast::ast::VariableDeclarator,
    ) -> Option<(SymbolId, Violation)> {
        let next_parent_node = ctx.nodes().parent_node(var_decl.node_id());

        if let AstKind::VariableDeclaration(var_declaration) = next_parent_node.kind()
            && var_declaration.kind == VariableDeclarationKind::Const
            && let AstKind::ExportNamedDeclaration(export_named_decl) =
                ctx.nodes().parent_node(next_parent_node.id()).kind()
        {
            if Self::is_variable_used_elsewhere(ctx, var_decl) {
                return None;
            }

            let target_name =
                if let BindingPattern::BindingIdentifier(binding_identifier) = &var_decl.id {
                    binding_identifier.name.to_string()
                } else {
                    return None;
                };

            let full_target_name = format!(
                "{} as {}",
                Self::get_export_name_for_export_decl(specifier_spec),
                target_name
            );

            let is_namespace = full_target_name.starts_with('*');

            let violation = Violation {
                export_name: full_target_name,
                export_node_id: export_named_decl.node_id(),
                import_specifier_id: reference_node_id,
                is_namespace_export: is_namespace,
                is_typescript_type: false,
                needs_source: false,
                original_export_span: None,
            };

            return Some((symbol_id, violation));
        }

        None
    }

    //Check if the exported variable is subsequently used
    fn is_variable_used_elsewhere(
        ctx: &LintContext<'_>,
        var_decl: &oxc_ast::ast::VariableDeclarator,
    ) -> bool {
        let var_decl_id_symbol_id =
            if let BindingPattern::BindingIdentifier(binding_identifier) = &var_decl.id {
                Some(binding_identifier.symbol_id())
            } else {
                None
            };

        if let Some(id_symbol_id) = var_decl_id_symbol_id {
            ctx.symbol_references(id_symbol_id).next().is_some()
        } else {
            true
        }
    }

    fn get_target_name_for_default_export(specifier_spec: &SpecifierSpec<'_>) -> String {
        match specifier_spec.specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "default".to_string(),
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                let imported_name = match &import_specifier.imported {
                    oxc_ast::ast::ModuleExportName::IdentifierName(ident_name) => {
                        ident_name.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::IdentifierReference(ident_ref) => {
                        ident_ref.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::StringLiteral(literal) => &literal.raw.unwrap(),
                };
                let imported_local_name = import_specifier.local.name.as_str();
                let temp_name = specifier_spec.name.as_str();

                if imported_name == "default" && imported_local_name == temp_name {
                    "default".to_string()
                } else if imported_name.contains('\'') {
                    format!("{imported_name} as default")
                } else {
                    format!("{temp_name} as default")
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => specifier_spec.name.clone(),
        }
    }

    fn is_export_as_default(specifiers: &[oxc_ast::ast::ExportSpecifier]) -> bool {
        specifiers.iter().any(|specifier| {
            if let oxc_ast::ast::ModuleExportName::IdentifierName(ident_name) = &specifier.exported
            {
                ident_name.name.as_str() == "default"
            } else if let oxc_ast::ast::ModuleExportName::IdentifierReference(ident_ref) =
                &specifier.exported
            {
                ident_ref.name.as_str() == "default"
            } else {
                false
            }
        })
    }

    fn get_export_name_for_export_decl(specifier_spec: &SpecifierSpec<'_>) -> String {
        match specifier_spec.specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "default".to_string(),
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                let imported_name = match &import_specifier.imported {
                    oxc_ast::ast::ModuleExportName::IdentifierName(ident_name) => {
                        ident_name.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::IdentifierReference(ident_ref) => {
                        ident_ref.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::StringLiteral(literal) => &literal.raw.unwrap(),
                };

                imported_name.to_string()
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => "*".to_string(),
        }
    }

    fn get_export_name(
        specifier_spec: &SpecifierSpec<'_>,
        export_specifier: &oxc_ast::ast::ExportSpecifier,
    ) -> String {
        match specifier_spec.specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                let temp_export = export_specifier.exported.to_string();
                if temp_export == "default" {
                    "default".to_string()
                } else {
                    format!("default as {temp_export}")
                }
            }
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                let imported_name = match &import_specifier.imported {
                    oxc_ast::ast::ModuleExportName::IdentifierName(ident_name) => {
                        ident_name.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::IdentifierReference(ident_ref) => {
                        ident_ref.name.as_str()
                    }
                    oxc_ast::ast::ModuleExportName::StringLiteral(literal) => &literal.raw.unwrap(),
                };
                // let temp_export = export_specifier.exported.to_string();
                let temp_export = if let oxc_ast::ast::ModuleExportName::StringLiteral(literal) =
                    &export_specifier.exported
                {
                    literal.raw.as_ref().unwrap()
                } else {
                    export_specifier.exported.name().as_str()
                };

                if imported_name == "default" {
                    format!("default as {temp_export}")
                } else if temp_export == "default" {
                    format!("{imported_name} as default")
                } else if imported_name != temp_export {
                    if Self::is_strings_equal_std(imported_name, temp_export) {
                        if temp_export.contains('"') {
                            return String::from(temp_export);
                        }
                        return String::from(imported_name);
                    }
                    format!("{imported_name} as {temp_export}")
                } else {
                    String::from(temp_export)
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                export_specifier.exported.to_string()
            }
        }
    }

    fn is_strings_equal_std(s1: &str, s2: &str) -> bool {
        let clean = |s: &str| -> Option<String> {
            let has_quotes = (s.starts_with('\'') || s.starts_with('"'))
                && (s.ends_with('\'') || s.ends_with('"'));
            if !has_quotes {
                return None;
            }

            let inner = &s[1..s.len() - 1];

            let json_str = format!("\"{inner}\"");

            let decoded = serde_json::from_str::<String>(&json_str).ok()?;

            Some(decoded.chars().filter(|c| !c.is_whitespace()).collect::<String>())
        };
        match (clean(s1), clean(s2)) {
            (Some(cleaned_s1), Some(cleaned_s2)) => cleaned_s1 == cleaned_s2,
            _ => false,
        }
    }

    fn get_replace_span<'a>(ctx: &LintContext<'a>, target_node_id: NodeId) -> Span {
        let target_node = ctx.nodes().get_node(target_node_id);
        let import_parent_node = ctx.nodes().parent_node(target_node.id());
        let statements: &[Statement<'a>] = match import_parent_node.kind() {
            AstKind::Program(program) => &program.body,
            _ => &[],
        };

        let import_index =
            statements.iter().position(|statement| statement.span() == target_node.span());

        let delete_import_start = target_node.span().end;
        let delete_import_end = if let Some(index) = import_index {
            if index + 1 < statements.len() {
                statements[index + 1].span().start
            } else {
                target_node.span().end
            }
        } else {
            target_node.span().end
        };

        Span::new(delete_import_start, delete_import_end)
    }
    fn process_violations<'a>(
        ctx: &LintContext<'a>,
        violations: &Vec<Violation>,
        used_specifiers: &[SymbolId],
        symbol_to_specifier: &FxIndexMap<SymbolId, SpecifierSpec<'a>>,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        re_export_decl: Option<&'a ExportNamedDeclaration<'a>>,
        source: &str,
        with_clause: Option<&String>,
        replace_span: Span,
        is_namespace: bool,
    ) {
        // Generate the export format based on whether it's a namespace export
        let export_format =
            Self::generate_export_format(violations, source, with_clause, is_namespace);

        if export_format.is_empty() {
            return;
        }

        let parent_nodes: Vec<&AstNode> =
            violations.iter().map(|v| ctx.nodes().get_node(v.export_node_id)).collect();

        let replace_export_spans: Vec<Span> = parent_nodes
            .iter()
            .map(|parent_node| Self::get_replace_span(ctx, parent_node.id()))
            .collect();

        let sorted_nodes = parent_nodes
            .iter()
            .sorted_by(|a, b| a.span().start.cmp(&b.span().start))
            .collect::<Vec<_>>();

        let end = sorted_nodes.last().unwrap().span().end;
        let start = sorted_nodes.first().unwrap().span().start;
        let delete_span = Span::new(start, end);

        let re_export_source_text =
            re_export_decl.map(|decl| ctx.source_range(decl.span())).unwrap_or_default();

        for violation in violations {
            let specifier_node = ctx.nodes().get_node(violation.import_specifier_id);

            if violation.needs_source {
                ctx.diagnostic_with_suggestion(
                    prefer_export_from_diagnostic(import_decl.span(), specifier_node.span()),
                    |fixer| {
                        Self::add_fix_for_export(
                            fixer,
                            violation.original_export_span.unwrap(),
                            &export_format,
                        )
                    },
                );
            } else {
                let join_names = &violations
                    .iter()
                    .map(|v| {
                        if v.is_typescript_type {
                            format!("type {}", v.export_name)
                        } else {
                            v.export_name.clone()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                ctx.diagnostic_with_suggestion(
                    prefer_export_from_diagnostic(import_decl.span(), specifier_node.span()),
                    |fixer| {
                        Self::create_fix_for_re_export(
                            fixer,
                            used_specifiers,
                            symbol_to_specifier,
                            import_decl,
                            re_export_decl,
                            if is_namespace { "" } else { re_export_source_text },
                            replace_span,
                            &replace_export_spans,
                            delete_span,
                            if is_namespace { &violation.export_name } else { join_names },
                            &export_format,
                            source,
                            is_namespace,
                        )
                    },
                );
            }
        }
    }

    /// Generate the export format based on whether it's a namespace export
    fn generate_export_format(
        violations: &[Violation],
        source: &str,
        with_clause: Option<&String>,
        is_namespace: bool,
    ) -> String {
        // For namespace exports, create separate export statements for each violation
        if is_namespace {
            violations
                .iter()
                .map(|violation| {
                    Self::format_namespace_export(
                        &violation.export_name,
                        source,
                        violation.is_typescript_type,
                        with_clause,
                    )
                })
                .collect::<String>()
        } else {
            // For regular exports, combine them into a single export statement
            let export_names: Vec<String> = violations
                .iter()
                .map(|violation| {
                    if violation.is_typescript_type {
                        format!("type {}", violation.export_name)
                    } else {
                        violation.export_name.clone()
                    }
                })
                .collect();

            if export_names.is_empty() {
                return String::new();
            }

            Self::format_export_statement(&export_names, source, with_clause)
        }
    }
    /// Format a regular export statement with multiple exports
    fn format_export_statement(
        exports: &[String],
        source: &str,
        with_clause: Option<&String>,
    ) -> String {
        let joined_names = exports.join(", ");
        let mut result = format!("export {{ {joined_names} }} from '{source}'");

        if let Some(clause) = with_clause {
            let _ = write!(result, " {clause}");
        }

        result.push(';');
        result.push('\n');

        result
    }

    /// Format a namespace export statement
    fn format_namespace_export(
        name: &str,
        source: &str,
        is_typescript_type: bool,
        with_clause: Option<&String>,
    ) -> String {
        let formatted_name =
            if is_typescript_type { format!("type {name}") } else { name.to_string() };

        let mut result = format!("export {formatted_name} from '{source}'");

        if let Some(clause) = with_clause {
            let _ = write!(result, " {clause}");
        }

        result.push(';');
        result.push('\n');

        result
    }
    fn add_fix_for_export(
        fixer: RuleFixer<'_, '_>,
        add_span: Span,
        replacement_str: &str,
    ) -> RuleFix {
        let fixer = fixer.for_multifix();
        let mut rule_fixes = fixer.new_fix_with_capacity(2);
        rule_fixes.push(fixer.replace(add_span, replacement_str.to_string()));

        rule_fixes.with_message("use `export ... from ...;`")
    }

    fn create_fix_for_re_export<'a>(
        fixer: RuleFixer<'_, '_>,
        used_specifiers: &[SymbolId],
        symbol_to_specifier: &FxIndexMap<SymbolId, SpecifierSpec<'a>>,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        re_export_decl: Option<&'a ExportNamedDeclaration<'a>>,
        re_export_source_text: &str,
        replace_span: Span,
        replace_export_spans: &[Span],
        delete_span: Span,
        exports_str: &str,
        replacement_str: &str,
        source: &str,
        is_namespace: bool,
    ) -> RuleFix {
        let fixer = fixer.for_multifix();
        let mut rule_fixes = fixer.new_fix_with_capacity(2);

        if used_specifiers.len() == symbol_to_specifier.len() {
            Self::apply_complete_export_fix(
                fixer,
                &mut rule_fixes,
                import_decl,
                re_export_decl,
                re_export_source_text,
                replace_span,
                replace_export_spans,
                delete_span,
                exports_str,
                replacement_str,
                is_namespace,
            );
        } else {
            Self::apply_partial_export_fix(
                fixer,
                &mut rule_fixes,
                used_specifiers,
                symbol_to_specifier,
                import_decl,
                replace_span,
                re_export_decl,
                delete_span,
                exports_str,
                replacement_str,
                source,
            );
        }

        rule_fixes.with_message("use `export ... from ...;`")
    }
    /// Apply fix when all specifiers are used
    fn apply_complete_export_fix<'a>(
        fixer: RuleFixer<'_, '_>,
        rule_fixes: &mut RuleFix,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        re_export_decl: Option<&'a ExportNamedDeclaration<'a>>,
        re_export_source_text: &str,
        replace_span: Span,
        replace_export_spans: &[Span],
        delete_span: Span,
        exports_str: &str,
        replacement_str: &str,
        is_namespace: bool,
    ) {
        if let Some(re_export_inner) = re_export_decl {
            Self::handle_reexport_case(
                fixer,
                rule_fixes,
                re_export_inner,
                re_export_source_text,
                import_decl,
                replace_span,
                replace_export_spans,
                exports_str,
                is_namespace,
            );
        } else {
            rule_fixes.push(fixer.delete(&replace_span));
            rule_fixes.push(fixer.replace(import_decl.span(), replacement_str.to_string()));
        }
        rule_fixes.push(fixer.replace(delete_span, ""));
    }

    /// Handle the case when there's a re-export declaration
    fn handle_reexport_case<'a>(
        fixer: RuleFixer<'_, '_>,
        rule_fixes: &mut RuleFix,
        re_export: &'a ExportNamedDeclaration<'a>,
        re_export_source_text: &str,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        replace_span: Span,
        replace_export_spans: &[Span],
        exports_str: &str,
        is_namespace: bool,
    ) {
        let last_specifier = re_export.specifiers.last();
        let last_export_span =
            Self::get_last_export_span(last_specifier, re_export_source_text, re_export);
        let processed_exports_str = Self::get_processed_exports_str(exports_str, re_export);

        if is_namespace {
            let source = re_export.source.as_ref().unwrap().raw.unwrap();
            let final_replacement = format!("export {processed_exports_str} from {source}");
            rule_fixes.push(fixer.replace(import_decl.span(), final_replacement));
        } else {
            let insert_text = Self::get_insertion_text_for_regular_export(
                last_specifier,
                &processed_exports_str,
                re_export,
                re_export_source_text,
            );

            rule_fixes.push(fixer.insert_text_after_range(last_export_span, insert_text));
            rule_fixes.push(fixer.replace(import_decl.span(), ""));
            rule_fixes.push(fixer.replace(replace_span, ""));
            for span in replace_export_spans {
                rule_fixes.push(fixer.replace(*span, ""));
            }
        }
    }
    /// Get the span of the last specifier in an export declaration
    fn get_last_export_span(
        last_specifier: Option<&oxc_ast::ast::ExportSpecifier>,
        re_export_source_text: &str,
        re_export: &ExportNamedDeclaration,
    ) -> Span {
        if let Some(specifier) = last_specifier {
            specifier.span()
        } else {
            let index = re_export_source_text.find('{').unwrap_or(0);
            let start = re_export.span().start;
            let end = start + u32::try_from(index).unwrap_or_default() + 1;
            Span::new(start, end)
        }
    }

    /// Apply fix when only some specifiers are used
    fn apply_partial_export_fix<'a>(
        fixer: RuleFixer<'_, '_>,
        rule_fixes: &mut RuleFix,
        used_specifiers: &[SymbolId],
        symbol_to_specifier: &FxIndexMap<SymbolId, SpecifierSpec<'a>>,
        import_decl: &'a oxc_ast::ast::ImportDeclaration<'a>,
        replace_span: Span,
        re_export_decl: Option<&'a ExportNamedDeclaration<'a>>,
        delete_span: Span,
        exports_str: &str,
        replacement_str: &str,
        source: &str,
    ) {
        rule_fixes.push(fixer.delete(&replace_span));

        let unused_specifiers: Vec<(&SymbolId, &SpecifierSpec)> = symbol_to_specifier
            .into_iter()
            .filter(|(symbol_id, _)| !used_specifiers.contains(symbol_id))
            .collect();

        if unused_specifiers.is_empty() {
            rule_fixes.push(fixer.delete(&import_decl.span()));
        } else {
            let new_import_str = Self::build_new_import_declaration(&unused_specifiers, source);
            if let Some(item) = re_export_decl {
                let last_export_span = item.specifiers.last().unwrap().span();
                let replacement_str = format!(", {}", &exports_str);
                rule_fixes.push(fixer.insert_text_after_range(last_export_span, replacement_str));
                rule_fixes.push(fixer.replace(import_decl.span(), new_import_str));
            } else {
                let new_import_replacement_str = format!("{new_import_str}{replacement_str}");
                rule_fixes.push(fixer.replace(import_decl.span(), new_import_replacement_str));
            }

            rule_fixes.push(fixer.replace(delete_span, ""));
        }
    }

    fn build_new_import_declaration<'a>(
        unused_specifiers: &[(&SymbolId, &'a SpecifierSpec<'a>)],
        source: &str,
    ) -> String {
        let mut import_parts: Vec<String> = Vec::new();
        let mut has_default = false;
        let mut named_imports: Vec<String> = Vec::new();

        for (_, spec) in unused_specifiers {
            match spec.specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    Self::add_default_import_part(&mut import_parts, &mut has_default, spec);
                }

                ImportDeclarationSpecifier::ImportSpecifier(import_spec) => {
                    Self::add_named_import_part(&mut named_imports, import_spec);
                }

                ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_spec) => {
                    Self::add_namespace_import_part(&mut import_parts, namespace_spec);
                }
            }
        }
        Self::finalize_import_parts(&import_parts, &named_imports, source)
    }

    /// Add default import part to import parts vector
    fn add_default_import_part(
        import_parts: &mut Vec<String>,
        has_default: &mut bool,
        spec: &SpecifierSpec,
    ) {
        if !*has_default {
            *has_default = true;
            import_parts.push(spec.name.clone());
        }
    }
    /// Add named import part to named imports vector
    fn add_named_import_part(
        named_imports: &mut Vec<String>,
        import_spec: &oxc_ast::ast::ImportSpecifier,
    ) {
        let local_name = &import_spec.local.name;
        let imported_name = match &import_spec.imported {
            oxc_ast::ast::ModuleExportName::IdentifierName(ident) => ident.name.as_str(),
            oxc_ast::ast::ModuleExportName::IdentifierReference(ref_name) => ref_name.name.as_str(),
            oxc_ast::ast::ModuleExportName::StringLiteral(_) => "",
        };

        if imported_name == local_name.as_str() {
            named_imports.push(local_name.to_string());
        } else {
            named_imports.push(format!("{imported_name} as {local_name}"));
        }
    }

    /// Add namespace import part to import parts vector
    fn add_namespace_import_part(
        import_parts: &mut Vec<String>,
        namespace_spec: &oxc_ast::ast::ImportNamespaceSpecifier,
    ) {
        import_parts.push(format!(" * as {}", namespace_spec.local.name));
    }

    /// Finalize the import parts and return the complete import declaration string
    fn finalize_import_parts(
        import_parts: &[String],
        named_imports: &[String],
        source: &str,
    ) -> String {
        let mut result_parts = import_parts.to_vec();

        if !named_imports.is_empty() {
            result_parts.push(format!("{{{}}}", named_imports.join(", ")));
        }

        if result_parts.is_empty() {
            String::new()
        } else {
            format!("import {} from '{}';\n", result_parts.join(", "), source)
        }
    }

    fn get_processed_exports_str(exports_str: &str, re_export: &ExportNamedDeclaration) -> String {
        if matches!(re_export.export_kind, ImportOrExportKind::Type) {
            exports_str.cow_replace("type ", "").to_string()
        } else {
            exports_str.to_string()
        }
    }

    fn get_insertion_text_for_regular_export(
        last_specifier: Option<&oxc_ast::ast::ExportSpecifier>,
        processed_exports_str: &str,
        re_export: &ExportNamedDeclaration,
        re_export_source_text: &str,
    ) -> String {
        match last_specifier {
            None => processed_exports_str.to_string(),
            Some(specifier) => {
                let last_dot_end = specifier.span().end + 1;
                let dot_index = (last_dot_end - re_export.span().start) as usize;

                if re_export_source_text.chars().nth(dot_index) == Some(',') {
                    processed_exports_str.to_string()
                } else {
                    format!(", {processed_exports_str}")
                }
            }
        }
    }

    fn get_symbol_to_specifier<'a>(
        import_decl: &'a ImportDeclaration<'a>,
    ) -> FxIndexMap<SymbolId, SpecifierSpec<'a>> {
        import_decl
            .specifiers
            .as_ref()
            .unwrap()
            .iter()
            .map(|specifier| {
                let symbol_id = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        specifier.local.symbol_id()
                    }

                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        specifier.local.symbol_id()
                    }

                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        specifier.local.symbol_id()
                    }
                };

                let spec = SpecifierSpec {
                    specifier,
                    name: specifier.local().name.to_string(),
                    decl_type: import_decl.import_kind == ImportOrExportKind::Type,
                };

                (symbol_id, spec)
            })
            .collect()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass: Vec<&str> = vec![
        r#"import "foo";"#,
        r#"import {} from "foo";"#,
        r#"import * as namespace from "foo";"#,
        r#"import defaultExport from "foo";"#,
        r#"import {named} from "foo";"#,
        "const named = import(foo);
            export {named};",
        r#"export * from "foo";"#,
        r#"export {default} from "foo";"#,
        r#"export {named} from "foo";"#,
        "const defaultExport = require('foo');
            export default defaultExport;",
        "import defaultExport from 'foo';
            export var variable = defaultExport;",
        "import defaultExport from 'foo';
            export let variable = defaultExport;",
        "import defaultExport from 'foo';
            export const variable = defaultExport;
            use(variable);",
        "import defaultExport from 'foo';
            export let variable = defaultExport;
            variable = 1;",
        "import * as namespace from 'foo';
            export default namespace;",
        "import * as namespace from 'foo';
            export {namespace as default};",
        "import defaultExport from 'foo';
            const variable = defaultExport;
            export {variable}",
        "import defaultExport from 'foo';
            export const {variable} = {variable: defaultExport};",
        "import {useDispatch as reduxUseDispatch} from 'react-redux'
            type MyDispatchType = Dispatch<MyActions>
            export const useDispatch: () => DispatchAllActions = reduxUseDispatch",
        //it will trigger re_declare rule so comment it
        // r#"import React from "react";
        //     import React from "react";
        //     export {React}"#,
        r#"type AceEditor = import("ace-builds").Ace.Editor;
            import AceEditor from "./advanced-editor";"#,
        r#"type AceEditor = import("ace-builds").Ace.Editor;
            import AceEditor from "./advanced-editor";
            export {AceEditor};"#,
        r#"import AceEditor from "./advanced-editor";
            type AceEditor = import("ace-builds").Ace.Editor;
            export {AceEditor};"#,
        r#"export type { bar, foo } from "foo";"#,
    ];

    let fail = vec![
        "import defaultExport from 'foo';
            export default defaultExport;",
        "import defaultExport from 'foo';
                export {defaultExport as default};",
        "import defaultExport from 'foo';
                export {defaultExport as named};",
        "import defaultExport from 'foo';
            export const variable = defaultExport;",
        "import {default as defaultExport} from 'foo';
                export default defaultExport;",
        "import {default as defaultExport} from 'foo';
                export {defaultExport as named};",
        "import defaultExport from 'foo';
                defaultExport.bar = 1;
                export {defaultExport as named};
                export {defaultExport as default};
                export const variable = defaultExport;",
        "import {named} from 'foo';
            export default named;",
        "import {named} from 'foo';
                export {named as default};",
        "import {named} from 'foo';
                export {named as named};",
        "import {named} from 'foo';
                export {named as renamed};",
        "import {named} from 'foo';
            export const variable = named;",
        "import {named} from 'foo';
            named.bar = 1;
            export {named as named};
            export {named as default};
            export const variable = named;",
        "import * as namespace from 'foo';
            export {namespace as namespace};",
        "import * as namespace from 'foo';
            export {namespace as renamed};",
        "import * as namespace from 'foo';
            export const variable = namespace;",
        "import * as namespace from 'foo';
            namespace.bar = 1;
            export {namespace as named};
            export {namespace as default};
            export const variable = namespace;",
        "import {named1, named2} from 'foo';
            export {named1};",
        "import defaultExport, {named} from 'foo';
            export {defaultExport};",
        "import defaultExport, {named} from 'foo';
            export {named};",
        "import defaultExport, * as namespace from 'foo';
            export {defaultExport};",
        "import * as foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export {} from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export * from 'foo';",
        "import foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {bar,} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export * from 'foo';",
        "import {named1, named2} from 'foo';
                export {named1, named2};",
        "import {named} from 'foo';
                export {named as default, named};",
        "import {named, named as renamed} from 'foo';
                export {named, renamed};",
        "import defaultExport, {named1, named2} from 'foo';
                export {named1 as default};
                export {named2};
                export {defaultExport};",
        "import * as foo from 'foo';
        import * as bar from 'foo';
         export {foo, bar};",
        "import * as foo from 'foo';
                export {foo, foo as bar};",
        "import defaultExport from 'foo';
        export * from 'foo';
         export default defaultExport;",
        "import defaultExport from 'foo';
            export {named} from 'foo';
            export * from 'foo';
            export default defaultExport;",
        "import defaultExport from './foo.js';
                export {named} from './foo.js';
                export default defaultExport;",
        "import defaultExport from './foo.js';
                export {named} from './foo.js?query';
                export default defaultExport;",
        "import * as namespace from 'foo';
                export default namespace;
                export {namespace};",
        "import * as namespace from 'foo';
                export {namespace};
                export default namespace;",
        "import {'foo' as foo} from 'foo';
            export default foo;",
        "import {'foo' as foo} from 'foo';
            export {foo};",
        "import {'foo' as foo} from 'foo';
            export const bar = foo;",
        "import {'foo' as foo} from 'foo';
            export {foo as 'foo'};",
        r#"import {'foo' as foo} from 'foo';
            export {foo as "foo"};"#,
        "import {'fo\u{20}o' as foo} from 'foo';
            export {foo as \"fo o\"};",
        "import {'fo\\no' as foo} from 'foo';
           export {foo as \"fo\\u000ao\"};",
        "import {'default' as foo} from 'foo';
            export {foo};",
        "import {'default' as foo} from 'foo';
            export default foo;",
        "import {'*' as foo} from 'foo';
            export {foo};",
        r#"import { foo } from "foo";
            export { foo };
            export type { bar } from "foo";"#,
        r#"import { foo } from "foo";
            export { foo };
            export { type bar } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
        r#"import type { foo } from "foo";
            export type { foo };
            export type { bar } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export { baz } from "foo" ;
            export { type bar } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { type bar } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { baz } from "foo";
            export type { bar } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
        "import { type foo } from 'foo';
            export type { foo };",
        "import { foo } from 'foo';
            export type { foo };",
        "import type { foo } from 'foo';
            export { type foo };",
        r#"import type foo from "foo";
            export default foo"#,
        "import {type foo} from 'foo';
            export {type foo as bar};",
        "import {type foo} from 'foo';
            export {type foo as bar};
            export {type bar} from 'foo';",
        "import {type foo as bar} from 'foo';
            export {type bar as baz};",
        "import {type foo as foo} from 'foo';
            export {type foo as bar};",
        "import {type foo as bar} from 'foo';
            export {type bar as bar};",
        "import {type foo as bar} from 'foo';
            export {type bar as foo};",
        "import json from './foo.json' assert { type: 'json' };
                export default json;",
        "import * as json from './foo.json' assert { type: 'json' };
                export {json};",
        "import {foo} from './foo.json' assert { type: 'unknown' };
            export {foo};
                export {bar} from './foo.json';",
        "import {foo} from './foo.json';
            export {foo};
                export {bar} from './foo.json' assert { type: 'unknown' };",
        "import type * as X from 'foo';
                export { X };",
        "import * as X from 'foo';
                export type { X };",
        "import type * as X from 'foo';
                export type { X };",
        "import * as X from 'foo';
                export { X };",
        "import json from './foo.json' with { type: 'json' };
                export default json;",
        "import * as json from './foo.json' with { type: 'json' };
            export {json};",
        "import {foo} from './foo.json' with { type: 'unknown' };
            export {foo};
                export {bar} from './foo.json';",
        "import {foo} from './foo.json';
            export {foo};
                export {bar} from './foo.json' with { type: 'unknown' };",
    ];

    let fix = vec![
        (
            "import defaultExport from 'foo';
            export default defaultExport;",
            "export { default } from 'foo';\n",
        ),
        (
            "import defaultExport from 'foo';
            export {defaultExport as default};",
            "export { default } from 'foo';\n",
        ),
        (
            "import defaultExport from 'foo';
            export {defaultExport as named};",
            "export { default as named } from 'foo';\n",
        ),
        (
            "import defaultExport from 'foo';
            export const variable = defaultExport;",
            "export { default as variable } from 'foo';\n",
        ),
        (
            "import {default as defaultExport} from 'foo';
                export default defaultExport;",
            "export { default } from 'foo';\n",
        ),
        (
            "import {default as defaultExport} from 'foo';
        export {defaultExport as named};",
            "export { default as named } from 'foo';\n",
        ),
        (
            "import {named} from 'foo';
            export default named;",
            "export { named as default } from 'foo';\n",
        ),
        (
            "import defaultExport from 'foo';
                defaultExport.bar = 1;
                export {defaultExport as named};
                export {defaultExport as default};
                export const variable = defaultExport;",
            "export { default as named, default, default as variable } from 'foo';\ndefaultExport.bar = 1;\n                ",
        ),
        (
            "import {named} from 'foo';
            export {named as default};",
            "export { named as default } from 'foo';\n",
        ),
        (
            "import {named} from 'foo';
            export {named as named};",
            "export { named } from 'foo';\n",
        ),
        (
            "import {named} from 'foo';
             export {named as renamed};",
            "export { named as renamed } from 'foo';\n",
        ),
        (
            "import * as namespace from 'foo';
            export {namespace as namespace};",
            "export * as namespace from 'foo';\n",
        ),
        (
            "import * as namespace from 'foo';
            export {namespace as renamed};",
            "export * as renamed from 'foo';\n",
        ),
        (
            "import * as foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
            "export * as foo from 'foo';\n\n            export * as bar from 'foo';",
        ),
        (
            "import * as foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
            "export * as foo from 'foo'\n            \n            export {bar} from 'foo';",
        ),
        (
            "import * as foo from 'foo';
            export {foo};
            export {} from 'foo';",
            "export * as foo from 'foo'\n            \n            export {} from 'foo';",
        ),
        (
            "import * as foo from 'foo';
            export {foo};
            export * from 'foo';",
            "export * as foo from 'foo';\n\n            export * from 'foo';",
        ),
        (
            "import * as foo from 'foo';
          import * as bar from 'foo';
         export {foo, bar};",
            "export * as foo from 'foo';\nimport * as bar from 'foo';\n         ",
        ),
        (
            "import * as foo from 'foo';
                export {foo, foo as bar};",
            "export * as foo from 'foo';\nexport * as bar from 'foo';\n",
        ),
        (
            "import * as namespace from 'foo';
                export default namespace;
                export {namespace};",
            "export * as namespace from 'foo';\nexport default namespace;\n                ",
        ),
        (
            "import * as namespace from 'foo';
                export {namespace};
                export default namespace;",
            "export * as namespace from 'foo';\n\n                export default namespace;",
        ),
        (
            "import {'foo' as foo} from 'foo';
            export default foo;",
            "export { 'foo' as default } from 'foo';\n",
        ),
        (
            "import {'foo' as foo} from 'foo';
            export {foo};",
            "export { 'foo' as foo } from 'foo';\n",
        ),
        (
            "import {'foo' as foo} from 'foo';
            export {foo as 'foo'};",
            "export { 'foo' } from 'foo';\n",
        ),
        (
            r#"import {'foo' as foo} from 'foo';
            export {foo as "foo"};"#,
            "export { \"foo\" } from 'foo';\n",
        ),
        (
            "import {'fo\u{20}o' as foo} from 'foo';
            export {foo as \"fo o\"};",
            "export { \"fo o\" } from 'foo';\n",
        ),
        (
            "import {'fo\\no' as foo} from 'foo';
           export {foo as \"fo\\u000ao\"};",
            "export { \"fo\\u000ao\" } from 'foo';\n",
        ),
        (
            "import {'default' as foo} from 'foo';
            export {foo};",
            "export { 'default' as foo } from 'foo';\n",
        ),
        (
            "import {'default' as foo} from 'foo';
            export default foo;",
            "export { 'default' as default } from 'foo';\n",
        ),
        (
            "import {'*' as foo} from 'foo';
            export {foo};",
            "export { '*' as foo } from 'foo';\n",
        ),
        (
            "import foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
            "export {bar, default as foo} from 'foo';",
        ),
        (
            "import foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
            "export { default as foo } from 'foo';\n\n            export * as bar from 'foo';",
        ),
        (
            "import {named1, named2} from 'foo';
            export {named1, named2};",
            "export { named1, named2 } from 'foo';\n",
        ),
        (
            "import {named} from 'foo';
            export {named as default, named};",
            "export { named as default, named } from 'foo';\n",
        ),
        (
            "import {named, named as renamed} from 'foo';
            export {named, renamed};",
            "export { named, named as renamed } from 'foo';\n",
        ),
        (
            "import defaultExport, {named1, named2} from 'foo';
            export {named1 as default};
            export {named2};
            export {defaultExport};",
            "export { default as defaultExport, named1 as default, named2 } from 'foo';\n",
        ),
        (
            "import {named1, named2} from 'foo';
            export {named1};",
            "import {named2} from 'foo';\nexport { named1 } from 'foo';\n",
        ),
        (
            "import defaultExport, {named} from 'foo'
               export {defaultExport}",
            "import {named} from 'foo';\nexport { default as defaultExport } from 'foo';\n",
        ),
        (
            "import defaultExport, {named} from 'foo';
            export {named};",
            "import defaultExport from 'foo';\nexport { named } from 'foo';\n",
        ),
        (
            "import defaultExport from './foo.js';
            export {named} from './foo.js';
            export default defaultExport;",
            "export {named, default} from './foo.js';\n            ",
        ),
        (
            "import defaultExport from './foo.js';
            export {named} from './foo.js?query';
            export default defaultExport;",
            "export { default } from './foo.js';\nexport {named} from './foo.js?query';\n            ",
        ),
        (
            "import json from './foo.json' assert { type: 'json' };
            export default json;",
            "export { default } from './foo.json' assert { type: 'json' };\n",
        ),
        (
            "import {foo} from './foo.json' assert { type: 'unknown' };
            export {foo};
            export {bar} from './foo.json';",
            "export {bar, foo} from './foo.json';",
        ),
        (
            "import {foo} from './foo.json';
            export {foo};
            export {bar} from './foo.json' assert { type: 'unknown' };",
            "export {bar, foo} from './foo.json' assert { type: 'unknown' };",
        ),
        (
            "import json from './foo.json' with { type: 'json' };
            export default json;",
            "export { default } from './foo.json' with { type: 'json' };\n",
        ),
        (
            "import * as json from './foo.json' assert { type: 'json' };
                export {json};",
            "export * as json from './foo.json' assert { type: 'json' };\n",
        ),
        (
            "import {foo} from './foo.json' with { type: 'unknown' };
            export {foo};
            export {bar} from './foo.json';",
            "export {bar, foo} from './foo.json';",
        ),
        (
            "import {foo} from './foo.json';
            export {foo};
            export {bar} from './foo.json' with { type: 'unknown' };",
            "export {bar, foo} from './foo.json' with { type: 'unknown' };",
        ),
        (
            r#"import { foo } from "foo";
            export { foo };
            export type { bar } from "foo";"#,
            "export { foo } from 'foo';\n\n            export type { bar } from \"foo\";",
        ),
        (
            r#"import { foo } from "foo";
            export { foo };
            export { type bar } from "foo";"#,
            "export { type bar, foo } from \"foo\";",
        ),
        (
            r#"import { foo } from 'foo';
            export { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
            "export type { bar } from \"foo\";\n            export { baz, foo } from \"foo\";",
        ),
        (
            r#"import { foo } from 'foo';
            export { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
            "export { type bar, foo } from \"foo\";\n            export { baz } from \"foo\";",
        ),
        (
            r#"import type { foo } from "foo";
            export type { foo };
            export type { bar } from "foo";"#,
            "export type { bar, foo } from \"foo\";",
        ),
        (
            r#"import { foo } from 'foo';
            export { foo };
            export { baz } from "foo";
            export { type bar } from "foo";"#,
            "export { baz, foo } from \"foo\";\n            export { type bar } from \"foo\";",
        ),
        (
            r#"import type { foo } from 'foo';
            export type { foo };
            export { type bar } from "foo";"#,
            r#"export { type bar, type foo } from "foo";"#,
        ),
        (
            r#"import type { foo } from 'foo';
            export type { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
            "export type { bar, foo } from \"foo\";\n            export { baz } from \"foo\";",
        ),
        (
            r#"import type { foo } from 'foo';
            export type { foo };
            export { baz } from "foo";
            export type { bar } from "foo";"#,
            "export { baz } from \"foo\";\n            export type { bar, foo } from \"foo\";",
        ),
        (
            r#"import type { foo } from 'foo';
            export type { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
            "export { type bar, type foo } from \"foo\";\n            export { baz } from \"foo\";",
        ),
        (
            "import { type foo } from 'foo';
            export type { foo };",
            "export { type foo } from 'foo';\n",
        ),
        (
            "import { foo } from 'foo';
            export type { foo };",
            "import { foo } from 'foo';\n            export { foo } from 'foo';\n",
        ),
        (
            r#"import type foo from "foo";
            export default foo"#,
            "export { type default } from 'foo';\n",
        ),
        (
            "import {type foo} from 'foo';
            export {type foo as bar};",
            "export { type foo as bar } from 'foo';\n",
        ),
        (
            "import {type foo} from 'foo';
            export {type foo as bar};
            export {type bar} from 'foo';",
            "export {type bar, type foo as bar} from 'foo';",
        ),
        (
            "import {type foo as bar} from 'foo';
            export {type bar as baz};",
            "export { type foo as baz } from 'foo';\n",
        ),
        (
            "import {type foo as foo} from 'foo';
            export {type foo as bar};",
            "export { type foo as bar } from 'foo';\n",
        ),
        (
            "import {type foo as bar} from 'foo';
            export {type bar as bar};",
            "export { type foo as bar } from 'foo';\n",
        ),
        (
            "import {type foo as bar} from 'foo';
            export {type bar as foo};",
            "export { type foo } from 'foo';\n",
        ),
        (
            "import type * as X from 'foo';
                export { X };",
            "export type * as X from 'foo';\n",
        ),
        (
            "import * as X from 'foo';
                export type { X };",
            "import * as X from 'foo';\n                export type * as X from 'foo';\n",
        ),
        (
            "import type * as X from 'foo';
                export type { X };",
            "export type * as X from 'foo';\n",
        ),
        (
            "import * as X from 'foo';
                export { X };",
            "export * as X from 'foo';\n",
        ),
        (
            "import {'foo' as foo} from 'foo';
            export const bar = foo;",
            "export { 'foo' as bar } from 'foo';\n",
        ),
        (
            "import defaultExport, * as namespace from 'foo';
            export {defaultExport};",
            "import  * as namespace from 'foo';\nexport { default as defaultExport } from 'foo';\n",
        ),
        (
            "import * as namespace from 'foo';
                export const variable = namespace;",
            "export * as variable from 'foo';\n",
        ),
        (
            "import * as namespace from 'foo';
            namespace.bar = 1;
            export {namespace as named};
            export {namespace as default};
            export const variable = namespace;",
            "export * as named from 'foo';\nexport * as variable from 'foo';\nnamespace.bar = 1;\n            ",
        ),
        (
            "import foo from 'foo';
               export {foo};
               export {} from 'foo';",
            "export {default as foo} from 'foo';",
        ),
        (
            "import foo from 'foo';
            export {foo};
            export {bar,} from 'foo';",
            "export {bar, default as foo,} from 'foo';",
        ),
    ];
    Tester::new(PreferExportFrom::NAME, PreferExportFrom::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
