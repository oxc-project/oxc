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

// Lazy load oxc plugin
// FormatOptions are cached on Rust side, so no need to pass options here
let oxcPluginCache: Plugin;

async function loadOxcPlugin(): Promise<Plugin> {
  if (oxcPluginCache) return oxcPluginCache;

  const module = await import("./prettier-plugin-oxc");
  oxcPluginCache = module.default;
  return oxcPluginCache;
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
  options: Options & { _tailwindPluginEnabled?: boolean };
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
  options: Options & { _tailwindPluginEnabled?: boolean };
};

/**
 * Serialize an object option to a JSON string for Prettier plugin preservation.
 */
function serializeObjectOption(options: Options, sourceKey: string, targetKey: string): void {
  const anyOptions = options as Record<string, unknown>;
  if (anyOptions[sourceKey] !== undefined) {
    anyOptions[targetKey] = JSON.stringify(anyOptions[sourceKey]);
  }
}

// Parsers that contain embedded JavaScript (need oxc plugin)
const PARSERS_WITH_EMBEDDED_JS = new Set(["vue", "html", "angular", "svelte", "astro"]);

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

  // Add oxc plugin for files with embedded JavaScript AFTER tailwind plugin
  // This overrides babel/typescript parsers so that <script> content
  // is formatted by oxc_formatter instead of Prettier's built-in formatter
  // Prettier resolves parsers from the LAST plugin first, so oxc must be last
  // FormatOptions are cached on Rust side before this function is called
  if (PARSERS_WITH_EMBEDDED_JS.has(parserName)) {
    const oxcPlugin = await loadOxcPlugin();
    options.plugins = options.plugins || [];
    options.plugins.push(oxcPlugin);

    // Convert object options to JSON strings so they survive Prettier's option normalization
    // Prettier only preserves options that are defined in plugins, and only supports primitive types
    serializeObjectOption(options, "experimentalSortImports", "_experimentalSortImportsJson");
    serializeObjectOption(options, "experimentalTailwindcss", "_experimentalTailwindcssJson");
  }

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
 * Set up Tailwind CSS plugin for Prettier when:
 * - `options._tailwindPluginEnabled` is set
 * - And, the parser is relevant for Tailwind CSS
 * Loads the plugin lazily. Option mapping is done in Rust side.
 */
async function setupTailwindPlugin(
  options: Options & { _tailwindPluginEnabled?: boolean },
): Promise<void> {
  if (!options._tailwindPluginEnabled) return;

  // Clean up internal flag
  delete options._tailwindPluginEnabled;

  // PERF: Skip loading Tailwind plugin for parsers that don't use it
  if (!TAILWIND_RELEVANT_PARSERS.has(options.parser as string)) return;

  const tailwindPlugin = await loadTailwindPlugin();

  options.plugins = options.plugins || [];
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
