import Tinypool from "tinypool";
import type { Options } from "prettier";
import type { FormatEmbeddedCodeArgs, FormatFileArgs } from "./prettier-worker.ts";

// Worker pool for parallel Prettier formatting
// Used by each exported function
let pool: Tinypool | null = null;

type InitResult = string[];
let initResultCache: InitResult | null = null;

// ---

/**
 * Setup worker pool for Prettier formatting.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param numThreads - Number of worker threads to use (same as Rayon thread count)
 * @returns Array of loaded plugin's `languages` info
 */
export async function initExternalFormatter(numThreads: number): Promise<InitResult> {
  // NOTE: When called from CLI, it's only called once at the beginning.
  // However, when called via API, like `format(fileName, code)`, it may be called multiple times.
  // Therefore, allow it by returning cached result.
  if (initResultCache !== null) return initResultCache;

  // Initialize worker pool for parallel Prettier formatting
  pool = new Tinypool({
    filename: new URL("./prettier-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
  });

  // TODO: Plugins support
  // - Read `plugins` field
  // - Load plugins dynamically and parse `languages` field
  // - Map file extensions and filenames to Prettier parsers
  initResultCache = [];

  return initResultCache;
}

// ---

// Map template tag names to Prettier parsers
const TAG_TO_PARSER: Record<string, string> = {
  // CSS
  css: "css",
  styled: "css",
  // GraphQL
  gql: "graphql",
  graphql: "graphql",
  // HTML
  html: "html",
  // Markdown
  md: "markdown",
  markdown: "markdown",
};

/**
 * Format embedded code using Prettier.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param options - Prettier configuration as object
 * @param tagName - The template tag name (e.g., "css", "gql", "html")
 * @param code - The code to format
 * @returns Formatted code
 */
export async function formatEmbeddedCode(
  options: Options,
  tagName: string,
  code: string,
): Promise<string> {
  const parser = TAG_TO_PARSER[tagName];

  // Unknown tag, return original code
  if (!parser) {
    return code;
  }

  options.parser = parser;

  return pool!.run({ options, code } satisfies FormatEmbeddedCodeArgs, {
    name: "formatEmbeddedCode",
  });
}

// ---

/**
 * Format whole file content using Prettier.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param options - Prettier configuration as object
 * @param parserName - The parser name
 * @param fileName - The file name (e.g., "package.json")
 * @param code - The code to format
 * @returns Formatted code
 */
export async function formatFile(
  options: Options,
  parserName: string,
  fileName: string,
  code: string,
): Promise<string> {
  options.parser = parserName;
  options.filepath = fileName;

  return pool!.run({ options, code } satisfies FormatFileArgs, {
    name: "formatFile",
  });
}
