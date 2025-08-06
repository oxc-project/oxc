/**
 * @oxc-project/oxc - MCP Server for Oxc
 * 
 * This package provides a Model Context Protocol (MCP) server for Oxc,
 * enabling AI assistants to interact with Oxc's JavaScript/TypeScript
 * parsing, linting, and analysis capabilities.
 */

export { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
export { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
export { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";

// Re-export zod for schema definitions
export { z } from "zod";

// Version and metadata
export const version = "0.1.0";
export const name = "@oxc-project/oxc";

/**
 * Default configuration for the Oxc MCP server
 */
export const defaultConfig = {
  name: "@oxc-project/oxc",
  version: "0.1.0",
  capabilities: {
    tools: {},
    resources: {},
    prompts: {}
  }
};

// Type definitions for Oxc-specific operations
export interface CodeAnalysis {
  language: string;
  codeLength: number;
  lineCount: number;
  hasImports: boolean;
  hasExports: boolean;
  functions: number;
  arrows: number;
}

export interface LintResult {
  issues: string[];
  hasErrors: boolean;
  hasWarnings: boolean;
}

export interface OxcServerOptions {
  enableParser?: boolean;
  enableLinter?: boolean;
  enableAnalysis?: boolean;
  customRules?: string[];
}