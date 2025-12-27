/**
 * Custom parser support for oxlint.
 *
 * This module provides the JS-side implementation for loading and invoking custom ESLint-compatible parsers.
 * Parsers can implement either:
 * - `parse(code, options)` returning an ESTree-compatible AST
 * - `parseForESLint(code, options)` returning `{ ast, scopeManager?, visitorKeys?, services? }`
 */

import { getErrorMessage } from "../utils/utils.ts";
import { serializeScopeManagerFrom } from "./scope.ts";

/**
 * Stringify an object to JSON, handling circular references by replacing them with null.
 * Many ESLint parsers return ASTs with parent pointers that create circular references.
 */
function safeJsonStringify(obj: unknown): string {
  const seen = new WeakSet();
  return JSON.stringify(obj, (_key, value) => {
    if (typeof value === "object" && value !== null) {
      if (seen.has(value)) {
        // Circular reference found, replace with null
        return null;
      }
      seen.add(value);
    }
    return value;
  });
}

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
 * Cached parse result from parseForESLint().
 * Contains the AST and scopeManager that can't be serialized to JSON due to
 * circular references (parent pointers) and methods.
 */
interface CachedParseResult {
  ast: unknown;
  scopeManager: unknown;
}

/**
 * Cache for parse results from parseForESLint().
 * Keyed by file path, stores the AST and scopeManager until lintFileWithCustomAst retrieves them.
 * This allows passing these objects between parseFile and lintFile without JSON serialization,
 * preserving parent pointers and scopeManager methods.
 */
const parseResultCache = new Map<string, CachedParseResult>();

/**
 * Get and remove the cached parse result for a file path.
 * @param filePath - Absolute path of file
 * @returns The cached parse result if available, or undefined
 */
export function getAndClearCachedParseResult(filePath: string): CachedParseResult | undefined {
  const result = parseResultCache.get(filePath);
  parseResultCache.delete(filePath);
  return result;
}

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
    const imported = await import(url);
    // Handle both ES modules and CommonJS modules (where exports are under `default`)
    const parser = (imported.parseForESLint || imported.parse
      ? imported
      : imported.default ?? imported) as EslintParser;

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
    let visitorKeys: Record<string, string[]> | undefined;
    let scopeManager: unknown;

    const { parser, hasParseForEslint } = loadedParser;

    if (hasParseForEslint && parser.parseForESLint) {
      // Use parseForESLint to get full information
      const result = parser.parseForESLint(sourceText, parserOptions);
      ast = result.ast;
      visitorKeys = result.visitorKeys;
      scopeManager = result.scopeManager;
    } else if (parser.parse) {
      // Fall back to simple parse
      ast = parser.parse(sourceText, parserOptions);
    } else {
      // This shouldn't happen if loadParser validated correctly
      throw new Error("Parser has no parse method");
    }

    // Cache the AST and scopeManager for retrieval by lintFileWithCustomAst.
    // This avoids JSON serialization issues:
    // - AST has parent pointers (circular references) that would be lost
    // - scopeManager has methods and circular references
    parseResultCache.set(filePath, { ast, scopeManager });

    // Serialize the scope manager for Rust consumption (Phase 3 ESTree deserialization).
    // This extracts scope, variable, and reference information into a flat JSON structure
    // that can be used by Rust to inject external scope data into the linter.
    const serializedScopeManager = serializeScopeManagerFrom(scopeManager);

    // We still serialize the AST to JSON for the Rust side, but the JS side will use
    // the cached original AST with parent pointers intact.
    const result: ParseFileResult = {
      astJson: safeJsonStringify(ast),
      scopeManagerJson: serializedScopeManager ? JSON.stringify(serializedScopeManager) : null,
      visitorKeysJson: visitorKeys ? safeJsonStringify(visitorKeys) : null,
      servicesJson: null, // Not serializable (may have circular refs)
    };

    return JSON.stringify({ Success: result });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Parser interface for stripping custom syntax.
 *
 * If a parser implements this interface, it can be used for Phase 2 custom parser
 * support, where custom syntax is stripped and Rust rules can run on the result.
 */
interface EslintParserWithStrip extends EslintParser {
  /**
   * Strip custom syntax from the source, returning valid JavaScript.
   *
   * @param code - Source text content
   * @param options - Parser options
   * @returns Object containing stripped source and span mappings, or undefined if not supported
   */
  stripCustomSyntax?: (
    code: string,
    options?: Record<string, unknown>,
  ) =>
    | {
        /** The stripped source code (valid JavaScript) */
        source: string;
        /** Source type hints (optional) */
        sourceType?: {
          module?: boolean;
          typescript?: boolean;
          jsx?: boolean;
        };
        /** Span mappings from stripped positions to original positions */
        mappings: Array<{
          strippedStart: number;
          strippedEnd: number;
          originalStart: number;
          originalEnd: number;
        }>;
      }
    | undefined;
}

/**
 * Result from stripping a file, returned to Rust
 */
interface StripFileResult {
  source: string;
  sourceType?: {
    module?: boolean;
    typescript?: boolean;
    jsx?: boolean;
  };
  mappings: Array<{
    strippedStart: number;
    strippedEnd: number;
    originalStart: number;
    originalEnd: number;
  }>;
}

/**
 * Strip custom syntax from a file using a custom parser.
 *
 * This is used in Phase 2 to enable Rust rules on files with custom syntax.
 * The parser strips non-JS syntax and provides span mappings for diagnostic remapping.
 *
 * @param parserId - ID of the parser to use
 * @param filePath - Absolute path of file being stripped
 * @param sourceText - Source text content
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Strip result as JSON string, or null if not supported
 */
export function stripFile(
  parserId: number,
  filePath: string,
  sourceText: string,
  parserOptionsJson: string,
): string | null {
  try {
    const loadedParser = loadedParsers.get(parserId);
    if (!loadedParser) {
      throw new Error(`Parser with ID ${parserId} not found`);
    }

    const parser = loadedParser.parser as EslintParserWithStrip;

    // Check if the parser supports stripping
    if (typeof parser.stripCustomSyntax !== "function") {
      // Parser doesn't support stripping - return NotSupported
      // Note: Rust serde expects unit variants as null, not true
      return JSON.stringify({ NotSupported: null });
    }

    // Parse options from JSON
    let parserOptions: Record<string, unknown> = {};
    if (parserOptionsJson && parserOptionsJson !== "null") {
      parserOptions = JSON.parse(parserOptionsJson) as Record<string, unknown>;
    }

    // Add common options
    parserOptions.filePath = filePath;

    // Call the strip function
    const stripResult = parser.stripCustomSyntax(sourceText, parserOptions);

    if (!stripResult) {
      // Parser declined to strip this file
      // Note: Rust serde expects unit variants as null, not true
      return JSON.stringify({ NotSupported: null });
    }

    const result: StripFileResult = {
      source: stripResult.source,
      sourceType: stripResult.sourceType,
      mappings: stripResult.mappings,
    };

    return JSON.stringify({ Success: result });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}
