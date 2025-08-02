use std::collections::HashMap;

use anyhow::Result;
use log::{debug, info};
use rust_mcp_sdk::{Server, ServerInfo};
use rust_mcp_schema::{
    CallToolRequest, CallToolResult, GetPromptRequest, GetPromptResult, ListPromptsRequest,
    ListPromptsResult, ListResourcesRequest, ListResourcesResult, ListToolsRequest,
    ListToolsResult, Prompt, PromptMessage, ReadResourceRequest, ReadResourceResult, Resource,
    Role, SamplingMessageContent, TextContent, Tool,
};
use serde_json::{json, Value};

mod resources;
mod tools;

use resources::get_resource_content;
use tools::execute_tool;

/// Oxc Model Context Protocol Server
/// 
/// This server provides AI models with access to Oxc's JavaScript/TypeScript tooling capabilities
/// including linting, parsing, and code analysis.
pub struct OxcMcpServer {
    /// Information about this server
    server_info: ServerInfo,
}

impl OxcMcpServer {
    pub fn new() -> Self {
        Self {
            server_info: ServerInfo {
                name: "oxc-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

impl Default for OxcMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[rust_mcp_sdk::server]
impl Server for OxcMcpServer {
    fn get_server_info(&self) -> ServerInfo {
        self.server_info.clone()
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> Result<ListResourcesResult> {
        debug!("Listing resources");
        
        let resources = vec![
            Resource {
                uri: "oxc://linter/rules".to_string(),
                name: Some("Linter Rules".to_string()),
                description: Some("List of all available linter rules with descriptions".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "oxc://project/info".to_string(),
                name: Some("Project Information".to_string()),
                description: Some("Information about the Oxc project structure and capabilities".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
            Resource {
                uri: "oxc://ast/schema".to_string(),
                name: Some("AST Schema".to_string()),
                description: Some("Schema definition for Oxc's Abstract Syntax Tree".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ];

        Ok(ListResourcesResult { resources })
    }

    async fn read_resource(&self, request: ReadResourceRequest) -> Result<ReadResourceResult> {
        debug!("Reading resource: {}", request.uri);
        
        let content = get_resource_content(&request.uri).await?;
        
        Ok(ReadResourceResult {
            contents: vec![content],
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> Result<ListToolsResult> {
        debug!("Listing tools");
        
        let tools = vec![
            Tool {
                name: "lint_code".to_string(),
                description: Some("Lint JavaScript/TypeScript code using Oxc linter".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The JavaScript/TypeScript code to lint"
                        },
                        "filename": {
                            "type": "string",
                            "description": "Optional filename for the code (affects which rules are applied)"
                        }
                    },
                    "required": ["code"]
                }),
            },
            Tool {
                name: "parse_code".to_string(),
                description: Some("Parse JavaScript/TypeScript code and return AST".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The JavaScript/TypeScript code to parse"
                        },
                        "filename": {
                            "type": "string",
                            "description": "Optional filename for the code"
                        },
                        "source_type": {
                            "type": "string",
                            "enum": ["script", "module"],
                            "description": "Source type of the code"
                        }
                    },
                    "required": ["code"]
                }),
            },
            Tool {
                name: "analyze_code".to_string(),
                description: Some("Perform semantic analysis on JavaScript/TypeScript code".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The JavaScript/TypeScript code to analyze"
                        },
                        "filename": {
                            "type": "string",
                            "description": "Optional filename for the code"
                        }
                    },
                    "required": ["code"]
                }),
            },
            Tool {
                name: "format_code".to_string(),
                description: Some("Format JavaScript/TypeScript code using Oxc formatter".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The JavaScript/TypeScript code to format"
                        },
                        "filename": {
                            "type": "string",
                            "description": "Optional filename for the code"
                        }
                    },
                    "required": ["code"]
                }),
            },
        ];

        Ok(ListToolsResult { tools })
    }

    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        debug!("Calling tool: {}", request.name);
        
        let result = execute_tool(&request.name, &request.arguments).await?;
        
        Ok(CallToolResult {
            content: vec![SamplingMessageContent::TextContent(TextContent::new(
                result,
                None,
                None,
            ))],
            is_error: Some(false),
        })
    }

    async fn list_prompts(&self, _request: ListPromptsRequest) -> Result<ListPromptsResult> {
        debug!("Listing prompts");
        
        let prompts = vec![
            Prompt {
                name: "analyze_js_code".to_string(),
                description: Some("Analyze JavaScript/TypeScript code for issues and suggestions".to_string()),
                arguments: Some(vec![
                    json!({
                        "name": "code",
                        "description": "The JavaScript/TypeScript code to analyze",
                        "required": true
                    })
                ]),
            },
            Prompt {
                name: "explain_linter_rules".to_string(),
                description: Some("Explain linter rules and their purpose".to_string()),
                arguments: Some(vec![
                    json!({
                        "name": "rule_name",
                        "description": "Name of the linter rule to explain",
                        "required": false
                    })
                ]),
            },
            Prompt {
                name: "code_quality_review".to_string(),
                description: Some("Perform a comprehensive code quality review".to_string()),
                arguments: Some(vec![
                    json!({
                        "name": "code",
                        "description": "The JavaScript/TypeScript code to review",
                        "required": true
                    }),
                    json!({
                        "name": "focus_areas",
                        "description": "Specific areas to focus on (performance, security, maintainability, etc.)",
                        "required": false
                    })
                ]),
            },
        ];

        Ok(ListPromptsResult { prompts })
    }

    async fn get_prompt(&self, request: GetPromptRequest) -> Result<GetPromptResult> {
        debug!("Getting prompt: {}", request.name);
        
        let messages = match request.name.as_str() {
            "analyze_js_code" => {
                let code = request.arguments.as_ref()
                    .and_then(|args| args.get("code"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                vec![PromptMessage {
                    role: Role::User,
                    content: SamplingMessageContent::TextContent(TextContent::new(
                        format!(
                            "Please analyze this JavaScript/TypeScript code for potential issues, \
                            code quality problems, and improvement suggestions:\n\n```javascript\n{}\n```\n\n\
                            Focus on:\n\
                            - Syntax and semantic errors\n\
                            - Code style and formatting issues\n\
                            - Performance considerations\n\
                            - Security vulnerabilities\n\
                            - Best practices adherence\n\
                            - Maintainability concerns",
                            code
                        ),
                        None,
                        None,
                    )),
                }]
            },
            
            "explain_linter_rules" => {
                let rule_name = request.arguments.as_ref()
                    .and_then(|args| args.get("rule_name"))
                    .and_then(|v| v.as_str());
                
                let text = if let Some(rule) = rule_name {
                    format!(
                        "Please explain the '{}' linter rule:\n\n\
                        - What does this rule check for?\n\
                        - Why is this rule important?\n\
                        - Common violations and examples\n\
                        - How to fix violations\n\
                        - When might you want to disable this rule?",
                        rule
                    )
                } else {
                    "Please explain JavaScript/TypeScript linter rules in general:\n\n\
                    - What are linter rules and why are they important?\n\
                    - Categories of linter rules (syntax, style, best practices, etc.)\n\
                    - How to configure and customize linter rules\n\
                    - Common rules that help improve code quality\n\
                    - Balance between strictness and productivity".to_string()
                };
                
                vec![PromptMessage {
                    role: Role::User,
                    content: SamplingMessageContent::TextContent(TextContent::new(
                        text,
                        None,
                        None,
                    )),
                }]
            },
            
            "code_quality_review" => {
                let code = request.arguments.as_ref()
                    .and_then(|args| args.get("code"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let focus_areas = request.arguments.as_ref()
                    .and_then(|args| args.get("focus_areas"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("general code quality");
                
                vec![PromptMessage {
                    role: Role::User,
                    content: SamplingMessageContent::TextContent(TextContent::new(
                        format!(
                            "Please perform a comprehensive code quality review of this JavaScript/TypeScript code, \
                            focusing on {}:\n\n```javascript\n{}\n```\n\n\
                            Please provide:\n\
                            1. Overall assessment of code quality\n\
                            2. Specific issues found (with line numbers if possible)\n\
                            3. Improvement suggestions with examples\n\
                            4. Best practices recommendations\n\
                            5. Refactoring opportunities\n\
                            6. Performance considerations\n\
                            7. Security concerns (if any)\n\
                            8. Testing recommendations",
                            focus_areas, code
                        ),
                        None,
                        None,
                    )),
                }]
            },
            
            _ => {
                return Err(anyhow::anyhow!("Unknown prompt: {}", request.name));
            }
        };

        Ok(GetPromptResult {
            description: Some(format!("Prompt for {}", request.name)),
            messages,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("Starting Oxc MCP Server v{}", env!("CARGO_PKG_VERSION"));
    
    let server = OxcMcpServer::new();
    
    // Use stdio transport (standard for MCP servers)
    rust_mcp_sdk::transport::stdio::run_server(server).await?;
    
    Ok(())
}