use anyhow::{anyhow, Result};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use serde_json::{json, Value};
use std::path::Path;

/// Execute a tool with the given arguments
pub async fn execute_tool(name: &str, arguments: &Option<Value>) -> Result<String> {
    match name {
        "lint_code" => lint_code(arguments).await,
        "parse_code" => parse_code(arguments).await,
        "analyze_code" => analyze_code(arguments).await,
        "format_code" => format_code(arguments).await,
        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}

/// Lint JavaScript/TypeScript code using Oxc linter (simplified version)
async fn lint_code(arguments: &Option<Value>) -> Result<String> {
    let args = arguments.as_ref().ok_or_else(|| anyhow!("Missing arguments"))?;
    
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'code' argument"))?;
    
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("input.js");

    // Determine source type from filename
    let source_type = SourceType::from_path(Path::new(filename))
        .unwrap_or_else(|_| SourceType::default().with_module(true));

    let allocator = Allocator::default();
    
    // Parse the code first
    let parser = Parser::new(&allocator, code, source_type);
    let parser_result = parser.parse();

    if !parser_result.errors.is_empty() {
        let mut result = json!({
            "success": false,
            "parse_errors": []
        });
        
        for error in &parser_result.errors {
            result["parse_errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "message": error.to_string(),
                    "severity": "error"
                }));
        }
        
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // For now, just return parse success - full linting requires more complex setup
    let result = json!({
        "success": true,
        "message": "Code parsed successfully. Full linting functionality requires additional configuration.",
        "diagnostics": [],
        "diagnostics_count": 0,
        "filename": filename,
        "note": "This is a simplified implementation. The full linter requires proper configuration setup."
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

/// Parse JavaScript/TypeScript code and return AST information
async fn parse_code(arguments: &Option<Value>) -> Result<String> {
    let args = arguments.as_ref().ok_or_else(|| anyhow!("Missing arguments"))?;
    
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'code' argument"))?;
    
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("input.js");

    let source_type_arg = args.get("source_type").and_then(|v| v.as_str());
    
    // Determine source type
    let mut source_type = SourceType::from_path(Path::new(filename))
        .unwrap_or_else(|_| SourceType::default().with_module(true));
    
    if let Some(st) = source_type_arg {
        match st {
            "script" => source_type = source_type.with_script(true),
            "module" => source_type = source_type.with_module(true),
            _ => return Err(anyhow!("Invalid source_type: {}", st)),
        }
    }

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, code, source_type);
    let parser_result = parser.parse();

    if !parser_result.errors.is_empty() {
        let mut result = json!({
            "success": false,
            "errors": []
        });
        
        for error in &parser_result.errors {
            result["errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "message": error.to_string(),
                    "severity": "error"
                }));
        }
        
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // Generate basic AST info (avoiding full serialization for performance)
    let program = &parser_result.program;
    
    let result = json!({
        "success": true,
        "ast_info": {
            "type": "Program",
            "source_type": {
                "is_script": source_type.is_script(),
                "is_module": source_type.is_module(),
                "is_typescript": source_type.is_typescript(),
                "is_jsx": source_type.is_jsx()
            },
            "body_length": program.body.len(),
            "directives_length": program.directives.len(),
            "has_hashbang": program.hashbang.is_some(),
        },
        "filename": filename,
        "code_length": code.len(),
        "parse_time_info": "Fast parsing completed successfully"
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

/// Perform semantic analysis on JavaScript/TypeScript code
async fn analyze_code(arguments: &Option<Value>) -> Result<String> {
    let args = arguments.as_ref().ok_or_else(|| anyhow!("Missing arguments"))?;
    
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'code' argument"))?;
    
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("input.js");

    let source_type = SourceType::from_path(Path::new(filename))
        .unwrap_or_else(|_| SourceType::default().with_module(true));

    let allocator = Allocator::default();
    
    // Parse the code
    let parser = Parser::new(&allocator, code, source_type);
    let parser_result = parser.parse();

    if !parser_result.errors.is_empty() {
        let mut result = json!({
            "success": false,
            "parse_errors": []
        });
        
        for error in &parser_result.errors {
            result["parse_errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "message": error.to_string()
                }));
        }
        
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // Build semantic model
    let program = allocator.alloc(parser_result.program);
    let semantic_result = SemanticBuilder::new()
        .build(program);

    if !semantic_result.errors.is_empty() {
        let mut result = json!({
            "success": false,
            "semantic_errors": []
        });
        
        for error in &semantic_result.errors {
            result["semantic_errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "message": error.to_string()
                }));
        }
        
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    let _semantic = &semantic_result.semantic;
    
    // Gather semantic analysis information (simplified to avoid API issues)
    let result = json!({
        "success": true,
        "analysis": {
            "semantic_analysis": "completed",
            "diagnostics": semantic_result.errors.len(),
            "note": "Detailed symbol and scope information available but not exposed in this simplified implementation."
        },
        "filename": filename,
        "source_info": {
            "is_typescript": source_type.is_typescript(),
            "is_jsx": source_type.is_jsx(),
            "is_module": source_type.is_module(),
        }
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

/// Format JavaScript/TypeScript code (basic implementation using codegen)
async fn format_code(arguments: &Option<Value>) -> Result<String> {
    let args = arguments.as_ref().ok_or_else(|| anyhow!("Missing arguments"))?;
    
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'code' argument"))?;
    
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("input.js");

    let source_type = SourceType::from_path(Path::new(filename))
        .unwrap_or_else(|_| SourceType::default().with_module(true));

    let allocator = Allocator::default();
    
    // Parse the code
    let parser = Parser::new(&allocator, code, source_type);
    let parser_result = parser.parse();

    if !parser_result.errors.is_empty() {
        let mut result = json!({
            "success": false,
            "errors": []
        });
        
        for error in &parser_result.errors {
            result["errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "message": error.to_string()
                }));
        }
        
        return Ok(serde_json::to_string_pretty(&result)?);
    }

    // Use codegen to regenerate the code (basic formatting)
    let program = &parser_result.program;
    let formatted_code = Codegen::new().build(program).code;
    
    let result = json!({
        "success": true,
        "formatted_code": formatted_code,
        "original_length": code.len(),
        "formatted_length": formatted_code.len(),
        "filename": filename,
        "note": "Basic formatting using Oxc codegen. Full formatter is work in progress."
    });

    Ok(serde_json::to_string_pretty(&result)?)
}