/**
 * API functions for Prettier integration.
 *
 * These must be plain functions because:
 * - They can be called as `tinypool` RPC functions via `cli-worker.ts`
 *   - Tinypool runs workers as `child_process`, so each worker is an isolated process
 *   - Module-level caches are shared only within each worker process
 * - They can also be imported directly via `index.ts` (Node.js API)
 *   - In this case, module-level caches are shared globally
 *
 * The caches (`xxxCache`) are for lazy loading
 * and avoiding redundant dynamic imports within the same process.
 */

import type { Options, Plugin } from "prettier";

// Lazy load Prettier
//
// NOTE: In the past, statically importing caused issues with `oxfmt --lsp` not starting.
// However, this issue has not been observed recently, possibly due to changes in the bundling configuration.
// Anyway, we keep lazy loading for now to minimize initial load time.
let prettierCache: typeof import("prettier");

async function loadPrettier(): Promise<typeof import("prettier")> {
  if (prettierCache) return prettierCache;

  prettierCache = await import("prettier");
  return prettierCache;
}

// ---

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

export type FormatEmbeddedCodeParam = {
  code: string;
  parserName: string;
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
  parserName,
  options,
}: FormatEmbeddedCodeParam): Promise<string> {
  const prettier = await loadPrettier();

  // SAFETY: `options` is created in Rust side, so it's safe to mutate here
  options.parser = parserName;

  // Enable Tailwind CSS plugin for embedded code (e.g., html`...` in JS) if needed
  await setupTailwindPlugin(options);

  // NOTE: This will throw if:
  // - Specified parser is not available
  // - Or, code has syntax errors
  // In such cases, Rust side will fallback to original code
  return prettier.format(code, options);
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
  const prettier = await loadPrettier();

  // SAFETY: `options` is created in Rust side, so it's safe to mutate here
  // We specify `parser` to skip parser inference for performance
  options.parser = parserName;
  // But some plugins rely on `filepath`, so we set it too
  options.filepath = fileName;

  // Enable Tailwind CSS plugin for non-JS files if needed
  await setupTailwindPlugin(options);
  // Add oxfmt plugin for (j|t)-in-xxx files to use `oxc_formatter` instead of built-in formatter.
  // NOTE: This must be last since Prettier plugins are applied in order
  await setupOxfmtPlugin(options);

  return prettier.format(code, options);
}

// ---
// Tailwind CSS support
// ---

// Import types only to avoid runtime error if plugin is not installed
import type { TransformerEnv } from "prettier-plugin-tailwindcss";

// Shared cache for prettier-plugin-tailwindcss
let tailwindPluginCache: typeof import("prettier-plugin-tailwindcss");

async function loadTailwindPlugin(): Promise<typeof import("prettier-plugin-tailwindcss")> {
  if (tailwindPluginCache) return tailwindPluginCache;

  tailwindPluginCache = await import("prettier-plugin-tailwindcss");
  return tailwindPluginCache;
}

// ---

const TAILWIND_RELEVANT_PARSERS = new Set(["html", "vue", "angular", "glimmer"]);

/**
 * Load Tailwind CSS plugin lazily for Prettier when:
 * - Option flag is set (by Rust side)
 * - And, the `parser` is relevant for Tailwind CSS
 *
 * Option mapping (experimentalTailwindcss.xxx → tailwindXxx) is done in Rust side.
 */
async function setupTailwindPlugin(options: Options): Promise<void> {
  if ("_useTailwindPlugin" in options === false) return;
  // PERF: Skip loading Tailwind plugin for parsers that don't use it
  if (!TAILWIND_RELEVANT_PARSERS.has(options.parser as string)) return;

  const tailwindPlugin = await loadTailwindPlugin();

  options.plugins ??= [];
  options.plugins.push(tailwindPlugin as Plugin);
}

// ---

export interface SortTailwindClassesArgs {
  filepath: string;
  classes: string[];
  options?: Record<string, unknown>;
}

/**
 * Process Tailwind CSS classes found in JSX attributes.
 * Option mapping (`experimentalTailwindcss.xxx` → `tailwindXxx`) is done in Rust side.
 * @param args - Object containing filepath, classes, and options
 * @returns Array of sorted class strings (same order/length as input)
 */
export async function sortTailwindClasses({
  filepath,
  classes,
  options = {},
}: SortTailwindClassesArgs): Promise<string[]> {
  const tailwindPlugin = await loadTailwindPlugin();

  // SAFETY: `options` is created in Rust side, so it's safe to mutate here
  options.filepath = filepath;

  // Load Tailwind context
  const context = await tailwindPlugin.getTailwindConfig(options);
  if (!context) return classes;

  // Create transformer env with options
  const env: TransformerEnv = { context, options };

  // Sort all classes
  return classes.map((classStr) => {
    try {
      return tailwindPlugin.sortClasses(classStr, { env });
    } catch {
      // Failed to sort, return original
      return classStr;
    }
  });
}

// ---
// Oxfmt plugin support for (j|t)-in-xxx files
// ---

let oxfmtPluginCache: Plugin;

async function loadOxfmtPlugin(): Promise<Plugin> {
  if (oxfmtPluginCache) return oxfmtPluginCache;

  oxfmtPluginCache = (await import("./prettier-plugin-oxfmt")) as Plugin;
  return oxfmtPluginCache;
}

// ---

// Parsers that can embed JS/TS code
const OXFMT_RELEVANT_PARSERS = new Set([
  "html",
  "angular",
  "vue",
  "lwc",
  "mjml",
  "markdown",
  "mdx",
]);

async function setupOxfmtPlugin(options: Options): Promise<void> {
  // Skip loading oxfmt plugin for parsers that don't embed JS/TS
  if (!OXFMT_RELEVANT_PARSERS.has(options.parser as string)) return;

  const oxcPlugin = await loadOxfmtPlugin();

  options.plugins ??= [];
  options.plugins.push(oxcPlugin);
}
