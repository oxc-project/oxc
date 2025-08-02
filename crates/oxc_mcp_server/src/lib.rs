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
//! oxc_mcp_server
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
//!     "name": "lint_code",
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