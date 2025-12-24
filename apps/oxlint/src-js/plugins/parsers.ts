/**
 * Custom parser support for oxlint.
 *
 * This module provides the JS-side implementation for loading and invoking custom ESLint-compatible parsers.
 * Parsers can implement either:
 * - `parse(code, options)` returning an ESTree-compatible AST
 * - `parseForESLint(code, options)` returning `{ ast, scopeManager?, visitorKeys?, services? }`
 */

import { getErrorMessage } from "../utils/utils.ts";

/**
 * ESLint parser interface - `parse()` method signature
 */
interface EslintParseFunction {
  (code: string, options?: Record<string, unknown>): unknown; // Returns AST
}

/**
 * ESLint parser interface - `parseForESLint()` method signature
 */
interface EslintParseForESLintFunction {
  (code: string, options?: Record<string, unknown>): {
    ast: unknown;
    scopeManager?: unknown;
    visitorKeys?: Record<string, string[]>;
    services?: Record<string, unknown>;
  };
}

/**
 * ESLint parser module structure
 */
interface EslintParser {
  parse?: EslintParseFunction;
  parseForESLint?: EslintParseForESLintFunction;
}

/**
 * Loaded parser instance
 */
interface LoadedParser {
  id: number;
  parser: EslintParser;
  hasParseForEslint: boolean;
}

// Map of parser ID to loaded parser instance
const loadedParsers = new Map<number, LoadedParser>();

// Counter for generating unique parser IDs
let nextParserId = 0;

/**
 * Result from loading a parser, returned to Rust
 */
interface LoadParserResult {
  parserId: number;
  hasParseForEslint: boolean;
}

/**
 * Result from parsing a file, returned to Rust
 */
interface ParseFileResult {
  astJson: string;
  scopeManagerJson: string | null;
  visitorKeysJson: string | null;
  servicesJson: string | null;
}

/**
 * Load a custom parser from a file URL.
 *
 * @param url - Absolute path of parser file as a `file://...` URL
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Parser details or error serialized to JSON string
 */
export async function loadParser(url: string, parserOptionsJson: string): Promise<string> {
  try {
    const parser = (await import(url)) as EslintParser;

    // Validate the parser has at least one of the required methods
    const hasParseForEslint = typeof parser.parseForESLint === "function";
    const hasParse = typeof parser.parse === "function";

    if (!hasParseForEslint && !hasParse) {
      throw new Error(
        "Parser must export either a `parse` or `parseForESLint` function. " +
          "See https://eslint.org/docs/developer-guide/working-with-custom-parsers for the expected interface.",
      );
    }

    const id = nextParserId++;
    const loadedParser: LoadedParser = {
      id,
      parser,
      hasParseForEslint,
    };

    loadedParsers.set(id, loadedParser);

    const result: LoadParserResult = {
      parserId: id,
      hasParseForEslint,
    };

    return JSON.stringify({ Success: result });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Parse a file using a custom parser.
 *
 * @param parserId - ID of the parser to use
 * @param filePath - Absolute path of file being parsed
 * @param sourceText - Source text content
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Parse result or error serialized to JSON string
 */
export function parseFile(
  parserId: number,
  filePath: string,
  sourceText: string,
  parserOptionsJson: string,
): string {
  try {
    const loadedParser = loadedParsers.get(parserId);
    if (!loadedParser) {
      throw new Error(`Parser with ID ${parserId} not found`);
    }

    // Parse options from JSON
    let parserOptions: Record<string, unknown> = {};
    if (parserOptionsJson && parserOptionsJson !== "null") {
      parserOptions = JSON.parse(parserOptionsJson) as Record<string, unknown>;
    }

    // Add common options that parsers typically expect
    parserOptions.filePath = filePath;

    let ast: unknown;
    let scopeManager: unknown = null;
    let visitorKeys: Record<string, string[]> | undefined;
    let services: Record<string, unknown> | undefined;

    const { parser, hasParseForEslint } = loadedParser;

    if (hasParseForEslint && parser.parseForESLint) {
      // Use parseForESLint to get full information
      const result = parser.parseForESLint(sourceText, parserOptions);
      ast = result.ast;
      scopeManager = result.scopeManager;
      visitorKeys = result.visitorKeys;
      services = result.services;
    } else if (parser.parse) {
      // Fall back to simple parse
      ast = parser.parse(sourceText, parserOptions);
    } else {
      // This shouldn't happen if loadParser validated correctly
      throw new Error("Parser has no parse method");
    }

    const result: ParseFileResult = {
      astJson: JSON.stringify(ast),
      scopeManagerJson: scopeManager ? JSON.stringify(scopeManager) : null,
      visitorKeysJson: visitorKeys ? JSON.stringify(visitorKeys) : null,
      servicesJson: services ? JSON.stringify(services) : null,
    };

    return JSON.stringify({ Success: result });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}
