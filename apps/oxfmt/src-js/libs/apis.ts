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
 * Format xxx-in-js code snippets into formatted string.
 *
 * This will be gradually replaced by `formatEmbeddedDoc` which returns `Doc`.
 * For now, html|css|md-in-js are using this.
 *
 * @returns Formatted code snippet
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

export type FormatEmbeddedDocParam = {
  texts: string[];
  options: Options;
};

/**
 * Format xxx-in-js code snippets into Prettier `Doc` JSON strings.
 *
 * This makes `oxc_formatter` correctly handle `printWidth` even for embedded code.
 * - For gql-in-js, `texts` contains multiple parts split by `${}` in a template literal
 * - For others, `texts` always contains a single string with `${}` parts replaced by placeholders
 * However, this function does not need to be aware of that,
 * as it simply formats each text part independently and returns an array of formatted parts.
 *
 * @returns Doc JSON strings (one per input text)
 */
export async function formatEmbeddedDoc({
  texts,
  options,
}: FormatEmbeddedDocParam): Promise<string[]> {
  const prettier = await loadPrettier();

  // Enable Tailwind CSS plugin for embedded code (e.g., html`...` in JS) if needed
  await setupTailwindPlugin(options);

  // NOTE: This will throw if:
  // - Specified parser is not available
  // - Or, code has syntax errors
  // In such cases, Rust side will fallback to original code
  return Promise.all(
    texts.map(async (text) => {
      // @ts-expect-error: Use internal API, but it's necessary and only way to get `Doc`
      const doc = await prettier.__debug.printToDoc(text, options);

      // Serialize Doc to JSON, handling special values in a single pass:
      // - Symbol group IDs (used by `group`, `if-break`, `indent-if-break`) → numeric counters
      // - -Infinity (used by `dedentToRoot` via `align`) → marker string
      const symbolToNumber = new Map<symbol, number>();
      let nextId = 1;

      return JSON.stringify(doc, (_key, value) => {
        if (typeof value === "symbol") {
          if (!symbolToNumber.has(value)) symbolToNumber.set(value, nextId++);
          return symbolToNumber.get(value);
        }
        if (value === -Infinity) return "__NEGATIVE_INFINITY__";
        return value;
      });
    }),
  );
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

  // 1. Svelte plugin first (parser plugin, required for .svelte)
  await setupSveltePlugin(options);
  // 2. Tailwind plugin (transformation, wraps Svelte's parsers)
  await setupTailwindPlugin(options);
  // 3. Oxfmt plugin last (overrides babel/typescript parsers)
  await setupOxfmtPlugin(options);

  return prettier.format(code, options);
}

// ---
// Svelte support
// ---

let sveltePluginCache: typeof import("prettier-plugin-svelte");

async function loadSveltePlugin(): Promise<typeof import("prettier-plugin-svelte")> {
  if (sveltePluginCache) return sveltePluginCache;

  sveltePluginCache = await import("prettier-plugin-svelte");
  return sveltePluginCache;
}

/**
 * Load Svelte plugin lazily when `options._useSveltePlugin` flag is set.
 * The flag is added by Rust side only for `.svelte` files.
 *
 * Option mapping (svelte.xxx → svelteXxx) is also done in Rust side.
 */
async function setupSveltePlugin(options: Options): Promise<void> {
  if ("_useSveltePlugin" in options === false) return;

  const sveltePlugin = await loadSveltePlugin();

  options.plugins ??= [];
  options.plugins.push(sveltePlugin as Plugin);
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
