use std::io::{self, BufRead, BufReader, Write};

use anyhow::Result;
use log::{debug, error, info};
use serde_json::{json, Value};

mod resources;
mod tools;

use resources::get_resource_content;
use tools::execute_tool;

/// Simple JSON-RPC based MCP server for Oxc
/// 
/// This server provides AI models with access to Oxc's JavaScript/TypeScript tooling capabilities
/// including linting, parsing, and code analysis.
pub struct OxcMcpServer;

impl OxcMcpServer {
    pub fn new() -> Self {
        Self
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: Value) -> Result<Value> {
        let method = request["method"].as_str().unwrap_or("");
        let params = &request["params"];
        let id = &request["id"];

        debug!("Handling request: method={}, id={:?}", method, id);

        let result = match method {
            "initialize" => self.handle_initialize(params).await,
            "resources/list" => self.handle_list_resources(params).await,
            "resources/read" => self.handle_read_resource(params).await,
            "tools/list" => self.handle_list_tools(params).await,
            "tools/call" => self.handle_call_tool(params).await,
            "prompts/list" => self.handle_list_prompts(params).await,
            "prompts/get" => self.handle_get_prompt(params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
        };

        match result {
            Ok(result) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            })),
            Err(e) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                }
            })),
        }
    }

    async fn handle_initialize(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "resources": {},
                "tools": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "oxc-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    async fn handle_list_resources(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "resources": [
                {
                    "uri": "oxc://linter/rules",
                    "name": "Linter Rules",
                    "description": "List of all available linter rules with descriptions",
                    "mimeType": "application/json"
                },
                {
                    "uri": "oxc://project/info",
                    "name": "Project Information",
                    "description": "Information about the Oxc project structure and capabilities",
                    "mimeType": "text/markdown"
                },
                {
                    "uri": "oxc://ast/schema",
                    "name": "AST Schema",
                    "description": "Schema definition for Oxc's Abstract Syntax Tree",
                    "mimeType": "application/json"
                }
            ]
        }))
    }

    async fn handle_read_resource(&self, params: &Value) -> Result<Value> {
        let uri = params["uri"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;
        
        let content = get_resource_content(uri).await?;
        
        Ok(json!({
            "contents": [content]
        }))
    }

    async fn handle_list_tools(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "tools": [
                {
                    "name": "lint_code",
                    "description": "Lint JavaScript/TypeScript code using Oxc linter",
                    "inputSchema": {
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
                    }
                },
                {
                    "name": "parse_code",
                    "description": "Parse JavaScript/TypeScript code and return AST information",
                    "inputSchema": {
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
                    }
                },
                {
                    "name": "analyze_code",
                    "description": "Perform semantic analysis on JavaScript/TypeScript code",
                    "inputSchema": {
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
                    }
                },
                {
                    "name": "format_code",
                    "description": "Format JavaScript/TypeScript code using Oxc codegen",
                    "inputSchema": {
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
                    }
                }
            ]
        }))
    }

    async fn handle_call_tool(&self, params: &Value) -> Result<Value> {
        let name = params["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;
        let arguments = params.get("arguments");
        
        let result = execute_tool(name, &arguments.cloned()).await?;
        
        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": result
                }
            ]
        }))
    }

    async fn handle_list_prompts(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "prompts": [
                {
                    "name": "analyze_js_code",
                    "description": "Analyze JavaScript/TypeScript code for issues and suggestions",
                    "arguments": [
                        {
                            "name": "code",
                            "description": "The JavaScript/TypeScript code to analyze",
                            "required": true
                        }
                    ]
                },
                {
                    "name": "explain_linter_rules",
                    "description": "Explain linter rules and their purpose",
                    "arguments": [
                        {
                            "name": "rule_name",
                            "description": "Name of the linter rule to explain",
                            "required": false
                        }
                    ]
                },
                {
                    "name": "code_quality_review",
                    "description": "Perform a comprehensive code quality review",
                    "arguments": [
                        {
                            "name": "code",
                            "description": "The JavaScript/TypeScript code to review",
                            "required": true
                        },
                        {
                            "name": "focus_areas",
                            "description": "Specific areas to focus on (performance, security, maintainability, etc.)",
                            "required": false
                        }
                    ]
                }
            ]
        }))
    }

    async fn handle_get_prompt(&self, params: &Value) -> Result<Value> {
        let name = params["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;
        let empty_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty_args);

        let messages = match name {
            "analyze_js_code" => {
                let code = arguments.get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                vec![json!({
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
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
                        )
                    }
                })]
            },
            
            "explain_linter_rules" => {
                let rule_name = arguments.get("rule_name")
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
                
                vec![json!({
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": text
                    }
                })]
            },
            
            "code_quality_review" => {
                let code = arguments.get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let focus_areas = arguments.get("focus_areas")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general code quality");
                
                vec![json!({
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
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
                        )
                    }
                })]
            },
            
            _ => {
                return Err(anyhow::anyhow!("Unknown prompt: {}", name));
            }
        };

        Ok(json!({
            "description": format!("Prompt for {}", name),
            "messages": messages
        }))
    }

    /// Run the MCP server on stdio
    pub async fn run(&self) -> Result<()> {
        info!("Starting Oxc MCP Server v{}", env!("CARGO_PKG_VERSION"));
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = BufReader::new(stdin);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            
            if line.is_empty() {
                continue;
            }

            debug!("Received: {}", line);

            match serde_json::from_str::<Value>(line) {
                Ok(request) => {
                    let response = self.handle_request(request).await?;
                    let response_str = serde_json::to_string(&response)?;
                    
                    debug!("Sending: {}", response_str);
                    writeln!(stdout, "{}", response_str)?;
                    stdout.flush()?;
                }
                Err(e) => {
                    error!("Failed to parse JSON: {}", e);
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": "Parse error"
                        }
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }
}

impl Default for OxcMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let server = OxcMcpServer::new();
    server.run().await?;
    
    Ok(())
}