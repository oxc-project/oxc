import Tinypool from "tinypool";
import type { WorkerData, FormatEmbeddedCodeArgs, FormatFileArgs } from "./prettier-worker.ts";

// Worker pool for parallel Prettier formatting
// Used by each exported function
let pool: Tinypool | null = null;

type SetupResult = string[];
let setupCache: SetupResult | null = null;

// ---

/**
 * Setup Prettier configuration.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param configJSON - Prettier configuration as JSON string
 * @param numThreads - Number of worker threads to use (same as Rayon thread count)
 * @returns Array of loaded plugin's `languages` info
 * */
export async function setupConfig(configJSON: string, numThreads: number): Promise<SetupResult> {
  // NOTE: When called from CLI, it's only called once at the beginning.
  // However, when called via API, like `format(fileName, code)`, it may be called multiple times.
  // Therefore, allow it by returning cached result.
  if (setupCache !== null) return setupCache;

  const workerData: WorkerData = {
    // SAFETY: Always valid JSON constructed by Rust side
    prettierConfig: JSON.parse(configJSON),
  };

  // Initialize worker pool for parallel Prettier formatting
  // Pass config via workerData so all workers get it on initialization
  pool = new Tinypool({
    filename: new URL("./prettier-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
    workerData,
  });

  // TODO: Plugins support
  // - Read `plugins` field
  // - Load plugins dynamically and parse `languages` field
  // - Map file extensions and filenames to Prettier parsers
  setupCache = [];

  return setupCache;
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
 * @param tagName - The template tag name (e.g., "css", "gql", "html")
 * @param code - The code to format
 * @returns Formatted code
 */
export async function formatEmbeddedCode(tagName: string, code: string): Promise<string> {
  const parser = TAG_TO_PARSER[tagName];

  // Unknown tag, return original code
  if (!parser) {
    return code;
  }

  return pool!.run({ parser, code } satisfies FormatEmbeddedCodeArgs, {
    name: "formatEmbeddedCode",
  });
}

// ---

/**
 * Format whole file content using Prettier.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param parserName - The parser name
 * @param fileName - The file name (e.g., "package.json")
 * @param code - The code to format
 * @returns Formatted code
 */
export async function formatFile(
  parserName: string,
  fileName: string,
  code: string,
): Promise<string> {
  return pool!.run({ parserName, fileName, code } satisfies FormatFileArgs, {
    name: "formatFile",
  });
}
