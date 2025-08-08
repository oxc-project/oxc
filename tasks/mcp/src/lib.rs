use serde_json::Value;
use std::collections::HashMap;

/// A simple MCP server boilerplate demonstrating basic functionality
pub struct OxcMcpServer {
    /// Server metadata
    name: String,
    version: String,
}

impl OxcMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            name: "oxc-mcp-server".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Get server information
    pub fn get_server_info(&self) -> HashMap<String, Value> {
        let mut info = HashMap::new();
        info.insert("name".to_string(), Value::String(self.name.clone()));
        info.insert("version".to_string(), Value::String(self.version.clone()));
        info.insert("description".to_string(), Value::String(
            "An MCP server boilerplate integrated with Oxc tools".to_string()
        ));
        info
    }

    /// Initialize the server with basic capabilities
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing Oxc MCP Server...");
        println!("Server: {} v{}", self.name, self.version);
        Ok(())
    }
}

impl Default for OxcMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Example tool: Echo tool that simply returns the input
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EchoTool {
    pub message: String,
}

impl EchoTool {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn execute(&self) -> String {
        format!("Echo: {}", self.message)
    }
}

/// Example tool: Simple text analyzer
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextAnalyzerTool {
    pub text: String,
}

impl TextAnalyzerTool {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn execute(&self) -> serde_json::Value {
        let word_count = self.text.split_whitespace().count();
        let char_count = self.text.chars().count();
        let line_count = self.text.lines().count();

        serde_json::json!({
            "analysis": {
                "word_count": word_count,
                "character_count": char_count,
                "line_count": line_count,
                "text_length": self.text.len()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = OxcMcpServer::new();
        assert_eq!(server.name, "oxc-mcp-server");
        assert_eq!(server.version, "0.1.0");
    }

    #[test]
    fn test_echo_tool() {
        let tool = EchoTool::new("Hello, World!".to_string());
        assert_eq!(tool.execute(), "Echo: Hello, World!");
    }

    #[test]
    fn test_text_analyzer_tool() {
        let tool = TextAnalyzerTool::new("Hello world\nThis is a test".to_string());
        let result = tool.execute();
        
        assert_eq!(result["analysis"]["word_count"], 6);
        assert_eq!(result["analysis"]["line_count"], 2);
    }
}