import Tinypool from "tinypool";
import type { WorkerData, FormatEmbeddedCodeArgs, FormatFileArgs } from "./prettier-worker.ts";
import { createBatchSorter, type BatchSortContext } from "prettier-plugin-tailwindcss";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;

// Tailwind sorter (initialized lazily on first use)
let tailwindSorter: BatchSortContext | null = null;

// ---

/**
 * Setup Prettier configuration.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param configJSON - Prettier configuration as JSON string
 * @param numThreads - Number of worker threads to use (same as Rayon thread count)
 * @returns Array of loaded plugin's `languages` info
 * */
export async function setupConfig(configJSON: string, numThreads: number): Promise<string[]> {
  const workerData: WorkerData = {
    // SAFETY: Always valid JSON constructed by Rust side
    prettierConfig: JSON.parse(configJSON),
  };

  if (pool) throw new Error("`setupConfig()` has already been called");

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
  return [];
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

// ---

/**
 * Process Tailwind CSS classes found in JSX attributes.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param classes - Array of class strings found in JSX class/className attributes
 * @returns Array of sorted class strings (same order/length as input)
 */
export async function processTailwindClasses(classes: string[]): Promise<string[]> {
  // Initialize sorter on first call (lazy)
  if (!tailwindSorter) {
    try {
      tailwindSorter = await createBatchSorter({
        // Auto-detect Tailwind config from cwd
        // Could add config options here later
      });
      console.log("[oxfmt:tailwind] Initialized Tailwind sorter");
    } catch (err) {
      console.error("[oxfmt:tailwind] Failed to initialize sorter:", err);
      // Return original classes on error
      return classes;
    }
  }

  // Sort all classes
  const sorted = tailwindSorter.sortClasses(classes);

  // Log results
  console.log("[oxfmt:tailwind] Sorted classes:");
  sorted.forEach((classStr: string, idx: number) => {
    const original = classes[idx];
    const changed = classStr !== original;
    console.log(`  [${idx}]: "${classStr}"${changed ? ` (was: "${original}")` : ""}`);
  });
  console.log(`  Total: ${sorted.length} class/className attributes\n`);

  // Return sorted classes to Rust to replace originals in the output
  return sorted;
}
