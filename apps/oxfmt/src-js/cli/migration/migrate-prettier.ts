/* oxlint-disable no-console */

import { basename, extname, join } from "node:path";
import { readdir, readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { hasOxfmtrcFile, createBlankOxfmtrcFile, saveOxfmtrcFile, exitWithError } from "./shared";
import type { Language } from "../../config.generated";

/**
 * Run the `--migrate prettier` command to migrate Prettier's config in the cwd to `.oxfmtrc.json` file.
 * https://prettier.io/docs/configuration
 */
export async function runMigratePrettier() {
  const cwd = process.cwd();

  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt config file already exists.\nRemove it and re-run.");
  }

  // Only the cwd is searched, nested configs need to be migrated by running in each directory.
  const prettierConfigPath = await findPrettierConfigFile(cwd);
  if (!prettierConfigPath) {
    return exitWithError(
      "No Prettier config file found in the current directory.\nRun `oxfmt --migrate prettier` in the directory where the Prettier config is, or use `--init` to create a blank `.oxfmtrc.json`.",
    );
  }

  let prettierConfig: Record<string, unknown>;
  try {
    prettierConfig = await loadPrettierConfig(prettierConfigPath);
    console.log("Found Prettier config at:", prettierConfigPath);
  } catch (err) {
    return exitWithError(
      `Failed to load Prettier config at: ${prettierConfigPath}\n${err instanceof Error ? err.message : String(err)}`,
    );
  }

  // Start with blank, then fill in from `prettierConfig`.
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);

  const { plugins, overrides, $schema: _, ...options } = prettierConfig;

  // Handle plugins - check for known plugins and warn about others
  const enabledPlugins = new Set<string>();
  for (const plugin of Array.isArray(plugins) ? plugins : []) {
    if (KNOWN_PLUGINS.has(plugin)) {
      enabledPlugins.add(plugin);
    } else if (typeof plugin === "string") {
      warnings.push(`plugins: "${plugin}" is not supported, skipping...`);
    } else {
      warnings.push(`plugins: custom plugin module is not supported, skipping...`);
    }
  }

  Object.assign(oxfmtrc, migrateOptions(options, true));

  // `printWidth` has different default between Prettier and Oxfmt.
  // Oxfmt default is 100, Prettier default is 80.
  if (typeof oxfmtrc.printWidth !== "number") {
    warnings.push(
      `"printWidth" is not set in Prettier config, defaulting to 80 (Oxfmt default: 100)`,
    );
    oxfmtrc.printWidth = 80;
  }
  // `sortPackageJson` is enabled by default in Oxfmt, but Prettier does not have this.
  // Only enable if `prettier-plugin-packagejson` is used.
  if (enabledPlugins.has("prettier-plugin-packagejson")) {
    oxfmtrc.sortPackageJson = {};
    console.log(`  - Migrated "prettier-plugin-packagejson" to "sortPackageJson"`);
  } else {
    oxfmtrc.sortPackageJson = false;
  }
  Object.assign(oxfmtrc, migratePluginOptions(options, enabledPlugins, true));

  if (Array.isArray(overrides)) {
    const oxfmtOverrides = [];
    const associations = [];
    for (const { files, excludeFiles, options = {} } of overrides) {
      const globs = {
        files: [files].flat(),
        ...(excludeFiles !== undefined && { excludeFiles: [excludeFiles].flat() }),
      };
      // e.g. `{ files: "*.svg", options: { parser: "html" } }`
      const { parser, ...rest } = options;
      const language = PARSER_TO_LANGUAGE.get(parser);
      if (language) {
        associations.push({ ...globs, language });
      } else if (parser !== undefined) {
        warnings.push(`overrides: "parser": "${parser}" is not supported, skipping...`);
      }
      const migrated = {
        ...migrateOptions(rest, false),
        ...migratePluginOptions(rest, enabledPlugins, false),
      };
      if (Object.keys(migrated).length === 0) continue;
      oxfmtOverrides.push({ ...globs, options: migrated });
    }
    if (oxfmtOverrides.length > 0) {
      oxfmtrc.overrides = oxfmtOverrides;
      console.log(`  - Migrated "overrides"`);
    }
    if (associations.length > 0) {
      oxfmtrc.associations = associations;
      console.log(`  - Migrated "parser" in "overrides" to "associations"`);
    }
  }

  // Migrate `ignorePatterns` from `.prettierignore`
  const ignores = await resolvePrettierIgnore(cwd);
  if (ignores.length > 0) {
    console.log("  - Migrated ignore patterns from `.prettierignore`");
  }
  // Keep ignorePatterns at the bottom
  delete oxfmtrc.ignorePatterns;
  oxfmtrc.ignorePatterns = ignores;

  if (JS_EXTENSIONS.has(extname(prettierConfigPath))) {
    warnings.push(
      `\`${basename(prettierConfigPath)}\` was evaluated and migrated as static values, any logic in it is not preserved,\n    port it to \`oxfmt.config.ts\` manually if needed`,
    );
  }

  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  try {
    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json`.\nIt may not be formatted yet, run `oxfmt` to format it.");
    if (warnings.length > 0) {
      console.error(`\nPlease review:\n${warnings.map((w) => `  - ${w}`).join("\n")}`);
    }
  } catch {
    return exitWithError("Failed to create `.oxfmtrc.json`.");
  }
}

// ---

// Collected and printed at the end, not to be mixed with progress logs
const warnings: string[] = [];

// Same order as Prettier's config searcher.
// https://github.com/prettier/prettier/blob/main/src/config/prettier-config/config-searcher.js
const CONFIG_FILES = [
  "package.json",
  // ponytail: `package.yaml` is not searched, since checking its `prettier` field requires YAML parser
  ".prettierrc",
  ".prettierrc.json",
  ".prettierrc.yml",
  ".prettierrc.yaml",
  ".prettierrc.json5",
  ".prettierrc.js",
  "prettier.config.js",
  ".prettierrc.ts",
  "prettier.config.ts",
  ".prettierrc.mjs",
  "prettier.config.mjs",
  ".prettierrc.mts",
  "prettier.config.mts",
  ".prettierrc.cjs",
  "prettier.config.cjs",
  ".prettierrc.cts",
  "prettier.config.cts",
  ".prettierrc.toml",
];

const JS_EXTENSIONS = new Set([".js", ".mjs", ".cjs", ".ts", ".mts", ".cts"]);

async function findPrettierConfigFile(cwd: string) {
  const names = new Set(await readdir(cwd));
  for (const name of CONFIG_FILES) {
    if (!names.has(name)) continue;
    const path = join(cwd, name);
    // oxlint-disable-next-line no-await-in-loop -- only for `package.json`
    if (name === "package.json" && (await importJson(path)).prettier === undefined) continue;
    return path;
  }
  return null;
}

// Only the formats loadable without extra parsers are supported.
async function loadPrettierConfig(path: string): Promise<Record<string, unknown>> {
  const name = basename(path);
  const ext = extname(path);

  let config;
  if (name === "package.json") {
    config = (await importJson(path)).prettier;
  } else if (name === ".prettierrc") {
    try {
      config = JSON.parse(await readFile(path, "utf8"));
    } catch {
      throw new Error(
        "Only JSON format is supported for `.prettierrc`. Convert it to JSON and re-run.",
      );
    }
  } else if (ext === ".json") {
    config = await importJson(path);
  } else if (JS_EXTENSIONS.has(ext)) {
    config = (await import(pathToFileURL(path).href)).default;
  } else {
    throw new Error(
      "YAML, JSON5 and TOML formats are not supported. Convert it to JSON or JS and re-run.",
    );
  }

  // Shareable config, e.g. `"prettier": "@company/prettier-config"`
  if (typeof config === "string") {
    const mod = createRequire(path)(config);
    config = mod?.__esModule ? mod.default : mod;
  }

  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new Error("Config must be an object.");
  }
  return config;
}

async function importJson(path: string) {
  return (await import(pathToFileURL(path).href, { with: { type: "json" } })).default;
}

async function resolvePrettierIgnore(cwd: string) {
  const ignores = [];

  try {
    const content = await readFile(join(cwd, ".prettierignore"), "utf8");

    const lines = content.split("\n");
    for (let line of lines) {
      line = line.trim();
      if (line === "" || line.startsWith("#")) {
        continue;
      }
      ignores.push(line);
    }
  } catch {}

  return ignores;
}

// ---

// Prettier options that Oxfmt supports as-is.
const COMPATIBLE_OPTIONS = new Set([
  "arrowParens",
  "bracketSameLine",
  "bracketSpacing",
  "embeddedLanguageFormatting",
  "endOfLine",
  "experimentalOperatorPosition",
  "htmlWhitespaceSensitivity",
  "jsxSingleQuote",
  "objectWrap",
  "printWidth",
  "proseWrap",
  "quoteProps",
  "semi",
  "singleAttributePerLine",
  "singleQuote",
  "tabWidth",
  "trailingComma",
  "useTabs",
  "vueIndentScriptAndStyle",
]);

// Migrate top-level or `overrides[].options` Prettier options, except plugin-specific ones.
function migrateOptions(
  options: Record<string, unknown>,
  isTopLevel: boolean,
): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  const prefix = isTopLevel ? "" : "overrides: ";

  for (const [key, value] of Object.entries(options)) {
    // Plugin-specific options - handled by `migratePluginOptions()`
    if (PLUGIN_OPTIONS.has(key)) continue;
    if (!COMPATIBLE_OPTIONS.has(key)) {
      warnings.push(`${prefix}"${key}" is not supported, skipping...`);
      continue;
    }
    // Oxfmt does not support this, fallback to default
    if (key === "endOfLine" && value === "auto") {
      warnings.push(`${prefix}"endOfLine": "auto" is not supported, skipping...`);
      continue;
    }
    result[key] = value;
  }

  return result;
}

// Prettier's `parser` (built-in and known plugins) to Oxfmt's `associations[].language`.
// Not listed (e.g. `flow`, `hermes`, `json-stringify`, `lwc`, `mjml`) are not supported.
const PARSER_TO_LANGUAGE = new Map<unknown, Language>([
  ["babel", "javascript"],
  ["acorn", "javascript"],
  ["espree", "javascript"],
  ["meriyah", "javascript"],
  // `@prettier/plugin-oxc`, `@prettier/plugin-yuku`
  ["oxc", "javascript"],
  ["yuku", "javascript"],
  ["typescript", "typescript"],
  ["babel-ts", "typescript"],
  ["oxc-ts", "typescript"],
  ["yuku-ts", "typescript"],
  ["json", "json"],
  ["jsonc", "jsonc"],
  ["json5", "json5"],
  ["css", "css"],
  ["scss", "scss"],
  ["less", "less"],
  ["graphql", "graphql"],
  ["yaml", "yaml"],
  ["markdown", "markdown"],
  ["remark", "markdown"],
  ["mdx", "mdx"],
  ["html", "html"],
  ["angular", "angular-html"],
  ["vue", "vue"],
  ["glimmer", "handlebars"],
  ["svelte", "svelte"],
  ["astro", "astro"],
]);

// Plugin options: only enable when the corresponding Prettier plugin is used.
// Empty object means "enabled with defaults"; Tailwind, Svelte and Astro are disabled by default.
// In overrides, namespaces are deep-merged with top-level, so only set ones are needed.
function migratePluginOptions(
  options: Record<string, unknown>,
  enabledPlugins: Set<string>,
  isTopLevel: boolean,
): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [plugin, oxfmtKey, mapping, transform] of NAMESPACED_PLUGINS) {
    if (!enabledPlugins.has(plugin)) continue;
    const migrated = migrateMappedOptions(options, mapping, transform);
    if (!isTopLevel && Object.keys(migrated).length === 0) continue;
    result[oxfmtKey] = migrated;
    if (isTopLevel) console.log(`  - Migrated "${plugin}" options to "${oxfmtKey}"`);
  }

  return result;
}

// ---

// Map Oxfmt's namespaced option keys (left) to Prettier's flat option keys (right).
// Used by `migrateMappedOptions` to copy values from Prettier's flat options
// into a single Oxfmt namespace (e.g. `sortTailwindcss`, `svelte`, `astro`).
const TAILWIND_OPTION_MAPPING: Record<string, string> = {
  config: "tailwindConfig",
  stylesheet: "tailwindStylesheet",
  functions: "tailwindFunctions",
  attributes: "tailwindAttributes",
  preserveWhitespace: "tailwindPreserveWhitespace",
  preserveDuplicates: "tailwindPreserveDuplicates",
};

const SVELTE_OPTION_MAPPING: Record<string, string> = {
  allowShorthand: "svelteAllowShorthand",
  indentScriptAndStyle: "svelteIndentScriptAndStyle",
  sortOrder: "svelteSortOrder",
};

const ASTRO_OPTION_MAPPING: Record<string, string> = {
  allowShorthand: "astroAllowShorthand",
  skipFrontmatter: "astroSkipFrontmatter",
  compressHTML: "astroCompressHTML",
};

const NAMESPACED_PLUGINS = [
  ["prettier-plugin-tailwindcss", "sortTailwindcss", TAILWIND_OPTION_MAPPING, filterTailwindRegex],
  ["prettier-plugin-svelte", "svelte", SVELTE_OPTION_MAPPING, undefined],
  ["prettier-plugin-astro", "astro", ASTRO_OPTION_MAPPING, normalizeAstroCompressHTML],
] as const;

const KNOWN_PLUGINS = new Set<unknown>([
  "prettier-plugin-packagejson",
  ...NAMESPACED_PLUGINS.map(([plugin]) => plugin),
]);

const PLUGIN_OPTIONS = new Set(
  NAMESPACED_PLUGINS.flatMap(([, , mapping]) => Object.values(mapping)),
);

function migrateMappedOptions(
  options: Record<string, unknown>,
  mapping: Record<string, string>,
  transform?: (prettierKey: string, value: unknown) => unknown,
): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [oxfmtKey, prettierKey] of Object.entries(mapping)) {
    const value = options[prettierKey];
    if (value === undefined) continue;
    result[oxfmtKey] = transform ? transform(prettierKey, value) : value;
  }
  return result;
}

// `tailwindFunctions` / `tailwindAttributes` accept regex strings (e.g. `/^tw-/`)
// which Oxfmt does not support. Drop them and warn.
function filterTailwindRegex(prettierKey: string, value: unknown): unknown {
  if (
    (prettierKey !== "tailwindFunctions" && prettierKey !== "tailwindAttributes")
    || !Array.isArray(value)
  ) {
    return value;
  }
  return (value as unknown[]).filter((item): item is string => {
    if (typeof item !== "string") return false;
    const isRegex = item.startsWith("/") && item.endsWith("/");
    if (isRegex) {
      warnings.push(`Regexp in "${prettierKey}" option is not supported, skipping: "${item}"`);
    }
    return !isRegex;
  });
}

// `astroCompressHTML` also accepts `true` / `false` as aliases of `"html"` / `"none"`.
// Oxfmt accepts only the canonical strings.
function normalizeAstroCompressHTML(prettierKey: string, value: unknown): unknown {
  if (prettierKey !== "astroCompressHTML" || typeof value !== "boolean") return value;
  return value ? "html" : "none";
}
