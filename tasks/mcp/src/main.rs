use oxc_mcp::{EchoTool, OxcMcpServer, TextAnalyzerTool};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Oxc MCP Server Boilerplate");

    // Create and initialize the MCP server
    let server = OxcMcpServer::new();
    server.initialize().await?;

    // Display server information
    let info = server.get_server_info();
    println!("\n📊 Server Information:");
    for (key, value) in &info {
        println!("  {}: {}", key, value);
    }

    // Demonstrate tools
    println!("\n🔧 Demonstrating MCP Tools:");

    // Echo tool example
    println!("\n1. Echo Tool:");
    let echo_tool = EchoTool::new("Hello from Oxc MCP!".to_string());
    println!("   Result: {}", echo_tool.execute());

    // Text analyzer tool example
    println!("\n2. Text Analyzer Tool:");
    let sample_text = "Welcome to the Oxc project!\nThis is a Model Context Protocol server boilerplate.\nIt demonstrates basic MCP functionality.";
    let analyzer_tool = TextAnalyzerTool::new(sample_text.to_string());
    let analysis = analyzer_tool.execute();
    println!("   Analysis: {}", serde_json::to_string_pretty(&analysis)?);

    // Interactive mode
    println!("\n🎯 Interactive Mode - Try the tools yourself!");
    println!("Enter 'echo <message>' or 'analyze <text>' or 'quit' to exit:");

    let stdin = io::stdin();
    loop {
        let mut input = String::new();
        print!("\n> ");
        io::Write::flush(&mut io::stdout())?;

        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input == "quit" {
            break;
        }

        if let Some(message) = input.strip_prefix("echo ") {
            let tool = EchoTool::new(message.to_string());
            println!("   {}", tool.execute());
        } else if let Some(text) = input.strip_prefix("analyze ") {
            let tool = TextAnalyzerTool::new(text.to_string());
            let result = tool.execute();
            println!("   {}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("   Commands: 'echo <message>', 'analyze <text>', or 'quit'");
        }
    }

    println!("\n👋 Goodbye from Oxc MCP Server!");
    Ok(())
}
