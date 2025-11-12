/**
 * Custom ESLint parser integration for oxlint.
 *
 * This module handles loading and executing custom ESLint parsers,
 * and serializing ESTree ASTs to raw transfer buffers for efficient
 * transfer to Rust.
 */

import { pathToFileURL } from 'node:url';

import type * as ESTree from '../generated/types.d.ts';
import { getErrorMessage } from './utils.js';
import { stripCustomNodes } from './strip-nodes.js';

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

// Absolute paths of parsers which have been loaded
const registeredParserPaths = new Set<string>();

// Parser instances indexed by path
const registeredParsers = new Map<string, CustomParser>();

// Parser details returned to Rust
interface ParserDetails {
  // Parser name
  name: string;
  // Path used to load the parser
  path: string;
}

/**
 * Load a custom parser module.
 *
 * Main logic is in separate function `loadCustomParserImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param path - Absolute path of parser file
 * @param packageName - Optional package name from package.json (fallback if parser.meta.name is missing)
 * @returns JSON result
 */
export async function loadCustomParser(path: string, packageName?: string): Promise<string> {
  try {
    const res = await loadCustomParserImpl(path, packageName);
    return JSON.stringify({ Success: res });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Load a custom parser module.
 *
 * @param path - Absolute path of parser file
 * @param packageName - Optional package name from package.json (fallback if parser.meta.name is missing)
 * @returns - Parser details
 * @throws {Error} If parser has already been registered
 * @throws {TypeError} If parser doesn't match CustomParser interface
 * @throws {*} If parser throws an error during import
 */
async function loadCustomParserImpl(path: string, packageName?: string): Promise<ParserDetails> {
  if (registeredParserPaths.has(path)) {
    throw new Error('This parser has already been registered. This is a bug in Oxlint. Please report it.');
  }

  // Import the parser module
  // Parsers can export either:
  // 1. A default export that is the parser object/function
  // 2. Named exports (parse, parseForESLint)
  const module = await import(pathToFileURL(path).href);

  let parser: CustomParser;

  // Check for default export first (most common)
  if (module.default) {
    const defaultExport = module.default;

    // If default is a function, it's the parse function
    if (typeof defaultExport === 'function') {
      parser = {
        parse: defaultExport,
        parseForESLint: module.parseForESLint,
      };
    } else if (typeof defaultExport === 'object' && defaultExport !== null) {
      // If default is an object, it should have parse/parseForESLint methods
      parser = {
        parse: defaultExport.parse,
        parseForESLint: defaultExport.parseForESLint,
      };
    } else {
      throw new TypeError('Parser default export must be a function or object with parse method');
    }
  } else {
    // No default export, look for named exports
    if (typeof module.parse !== 'function') {
      throw new TypeError('Parser must export a `parse` function (either as default or named export)');
    }
    parser = {
      parse: module.parse,
      parseForESLint: module.parseForESLint,
    };
  }

  // Validate parser interface
  if (typeof parser.parse !== 'function') {
    throw new TypeError('Parser must have a `parse` method that is a function');
  }

  if (parser.parseForESLint !== undefined && typeof parser.parseForESLint !== 'function') {
    throw new TypeError('Parser `parseForESLint` method must be a function if provided');
  }

  registeredParserPaths.add(path);
  registeredParsers.set(path, parser);

  // Get parser name from parser.meta.name, or fall back to package name from package.json
  const parserName = (parser as any).meta?.name ?? packageName ?? path;

  return { name: parserName, path };
}

/**
 * Get a loaded parser by path.
 *
 * @param path - Absolute path of parser file
 * @returns The parser instance, or undefined if not loaded
 */
export function getCustomParser(path: string): CustomParser | undefined {
  return registeredParsers.get(path);
}

/**
 * Parse source code with a custom parser and prepare for raw transfer.
 *
 * This version produces a STRIPPED AST for Rust rules (custom nodes removed).
 * For JS plugins, use parseWithCustomParserFull to get the unstripped AST.
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
  const result = parser.parseForESLint ? parser.parseForESLint(code, options) : { ast: parser.parse(code, options) };

  // Strip custom nodes for Rust rules (dual-path architecture)
  const { ast: strippedAst } = stripCustomNodes(result.ast, {
    preserveLocations: true,
    replacementComment: 'Custom node removed for standard ESTree processing',
  });

  // Add hints to identifiers (Strategy 3 from plan)
  const astWithHints = addOxcHints(strippedAst);

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
 * Parse source code with a custom parser and prepare FULL (unstripped) AST.
 *
 * This version is for JS plugins that need to see framework-specific nodes.
 * For Rust rules, use parseWithCustomParser which strips custom nodes.
 *
 * @param parser - The custom parser instance
 * @param code - Source code to parse
 * @param options - Parser options
 * @returns Result containing buffer and metadata
 */
export function parseWithCustomParserFull(
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
  const result = parser.parseForESLint ? parser.parseForESLint(code, options) : { ast: parser.parse(code, options) };

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
  node: any, // ESTree Identifier from external parsers (not oxc's specific types)
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
function serializeEstreeToBuffer(ast: ESTree.Program, sourceText: string): { buffer: Uint8Array; offset: number } {
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
