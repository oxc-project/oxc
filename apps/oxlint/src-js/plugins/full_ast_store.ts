/**
 * Storage for full (unstripped) ASTs from custom parsers.
 *
 * When using custom parsers like ember-eslint-parser, we maintain two ASTs:
 * 1. Stripped AST (in buffer) - for Rust rules, has only standard ESTree nodes
 * 2. Full AST (stored here) - for JS plugin rules, has custom framework nodes
 *
 * This enables the dual-path architecture:
 * - Rust rules work fast on standard JavaScript/TypeScript
 * - JS plugin rules see framework-specific syntax (templates, etc.)
 */

// Map from file path to full AST with custom nodes
const fullAstStore = new Map<string, any>();

/**
 * Store a full AST for a file.
 *
 * Called after parsing with custom parser, before the AST is stripped.
 *
 * @param filePath - Absolute path of the file
 * @param ast - Full ESTree AST including custom nodes
 */
export function storeFullAst(filePath: string, ast: any): void {
  fullAstStore.set(filePath, ast);
}

/**
 * Get the full AST for a file, if one was stored.
 *
 * @param filePath - Absolute path of the file
 * @returns Full AST with custom nodes, or undefined if not available
 */
export function getFullAst(filePath: string): any | undefined {
  return fullAstStore.get(filePath);
}

/**
 * Check if a full AST is available for a file.
 *
 * @param filePath - Absolute path of the file
 * @returns True if full AST is stored
 */
export function hasFullAst(filePath: string): boolean {
  return fullAstStore.has(filePath);
}

/**
 * Clear the full AST for a file after linting.
 *
 * Called to free memory after a file has been linted.
 *
 * @param filePath - Absolute path of the file
 */
export function clearFullAst(filePath: string): void {
  fullAstStore.delete(filePath);
}

/**
 * Clear all stored full ASTs.
 *
 * Useful for testing or when starting a new lint run.
 */
export function clearAllFullAsts(): void {
  fullAstStore.clear();
}

/**
 * Get statistics about stored full ASTs.
 *
 * Useful for debugging and monitoring memory usage.
 *
 * @returns Object with store statistics
 */
export function getStoreStats(): { count: number; paths: string[] } {
  return {
    count: fullAstStore.size,
    paths: Array.from(fullAstStore.keys()),
  };
}
