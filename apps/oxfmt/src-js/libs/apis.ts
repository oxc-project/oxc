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
  options,
}: FormatEmbeddedCodeParam): Promise<string> {
  const prettier = await loadPrettier();

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
  options: Options;
};

/**
 * Format non-js file
 *
 * @returns Formatted code
 */
export async function formatFile({ code, options }: FormatFileParam): Promise<string> {
  const prettier = await loadPrettier();

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

// Shared cache for prettier-plugin-tailwindcss
let tailwindPluginCache: typeof import("prettier-plugin-tailwindcss");

async function loadTailwindPlugin(): Promise<typeof import("prettier-plugin-tailwindcss")> {
  if (tailwindPluginCache) return tailwindPluginCache;

  tailwindPluginCache = await import("prettier-plugin-tailwindcss");
  return tailwindPluginCache;
}

// ---

/**
 * Load Tailwind CSS plugin lazily when `options._useTailwindPlugin` flag is set.
 * The flag is added by Rust side only for relevant parsers.
 *
 * Option mapping (sortTailwindcss.xxx → tailwindXxx) is also done in Rust side.
 */
async function setupTailwindPlugin(options: Options): Promise<void> {
  if ("_useTailwindPlugin" in options === false) return;

  const tailwindPlugin = await loadTailwindPlugin();

  options.plugins ??= [];
  options.plugins.push(tailwindPlugin as Plugin);
}

// ---

export interface SortTailwindClassesArgs {
  classes: string[];
  options: {
    filepath?: string;
    tailwindStylesheet?: string;
    tailwindConfig?: string;
    tailwindPreserveWhitespace?: boolean;
    tailwindPreserveDuplicates?: boolean;
  };
}

/**
 * Process Tailwind CSS classes found in JS/TS files in batch.
 * @param args - Object containing classes and options (filepath is in options.filepath)
 * @returns Array of sorted class strings (same order/length as input)
 */
export async function sortTailwindClasses({
  classes,
  options,
}: SortTailwindClassesArgs): Promise<string[]> {
  const { createSorter } = await import("prettier-plugin-tailwindcss/sorter");

  const sorter = await createSorter({
    filepath: options.filepath,
    stylesheetPath: options.tailwindStylesheet,
    configPath: options.tailwindConfig,
    preserveWhitespace: options.tailwindPreserveWhitespace,
    preserveDuplicates: options.tailwindPreserveDuplicates,
  });

  return sorter.sortClassAttributes(classes);
}

// ---
// Oxfmt plugin support for (j|t)-in-xxx files
// ---

let oxfmtPluginCache: Plugin;

async function loadOxfmtPlugin(): Promise<Plugin> {
  if (oxfmtPluginCache) return oxfmtPluginCache;

  oxfmtPluginCache = (await import("./prettier-plugin-oxfmt/index")) as Plugin;
  return oxfmtPluginCache;
}

// ---

/**
 * Load oxfmt plugin for js-in-xxx parsers when `options._oxfmtPluginOptionsJson` is set.
 * The flag is added by Rust side only for relevant parsers.
 */
async function setupOxfmtPlugin(options: Options): Promise<void> {
  if ("_oxfmtPluginOptionsJson" in options === false) return;

  const oxcPlugin = await loadOxfmtPlugin();

  options.plugins ??= [];
  options.plugins.push(oxcPlugin);
}
