// Port of internal/rules/consistent_type_exports/consistent_type_exports.go.

use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol, SymbolFlags};
use tsrs_checker::Checker;
use tsrs_core::P;

use crate::rule::{
    Ctx, LabeledRange, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor, opt_bool,
    options_object,
};
use crate::utils;

fn build_type_over_value_message() -> RuleMessage {
    RuleMessage::with_help(
        "typeOverValue",
        "All exports in the declaration are only used as types.",
        "Use `export type`.",
    )
}

fn build_single_export_is_type_message(export_name: &str) -> RuleMessage {
    RuleMessage::with_help(
        "singleExportIsType",
        format!(
            "Type export {export_name} is not a value and should be exported using `export type`."
        ),
        format!("Try adding the `type` keyword: `type {export_name}`"),
    )
}

fn build_multiple_exports_are_types_message(export_names: &str) -> RuleMessage {
    RuleMessage::new(
        "multipleExportsAreTypes",
        format!(
            "Type exports {export_names} are not values and should be exported using `export type`."
        ),
    )
}

struct AnalyzedNamedExport {
    node: P<Node>,
    type_texts: Vec<String>,
    type_based_nodes: Vec<P<Node>>,
    type_based_names: Vec<String>,
    value_texts: Vec<String>,
    module_source: String,
}

/// Returns (is_type, resolved).
fn is_symbol_type_based(c: &mut Checker, mut symbol: Option<P<Symbol>>) -> (bool, bool) {
    let mut visited: FxHashSet<P<Symbol>> = FxHashSet::default();
    while let Some(s) = symbol {
        if !visited.insert(s) {
            break;
        }
        if s.declarations().iter().any(|&d| ast::is_type_only_import_or_export_declaration(d)) {
            return (true, true);
        }
        let flags = s.flags.get();
        if flags.intersects(SymbolFlags::Value) {
            return (false, true);
        }
        if flags.intersects(SymbolFlags::Alias) {
            let Some(next) = c.get_immediate_aliased_symbol(s) else {
                return (false, false);
            };
            symbol = Some(next);
            continue;
        }
        return (true, true);
    }
    (false, false)
}

fn get_node_text(source_file: P<SourceFile>, node: P<Node>) -> &'static str {
    let (pos, end) = utils::trim_node_text_range(source_file, node);
    &source_file.text()[pos as usize..end as usize]
}

/// Go unicode.IsSpace for a single byte converted to a rune.
fn go_is_space(c: u8) -> bool {
    matches!(c, b'\t' | b'\n' | 0x0B | 0x0C | b'\r' | b' ' | 0x85 | 0xA0)
}

fn strip_leading_type_keyword(specifier_text: &str) -> String {
    if !specifier_text.starts_with("type") {
        return specifier_text.to_string();
    }
    if specifier_text.len() == 4 {
        return String::new();
    }
    let next = specifier_text.as_bytes()[4];
    if !go_is_space(next) && next != b'/' {
        return specifier_text.to_string();
    }
    specifier_text[4..].trim_start_matches([' ', '\t', '\r', '\n']).to_string()
}

fn get_export_specifier_text(source_file: P<SourceFile>, specifier_node: P<Node>) -> String {
    let specifier = specifier_node.as_export_specifier();
    let text = get_node_text(source_file, specifier_node).trim();
    if specifier.is_type_only() {
        return strip_leading_type_keyword(text);
    }
    let local = specifier.property_name().unwrap_or_else(|| specifier.name());
    let exported = specifier.name();
    let local_text = get_node_text(source_file, local).trim();
    let exported_text = get_node_text(source_file, exported).trim();
    if local_text == exported_text {
        return local_text.to_string();
    }
    format!("{local_text} as {exported_text}")
}

fn get_export_keyword_range(source_file: P<SourceFile>, node: P<Node>) -> (i32, i32) {
    let mut s = tsrs_scanner::get_scanner_for_source_file(source_file, node.pos());
    loop {
        if s.token() == Kind::ExportKeyword {
            let r = s.token_range();
            return (r.pos(), r.end());
        }
        if s.token() == Kind::EndOfFile || s.token_range().pos() >= node.end() {
            break;
        }
        s.scan();
    }
    utils::trim_node_text_range(source_file, node)
}

fn join_word_list(words: &[String]) -> String {
    match words.len() {
        0 => String::new(),
        1 => words[0].clone(),
        2 => format!("{} and {}", words[0], words[1]),
        n => format!("{}, and {}", words[..n - 1].join(", "), words[n - 1]),
    }
}

fn build_named_export_statement(type_only: bool, specifiers: &[String], source: &str) -> String {
    let mut statement = "export ".to_string();
    if type_only {
        statement.push_str("type ");
    }
    statement.push_str("{ ");
    statement.push_str(&specifiers.join(", "));
    statement.push_str(" }");
    if !source.is_empty() {
        statement.push_str(" from ");
        statement.push_str(source);
    }
    statement.push(';');
    statement
}

pub struct ConsistentTypeExports {
    fix_mixed_exports_with_inline_type_specifier: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let m = options_object(options);
    Ok(Box::new(ConsistentTypeExports {
        fix_mixed_exports_with_inline_type_specifier: opt_bool(
            &m,
            "fixMixedExportsWithInlineTypeSpecifier",
            false,
        ),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::ExportDeclaration)];

impl Rule for ConsistentTypeExports {
    fn name(&self) -> &'static str {
        "consistent-type-exports"
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { o: self })
    }
}

struct Visitor {
    o: &'static ConsistentTypeExports,
}

impl Visitor {
    fn check_star_export(&self, ctx: &mut Ctx, node: P<Node>) {
        let export_decl = node.as_export_declaration();
        if export_decl.is_type_only() {
            return;
        }
        let Some(module_specifier) = export_decl.module_specifier() else {
            return;
        };
        let Some(module_symbol) = ctx.checker.get_symbol_at_location_exported(module_specifier)
        else {
            return;
        };
        let source_file_type = ctx.checker.get_type_of_symbol(module_symbol);
        let mut is_there_any_exported_value = false;
        for &property_type_symbol in ctx.checker.get_properties_of_type(source_file_type) {
            if ctx
                .checker
                .get_property_of_type(source_file_type, property_type_symbol.name())
                .is_some()
            {
                is_there_any_exported_value = true;
                break;
            }
        }
        if is_there_any_exported_value {
            return;
        }
        let (pos, end) = get_export_keyword_range(ctx.file, node);
        ctx.report_diagnostic_with_fixes(
            RuleDiagnostic {
                pos,
                end,
                message: build_type_over_value_message(),
                labeled_ranges: Vec::new(),
            },
            |ctx| {
                let mut s = tsrs_scanner::get_scanner_for_source_file(ctx.file, node.pos());
                loop {
                    if s.token() == Kind::AsteriskToken {
                        let r = s.token_range();
                        return vec![ctx.fix_replace_range(r.pos(), r.pos(), "type ")];
                    }
                    if s.token() == Kind::EndOfFile || s.token_range().pos() >= node.end() {
                        break;
                    }
                    s.scan();
                }
                Vec::new()
            },
        );
    }

    fn analyze_named_export(&self, ctx: &mut Ctx, node: P<Node>) -> Option<AnalyzedNamedExport> {
        let export_decl = node.as_export_declaration();
        if export_decl.is_type_only() {
            return None;
        }
        let mut report = AnalyzedNamedExport {
            node,
            type_texts: Vec::with_capacity(2),
            type_based_nodes: Vec::with_capacity(2),
            type_based_names: Vec::with_capacity(2),
            value_texts: Vec::with_capacity(2),
            module_source: String::new(),
        };
        if let Some(module_specifier) = export_decl.module_specifier() {
            report.module_source = get_node_text(ctx.file, module_specifier).trim().to_string();
        }
        for &specifier_node in export_decl.export_clause().unwrap().elements() {
            let specifier = specifier_node.as_export_specifier();
            let specifier_text = get_export_specifier_text(ctx.file, specifier_node);
            if specifier.is_type_only() {
                report.type_texts.push(specifier_text);
                continue;
            }
            let name_node = specifier.property_name().unwrap_or_else(|| specifier.name());
            let symbol = ctx.checker.get_symbol_at_location_exported(name_node);
            let (is_type, resolved) = is_symbol_type_based(ctx.checker, symbol);
            if !resolved {
                continue;
            }
            if is_type {
                report.type_texts.push(specifier_text.clone());
                report.type_based_nodes.push(specifier_node);
                report.type_based_names.push(specifier_text);
            } else {
                report.value_texts.push(specifier_text);
            }
        }
        Some(report)
    }

    fn check_named_export(&self, ctx: &mut Ctx, report: Option<AnalyzedNamedExport>) {
        let Some(report) = report else { return };
        if report.type_based_nodes.is_empty() {
            return;
        }
        let (kpos, kw_end) = get_export_keyword_range(ctx.file, report.node);
        if report.value_texts.is_empty() {
            ctx.report_diagnostic_with_fixes(
                RuleDiagnostic {
                    pos: kpos,
                    end: kw_end,
                    message: build_type_over_value_message(),
                    labeled_ranges: Vec::new(),
                },
                |ctx| {
                    vec![ctx.fix_replace(
                        report.node,
                        build_named_export_statement(
                            true,
                            &report.type_texts,
                            &report.module_source,
                        ),
                    )]
                },
            );
            return;
        }
        let msg;
        let mut labeled_ranges = Vec::new();
        let (mut pos, mut end) = (0, 0);
        if report.type_based_names.len() == 1 {
            msg = build_single_export_is_type_message(&report.type_based_names[0]);
            (pos, end) = ctx.trim(report.type_based_nodes[0]);
        } else {
            msg =
                build_multiple_exports_are_types_message(&join_word_list(&report.type_based_names));
            for (i, &specifier_node) in report.type_based_nodes.iter().enumerate() {
                let (p, e) = ctx.trim(specifier_node);
                let name = &report.type_based_names[i];
                labeled_ranges.push(LabeledRange {
                    label: format!("{name} is a type export, try `type {name}`"),
                    pos: p,
                    end: e,
                });
            }
        }
        let inline = self.o.fix_mixed_exports_with_inline_type_specifier;
        ctx.report_diagnostic_with_fixes(
            RuleDiagnostic { pos, end, message: msg, labeled_ranges },
            |ctx| {
                if inline {
                    return report
                        .type_based_nodes
                        .iter()
                        .map(|&n| ctx.fix_insert_before(n, "type "))
                        .collect();
                }
                let replacement = format!(
                    "{}\n{}",
                    build_named_export_statement(true, &report.type_texts, &report.module_source),
                    build_named_export_statement(false, &report.value_texts, &report.module_source)
                );
                vec![ctx.fix_replace(report.node, replacement)]
            },
        );
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let export_clause = node.as_export_declaration().export_clause();
        match export_clause {
            None => self.check_star_export(ctx, node),
            Some(c) if ast::is_namespace_export(c) => self.check_star_export(ctx, node),
            Some(c) if ast::is_named_exports(c) => {
                let report = self.analyze_named_export(ctx, node);
                self.check_named_export(ctx, report);
            }
            _ => {}
        }
    }
}
