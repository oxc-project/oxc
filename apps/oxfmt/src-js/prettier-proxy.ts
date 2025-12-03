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
      parser,
      // TODO: Read config
      printWidth: 80,
      tabWidth: 2,
      semi: true,
      singleQuote: false,
    })
    .then((formatted) => formatted.trimEnd())
    .catch(() => code);
}

/**
 * Format whole file content using Prettier.
 * NOTE: Called from Rust via NAPI ThreadsafeFunction with FnArgs
 * @param fileName - The file name (used to infer parser)
 * @param code - The code to format
 * @returns Formatted code
 */
export async function formatFile(fileName: string, code: string): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  // TODO: Tweak parser for `tsconfig.json` with `jsonc` parser?

  return prettierCache.format(code, {
    // Let Prettier infer the parser
    filepath: fileName,
    // TODO: Read config
  });
}
