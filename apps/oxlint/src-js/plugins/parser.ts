/**
 * Custom ESLint parser integration for oxlint.
 *
 * This module handles loading and executing custom ESLint parsers,
 * and serializing ESTree ASTs to raw transfer buffers for efficient
 * transfer to Rust.
 */

import type { ESTree } from '@typescript-eslint/types';

/**
 * Custom parser interface matching ESLint's parser specification.
 */
export interface CustomParser {
  /**
   * Parse source code and return ESTree AST.
   * This is the standard ESLint parser interface.
   */
  parse(code: string, options?: any): ESTree.Program;

  /**
   * Parse source code and return ESTree AST with additional ESLint-specific data.
   * This is the extended interface used by ESLint.
   */
  parseForESLint?(
    code: string,
    options?: any,
  ): {
    ast: ESTree.Program;
    services?: any;
    scopeManager?: any;
    visitorKeys?: any;
  };
}

/**
 * Load a custom parser module.
 *
 * @param parserPath - Path to the parser module (can be a package name or file path)
 * @param packageName - Optional package name for better error messages
 * @returns The loaded parser instance
 */
export async function loadCustomParser(
  parserPath: string,
  packageName?: string,
): Promise<CustomParser> {
  // TODO: Implement parser loading
  // This should:
  // 1. Resolve the parser path (similar to plugin loading)
  // 2. Load the module
  // 3. Extract the parser function/object
  // 4. Validate it matches the CustomParser interface
  throw new Error('Parser loading not yet implemented');
}

/**
 * Parse source code with a custom parser and prepare for raw transfer.
 *
 * @param parser - The custom parser instance
 * @param code - Source code to parse
 * @param options - Parser options
 * @returns Result containing buffer and metadata
 */
export function parseWithCustomParser(
  parser: CustomParser,
  code: string,
  options?: any,
): {
  buffer: Uint8Array;
  estreeOffset: number;
  services?: any;
  scopeManager?: any;
  visitorKeys?: any;
} {
  // Call parser
  const result = parser.parseForESLint
    ? parser.parseForESLint(code, options)
    : { ast: parser.parse(code, options) };

  // Add hints to identifiers (Strategy 3 from plan)
  const astWithHints = addOxcHints(result.ast);

  // Serialize ESTree AST to buffer
  // TODO: Implement efficient binary serialization
  // For now, this is a placeholder
  const buffer = serializeEstreeToBuffer(astWithHints, code);

  return {
    buffer,
    estreeOffset: 0, // TODO: Calculate actual offset
    services: result.services,
    scopeManager: result.scopeManager,
    visitorKeys: result.visitorKeys,
  };
}

/**
 * Add oxc-specific hints to ESTree AST nodes.
 *
 * This adds `_oxc_identifierKind` properties to Identifier nodes
 * to help with disambiguation during conversion. These hints are
 * ESLint-compatible (unknown properties are ignored).
 *
 * Format: `_oxc_identifierKind: "binding" | "reference" | "name" | "label"`
 * This structure is designed for potential future standardization.
 */
function addOxcHints(ast: ESTree.Program): ESTree.Program {
  // TODO: Implement AST traversal and hint addition
  // This should:
  // 1. Walk the ESTree AST
  // 2. For each Identifier node, determine its kind based on context
  // 3. Add `_oxc_identifierKind` property
  // 4. Return modified AST
  return ast;
}

/**
 * Serialize ESTree AST to a binary buffer for raw transfer.
 *
 * This creates an efficient binary representation of the ESTree AST
 * that can be read directly by Rust without JSON parsing overhead.
 *
 * @param ast - ESTree Program AST
 * @param sourceText - Original source code
 * @returns Buffer containing serialized AST
 */
function serializeEstreeToBuffer(
  ast: ESTree.Program,
  sourceText: string,
): Uint8Array {
  // TODO: Implement efficient binary serialization
  // This should:
  // 1. Allocate buffer (similar to raw transfer buffer size)
  // 2. Write source text at the start
  // 3. Write ESTree AST in binary format
  // 4. Write metadata at the end
  // For now, return empty buffer as placeholder
  const bufferSize = 2 * 1024 * 1024 * 1024; // 2 GiB (matching parser raw transfer)
  return new Uint8Array(bufferSize);
}

