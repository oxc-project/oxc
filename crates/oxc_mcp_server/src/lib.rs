//! # Oxc Model Context Protocol Server
//!
//! This crate provides a Model Context Protocol (MCP) server for the Oxc JavaScript/TypeScript toolchain.
//! The server exposes Oxc's parsing, linting, and analysis capabilities to AI models and other clients
//! that support the MCP protocol.
//!
//! ## Features
//!
//! ### Resources
//! - **Linter Rules**: Information about available linter rules and their descriptions
//! - **Project Info**: Overview of the Oxc project structure and capabilities  
//! - **AST Schema**: Schema definition for Oxc's Abstract Syntax Tree
//!
//! ### Tools
//! - **lint_code**: Lint JavaScript/TypeScript code using the Oxc linter
//! - **parse_code**: Parse code and return AST information
//! - **analyze_code**: Perform semantic analysis on code
//! - **format_code**: Format code using Oxc's codegen (basic formatting)
//!
//! ### Prompts
//! - **analyze_js_code**: Template for analyzing JavaScript/TypeScript code
//! - **explain_linter_rules**: Template for explaining linter rules
//! - **code_quality_review**: Template for comprehensive code quality reviews
//!
//! ## Usage
//!
//! The server runs as a standalone binary that communicates over stdio using the JSON-RPC 2.0 protocol,
//! following the MCP specification.
//!
//! ```bash
//! cargo run -p oxc_mcp_server
//! ```
//!
//! ## Example MCP Client Usage
//!
//! Once connected, clients can call tools like:
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "id": 1,
//!   "method": "tools/call",
//!   "params": {
//!     "name": "parse_code",
//!     "arguments": {
//!       "code": "const x = 1; var y = 2;",
//!       "filename": "example.js"
//!     }
//!   }
//! }
//! ```

pub mod resources;
pub mod tools;

pub use crate::resources::get_resource_content;
pub use crate::tools::execute_tool;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_parse_code_tool() {
        let args = Some(json!({
            "code": "const x = 1;",
            "filename": "test.js"
        }));
        
        let result = execute_tool("parse_code", &args).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["ast_info"]["body_length"], 1);
        assert_eq!(parsed["filename"], "test.js");
    }

    #[tokio::test]
    async fn test_analyze_code_tool() {
        let args = Some(json!({
            "code": "function test() { return 42; }",
            "filename": "test.js"
        }));
        
        let result = execute_tool("analyze_code", &args).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["filename"], "test.js");
    }

    #[tokio::test]
    async fn test_format_code_tool() {
        let args = Some(json!({
            "code": "const x=1;const y=2;",
            "filename": "test.js"
        }));
        
        let result = execute_tool("format_code", &args).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["success"], true);
        assert!(parsed["formatted_code"].as_str().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn test_lint_code_tool() {
        let args = Some(json!({
            "code": "var x = 1; console.log(x);",
            "filename": "test.js"
        }));
        
        let result = execute_tool("lint_code", &args).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["success"], true);
    }

    #[tokio::test]
    async fn test_invalid_tool() {
        let args = Some(json!({}));
        
        let result = execute_tool("invalid_tool", &args).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_info_resource() {
        let result = get_resource_content("oxc://project/info").await.unwrap();
        
        assert_eq!(result["uri"], "oxc://project/info");
        assert_eq!(result["mimeType"], "text/markdown");
        assert!(result["text"].as_str().unwrap().contains("Oxc Project Information"));
    }

    #[tokio::test]
    async fn test_linter_rules_resource() {
        let result = get_resource_content("oxc://linter/rules").await.unwrap();
        
        assert_eq!(result["uri"], "oxc://linter/rules");
        assert_eq!(result["mimeType"], "application/json");
        
        let text = result["text"].as_str().unwrap();
        let rules: Value = serde_json::from_str(text).unwrap();
        assert!(rules["rules"].is_object());
    }

    #[tokio::test]
    async fn test_ast_schema_resource() {
        let result = get_resource_content("oxc://ast/schema").await.unwrap();
        
        assert_eq!(result["uri"], "oxc://ast/schema");
        assert_eq!(result["mimeType"], "application/json");
        
        let text = result["text"].as_str().unwrap();
        let schema: Value = serde_json::from_str(text).unwrap();
        assert!(schema["core_types"].is_object());
    }

    #[tokio::test]
    async fn test_invalid_resource() {
        let result = get_resource_content("oxc://invalid/uri").await;
        
        assert!(result.is_err());
    }
}