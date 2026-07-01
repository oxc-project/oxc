//! Line-by-line port of typescript-go's symbol baseline harness
//! `internal/testutil/tsbaseline/type_symbol_baseline.go`.
//!
//! Only the `.symbols` half is ported: `.types` needs a type checker, which oxc does not
//! have. The walker visits the same nodes as the Go `forEachASTNode` (via `oxc_ast_visit`),
//! resolves the two lexically-resolvable kinds through `oxc_semantic`, and emits an
//! `<<unresolved>>` marker for member / `this` / lib nodes (which need a checker) so the
//! output stays line-aligned with the shipped baseline. Scoring is a line-diff against the
//! baseline counting matching `>` annotation lines.

use std::{fmt::Write as _, path::Path};

use oxc::{
    allocator::Allocator,
    ast::AstKind,
    ast_visit::Visit,
    parser::{Parser, config::TokensParserConfig},
    semantic::{Semantic, SemanticBuilder},
    span::{GetSpan, Span},
    syntax::{node::NodeId, symbol::SymbolId},
};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};

use super::scanner::{
    self, BRACKET_LINE, LINE_DELIMITER, full_start, line_and_character, line_of_position,
};
use crate::{CoverageResult, TestResult, TypeScriptFile, workspace_root};

/// Emitted for nodes oxc cannot resolve to a symbol (members, `this`/`super`, lib globals).
/// Must never be equal to a real `Symbol(...)` string.
const UNRESOLVED: &str = "<<unresolved>>";

/// Port of `typeWriterResult` (symbol fields only).
struct TypeWriterResult {
    line: u32,
    /// Raw node source text (may span lines); line-delimiters are stripped at output time.
    source_text: String,
    symbol: String,
}

/// Port of `typeWriterWalker`, backed by `oxc_ast_visit::Visit` in place of `forEachASTNode`.
struct SymbolWalker<'a> {
    semantic: &'a Semantic<'a>,
    content: &'a str,
    line_starts: &'a [u32],
    token_ends: &'a [u32],
    unit_name: &'a str,
    results: Vec<TypeWriterResult>,
}

impl<'a> Visit<'a> for SymbolWalker<'a> {
    // Port of `visitNode`'s filter (`IsExpressionNode || KindIdentifier || IsDeclarationName`)
    // fused with `writeTypeOrSymbol`. `enter_node` fires pre-order in source order, matching
    // the Go depth-first `forEachASTNode` walk.
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let symbol = match kind {
            AstKind::IdentifierReference(it) => self.resolve(
                it.reference_id
                    .get()
                    .and_then(|r| self.semantic.scoping().get_reference(r).symbol_id()),
            ),
            AstKind::BindingIdentifier(it) => self.resolve(it.symbol_id.get()),
            AstKind::IdentifierName(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::ThisExpression(_)
            | AstKind::Super(_) => UNRESOLVED.to_string(),
            // Literals, calls, binary/array/object/arrow, `TSQualifiedName`, etc. — the Go
            // checker returns nil for these, so they never appear in a `.symbols` baseline.
            _ => return,
        };
        self.write(kind.span(), symbol);
    }
}

impl SymbolWalker<'_> {
    fn resolve(&self, symbol_id: Option<SymbolId>) -> String {
        symbol_id.map_or_else(|| UNRESOLVED.to_string(), |sid| self.format_symbol(sid))
    }

    /// Port of `writeTypeOrSymbol`'s symbol branch: `Symbol(name, Decl(file, line, col), ...)`.
    fn format_symbol(&self, sid: SymbolId) -> String {
        let scoping = self.semantic.scoping();
        let mut s = format!("Symbol({}", scoping.symbol_name(sid));
        let decls: Vec<NodeId> = scoping.symbol_declarations(sid).collect();
        for (i, &nid) in decls.iter().enumerate() {
            if i >= 5 {
                let _ = write!(s, " ... and {} more", decls.len() - 5);
                break;
            }
            let full = full_start(self.token_ends, self.decl_start(nid));
            let (line, col) = line_and_character(self.content, self.line_starts, full);
            let _ = write!(s, ", Decl({}, {line}, {col})", self.unit_name);
        }
        s.push(')');
        s
    }

    /// Byte offset of the declaration node's start, before `full_start` is applied. Ports TS's
    /// choice of declaration node: exported declarations start before `export`, so climb to the
    /// export wrapper (but not for variables, whose declarator starts after `var`/`let`/`const`).
    fn decl_start(&self, nid: NodeId) -> u32 {
        let nodes = self.semantic.nodes();
        let span = nodes.kind(nid).span();
        match nodes.parent_kind(nid) {
            AstKind::ExportNamedDeclaration(export) => export.span().start,
            AstKind::ExportDefaultDeclaration(export) => export.span().start,
            _ => span.start,
        }
    }

    fn write(&mut self, span: Span, symbol: String) {
        let line = line_of_position(self.line_starts, span.start);
        let source_text = self.content[span.start as usize..span.end as usize].to_string();
        self.results.push(TypeWriterResult { line, source_text, symbol });
    }
}

/// Port of the per-file body of `iterateBaseline` (type_symbol_baseline.go:199-252).
fn iterate_unit(unit_name: &str, content: &str, results: &[TypeWriterResult]) -> String {
    let mut out = String::new();
    out.push_str("=== ");
    out.push_str(unit_name);
    out.push_str(" ===\r\n");

    let code_lines = scanner::split_lines(content);
    let mut last_index_written: Option<usize> = None;

    for result in results {
        let line = result.line as usize;
        match last_index_written {
            None => {
                out.push_str(&code_lines[..=line].join("\r\n"));
                out.push_str("\r\n");
            }
            Some(liw) if liw != line => {
                if !(liw + 1 < code_lines.len()
                    && (BRACKET_LINE.is_match(code_lines[liw + 1])
                        || code_lines[liw + 1].trim().is_empty()))
                {
                    out.push_str("\r\n");
                }
                out.push_str(&code_lines[liw + 1..=line].join("\r\n"));
                out.push_str("\r\n");
            }
            Some(_) => {}
        }
        last_index_written = Some(line);

        let line_text = LINE_DELIMITER.replace_all(&result.source_text, "");
        out.push('>');
        out.push_str(&line_text);
        out.push_str(" : ");
        out.push_str(&result.symbol);
        out.push_str("\r\n");
    }

    let next = last_index_written.map_or(0, |l| l + 1);
    if next < code_lines.len() {
        if !(BRACKET_LINE.is_match(code_lines[next]) || code_lines[next].trim().is_empty()) {
            out.push_str("\r\n");
        }
        out.push_str(&code_lines[next..].join("\r\n"));
    }
    out.push_str("\r\n");
    out
}

/// The `tests/cases/...` header path (strips the leading `typescript/` component).
fn header_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.strip_prefix("typescript/").unwrap_or(&s).to_string()
}

/// Port of `generateBaseline`: walk every unit, emit the `//// [path] ////` header + sections.
fn generate_for_file(f: &TypeScriptFile, header: &str) -> String {
    let mut joined = String::new();
    for unit in &f.units {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &unit.content, unit.source_type)
            .with_config(TokensParserConfig)
            .parse();
        let token_ends: Vec<u32> = ret.tokens.iter().map(oxc::parser::Token::end).collect();
        let line_starts = scanner::compute_line_starts(&unit.content);

        let results = if ret.panicked {
            Vec::new()
        } else {
            let semantic =
                SemanticBuilder::new_compiler().with_build_nodes(true).build(&ret.program).semantic;
            let mut walker = SymbolWalker {
                semantic: &semantic,
                content: &unit.content,
                line_starts: &line_starts,
                token_ends: &token_ends,
                unit_name: &unit.name,
                results: Vec::new(),
            };
            walker.visit_program(semantic.nodes().program());
            walker.results
        };

        joined.push_str(&iterate_unit(&unit.name, &unit.content, &results));
    }

    if joined.is_empty() {
        return String::new();
    }
    format!("//// [{header}] ////\r\n\r\n{joined}")
}

/// Line-diff score: of the baseline's `>` annotation lines, how many oxc reproduced exactly.
fn score(baseline: &str, oxc: &str) -> (usize, usize) {
    let diff = TextDiff::from_lines(baseline, oxc);
    let mut matched = 0;
    let mut total = 0;
    for change in diff.iter_all_changes() {
        let line = change.value().trim_end_matches(['\r', '\n']);
        if !line.starts_with('>') {
            continue;
        }
        match change.tag() {
            ChangeTag::Equal => {
                matched += 1;
                total += 1;
            }
            ChangeTag::Delete => total += 1,
            ChangeTag::Insert => {}
        }
    }
    (matched, total)
}

/// Conformance runner in the shared `CoverageResult` form (mirrors `run_semantic_typescript`),
/// so `AppArgs::run_tool` prints and snapshots it exactly like the other suites. A file passes
/// when oxc reproduces every `>` symbol line in the baseline; the remaining gap is the
/// type-checker surface (members, `this`, qualified names, lib globals). Use `--diff` to inspect
/// a mismatch.
pub fn run_symbols_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter_map(|f| {
            if f.should_fail {
                return None;
            }
            let stem = Path::new(&f.path).file_stem()?.to_string_lossy().into_owned();
            let baseline_path = workspace_root()
                .join("typescript/tests/baselines/reference")
                .join(format!("{stem}.symbols"));
            // Only score files that ship a `.symbols` baseline.
            let baseline = std::fs::read_to_string(&baseline_path).ok()?;

            let header = header_path(&f.path);
            let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                generate_for_file(f, &header)
            })) {
                Err(_) => {
                    TestResult::ParseError("Panicked while generating symbols".to_string(), true)
                }
                Ok(generated) => {
                    let (matched, total) = score(&baseline, &generated);
                    if matched == total {
                        TestResult::Passed
                    } else {
                        TestResult::Mismatch("Symbol Mismatch", generated, baseline)
                    }
                }
            };
            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}
