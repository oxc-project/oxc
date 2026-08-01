/* oxlint-disable no-console */

import { join } from "node:path";
import { readFile } from "node:fs/promises";
import { hasOxfmtrcFile, createBlankOxfmtrcFile, saveOxfmtrcFile, exitWithError } from "./shared";
import { Options } from "prettier";

/**
 * Run the `--migrate prettier` command to migrate various Prettier's config to `.oxfmtrc.json` file.
 * https://prettier.io/docs/configuration
 */
export async function runMigratePrettier() {
  const cwd = process.cwd();

  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt configuration file already exists.");
  }

  // XXX: If you statically import `prettier` here,
  // completely unsure why, but Prettier hangs forever when run via `napi`...
  const { resolveConfigFile, resolveConfig } = await import("prettier");

  // TODO: Support nested config?
  // For now, we assume the config for a dummy file at the `cwd`.
  const prettierConfigPath = await resolveConfigFile(join(cwd, "dummy.js"));

  // No Prettier config found, fallback with `--init` behavior
  if (!prettierConfigPath) {
    console.log("No Prettier configuration file found.");

    const oxfmtrc = await createBlankOxfmtrcFile(cwd);
    const jsonStr = JSON.stringify(oxfmtrc, null, 2);

    // TODO: Create napi `validateConfig()` and use to ensure validity?

    try {
      await saveOxfmtrcFile(cwd, jsonStr);
      console.log("Created `.oxfmtrc.json` instead.");
    } catch {
      exitWithError("Failed to create `.oxfmtrc.json`.");
    }

    return;
  }

  let prettierConfig;
  try {
    prettierConfig = await resolveConfig(prettierConfigPath, {
      // Avoid merging `.editorconfig` values
      editorconfig: false,
    });
    console.log("Found Prettier configuration at:", prettierConfigPath);
  } catch {
    return exitWithError(`Failed to parse: ${prettierConfigPath}`);
  }

  // Start with blank, then fill in from `prettierConfig`.
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);

  let hasTailwindcssPlugin = false;
  let hasSortPackageJsonPlugin = false;
  let hasSveltePlugin = false;
  for (const [key, value] of Object.entries(prettierConfig ?? {})) {
    // Handle plugins - check for known plugins and warn about others
    if (key === "plugins" && Array.isArray(value)) {
      for (const plugin of (value as Options["plugins"])!) {
        if (plugin === "prettier-plugin-tailwindcss") {
          hasTailwindcssPlugin = true;
        } else if (plugin === "prettier-plugin-packagejson") {
          hasSortPackageJsonPlugin = true;
        } else if (plugin === "prettier-plugin-svelte") {
          hasSveltePlugin = true;
        } else if (typeof plugin === "string") {
          console.error(`  - plugins: "${plugin}" is not supported, skipping...`);
        } else {
          console.error(`  - plugins: custom plugin module is not supported, skipping...`);
        }
      }
      continue;
    }
    // Per-file options that should not appear in a shared config; drop if leaked in
    if (key === "parser" || key === "filepath") {
      continue;
    }
    // Prettier-only options without an Oxfmt equivalent
    if (key === "requirePragma" || key === "insertPragma") {
      console.error(`  - "${key}" is not supported, skipping...`);
      continue;
    }
    // Oxfmt does not support this, fallback to default
    if (key === "endOfLine" && value === "auto") {
      console.error(`  - "endOfLine: auto" is not supported, skipping...`);
      continue;
    }
    // Oxfmt does not support these experimental options yet
    if (key === "experimentalTernaries" || key === "experimentalOperatorPosition") {
      console.error(`  - "${key}" is not supported yet`);
      continue;
    }

    // Skip plugin-specific options - handled separately
    if (key.startsWith("tailwind") || key.startsWith("svelte")) {
      continue;
    }

    // Otherwise, copy the value.
    // This may include options that do not affect Oxfmt, like `vueIndentScriptAndStyle`.
    oxfmtrc[key] = value;
  }

  // `printWidth` has different default between Prettier and Oxfmt.
  // Oxfmt default is 100, Prettier default is 80.
  if (typeof oxfmtrc.printWidth !== "number") {
    console.error(
      `  - "printWidth" is not set in Prettier config, defaulting to 80 (Oxfmt default: 100)`,
    );
    oxfmtrc.printWidth = 80;
  }
  // `sortPackageJson` is enabled by default in Oxfmt, but Prettier does not have this.
  // Only enable if `prettier-plugin-packagejson` is used.
  if (hasSortPackageJsonPlugin) {
    oxfmtrc.sortPackageJson = {};
    console.error(`  - Migrated "prettier-plugin-packagejson" to "sortPackageJson"`);
  } else {
    oxfmtrc.sortPackageJson = false;
  }
  // Plugin options: only enable when the corresponding Prettier plugin is used.
  // Empty object means "enabled with defaults"; both Tailwind and Svelte are disabled by default.
  if (hasTailwindcssPlugin) {
    oxfmtrc.sortTailwindcss = migrateMappedOptions(
      prettierConfig!,
      TAILWIND_OPTION_MAPPING,
      filterTailwindRegex,
    );
    console.log("Migrated prettier-plugin-tailwindcss options to sortTailwindcss");
  }
  if (hasSveltePlugin) {
    oxfmtrc.svelte = migrateMappedOptions(prettierConfig!, SVELTE_OPTION_MAPPING);
    console.log("Migrated prettier-plugin-svelte options to svelte");
  }

  // Migrate `ignorePatterns` from `.prettierignore`
  const ignores = await resolvePrettierIgnore(cwd);
  if (ignores.length > 0) {
    console.log("Migrated ignore patterns from `.prettierignore`");
  }
  // Keep ignorePatterns at the bottom
  delete oxfmtrc.ignorePatterns;
  oxfmtrc.ignorePatterns = ignores;

  // TODO: Oxfmt now supports `overrides`,
  // but `overrides` field is stripped from `resolveConfig()` result.
  // Automatic migration requires reading the raw config file and handling each format
  // (JSON, JSONC, YAML, JS/CJS/MJS, TOML, package.json).
  // See: https://github.com/oxc-project/oxc/issues/18215
  if (await rawConfigHasOverrides(prettierConfigPath)) {
    console.warn(
      `  - "overrides" cannot be migrated automatically. See: https://github.com/oxc-project/oxc/issues/18215`,
    );
  }

  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  try {
    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json`.");
  } catch {
    return exitWithError("Failed to create `.oxfmtrc.json`.");
  }
}

// ---

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

// Best-effort detection of a top-level `overrides` key in the raw config file.
// Uses string matching to cover all config formats (JSON/JSONC/JSON5/YAML/JS/TOML)
// Matches:
// - JSON/JSONC/JSON5/JS: `"overrides":` / `overrides:`
// - YAML: `overrides:`
// - TOML: `[[overrides]]` / `overrides =`
async function rawConfigHasOverrides(configPath: string): Promise<boolean> {
  try {
    const content = await readFile(configPath, "utf8");
    return /^\s*(?:\[\[\s*overrides\s*\]\]|["']?overrides["']?\s*[:=])/mv.test(content);
  } catch {
    return false;
  }
}

// ---

// Map Oxfmt's namespaced option keys (left) to Prettier's flat option keys (right).
// Used by `migrateMappedOptions` to copy values from Prettier's flat config
// into a single Oxfmt namespace (e.g. `sortTailwindcss`, `svelte`).
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

function migrateMappedOptions(
  prettierConfig: Record<string, unknown>,
  mapping: Record<string, string>,
  transform?: (prettierKey: string, value: unknown) => unknown,
): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [oxfmtKey, prettierKey] of Object.entries(mapping)) {
    const value = prettierConfig[prettierKey];
    if (value === undefined) continue;
    result[oxfmtKey] = transform ? transform(prettierKey, value) : value;
  }
  return result;
}

// `tailwindFunctions` / `tailwindAttributes` accept regex strings (e.g. `/^tw-/`)
// which Oxfmt does not support. Drop them and warn.
function filterTailwindRegex(prettierKey: string, value: unknown): unknown {
  if (
    (prettierKey !== "tailwindFunctions" && prettierKey !== "tailwindAttributes") ||
    !Array.isArray(value)
  ) {
    return value;
  }
  return (value as unknown[]).filter((item): item is string => {
    if (typeof item !== "string") return false;
    const isRegex = item.startsWith("/") && item.endsWith("/");
    if (isRegex) {
      console.warn(`  - Regexp in "${prettierKey}" option is not supported, skipping: ${item}`);
    }
    return !isRegex;
  });
}
