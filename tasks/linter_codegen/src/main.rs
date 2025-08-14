use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use syn::{visit::Visit, File, ImplItem, PatTupleStruct};

#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub name: String,
    pub plugin: String,
    pub path: PathBuf,
    pub ast_types: Vec<String>,
    pub any_node_type: bool,
}

pub struct AstKindVisitor {
    pub ast_kinds: HashSet<String>,
    pub has_return_early: bool,
    pub in_run_method: bool,
}

impl AstKindVisitor {
    fn new() -> Self {
        Self {
            ast_kinds: HashSet::new(),
            has_return_early: false,
            in_run_method: false,
        }
    }
}

impl<'ast> Visit<'ast> for AstKindVisitor {
    fn visit_impl_item(&mut self, impl_item: &'ast ImplItem) {
        if let ImplItem::Fn(method) = impl_item {
            if method.sig.ident == "run" {
                self.in_run_method = true;
                // Only analyze the first statement or expression to find the main pattern
                self.analyze_run_method_body(&method.block);
                self.in_run_method = false;
            }
        }
    }
}

impl AstKindVisitor {
    fn analyze_run_method_body(&mut self, block: &syn::Block) {
        // Look for the first pattern match that's likely the main conditional
        for stmt in &block.stmts {
            if self.find_main_ast_kind_pattern(stmt) {
                break; // Stop after finding the first meaningful pattern
            }
        }
    }

    fn find_main_ast_kind_pattern(&mut self, stmt: &syn::Stmt) -> bool {
        match stmt {
            syn::Stmt::Local(local) => {
                // Handle: let AstKind::Something(x) = node.kind() else { return; };
                if let syn::Pat::TupleStruct(pat) = &local.pat {
                    if self.extract_ast_kind_from_pattern(pat) {
                        return true;
                    }
                }
            }
            syn::Stmt::Expr(expr, _) => {
                // Handle: if let AstKind::Something(x) = node.kind() { ... }
                if let syn::Expr::If(if_expr) = expr {
                    if self.analyze_if_let_pattern(&if_expr.cond) {
                        return true;
                    }
                }
                // Handle: match node.kind() { AstKind::Something(..) => ... }
                if let syn::Expr::Match(match_expr) = expr {
                    self.analyze_match_arms(&match_expr.arms);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn analyze_if_let_pattern(&mut self, cond: &syn::Expr) -> bool {
        if let syn::Expr::Let(let_expr) = cond {
            if let syn::Pat::TupleStruct(pat) = let_expr.pat.as_ref() {
                return self.extract_ast_kind_from_pattern(pat);
            }
        }
        false
    }

    fn analyze_match_arms(&mut self, arms: &[syn::Arm]) {
        for arm in arms {
            if let syn::Pat::TupleStruct(pat) = &arm.pat {
                self.extract_ast_kind_from_pattern(pat);
            }
        }
    }

    fn extract_ast_kind_from_pattern(&mut self, pat: &PatTupleStruct) -> bool {
        if pat.path.segments.len() == 2 {
            if let (Some(first), Some(second)) = (pat.path.segments.first(), pat.path.segments.last()) {
                if first.ident == "AstKind" {
                    self.ast_kinds.insert(second.ident.to_string());
                    return true;
                }
            }
        }
        false
    }
}

pub fn analyze_rule_file(rule_path: &Path) -> Option<RuleInfo> {
    let content = fs::read_to_string(rule_path).ok()?;
    let syntax_tree: File = syn::parse_str(&content).ok()?;
    
    let mut visitor = AstKindVisitor::new();
    visitor.visit_file(&syntax_tree);

    // Extract rule name from path
    let rule_name = rule_path
        .file_stem()?
        .to_str()?
        .replace('-', "_");

    // Extract plugin name from path
    let plugin = rule_path
        .parent()?
        .file_name()?
        .to_str()?
        .to_string();

    let mut ast_types: Vec<String> = visitor.ast_kinds.into_iter().collect();
    ast_types.sort(); // Make output deterministic
    
    // If no AST kinds found, assume it processes all node types
    let any_node_type = ast_types.is_empty();

    Some(RuleInfo {
        name: rule_name,
        plugin,
        path: rule_path.to_path_buf(),
        ast_types,
        any_node_type,
    })
}

pub fn find_all_rules() -> Vec<RuleInfo> {
    let rules_dir = Path::new("crates/oxc_linter/src/rules");
    let mut rule_infos = Vec::new();
    
    if let Ok(entries) = fs::read_dir(rules_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // This is a plugin directory (eslint, typescript, etc.)
                if let Ok(plugin_entries) = fs::read_dir(&path) {
                    for plugin_entry in plugin_entries.flatten() {
                        let rule_path = plugin_entry.path();
                        if rule_path.is_file() && rule_path.extension().map_or(false, |ext| ext == "rs") {
                            if let Some(rule_info) = analyze_rule_file(&rule_path) {
                                rule_infos.push(rule_info);
                            }
                        } else if rule_path.is_dir() {
                            // Handle nested directories like no_unused_vars/
                            if let Ok(nested_entries) = fs::read_dir(&rule_path) {
                                for nested_entry in nested_entries.flatten() {
                                    let nested_rule_path = nested_entry.path();
                                    if nested_rule_path.is_file() && nested_rule_path.extension().map_or(false, |ext| ext == "rs") {
                                        if let Some(rule_info) = analyze_rule_file(&nested_rule_path) {
                                            rule_infos.push(rule_info);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    rule_infos
}

// Convert AstKind variant name to AstType enum value
pub fn ast_kind_to_ast_type(ast_kind: &str) -> String {
    ast_kind.to_string() // For now, they should match 1:1
}

pub fn generate_node_types_code(rules: &[RuleInfo]) -> String {
    let mut generated = String::new();
    
    generated.push_str("// Auto-generated code for rule AST node type optimization\n");
    generated.push_str("// DO NOT EDIT MANUALLY - run `just linter-codegen` to regenerate\n\n");
    generated.push_str("use oxc_ast::AstType;\n\n");
    
    // Generate constants for each rule with unique names
    for rule in rules {
        let plugin_pascal = to_pascal_case(&rule.plugin);
        let rule_name_pascal = to_pascal_case(&rule.name);
        let const_prefix = format!("{}{}", plugin_pascal, rule_name_pascal).to_uppercase();
        
        if rule.any_node_type {
            generated.push_str(&format!(
                "pub const {}_NODE_TYPES: &[AstType] = &[];\n",
                const_prefix
            ));
            generated.push_str(&format!(
                "pub const {}_ANY_NODE_TYPE: bool = true;\n\n",
                const_prefix
            ));
        } else {
            let ast_types: Vec<String> = rule.ast_types
                .iter()
                .map(|kind| format!("AstType::{}", ast_kind_to_ast_type(kind)))
                .collect();
            
            generated.push_str(&format!(
                "pub const {}_NODE_TYPES: &[AstType] = &[{}];\n",
                const_prefix,
                ast_types.join(", ")
            ));
            generated.push_str(&format!(
                "pub const {}_ANY_NODE_TYPE: bool = false;\n\n",
                const_prefix
            ));
        }
    }
    
    // Generate lookup functions
    generated.push_str("/// Get node types for a rule by its struct name\n");
    generated.push_str("pub fn get_node_types(rule_name: &str) -> &'static [AstType] {\n");
    generated.push_str("    match rule_name {\n");
    
    for rule in rules {
        let plugin_pascal = to_pascal_case(&rule.plugin);
        let rule_name_pascal = to_pascal_case(&rule.name);
        let const_prefix = format!("{}{}", plugin_pascal, rule_name_pascal).to_uppercase();
        generated.push_str(&format!(
            "        \"{}\" => {}_NODE_TYPES,\n",
            rule_name_pascal,
            const_prefix
        ));
    }
    
    generated.push_str("        _ => &[], // Fallback for unknown rules\n");
    generated.push_str("    }\n");
    generated.push_str("}\n\n");
    
    generated.push_str("/// Get any_node_type flag for a rule by its struct name\n");
    generated.push_str("pub fn get_any_node_type(rule_name: &str) -> bool {\n");
    generated.push_str("    match rule_name {\n");
    
    for rule in rules {
        let plugin_pascal = to_pascal_case(&rule.plugin);
        let rule_name_pascal = to_pascal_case(&rule.name);
        let const_prefix = format!("{}{}", plugin_pascal, rule_name_pascal).to_uppercase();
        generated.push_str(&format!(
            "        \"{}\" => {}_ANY_NODE_TYPE,\n",
            rule_name_pascal,
            const_prefix
        ));
    }
    
    generated.push_str("        _ => true, // Fallback for unknown rules - run on all nodes\n");
    generated.push_str("    }\n");
    generated.push_str("}\n");
    
    generated
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

fn main() {
    let rules = find_all_rules();
    println!("Found {} rules", rules.len());
    
    // Generate the constants file
    let generated_code = generate_node_types_code(&rules);
    let output_path = Path::new("crates/oxc_linter/src/generated/rule_node_types.rs");
    
    // Create directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }
    
    fs::write(output_path, generated_code).expect("Failed to write generated code");
    println!("Generated rule node types in {:?}", output_path);
    
    // Print some statistics
    let rules_with_specific_types = rules.iter().filter(|r| !r.any_node_type).count();
    let total_optimizable_types: usize = rules.iter()
        .filter(|r| !r.any_node_type)
        .map(|r| r.ast_types.len())
        .sum();
    
    println!("Rules with specific AST types: {} / {}", rules_with_specific_types, rules.len());
    println!("Total AST type mappings: {}", total_optimizable_types);
}