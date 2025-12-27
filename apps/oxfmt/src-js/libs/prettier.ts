import type { Options } from "prettier";

// Lazy load Prettier
//
// NOTE: In the past, statically importing caused issues with `oxfmt --lsp` not starting.
// However, this issue has not been observed recently, possibly due to changes in the bundling configuration.
// Anyway, we keep lazy loading for now to minimize initial load time.
let prettierCache: typeof import("prettier");

/**
 * TODO: Plugins support
 * - Read `plugins` field
 * - Load plugins dynamically and parse `languages` field
 * - Map file extensions and filenames to Prettier parsers
 *
 * @returns Array of loaded plugin's `languages` info
 */
export async function resolvePlugins(): Promise<string[]> {
  return [];
}

// ---

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

export type FormatEmbeddedCodeParam = {
  code: string;
  tagName: string;
  options: Options;
};

/**
 * Format xxx-in-js code snippets
 *
 * @returns Formatted code snippet
 * TODO: In the future, this should return `Doc` instead of string,
 * otherwise, we cannot calculate `printWidth` correctly.
 */
export async function formatEmbeddedCode({
  code,
  tagName,
  options,
}: FormatEmbeddedCodeParam): Promise<string> {
  // TODO: This should be resolved in Rust side
  const parserName = TAG_TO_PARSER[tagName];

  // Unknown tag, return original code
  if (!parserName) return code;

  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  // SAFETY: `options` is created in Rust side, so it's safe to mutate here
  options.parser = parserName;
  return prettierCache
    .format(code, options)
    .then((formatted) => formatted.trimEnd())
    .catch(() => code);
}

// ---

export type FormatFileParam = {
  code: string;
  parserName: string;
  fileName: string;
  options: Options;
};

/**
 * Format non-js file
 *
 * @returns Formatted code
 */
export async function formatFile({
  code,
  parserName,
  fileName,
  options,
}: FormatFileParam): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  // SAFETY: `options` is created in Rust side, so it's safe to mutate here
  // We specify `parser` to skip parser inference for performance
  options.parser = parserName;
  // But some plugins rely on `filepath`, so we set it too
  options.filepath = fileName;
  return prettierCache.format(code, options);
}

// ---
// Tailwind CSS class sorting support
// ---

// Import types only to avoid runtime error if plugin is not installed
import type { TransformerEnv } from "prettier-plugin-tailwindcss";

// Cache tailwind plugin functions for tree-shaking support
let cachedGetTailwindConfig: typeof import("prettier-plugin-tailwindcss").getTailwindConfig;
let cachedSortClasses: typeof import("prettier-plugin-tailwindcss").sortClasses;
let tailwindInitialized = false;

/**
 * Configuration options for Tailwind CSS class sorting.
 * These are flattened at root level (like Prettier).
 * See https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options
 */
export interface TailwindOptions {
  tailwindConfig?: string;
  tailwindStylesheet?: string;
  tailwindFunctions?: string[];
  tailwindAttributes?: string[];
  tailwindPreserveWhitespace?: boolean;
  tailwindPreserveDuplicates?: boolean;
}

export interface ProcessTailwindClassesArgs {
  filepath: string;
  classes: string[];
  options?: { experimentalTailwindcss?: TailwindOptions } & Record<string, unknown>;
}

/**
 * Process Tailwind CSS classes found in JSX attributes.
 * @param args - Object containing filepath, classes, and options
 * @returns Array of sorted class strings (same order/length as input)
 */
export async function processTailwindClasses({
  filepath,
  classes,
  options = {},
}: ProcessTailwindClassesArgs): Promise<string[]> {
  // Initialize tailwind plugin lazily on first call
  if (!tailwindInitialized) {
    tailwindInitialized = true;
    try {
      // Dynamic import with destructuring for tree-shaking
      ({ getTailwindConfig: cachedGetTailwindConfig, sortClasses: cachedSortClasses } =
        await import("prettier-plugin-tailwindcss"));
    } catch {
      // Plugin not installed or failed to initialize - sorting will be skipped
      return classes;
    }
  }

  // Options are flattened at root level (like Prettier)
  const configOptions = {
    filepath,
    ...options,
    // Oxfmt puts all Tailwind options under `experimentalTailwindcss`
    ...options?.experimentalTailwindcss,
  };

  // Load Tailwind context
  const tailwindContext = await cachedGetTailwindConfig(configOptions);

  // If context not available, return original classes
  if (!tailwindContext) {
    return classes;
  }

  // Create transformer env with options
  const env: TransformerEnv = {
    context: tailwindContext,
    options: configOptions,
  };

  // Sort all classes
  return classes.map((classStr) => {
    try {
      return cachedSortClasses(classStr, { env });
    } catch {
      // Failed to sort, return original
      return classStr;
    }
  });
}
