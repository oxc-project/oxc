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
 * The `CACHES.xxx` are for lazy loading
 * and avoiding redundant dynamic imports within the same process.
 */

import { createRequire } from "node:module";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { installPluginResolution } from "./plugin-resolution";
import type { Options, Plugin } from "prettier";

const CACHES = {
  prettier: null as typeof import("prettier") | null,
  sveltePlugin: null as Plugin | null,
  tailwindPlugin: null as typeof import("prettier-plugin-tailwindcss") | null,
  tailwindSorter: null as typeof import("prettier-plugin-tailwindcss/sorter") | null,
  oxfmtPlugin: null as Plugin | null,
};

async function loadCached<K extends keyof typeof CACHES>(
  key: K,
  loader: () => Promise<NonNullable<(typeof CACHES)[K]>>,
): Promise<NonNullable<(typeof CACHES)[K]>> {
  CACHES[key] ??= await loader();
  return CACHES[key]!;
}

// ---

async function loadPrettier(): Promise<typeof import("prettier")> {
  return loadCached("prettier", async () => {
    const prettier = await import("prettier");

    // NOTE: This is needed for xxx-in-js formatting to work correctly.
    //
    // Prettier internally extends `options` with hidden fields for embedded-formatters during printing.
    // However, `__debug.printToDoc()` runs `normalizeFormatOptions()` which strips unknown keys.
    // Only keys registered in `formatOptionsHiddenDefaults` survive (via `passThrough` option).
    // Since `__debug.printToDoc()` does NOT use `passThrough: true` (unlike internal `textToDoc()`!),
    // our custom fields would be dropped without this registration.
    //
    // The default values MUST be falsy, truthy default would affect all Prettier calls, not just ours.
    // In call sites, Prettier checks `if (!options.parentParser)`, so as long as the default is falsy,
    // there should be no side effects on other calls that don't set these fields.
    // @ts-expect-error: Use internal API
    const { formatOptionsHiddenDefaults } = prettier.__internal;
    // For html(angular)-in-js: Prevent attribute level formatting from running.
    // (e.g., CSS in `style="..."` attributes, JS in `onclick="..."` event handlers)
    // This does NOT affect `<style>`/`<script>` tags, they are always formatted.
    // Ideally we'd only block JS attributes while allowing CSS attributes (because no nesting is possible in CSS),
    // but Prettier's `!options.parentParser` check is all-or-nothing.
    formatOptionsHiddenDefaults.parentParser = null;
    // For html(angular)-in-js: Capture `htmlHasMultipleRootElements` from the HTML AST root during `__debug.printToDoc()`.
    // This is used to decide whether to wrap content with `indent`.
    // Without this, we'd need either:
    // - double parse AST
    // - or flaky traversal of the `Doc` output
    // to extract the same information, since this hooks into the AST.
    formatOptionsHiddenDefaults.__onHtmlRoot = null;
    // For md-in-js: Use `~` instead of `` ` `` for code fences
    formatOptionsHiddenDefaults.__inJsTemplate = null;

    return prettier;
  });
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
  const prettier = CACHES.prettier ?? (await loadPrettier());

  // NOTE: Plugins order matters here!
  // User plugins go first: like the bundled Svelte plugin they contribute parsers,
  // which later plugins may transform.
  if ("_userPlugins" in options) await setupUserPlugins(options);
  // This plugin add `svelte` parser to support for `.svelte` files, and is also needed for `svelte-in-md` to work
  if ("_useSveltePlugin" in options) await setupSveltePlugin(options);
  // Enable Tailwind CSS plugin, this plugin transforms `parsers` already installed by prior plugins
  if ("_useTailwindPlugin" in options) await setupTailwindPlugin(options);
  // This plugin overrides `babel(-ts)` and `typescript` parsers to use `oxc_formatter` instead of built-in parsers
  if ("_oxfmtPluginOptionsJson" in options) await setupOxfmtPlugin(options);

  // This is needed to detect tsx-in-vue properly, see `prettier-plugin-oxfmt` for details.
  // All `<script>` blocks in one SFC must share the same `lang` (Vue compiler restriction),
  // so a single per-file flag is enough.
  if (options.parser === "vue" && hasTsxScriptBlock(code)) {
    options._oxfmtVueScriptLang = "tsx";
  }

  return prettier.format(code, options);
}

// `<script` + a tag boundary (so `<scripts` does not match) + attributes + the tag-closing `>`.
// Quoted attribute values may contain `>` (e.g. `generic="T extends Record<string, string>"`),
// so they are consumed as whole quoted chunks before `>` can terminate the tag.
const SCRIPT_OPEN_TAG_RE = /<script(?=[\s>])((?:"[^"]*"|'[^']*'|[^"'>])*)>/gv;
// Standalone `lang` attribute (not e.g. `data-lang`) whose whole value is "tsx", quoted or unquoted
const LANG_TSX_ATTR_RE = /(?:^|\s)lang\s*=\s*(?:"tsx"|'tsx'|tsx(?=[\s\/]|$))/v;

/**
 * Whether any `<script ...>` open tag in `sourceText` carries `lang="tsx"`.
 *
 * A plain-text scan may have a false positive.
 * (e.g. the literal tag inside a template string or comment),
 * the block parses as `tsx`, worst case the lone generic comma is kept or,
 * if the block uses ts-only syntax, it is left unformatted.
 * Never destructive, so leniency is acceptable trade-off here.
 */
function hasTsxScriptBlock(sourceText: string): boolean {
  for (const [, attrs] of sourceText.matchAll(SCRIPT_OPEN_TAG_RE)) {
    if (LANG_TSX_ATTR_RE.test(attrs)) return true;
  }
  return false;
}

// ---

export type FormatEmbeddedCodeParam = {
  code: string;
  options: Options;
};

/**
 * Format non-js code snippets into formatted string.
 * Used for formatting code fences within JSDoc.
 *
 * @returns Formatted code snippet
 */
export async function formatEmbeddedCode({
  code,
  options,
}: FormatEmbeddedCodeParam): Promise<string> {
  const prettier = CACHES.prettier ?? (await loadPrettier());

  // Enable Tailwind CSS plugin for embedded code (e.g., html`...` in JS) if needed
  if ("_useTailwindPlugin" in options) await setupTailwindPlugin(options);

  // NOTE: This will throw if:
  // - Specified parser is not available
  // - Or, code has syntax errors
  // In such cases, Rust side will fallback to original code
  return prettier.format(code, options);
}

// ---

export type FormatEmbeddedDocParam = {
  code: string;
  options: Options;
};

/**
 * Format a non-js code snippet into a Prettier `Doc` JSON string.
 * This makes our printer correctly handle `printWidth` even for embedded code.
 *
 * @returns `Doc` JSON string
 */
export async function formatEmbeddedDoc({
  code,
  options,
}: FormatEmbeddedDocParam): Promise<string> {
  const prettier = CACHES.prettier ?? (await loadPrettier());

  // Enable Tailwind CSS plugin for embedded code (e.g., html`...` in JS) if needed
  if ("_useTailwindPlugin" in options) await setupTailwindPlugin(options);

  const metadata: Record<string, unknown> = {};

  // html(angular)-in-js specific options: see the comment in `loadPrettier()` for rationale
  if (options.parser === "html" || options.parser === "angular") {
    // Any truthy value works
    options.parentParser = "OXFMT";
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/html.js#L42-L44
    options.__onHtmlRoot = (root: { children?: unknown[] }) =>
      (metadata.htmlHasMultipleRootElements = (root.children?.length ?? 0) > 1);
  }

  // md-in-js specific options: see the comment in `loadPrettier()` for rationale
  if (options.parser === "markdown") {
    // https://github.com/prettier/prettier/blob/90983f40dce5e20beea4e5618b5e0426a6a7f4f0/src/language-js/embed/markdown.js#L21
    options.__inJsTemplate = true;
  }

  // NOTE: This will throw if:
  // - Specified parser is not available
  // - Or, code has syntax errors
  // In such cases, Rust side will fallback to original code
  // @ts-expect-error: Use internal API, but it's necessary and only way to get `Doc`
  const doc = await prettier.__debug.printToDoc(code, options);

  // Serialize as [doc, metadata], handling special values:
  // - Symbol group IDs → numeric counters
  // - -Infinity (dedentToRoot) → marker string
  const symbolToNumber = new Map<symbol, number>();
  let nextId = 1;
  return JSON.stringify([doc, metadata], (_key, value) => {
    if (typeof value === "symbol") {
      if (!symbolToNumber.has(value)) symbolToNumber.set(value, nextId++);
      return symbolToNumber.get(value);
    }
    if (value === -Infinity) return "__NEGATIVE_INFINITY__";
    return value;
  });
}

// ---
// Tailwind CSS support
// ---

/**
 * Load Tailwind CSS plugin.
 * Option mapping (sortTailwindcss.xxx → tailwindXxx) is also done in Rust side.
 */
async function setupTailwindPlugin(options: Options): Promise<void> {
  CACHES.tailwindPlugin ??= await loadCached(
    "tailwindPlugin",
    () => import("prettier-plugin-tailwindcss"),
  );
  options.plugins ??= [];
  options.plugins.push(CACHES.tailwindPlugin as Plugin);
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
  CACHES.tailwindSorter ??= await loadCached(
    "tailwindSorter",
    () => import("prettier-plugin-tailwindcss/sorter"),
  );
  const { createSorter } = CACHES.tailwindSorter;

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
// Svelte plugin support
// ---

/**
 * Load prettier-plugin-svelte to provide the `svelte` parser.
 */
async function setupSveltePlugin(options: Options): Promise<void> {
  CACHES.sveltePlugin ??= await loadCached(
    "sveltePlugin",
    async () => (await import("prettier-plugin-svelte")) as Plugin,
  );
  options.plugins ??= [];
  options.plugins.push(CACHES.sveltePlugin);
}

// ---
// Oxfmt plugin support for (j|t)-in-xxx files
// ---

/**
 * Load oxfmt plugin for js-in-xxx parsers.
 */
async function setupOxfmtPlugin(options: Options): Promise<void> {
  CACHES.oxfmtPlugin ??= await loadCached(
    "oxfmtPlugin",
    async () => (await import("./prettier-plugin-oxfmt/index")) as Plugin,
  );
  options.plugins ??= [];
  options.plugins.push(CACHES.oxfmtPlugin);
}

// ---
// User-configured Prettier plugin support
// ---

/**
 * Plugins named by the `plugins` config field.
 *
 * `base` is the directory the specifiers resolve from, which is the directory
 * of the config file that declared them, matching how Prettier resolves its own
 * `plugins` entries.
 */
export type UserPluginsParam = {
  base: string;
  specifiers: string[];
};

/** One entry of a plugin's `languages` declaration, narrowed to what routing needs. */
export type PluginLanguage = {
  parsers: string[];
  extensions: string[];
  filenames: string[];
};

export type ResolvePluginsResult = {
  languages: PluginLanguage[];
  /** Specifiers that could not be loaded, so the caller can report them once. */
  failures: { specifier: string; message: string }[];
};

// Keyed by base and specifier together: the same name can resolve to different
// packages under different config files.
const USER_PLUGINS = new Map<string, Promise<Plugin>>();

function loadUserPlugin(base: string, specifier: string): Promise<Plugin> {
  const key = `${base}\u0000${specifier}`;
  let cached = USER_PLUGINS.get(key);
  if (cached === undefined) {
    cached = importUserPlugin(base, specifier);
    USER_PLUGINS.set(key, cached);
  }
  return cached;
}

async function importUserPlugin(base: string, specifier: string): Promise<Plugin> {
  // Must precede the import: the plugin resolves `prettier` while it loads.
  installPluginResolution();

  // Resolve from the config's directory rather than from oxfmt's own location,
  // so a plugin in the user's `node_modules` is found. The anchor names a file
  // that need not exist; only its directory matters.
  const requireFromBase = createRequire(join(base, "__oxfmt_config__"));
  let resolved;
  try {
    resolved = requireFromBase.resolve(specifier);
  } catch {
    // Node's own message names the anchor, which would expose a path the user
    // never wrote. Report the directory they configured instead.
    throw new Error(`Cannot find module "${specifier}" from "${base}"`);
  }
  const mod = (await import(pathToFileURL(resolved).href)) as { default?: Plugin } & Plugin;
  // A CJS plugin's exports land on `default` once interop has run.
  return mod.default ?? mod;
}

async function setupUserPlugins(options: Options): Promise<void> {
  const { base, specifiers } = options._userPlugins as UserPluginsParam;
  const settled = await Promise.allSettled(specifiers.map((s) => loadUserPlugin(base, s)));

  // One unloadable plugin must not stop the others. `resolvePlugins()` already
  // reported the failure during init, and a file that needed the missing plugin
  // has no parser and fails on its own terms.
  const loaded = settled
    .filter((result) => result.status === "fulfilled")
    .map((result) => result.value);

  options.plugins ??= [];
  options.plugins.push(...loaded);
}

/**
 * Load the configured plugins and report the languages they declare, so the Rust
 * side can route their file extensions to Prettier.
 *
 * A plugin that fails to load is reported rather than thrown, so one broken
 * entry does not stop the rest of the run.
 */
export async function resolvePlugins({
  base,
  specifiers,
}: UserPluginsParam): Promise<ResolvePluginsResult> {
  const settled = await Promise.allSettled(specifiers.map((s) => loadUserPlugin(base, s)));

  const languages: PluginLanguage[] = [];
  const failures: ResolvePluginsResult["failures"] = [];

  for (const [index, result] of settled.entries()) {
    if (result.status === "rejected") {
      failures.push({
        specifier: specifiers[index]!,
        message: result.reason instanceof Error ? result.reason.message : String(result.reason),
      });
      continue;
    }
    for (const language of result.value.languages ?? []) {
      const parsers = language.parsers ?? [];
      if (parsers.length === 0) continue;
      languages.push({
        parsers,
        // Prettier stores extensions with the leading dot; routing compares bare ones.
        extensions: (language.extensions ?? []).map((ext) =>
          ext.startsWith(".") ? ext.slice(1) : ext,
        ),
        filenames: language.filenames ?? [],
      });
    }
  }

  return { languages, failures };
}
