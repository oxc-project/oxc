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
  const { buffer, offset } = serializeEstreeToBuffer(astWithHints, code);

  return {
    buffer,
    estreeOffset: offset,
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
  // Simple recursive walker to add hints
  function walk(node: any, parent: any, prop: string | null): void {
    if (!node || typeof node !== 'object') return;
    
    if (Array.isArray(node)) {
      for (const item of node) {
        walk(item, parent, null);
      }
      return;
    }
    
    if (node.type === 'Identifier') {
      // Determine identifier kind based on parent context
      const kind = inferIdentifierKind(node, parent, prop);
      if (kind) {
        (node as any)._oxc_identifierKind = kind;
      }
    }
    
    // Recursively walk all properties
    for (const [key, value] of Object.entries(node)) {
      if (key === 'type' || key === '_oxc_identifierKind') continue;
      if (value && typeof value === 'object') {
        walk(value, node, key);
      }
    }
  }
  
  // Start walking from root
  walk(ast, null, null);
  return ast;
}

/**
 * Infer identifier kind from context.
 * This mirrors the Rust-side logic for consistency.
 */
function inferIdentifierKind(
  node: ESTree.Identifier,
  parent: any,
  prop: string | null,
): 'binding' | 'reference' | 'name' | 'label' | null {
  if (!parent) return 'reference';
  
  switch (parent.type) {
    case 'VariableDeclarator':
      if (prop === 'id') return 'binding';
      break;
    case 'FunctionDeclaration':
    case 'FunctionExpression':
    case 'ClassDeclaration':
    case 'ClassExpression':
      if (prop === 'id') return 'binding';
      break;
    case 'MemberExpression':
      if (prop === 'property' && !parent.computed) return 'name';
      break;
    case 'LabeledStatement':
    case 'BreakStatement':
    case 'ContinueStatement':
      if (prop === 'label') return 'label';
      break;
    case 'Property':
      if (prop === 'key' && !parent.computed) return 'name';
      if (prop === 'value' && parent.shorthand) return 'binding';
      break;
    case 'ObjectPattern':
    case 'ArrayPattern':
    case 'AssignmentPattern':
      return 'binding';
    case 'CatchClause':
      if (prop === 'param') return 'binding';
      break;
    case 'ForInStatement':
    case 'ForOfStatement':
      if (prop === 'left') return 'binding';
      break;
  }
  
  // Default to reference (safest fallback)
  return 'reference';
}

/**
 * Serialize ESTree AST to a buffer for raw transfer.
 *
 * For MVP, we use JSON serialization. This can be optimized to binary format later.
 *
 * Buffer layout:
 * - [0-4]: Length of JSON string (u32, little-endian)
 * - [4-N]: JSON string (UTF-8 encoded)
 * - [N-N+4]: Offset where JSON starts (for consistency with raw transfer)
 *
 * @param ast - ESTree Program AST
 * @param sourceText - Original source code
 * @returns Object with buffer and offset where ESTree data starts
 */
function serializeEstreeToBuffer(
  ast: ESTree.Program,
  sourceText: string,
): { buffer: Uint8Array; offset: number } {
  // Serialize ESTree AST to JSON
  const jsonString = JSON.stringify(ast);
  const jsonBytes = new TextEncoder().encode(jsonString);
  
  // Allocate buffer: 4 bytes for length + JSON data + 4 bytes for offset
  const bufferSize = 4 + jsonBytes.length + 4;
  const buffer = new Uint8Array(bufferSize);
  
  // Write JSON length (u32, little-endian)
  const view = new DataView(buffer.buffer);
  view.setUint32(0, jsonBytes.length, true);
  
  // Write JSON data
  buffer.set(jsonBytes, 4);
  
  // Write offset where JSON starts (4)
  view.setUint32(4 + jsonBytes.length, 4, true);
  
  return {
    buffer,
    offset: 4, // JSON starts after length field
  };
}

