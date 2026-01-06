import type { Options, Plugin } from "prettier";

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

export type FormatFileParam = {
  code: string;
  parserName: string;
  fileName: string;
  options: Options & { experimentalTailwindcss?: TailwindcssOptions };
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

  // Enable Tailwind CSS plugin for non-JS files when experimentalTailwindcss is set
  await setupTailwindPlugin(options);

  return prettierCache.format(code, options);
}

// ---
// Tailwind CSS support
// ---

// Import types only to avoid runtime error if plugin is not installed
import type { TransformerEnv } from "prettier-plugin-tailwindcss";
import type { TailwindcssOptions } from "../index";

// Shared cache for prettier-plugin-tailwindcss
let tailwindPlugin: typeof import("prettier-plugin-tailwindcss") | null = null;

// Oxfmt to Prettier option name mapping (adds `tailwind` prefix)
export const TAILWIND_OPTION_MAPPING: Record<string, string> = {
  config: "tailwindConfig",
  stylesheet: "tailwindStylesheet",
  functions: "tailwindFunctions",
  attributes: "tailwindAttributes",
  preserveWhitespace: "tailwindPreserveWhitespace",
  preserveDuplicates: "tailwindPreserveDuplicates",
};

/**
 * Load prettier-plugin-tailwindcss lazily.
 * @returns The plugin module or null if not available.
 */
async function loadTailwindPlugin(): Promise<typeof import("prettier-plugin-tailwindcss") | null> {
  if (tailwindPlugin) return tailwindPlugin;

  try {
    tailwindPlugin = await import("prettier-plugin-tailwindcss");
    return tailwindPlugin;
  } catch {
    // Plugin not available
    return null;
  }
}

/**
 * Map Oxfmt Tailwind options to Prettier format.
 */
function mapTailwindOptions(
  tailwindcss: TailwindcssOptions,
  target: Record<string, unknown>,
): void {
  for (const [oxfmtKey, prettierKey] of Object.entries(TAILWIND_OPTION_MAPPING)) {
    const value = tailwindcss[oxfmtKey as keyof TailwindcssOptions];
    if (value !== undefined) {
      target[prettierKey] = value;
    }
  }
}

/**
 * Set up Tailwind CSS plugin for Prettier when experimentalTailwindcss is enabled.
 * Loads the plugin lazily and maps Oxfmt config options to Prettier format.
 */
async function setupTailwindPlugin(
  options: Options & { experimentalTailwindcss?: TailwindcssOptions },
): Promise<void> {
  const tailwindcss = options.experimentalTailwindcss;
  if (!tailwindcss) return;

  const plugin = await loadTailwindPlugin();
  if (plugin) {
    // Cast to `any` because the module type is not compatible with Prettier's plugin type
    options.plugins = options.plugins || [];
    options.plugins.push(plugin as Plugin);
    mapTailwindOptions(tailwindcss, options as Record<string, unknown>);
  }

  // Clean up experimentalTailwindcss from options to avoid passing it to Prettier
  delete options.experimentalTailwindcss;
}

// ---

export interface SortTailwindClassesArgs {
  filepath: string;
  classes: string[];
  options?: { experimentalTailwindcss?: TailwindcssOptions } & Record<string, unknown>;
}

/**
 * Process Tailwind CSS classes found in JSX attributes.
 * @param args - Object containing filepath, classes, and options
 * @returns Array of sorted class strings (same order/length as input)
 */
export async function sortTailwindClasses({
  filepath,
  classes,
  options = {},
}: SortTailwindClassesArgs): Promise<string[]> {
  const plugin = await loadTailwindPlugin();
  if (!plugin) return classes;

  const tailwindcss = options.experimentalTailwindcss || {};
  const configOptions: Record<string, unknown> = { filepath, ...options };
  mapTailwindOptions(tailwindcss, configOptions);

  // Load Tailwind context
  const tailwindContext = await plugin.getTailwindConfig(configOptions);
  if (!tailwindContext) return classes;

  // Create transformer env with options
  const env: TransformerEnv = {
    context: tailwindContext,
    options: configOptions,
  };

  // Sort all classes
  return classes.map((classStr) => {
    try {
      return plugin.sortClasses(classStr, { env });
    } catch {
      // Failed to sort, return original
      return classStr;
    }
  });
}
