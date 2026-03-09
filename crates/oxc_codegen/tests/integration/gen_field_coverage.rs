use std::fs;
use std::path::Path;

use phf::phf_map;
use rustc_hash::{FxHashMap, FxHashSet};
use syn::{self, Fields, File, FnArg, ImplItem, Item, Pat, Type, visit::Visit};

/// Fields that are internal/metadata and never need to be printed.
const GLOBAL_SKIP_FIELDS: &[&str] = &[
    "node_id",
    "scope_id",
    "symbol_id",
    "reference_id",
    "span",        // used for source mapping, always accessed separately
    "source_text", // metadata only
    "comments",    // handled separately in codegen
];

/// Type-specific skip list for fields intentionally not printed.
static TYPE_SPECIFIC_SKIPS: phf::Map<&'static str, &[&str]> = phf_map! {
    // `pure` is handled outside the impl block, in enum variant dispatch code
    "Function" => &["pure", "type"],
    "ArrowFunctionExpression" => &["pure"],
    // `type` is checked via `self.is_expression()` method
    "Class" => &["type"],
    // `expression` field is not printed; `directive` field is used instead (raw string)
    "Directive" => &["expression"],
    // Literal fields accessed via helper methods (e.g., `as_str()`, `raw_str()`)
    "BooleanLiteral" => &["value"],
    "StringLiteral" => &["raw"],
    "NumericLiteral" => &["raw", "base"],
    "BigIntLiteral" => &["raw", "base"],
    "RegExpLiteral" => &["raw"],
    "JSXText" => &["raw"],
    // `shorthand` is recomputed from key/value rather than using the AST field
    "BindingProperty" => &["shorthand"],
    // `kind` is printed by parent VariableDeclaration, not by VariableDeclarator
    "VariableDeclarator" => &["kind"],
    // `kind` is not printed directly (it describes the parameter list type)
    "FormalParameters" => &["kind"],
    // Rest parameters don't print decorators
    "FormalParameterRest" => &["decorators"],
    // Span-like metadata fields not used in codegen output
    "TSGlobalDeclaration" => &["global_span"],
    "TSThisParameter" => &["this_span"],
};

/// Normalize a raw identifier: `r#async` -> `async`.
fn normalize_ident(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

/// Collect all struct definitions from AST source files.
fn collect_ast_structs(ast_dir: &Path) -> FxHashMap<String, Vec<String>> {
    let mut structs = FxHashMap::default();
    for file_name in &["js.rs", "ts.rs", "jsx.rs", "literal.rs"] {
        let path = ast_dir.join(file_name);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let syntax = syn::parse_file(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
        for item in &syntax.items {
            if let Item::Struct(s) = item
                && let Fields::Named(named) = &s.fields
            {
                let fields: Vec<String> = named
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref().map(ToString::to_string))
                    .collect();
                if !fields.is_empty() {
                    structs.insert(s.ident.to_string(), fields);
                }
            }
        }
    }
    structs
}

/// Visitor that collects field accesses on specific variable names.
/// Tracks `var_name.field_name` patterns including through closures.
struct FieldAccessCollector<'a> {
    var_names: &'a [&'a str],
    fields: FxHashSet<String>,
}

impl<'ast> Visit<'ast> for FieldAccessCollector<'_> {
    fn visit_expr_field(&mut self, expr: &'ast syn::ExprField) {
        if let syn::Expr::Path(path) = &*expr.base
            && let Some(ident) = path.path.get_ident()
        {
            let ident_str = ident.to_string();
            if self.var_names.contains(&ident_str.as_str())
                && let syn::Member::Named(field_ident) = &expr.member
            {
                self.fields.insert(normalize_ident(&field_ident.to_string()).to_owned());
            }
        }
        syn::visit::visit_expr_field(self, expr);
    }
}

/// Extract the last type name from a type, unwrapping references.
fn extract_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Reference(r) => extract_type_name(&r.elem),
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

/// Collect enum variant -> inner type name mappings.
fn collect_enum_variant_types(syntax: &File) -> Vec<((String, String), String)> {
    let mut out = Vec::new();
    for item in &syntax.items {
        if let Item::Enum(e) = item {
            let enum_name = e.ident.to_string();
            for variant in &e.variants {
                if let Fields::Unnamed(fields) = &variant.fields
                    && fields.unnamed.len() == 1
                    && let Some(ty) = extract_type_name(&fields.unnamed[0].ty)
                {
                    out.push(((enum_name.clone(), variant.ident.to_string()), ty));
                }
            }
        }
    }
    out
}

/// Visitor that finds match arms binding variables from enum variants,
/// then tracks field accesses on those bindings.
struct MatchBindingCollector<'a> {
    variant_types: &'a FxHashMap<(String, String), String>,
    self_type_name: Option<&'a str>,
    fields: FxHashMap<String, FxHashSet<String>>,
}

impl<'ast> Visit<'ast> for MatchBindingCollector<'_> {
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let mut bindings: Vec<(String, String)> = Vec::new();
        self.extract_bindings(&arm.pat, &mut bindings);

        if !bindings.is_empty() {
            let var_names: Vec<&str> = bindings.iter().map(|(v, _)| v.as_str()).collect();
            let mut collector =
                FieldAccessCollector { var_names: &var_names, fields: FxHashSet::default() };
            if let Some((_, guard_expr)) = &arm.guard {
                collector.visit_expr(guard_expr);
            }
            collector.visit_expr(&arm.body);

            for (_, type_name) in &bindings {
                for field in &collector.fields {
                    self.fields.entry(type_name.clone()).or_default().insert(field.clone());
                }
            }
        }

        syn::visit::visit_arm(self, arm);
    }
}

impl MatchBindingCollector<'_> {
    fn extract_bindings(&self, pat: &syn::Pat, out: &mut Vec<(String, String)>) {
        match pat {
            Pat::TupleStruct(p) => {
                let segments = &p.path.segments;
                if segments.len() == 2
                    && p.elems.len() == 1
                    && let Some(enum_name) = if segments[0].ident == "Self" {
                        self.self_type_name.map(String::from)
                    } else {
                        Some(segments[0].ident.to_string())
                    }
                    && let Some(ty) =
                        self.variant_types.get(&(enum_name, segments[1].ident.to_string()))
                    && let Pat::Ident(id) = &p.elems[0]
                {
                    out.push((id.ident.to_string(), ty.clone()));
                }
            }
            Pat::Or(p) => {
                for case in &p.cases {
                    self.extract_bindings(case, out);
                }
            }
            _ => {}
        }
    }
}

/// Collect field accesses from function parameters that are AST types.
fn collect_fn_param_fields(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    block: &syn::Block,
    result: &mut FxHashMap<String, FxHashSet<String>>,
) {
    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
            && let Some(type_name) = extract_type_name(&pat_type.ty)
        {
            let param_name = pat_ident.ident.to_string();
            let names = [param_name.as_str()];
            let mut collector =
                FieldAccessCollector { var_names: &names, fields: FxHashSet::default() };
            collector.visit_block(block);
            if !collector.fields.is_empty() {
                result.entry(type_name).or_default().extend(collector.fields);
            }
        }
    }
}

/// Scan all source files in the codegen crate and collect field accesses.
fn collect_all_field_accesses(src_dir: &Path) -> FxHashMap<String, FxHashSet<String>> {
    let mut result: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();

    // Read and parse only files that reference AST types (skip options.rs, context.rs, etc.)
    let mut variant_types: FxHashMap<(String, String), String> = FxHashMap::default();
    let mut parsed_files: Vec<(std::path::PathBuf, File)> = Vec::new();

    for entry in fs::read_dir(src_dir).unwrap().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

        // Skip files that don't reference AST types — no point parsing them
        if !content.contains("oxc_ast") && !content.contains("Gen") {
            continue;
        }

        let syntax = syn::parse_file(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

        // Collect enum variant types in the same pass
        for ((e, v), ty) in collect_enum_variant_types(&syntax) {
            variant_types.insert((e, v), ty);
        }

        parsed_files.push((path, syntax));
    }

    // Process all parsed files
    for (path, syntax) in &parsed_files {
        let is_gen_rs = path.file_name().is_some_and(|n| n == "gen.rs");

        for item in &syntax.items {
            match item {
                Item::Impl(impl_item) => {
                    let self_type_name = if let syn::Type::Path(p) = &*impl_item.self_ty {
                        p.path.segments.last().map(|s| s.ident.to_string())
                    } else {
                        None
                    };

                    // For gen.rs: check if this is a Gen/GenExpr impl
                    if is_gen_rs {
                        let trait_name = impl_item
                            .trait_
                            .as_ref()
                            .and_then(|(_, p, _)| p.segments.last().map(|s| s.ident.to_string()));

                        if matches!(trait_name.as_deref(), Some("Gen" | "GenExpr"))
                            && let Some(type_name) = &self_type_name
                        {
                            // Collect `self.field` accesses in the impl body
                            let self_name = ["self"];
                            let mut collector = FieldAccessCollector {
                                var_names: &self_name,
                                fields: FxHashSet::default(),
                            };
                            for ii in &impl_item.items {
                                if let ImplItem::Fn(method) = ii {
                                    collector.visit_block(&method.block);
                                }
                            }
                            result.entry(type_name.clone()).or_default().extend(collector.fields);
                        }
                    }

                    // For all impl blocks: scan methods for param fields + match bindings
                    let self_name_ref = self_type_name.as_deref();
                    for ii in &impl_item.items {
                        if let ImplItem::Fn(method) = ii {
                            collect_fn_param_fields(&method.sig.inputs, &method.block, &mut result);
                            let mut mc = MatchBindingCollector {
                                variant_types: &variant_types,
                                self_type_name: self_name_ref,
                                fields: FxHashMap::default(),
                            };
                            mc.visit_block(&method.block);
                            for (ty, fields) in mc.fields {
                                result.entry(ty).or_default().extend(fields);
                            }
                        }
                    }
                }
                Item::Fn(func) => {
                    collect_fn_param_fields(&func.sig.inputs, &func.block, &mut result);
                }
                _ => {}
            }
        }
    }

    result
}

#[test]
fn test_gen_field_coverage() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let ast_dir = project_root.join("crates/oxc_ast/src/ast");
    let src_dir = manifest_dir.join("src");

    let ast_structs = collect_ast_structs(&ast_dir);
    let all_accesses = collect_all_field_accesses(&src_dir);

    let mut missing: Vec<String> = Vec::new();

    // NOTE: We iterate over types found in codegen (`all_accesses`), not all AST types.
    // Types whose fields are only accessed through chained expressions (e.g.,
    // `self.opening_element.name` for JSXOpeningElement) won't appear in `all_accesses`
    // and are silently skipped. This is a known limitation — the test only covers types
    // that have a direct Gen/GenExpr impl or are accessed via helper function parameters
    // or enum match arm bindings.
    for (type_name, accessed) in &all_accesses {
        let Some(struct_fields) = ast_structs.get(type_name) else {
            continue;
        };

        let per_type_skips = TYPE_SPECIFIC_SKIPS.get(type_name.as_str());

        for field in struct_fields {
            let bare = normalize_ident(field);
            if GLOBAL_SKIP_FIELDS.contains(&bare) {
                continue;
            }
            if per_type_skips.is_some_and(|s| s.contains(&bare)) {
                continue;
            }
            if !accessed.contains(bare) {
                missing.push(format!("{type_name}::{bare}"));
            }
        }
    }

    if !missing.is_empty() {
        missing.sort();
        panic!(
            "The following AST struct fields are not referenced in their Gen/GenExpr impl in gen.rs.\n\
             If a field is intentionally not printed, add it to TYPE_SPECIFIC_SKIPS in\n\
             crates/oxc_codegen/tests/integration/gen_field_coverage.rs:20\n\
             Missing fields:\n  - {}",
            missing.join("\n  - ")
        );
    }
}
