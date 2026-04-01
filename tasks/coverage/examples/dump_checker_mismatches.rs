#![expect(clippy::print_stdout)]
//! Dump ALL type mismatches for every failing checker_typescript test.
//!
//! Reads the snapshot file to find failing tests, runs the checker on each,
//! and outputs every mismatch (actual vs expected type).
//!
//! Usage:
//!   cargo run --release -p oxc_coverage --example dump_checker_mismatches 2>/dev/null > /tmp/mismatches.tsv

use std::fs;
use std::path::Path;

use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::{GetSpan as _, SourceType};
use oxc_checker::Checker;

fn main() {
    let workspace = find_workspace_root();
    let snap_path = workspace.join("tasks/coverage/snapshots/checker_typescript.snap");
    let baselines_dir = workspace.join("tasks/coverage/typescript/tests/baselines/reference");

    let snap_content = fs::read_to_string(&snap_path).expect("Cannot read snapshot file");

    // Extract failing test paths
    let failing: Vec<String> = snap_content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("checker: ").map(|s| s.to_string())
        })
        .collect();

    eprintln!("Processing {} failing tests...", failing.len());

    // Header
    println!("file\texpr\tactual_type\texpected_type\tmismatch_kind");

    let mut processed = 0;
    for rel_path in &failing {
        let full_path = workspace.join(rel_path);
        let source = match fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let stem = Path::new(rel_path).file_stem().unwrap().to_str().unwrap();
        let baseline_path = baselines_dir.join(format!("{stem}.types"));
        let baseline = match fs::read_to_string(&baseline_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let source_type = SourceType::from_path(&full_path).unwrap_or_default();
        let assertions = parse_types_baseline(&baseline);
        if assertions.is_empty() {
            continue;
        }

        // Parse → semantic → checker
        let type_arena = oxc_types::TypeArena::with_capacity(64);
        let project = oxc_project::Project::new(&type_arena);
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, source_type).parse();
        if !parsed.errors.is_empty() {
            println!("{rel_path}\t<parse_error>\t\t\tparse_error");
            processed += 1;
            continue;
        }
        let program = &parsed.program;
        let semantic = oxc::semantic::SemanticBuilder::new().build(program).semantic;
        let mut checker =
            Checker::new_with_host(&semantic, &type_arena, &project, String::new(), 1);

        // Collect computed types
        let actual = collect_checker_types(&mut checker, program, &source);

        // Match by (expression_text, occurrence_index).
        // Both baseline and walker emit in source order, so for each unique
        // expression text, the Nth baseline occurrence maps to the Nth walker
        // occurrence. This avoids the linear-scan displacement problem where
        // a single mismatch cascades into "missing" for all subsequent
        // occurrences of the same text.
        {
            use std::collections::HashMap;

            // Group actual entries by text, preserving order
            let mut actual_by_text: HashMap<&str, Vec<&str>> = HashMap::new();
            for (text, typ) in &actual {
                actual_by_text.entry(text.as_str()).or_default().push(typ.as_str());
            }

            // Track how many times we've seen each expression text in assertions
            let mut assertion_counts: HashMap<&str, usize> = HashMap::new();

            for (expr_text, expected_type) in &assertions {
                let idx = assertion_counts.entry(expr_text.as_str()).or_insert(0);
                let actual_types = actual_by_text.get(expr_text.as_str());

                match actual_types.and_then(|v| v.get(*idx)) {
                    Some(act_type) => {
                        if *act_type != expected_type.as_str() {
                            let kind = categorize_mismatch(act_type, expected_type);
                            println!(
                                "{rel_path}\t{expr_text}\t{act_type}\t{expected_type}\t{kind}"
                            );
                        }
                    }
                    None => {
                        println!(
                            "{rel_path}\t{expr_text}\t<missing>\t{expected_type}\tmissing_expression"
                        );
                    }
                }
                *idx += 1;
            }
        }

        processed += 1;
        if processed % 500 == 0 {
            eprintln!("  Processed {processed}/{}...", failing.len());
        }
    }

    eprintln!("Done. Processed {processed} tests.");
}

fn categorize_mismatch(actual: &str, expected: &str) -> &'static str {
    if actual == "any" && expected != "any" {
        return "got_any";
    }
    if actual == "error" || actual.contains("error") {
        return "got_error";
    }
    if actual == "never" && expected != "never" {
        return "got_never";
    }
    // Check for literal widening issues
    if (actual == "number" || actual == "string" || actual == "boolean")
        && expected != actual
    {
        if expected.starts_with('"')
            || expected.starts_with('\'')
            || expected.parse::<f64>().is_ok()
            || expected == "true"
            || expected == "false"
        {
            return "widened_literal";
        }
    }
    if expected.contains('|') && !actual.contains('|') {
        return "missing_union";
    }
    if expected.contains("typeof ") && !actual.contains("typeof ") {
        return "missing_typeof";
    }
    if expected.contains("=>") && !actual.contains("=>") {
        return "missing_function_type";
    }
    "type_mismatch"
}

fn parse_types_baseline(content: &str) -> Vec<(String, String)> {
    let mut assertions = Vec::new();
    let mut in_source = false;

    for line in content.lines() {
        if line.starts_with("=== ") && line.ends_with(" ===") {
            in_source = true;
            continue;
        }
        if line.starts_with("//// [") || !in_source {
            continue;
        }
        if let Some(rest) = line.strip_prefix('>') {
            let trimmed = rest.trim_start();
            if trimmed.starts_with(": ") && trimmed[2..].chars().all(|c| c == '^' || c == ' ') {
                continue;
            }
            if let Some((expr, typ)) = rest.split_once(" : ") {
                assertions.push((expr.to_string(), typ.to_string()));
            }
        }
    }

    assertions
}

fn collect_checker_types<'a>(
    checker: &mut Checker<'a>,
    program: &oxc::ast::ast::Program<'a>,
    source: &str,
) -> Vec<(String, String)> {
    use oxc::ast_visit::Visit;
    let mut walker = TypeCollectorVisitor { checker, source, results: Vec::new(), last_expression_type: None };
    walker.visit_program(program);
    walker.results
}

struct TypeCollectorVisitor<'a, 'b> {
    checker: &'b mut Checker<'a>,
    source: &'b str,
    results: Vec<(String, String)>,
    last_expression_type: Option<String>,
}

impl<'a> oxc::ast_visit::Visit<'a> for TypeCollectorVisitor<'a, '_> {
    fn visit_expression(&mut self, expr: &oxc::ast::ast::Expression<'a>) {
        let span = expr.span();
        if (span.start as usize) < self.source.len() && (span.end as usize) <= self.source.len() {
            let expr_text = &self.source[span.start as usize..span.end as usize];
            let type_id = self.checker.get_type_of_expression(expr, None);
            let type_str = self.checker.type_to_string(type_id);
            self.results.push((expr_text.to_string(), type_str.clone()));
            self.last_expression_type = Some(type_str);
        }
        oxc::ast_visit::walk::walk_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &oxc::ast::ast::StaticMemberExpression<'a>) {
        use oxc::span::GetSpan as _;
        if let Some(parent_type) = self.last_expression_type.take() {
            let prop_span = expr.property.span();
            if (prop_span.start as usize) < self.source.len()
                && (prop_span.end as usize) <= self.source.len()
            {
                let prop_text = &self.source[prop_span.start as usize..prop_span.end as usize];
                self.results.push((prop_text.to_string(), parent_type));
            }
        }
        oxc::ast_visit::walk::walk_static_member_expression(self, expr);
    }

    fn visit_binding_identifier(&mut self, id: &oxc::ast::ast::BindingIdentifier<'a>) {
        if let Some(symbol_id) = id.symbol_id.get() {
            use oxc::ast::AstKind;
            let node_id = self.checker.semantic().scoping().symbol_declaration(symbol_id);
            let node = self.checker.semantic().nodes().get_node(node_id);
            let type_id = match node.kind() {
                AstKind::Class(_)
                | AstKind::TSInterfaceDeclaration(_)
                | AstKind::TSTypeAliasDeclaration(_)
                | AstKind::TSEnumDeclaration(_) => {
                    self.checker.get_declared_type_of_symbol(symbol_id)
                }
                _ => self.checker.get_type_of_symbol(symbol_id),
            };
            self.results.push((id.name.to_string(), self.checker.type_to_string(type_id)));
        }
        oxc::ast_visit::walk::walk_binding_identifier(self, id);
    }


    fn visit_object_property(&mut self, prop: &oxc::ast::ast::ObjectProperty<'a>) {
        if prop.kind == oxc::ast::ast::PropertyKind::Init {
            if prop.key.static_name().is_some() {
                let key_span = prop.key.span();
                if (key_span.start as usize) < self.source.len()
                    && (key_span.end as usize) <= self.source.len()
                {
                    let prop_type = self.checker.get_type_of_expression(&prop.value, None);
                    let widened = self.checker.get_widened_literal_type(prop_type);
                    let key_text =
                        &self.source[key_span.start as usize..key_span.end as usize];
                    self.results
                        .push((key_text.to_string(), self.checker.type_to_string(widened)));
                }
            }
        }
        oxc::ast_visit::walk::walk_object_property(self, prop);
    }

    fn visit_property_definition(&mut self, prop: &oxc::ast::ast::PropertyDefinition<'a>) {
        if let Some(_name) = prop.key.static_name() {
            let key_span = prop.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let prop_type = if let Some(ann) = &prop.type_annotation {
                    self.checker.get_type_from_type_node(&ann.type_annotation)
                } else if let Some(init) = &prop.value {
                    self.checker.get_type_of_expression(init, None)
                } else {
                    self.checker.any_type
                };
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(prop_type)));
            }
        }
        oxc::ast_visit::walk::walk_property_definition(self, prop);
    }

    fn visit_method_definition(&mut self, method: &oxc::ast::ast::MethodDefinition<'a>) {
        if let Some(_name) = method.key.static_name() {
            let key_span = method.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let sig = self.checker.build_signature_from_function(&method.value);
                let method_type = self.checker.create_function_type(sig);
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(method_type)));
            }
        }
        oxc::ast_visit::walk::walk_method_definition(self, method);
    }

    fn visit_ts_property_signature(&mut self, prop: &oxc::ast::ast::TSPropertySignature<'a>) {
        if let Some(_name) = prop.key.static_name() {
            let key_span = prop.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let prop_type = if let Some(ann) = &prop.type_annotation {
                    self.checker.get_type_from_type_node(&ann.type_annotation)
                } else {
                    self.checker.any_type
                };
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(prop_type)));
            }
        }
        oxc::ast_visit::walk::walk_ts_property_signature(self, prop);
    }

    fn visit_ts_method_signature(&mut self, method: &oxc::ast::ast::TSMethodSignature<'a>) {
        if let Some(_name) = method.key.static_name() {
            let key_span = method.key.span();
            if (key_span.start as usize) < self.source.len()
                && (key_span.end as usize) <= self.source.len()
            {
                let sig = self.checker.build_signature_from_params(
                    &method.params,
                    method.return_type.as_deref(),
                );
                let method_type = self.checker.create_function_type(sig);
                let key_text =
                    &self.source[key_span.start as usize..key_span.end as usize];
                self.results
                    .push((key_text.to_string(), self.checker.type_to_string(method_type)));
            }
        }
        oxc::ast_visit::walk::walk_ts_method_signature(self, method);
    }

    fn visit_ts_enum_member(&mut self, member: &oxc::ast::ast::TSEnumMember<'a>) {
        let name_span = member.id.span();
        if (name_span.start as usize) < self.source.len()
            && (name_span.end as usize) <= self.source.len()
        {
            let member_type = if let Some(init) = &member.initializer {
                self.checker.get_type_of_expression(init, None)
            } else {
                self.checker.any_type
            };
            let name_text = &self.source[name_span.start as usize..name_span.end as usize];
            self.results
                .push((name_text.to_string(), self.checker.type_to_string(member_type)));
        }
        oxc::ast_visit::walk::walk_ts_enum_member(self, member);
    }
}

fn find_workspace_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join("Cargo.lock").exists() && dir.join("tasks").exists() {
            return dir;
        }
        if !dir.pop() {
            panic!("Could not find workspace root");
        }
    }
}
