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
export { z } from "zod";
export declare const version = "0.1.0";
export declare const name = "@oxc-project/oxc";
/**
 * Default configuration for the Oxc MCP server
 */
export declare const defaultConfig: {
    name: string;
    version: string;
    capabilities: {
        tools: {};
        resources: {};
        prompts: {};
    };
};
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
//# sourceMappingURL=index.d.ts.map