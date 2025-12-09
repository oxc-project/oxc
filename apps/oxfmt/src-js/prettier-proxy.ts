import type { Options } from "prettier";

// Import Prettier lazily.
// This helps to reduce initial load time if not needed.
//
// Also, this solves unknown issue described below...
//
// XXX: If import `prettier` directly here, it will add line like this to the output JS:
// ```js
// import process2 from 'process';
// ```
// Yes, this seems completely fine!
// But actually, this makes `oxfmt --lsp` immediately stop with `Parse error` JSON-RPC error
let prettierCache: typeof import("prettier");

// Cache for Prettier options.
// Set by `setupConfig` function once.
//
// Read `.oxfmtrc.json(c)` directly does not work,
// because our brand new defaults are not compatible with Prettier's defaults.
// So we need to pass the config from Rust side after merging with our defaults.
let configCache: Options = {};

// ---

/**
 * Setup Prettier configuration.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param configJSON - Prettier configuration as JSON string
 * @returns Array of loaded plugin's `languages` info
 * */
export async function setupConfig(configJSON: string): Promise<string[]> {
  // NOTE: `napi-rs` has ability to pass `Object` directly.
  // But since we don't know what options various plugins may specify,
  // we have to receive it as a JSON string and parse it.
  //
  // SAFETY: This is valid JSON string generated in Rust side
  configCache = JSON.parse(configJSON) as Options;

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

  if (!parser) {
    // Unknown tag, return original code
    return code;
  }

  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache
    .format(code, {
      ...configCache,
      parser,
    })
    .then((formatted) => formatted.trimEnd())
    .catch(() => code);
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
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache.format(code, {
    ...configCache,
    parser: parserName,
    filepath: fileName,
  });
}
