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

  // NOTE: Prettier's config resolving is based on each file,
  // but ours is based on the project root, typically `cwd`.
  // So we assume the config for a dummy file at the `cwd`.
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
  // NOTE: Some options unsupported by Oxfmt may still be valid when invoking Prettier.
  // However, to avoid inconsistency, we do not enable options that affect Oxfmt.
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);

  let hasSortPackageJsonPlugin = false;
  for (const [key, value] of Object.entries(prettierConfig ?? {})) {
    // Handle plugins - check for prettier-plugin-tailwindcss and warn about others
    if (key === "plugins" && Array.isArray(value)) {
      for (const plugin of (value as Options["plugins"])!) {
        if (plugin === "prettier-plugin-tailwindcss") {
          // Migrate `prettier-plugin-tailwindcss` options
          migrateTailwindOptions(prettierConfig!, oxfmtrc);
        } else if (plugin === "prettier-plugin-packagejson") {
          hasSortPackageJsonPlugin = true;
        } else if (typeof plugin === "string") {
          console.error(`  - plugins: "${plugin}" is not supported, skipping...`);
        } else {
          console.error(`  - plugins: custom plugin module is not supported, skipping...`);
        }
      }
      continue;
    }
    // Oxfmt does not support this, fallback to default
    if (key === "endOfLine" && value === "auto") {
      console.error(`  - "endOfLine: auto" is not supported, skipping...`);
      continue;
    }
    // Oxfmt does not support these experimental options yet
    if (key === "experimentalTernaries" || key === "experimentalOperatorPosition") {
      console.error(`  - "${key}" is not supported in JS/TS files yet`);
      continue;
    }

    // Skip Tailwind options - handled separately by migrateTailwindOptions
    if (key.startsWith("tailwind")) {
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
  // `experimentalSortPackageJson` is enabled by default in Oxfmt, but Prettier does not have this.
  // Only enable if `prettier-plugin-packagejson` is used.
  if (hasSortPackageJsonPlugin) {
    oxfmtrc.experimentalSortPackageJson = {};
    console.error(`  - Migrated "prettier-plugin-packagejson" to "experimentalSortPackageJson"`);
  } else {
    oxfmtrc.experimentalSortPackageJson = false;
  }
  // `embeddedLanguageFormatting` is not fully supported for JS-in-XXX yet.
  if (oxfmtrc.embeddedLanguageFormatting !== "off") {
    console.error(`  - "embeddedLanguageFormatting" in JS/TS files is not fully supported yet`);
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
  // but `overrides` field is not included in `resolveConfig()` result.
  // Automatic migration requires manual config file parsing.
  // See: https://github.com/oxc-project/oxc/issues/18215
  if ("overrides" in oxfmtrc) {
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

// ---

const TAILWIND_OPTION_MAPPING: Record<string, string> = {
  config: "tailwindConfig",
  stylesheet: "tailwindStylesheet",
  functions: "tailwindFunctions",
  attributes: "tailwindAttributes",
  preserveWhitespace: "tailwindPreserveWhitespace",
  preserveDuplicates: "tailwindPreserveDuplicates",
};

/**
 * Migrate prettier-plugin-tailwindcss options to Oxfmt's experimentalTailwindcss format.
 *
 * Prettier format:
 * ```json
 * {
 *   "plugins": ["prettier-plugin-tailwindcss"],
 *   "tailwindConfig": "./tailwind.config.js",
 *   "tailwindFunctions": ["clsx", "cn"]
 * }
 * ```
 *
 * Oxfmt format:
 * ```json
 * {
 *   "experimentalTailwindcss": {
 *     "config": "./tailwind.config.js",
 *     "functions": ["clsx", "cn"]
 *   }
 * }
 * ```
 */
function migrateTailwindOptions(
  prettierConfig: Record<string, unknown>,
  oxfmtrc: Record<string, unknown>,
): void {
  // Collect Tailwind options from Prettier config
  const tailwindOptions: Record<string, unknown> = {};
  for (const [oxfmtKey, prettierKey] of Object.entries(TAILWIND_OPTION_MAPPING)) {
    const value = prettierConfig[prettierKey];
    if (value !== undefined) {
      if (
        (prettierKey == "tailwindFunctions" || prettierKey == "tailwindAttributes") &&
        Array.isArray(value)
      ) {
        for (const item of value as string[]) {
          if (typeof item === "string" && item.startsWith("/") && item.endsWith("/")) {
            console.warn(
              `  - Do not support regex in "${prettierKey}" option yet, skipping: ${item}`,
            );
            continue;
          }
        }
      }
      tailwindOptions[oxfmtKey] = value;
    }
  }

  // Only add experimentalTailwindcss if plugin is used or options are present
  oxfmtrc.experimentalTailwindcss = tailwindOptions;
  console.log("Migrated prettier-plugin-tailwindcss options to experimentalTailwindcss");
}
