/// Integration test for ember-eslint-parser with oxc
///
/// This test verifies that GJS/GTS files parsed by ember-eslint-parser produce
/// valid stripped ESTree ASTs after removing custom Glimmer nodes.
use std::path::PathBuf;

#[test]
fn test_ember_gjs_stripped_ast_is_valid_estree() {
    // Load stripped AST from ember-eslint-parser
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ember-parser-test/sample.gjs.stripped.ast.json");

    let estree_json = std::fs::read_to_string(&test_file)
        .expect("Failed to read stripped GJS AST file");

    // Parse as JSON to verify it's valid
    let ast: serde_json::Value = serde_json::from_str(&estree_json)
        .expect("Stripped AST should be valid JSON");

    // Verify basic structure
    assert_eq!(ast["type"], "Program", "Root should be Program node");
    assert_eq!(ast["body"].as_array().unwrap().len(), 4, "Should have 4 top-level statements");

    // Check node types
    let body = ast["body"].as_array().unwrap();
    assert_eq!(body[0]["type"], "ImportDeclaration");
    assert_eq!(body[1]["type"], "ImportDeclaration");
    assert_eq!(body[2]["type"], "ImportDeclaration");
    assert_eq!(body[3]["type"], "ExportDefaultDeclaration");

    // Verify no Glimmer node types remain (check for node type patterns, not import paths or comments)
    assert!(!ast.to_string().contains("\"type\":\"Glimmer"), "Stripped AST should not contain Glimmer node types");

    println!("✅ GJS stripped AST is valid ESTree");
    println!("   - {} top-level statements", body.len());
}

#[test]
fn test_ember_gts_stripped_ast_is_valid_estree() {
    // Load stripped AST from ember-eslint-parser
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ember-parser-test/sample.gts.stripped.ast.json");

    let estree_json = std::fs::read_to_string(&test_file)
        .expect("Failed to read stripped GTS AST file");

    // Parse as JSON to verify it's valid
    let ast: serde_json::Value = serde_json::from_str(&estree_json)
        .expect("Stripped AST should be valid JSON");

    // Verify basic structure
    assert_eq!(ast["type"], "Program", "Root should be Program node");
    assert_eq!(ast["body"].as_array().unwrap().len(), 5, "Should have 5 top-level statements");

    // Check node types
    let body = ast["body"].as_array().unwrap();
    assert_eq!(body[0]["type"], "ImportDeclaration");
    assert_eq!(body[3]["type"], "TSInterfaceDeclaration");
    assert_eq!(body[4]["type"], "ExportDefaultDeclaration");

    // Verify no Glimmer node types remain (check for node type patterns, not import paths or comments)
    assert!(!ast.to_string().contains("\"type\":\"Glimmer"), "Stripped AST should not contain Glimmer node types");

    println!("✅ GTS stripped AST is valid ESTree");
    println!("   - {} top-level statements", body.len());
}

#[test]
#[ignore] // Run with: cargo test ember_parser -- --ignored
fn test_full_ast_contains_glimmer_nodes() {
    // This test verifies that the FULL (unstripped) AST contains Glimmer nodes
    // that would need to be handled by JS plugins
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ember-parser-test/sample.gjs.ast.json");

    let estree_json = std::fs::read_to_string(&test_file)
        .expect("Failed to read full GJS AST file");

    // Check that it contains Glimmer node types
    assert!(estree_json.contains("GlimmerTemplate"), "Full AST should contain GlimmerTemplate");
    assert!(estree_json.contains("GlimmerElementNode"), "Full AST should contain GlimmerElementNode");
    assert!(estree_json.contains("GlimmerMustacheStatement"), "Full AST should contain GlimmerMustacheStatement");

    println!("✅ Full AST contains Glimmer custom nodes");
    println!("   These would be passed to JS plugins for template-aware rules");
}
