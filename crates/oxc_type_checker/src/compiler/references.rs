//! Port of typescript-go's `internal/parser/references.go`.
//!
//! [`collect_external_module_references`] gathers the module specifiers a file references, in
//! tsgo's order: statically imported specifiers in source order (`collectModuleReferences`),
//! then dynamic `import()` calls and `import("...")` type queries in source order
//! (`ForEachDynamicImportOrRequireCall`), plus the file's `declare module "..."` augmentations.
//!
//! oxc's parser already collects top-level static imports/re-exports into the module record; the
//! AST walks here add the rest: `import x = require("...")`, statements nested inside ambient
//! module bodies, dynamic `import()`s, `import("...")` type queries, CommonJS `require("...")`
//! calls in JavaScript files, and the ambient module declarations themselves.

use oxc_ast::ast::{
    CallExpression, Declaration, Expression, ImportExpression, Program as AstProgram, Statement,
    TSImportType, TSModuleDeclarationBody, TSModuleDeclarationName, TSModuleReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use oxc_str::CompactStr;
use oxc_syntax::module_record::ModuleRecord;

use crate::tspath::is_external_module_name_relative;

/// A file's external module references (tsgo `SourceFile.Imports` + `ModuleAugmentations` +
/// `ReferencedFiles` + `TypeReferenceDirectives`).
#[derive(Debug, Default)]
pub(super) struct ExternalModuleReferences {
    /// Module specifiers referenced by imports: static imports/re-exports in source order,
    /// followed by dynamic `import()`s and `import("...")` type queries in source order.
    pub imports: Vec<CompactStr>,
    /// String-literal `declare module "..."` names that augment an existing external module.
    pub module_augmentations: Vec<CompactStr>,
    /// `/// <reference path="..." />` pragmas (tsgo `SourceFile.ReferencedFiles`).
    pub referenced_files: Vec<CompactStr>,
    /// `/// <reference types="..." />` pragmas (tsgo `SourceFile.TypeReferenceDirectives`).
    pub type_reference_directives: Vec<CompactStr>,
}

/// tsgo `collectExternalModuleReferences`: collect every module specifier the resolver should
/// see for this file.
pub(super) fn collect_external_module_references(
    program: &AstProgram<'_>,
    module_record: &ModuleRecord<'_>,
    is_declaration_file: bool,
) -> ExternalModuleReferences {
    let mut collector = Collector {
        // tsgo `ast.IsExternalModule`: the file has an external module indicator.
        is_external_module: module_record.has_module_syntax,
        is_declaration_file,
        statics: Vec::new(),
        module_augmentations: Vec::new(),
    };

    // Top-level static imports/re-exports, already collected by the parser (source position of
    // each request comes from the module record, so ordering below can interleave them with the
    // statements the walk finds).
    for (specifier, requests) in &module_record.requested_modules {
        let specifier = specifier.as_str();
        if specifier.is_empty() {
            continue;
        }
        for request in requests {
            collector.statics.push((request.span.start, CompactStr::from(specifier)));
        }
    }

    // tsgo `collectModuleReferences` over the top-level statements: adds `import x = require()`,
    // statements nested in ambient module bodies, and module augmentations.
    for statement in &program.body {
        collector.collect_module_references(statement, false);
    }
    collector.statics.sort_by_key(|(start, _)| *start);

    // tsgo `ForEachDynamicImportOrRequireCall` (string-literal-like arguments, including
    // type-space imports): one AST walk collects dynamic `import()`s, `import("...")` type
    // queries, and — in JavaScript files only — `require("...")` calls.
    let mut dynamics: Vec<(u32, CompactStr)> = Vec::new();
    let mut calls = CallCollector {
        dynamics: &mut dynamics,
        collect_require_calls: program.source_type.is_javascript(),
    };
    calls.visit_program(program);
    dynamics.sort_by_key(|(start, _)| *start);

    let imports =
        collector.statics.into_iter().chain(dynamics).map(|(_, specifier)| specifier).collect();

    let mut references = ExternalModuleReferences {
        imports,
        module_augmentations: collector.module_augmentations,
        ..ExternalModuleReferences::default()
    };
    collect_reference_pragmas(program, &mut references);
    references
}

/// tsgo's scanner collects `/// <reference ... />` pragmas from the file's leading trivia —
/// the comments before the first token (directive prologue or statement).
fn collect_reference_pragmas(program: &AstProgram<'_>, references: &mut ExternalModuleReferences) {
    let first_token_start = program
        .directives
        .first()
        .map(|directive| directive.span.start)
        .or_else(|| program.body.first().map(|statement| statement.span().start))
        .unwrap_or(u32::MAX);
    for comment in &program.comments {
        if comment.span.start >= first_token_start {
            break;
        }
        if !comment.is_line() {
            continue;
        }
        let content = comment.content_span().source_text(program.source_text);
        let Some((name, value)) = parse_reference_pragma(content) else { continue };
        if value.is_empty() {
            continue;
        }
        match name {
            "path" => references.referenced_files.push(CompactStr::from(value)),
            "types" => references.type_reference_directives.push(CompactStr::from(value)),
            // `lib="..."` references default lib files, which are not loaded yet;
            // `no-default-lib` carries no file reference.
            _ => {}
        }
    }
}

/// Parse a `/ <reference path|types = "..." />` line-comment body (the leading `//` is already
/// stripped), mirroring tsc's `fullTripleSlashReference(Path|TypeReference)RegEx`: the attribute
/// must directly follow `<reference`.
fn parse_reference_pragma(content: &str) -> Option<(&'static str, &str)> {
    let rest = content.strip_prefix('/')?.trim_start();
    let rest = rest.strip_prefix("<reference")?;
    let rest = rest.trim_start();
    let (name, rest) = if let Some(rest) = rest.strip_prefix("path") {
        ("path", rest)
    } else if let Some(rest) = rest.strip_prefix("types") {
        ("types", rest)
    } else if let Some(rest) = rest.strip_prefix("lib") {
        ("lib", rest)
    } else {
        return None;
    };
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next().filter(|&c| c == '"' || c == '\'')?;
    let rest = &rest[1..];
    let end = rest.find(quote)?;
    Some((name, &rest[..end]))
}

struct Collector {
    is_external_module: bool,
    is_declaration_file: bool,
    /// (source position, specifier) of statically referenced modules, for source-order sorting.
    statics: Vec<(u32, CompactStr)>,
    module_augmentations: Vec<CompactStr>,
}

impl Collector {
    /// tsgo `collectModuleReferences`. `in_ambient_module` is `true` inside the body of a
    /// top-level ambient module declaration, where only non-relative names may reference other
    /// external modules.
    fn collect_module_references(&mut self, statement: &Statement<'_>, in_ambient_module: bool) {
        match statement {
            // `import x = require("...")` — not in oxc's module record.
            Statement::TSImportEqualsDeclaration(decl) => {
                if let TSModuleReference::ExternalModuleReference(reference) =
                    &decl.module_reference
                {
                    let name = &reference.expression;
                    self.add_static(name.span.start, &name.value, in_ambient_module);
                }
            }
            // Top-level imports/re-exports are already in the module record; only the ones
            // nested inside ambient module bodies need collecting here.
            Statement::ImportDeclaration(decl) => {
                if in_ambient_module {
                    self.add_static(decl.source.span.start, &decl.source.value, true);
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if in_ambient_module && let Some(source) = &decl.source {
                    self.add_static(source.span.start, &source.value, true);
                }
                // `export import A = require("...")` wraps the import-equals declaration.
                if let Some(Declaration::TSImportEqualsDeclaration(decl)) = &decl.declaration
                    && let TSModuleReference::ExternalModuleReference(reference) =
                        &decl.module_reference
                {
                    let name = &reference.expression;
                    self.add_static(name.span.start, &name.value, in_ambient_module);
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                if in_ambient_module {
                    self.add_static(decl.source.span.start, &decl.source.value, true);
                }
            }
            Statement::TSModuleDeclaration(decl) => {
                // Only string-named ambient modules matter here (`declare global` has no module
                // name to resolve; tsgo drops it at resolution time).
                let TSModuleDeclarationName::StringLiteral(name) = &decl.id else { return };
                if !(in_ambient_module || decl.declare || self.is_declaration_file) {
                    return;
                }
                // Ambient module declarations can be interpreted as augmentations of existing
                // external modules: in an external module file, any of them; in a script file,
                // the non-relative ones immediately nested in a top-level ambient module.
                if self.is_external_module
                    || (in_ambient_module && !is_external_module_name_relative(&name.value))
                {
                    self.module_augmentations.push(CompactStr::from(name.value.as_str()));
                } else if !in_ambient_module {
                    // A top-level ambient module declaration in a script file *declares* the
                    // module — nothing to resolve, but its body may reference other modules.
                    if let Some(TSModuleDeclarationBody::TSModuleBlock(block)) = &decl.body {
                        for statement in &block.body {
                            self.collect_module_references(statement, true);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Record a statically referenced module name. Inside an ambient module, relative names
    /// cannot reference other external modules (TypeScript 1.0 spec §12.1.6).
    fn add_static(&mut self, start: u32, name: &str, in_ambient_module: bool) {
        if !name.is_empty() && (!in_ambient_module || !is_external_module_name_relative(name)) {
            self.statics.push((start, CompactStr::from(name)));
        }
    }
}

/// Collects dynamic `import()`s, `import("...")` type queries (tsgo's "type space imports",
/// which live in type positions anywhere in the AST), and — in JavaScript files —
/// `require("...")` calls.
struct CallCollector<'v> {
    dynamics: &'v mut Vec<(u32, CompactStr)>,
    collect_require_calls: bool,
}

impl CallCollector<'_> {
    /// Record a string-literal-like specifier (tsgo `requireStringLiteralLikeArgument`: a plain
    /// string literal or a no-substitution template), using the parsed (cooked) value so escape
    /// sequences resolve the same file TypeScript would.
    fn add_string_literal_like(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::StringLiteral(literal) => {
                if !literal.value.is_empty() {
                    self.dynamics
                        .push((literal.span.start, CompactStr::from(literal.value.as_str())));
                }
            }
            Expression::TemplateLiteral(template) if template.is_no_substitution_template() => {
                let value = template.quasis[0].value.cooked.as_ref().map_or_else(
                    || template.quasis[0].value.raw.as_str(),
                    |cooked| cooked.as_str(),
                );
                if !value.is_empty() {
                    self.dynamics.push((template.span.start, CompactStr::from(value)));
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for CallCollector<'_> {
    fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
        self.add_string_literal_like(&it.source);
        walk::walk_import_expression(self, it);
    }

    fn visit_ts_import_type(&mut self, it: &TSImportType<'a>) {
        if !it.source.value.is_empty() {
            self.dynamics.push((it.source.span.start, CompactStr::from(it.source.value.as_str())));
        }
        // Type arguments may nest further import types.
        walk::walk_ts_import_type(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // tsgo `IsRequireCall` (shape only, no shadowing check): a call to the identifier
        // `require` with exactly one string-literal-like argument.
        if self.collect_require_calls
            && let Expression::Identifier(callee) = &it.callee
            && callee.name == "require"
            && it.arguments.len() == 1
            && let Some(argument) = it.arguments[0].as_expression()
        {
            self.add_string_literal_like(argument);
        }
        walk::walk_call_expression(self, it);
    }
}
