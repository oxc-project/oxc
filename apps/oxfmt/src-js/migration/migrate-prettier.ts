/* oxlint-disable no-console */

import { join } from "node:path";
import { readFile } from "node:fs/promises";
import {
  hasOxfmtrcFile,
  createBlankOxfmtrcFile,
  saveOxfmtrcFile,
  exitWithError,
} from "./shared.js";

/**
 * Run the `--migrate prettier` command to migrate various Prettier's config to `.oxfmtrc.json` file.
 * https://prettier.io/docs/configuration
 */
export async function runMigratePrettier() {
  const cwd = process.cwd();

  console.log("Starting Prettier migration...");

  // Check if config file already exists
  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt configuration file already exists.");
  }

  // XXX: If you statically import `prettier` here,
  // completely unsure why, but Prettier hangs forever when run via `napi`.
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

    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json` instead.");
    return;
  }

  let prettierConfig;
  try {
    // Use `editorconfig: false` to avoid merging `.editorconfig` values
    prettierConfig = await resolveConfig(prettierConfigPath, { editorconfig: false });
    console.log("Found Prettier configuration at:", prettierConfigPath);
  } catch {
    return exitWithError(`Failed to parse: ${prettierConfigPath}`);
  }

  // Start with blank, then fill in from `prettierConfig`
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);
  for (const [key, value] of Object.entries(prettierConfig ?? {})) {
    // Skip unsupported options
    if (key === "plugins") {
      console.warn(`- "plugins" is not supported yet, skipping...`);
      continue;
    }
    if (key === "overrides") {
      console.warn(`- "overrides" is not supported, skipping...`);
      continue;
    }

    // Handle specific options
    if (key === "endOfLine" && value === "auto") {
      console.warn(`- "endOfLine: auto" is not supported, changing to "lf"`);
      oxfmtrc.endOfLine = "lf";
      continue;
    }

    // Warn partial support options
    if (key === "experimentalTernaries" || key === "experimentalOperatorPosition") {
      console.warn(`- "${key}" is not supported in JS/TS files yet`);
    }
    if (key === "embeddedLanguageFormatting" && value !== "off") {
      console.warn(`- "embeddedLanguageFormatting" in JS/TS files is not fully supported yet`);
    }
    oxfmtrc[key] = value;
  }

  // Fallback for missing `printWidth`
  if (typeof oxfmtrc.printWidth !== "number") {
    console.warn(
      `- "printWidth" is not set in Prettier config, defaulting to 80 (Oxfmt default: 100)`,
    );
    oxfmtrc.printWidth = 80;
  }

  // Migrate `ignorePatterns` from `.prettierignore`
  const ignores = await resolvePrettierIgnore(cwd);
  if (0 < ignores.length) {
    oxfmtrc.ignorePatterns = ignores;
    console.log("Migrated ignore patterns from `.prettierignore`");
  }

  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  await saveOxfmtrcFile(cwd, jsonStr);
  console.log("Created `.oxfmtrc.json` from Prettier configuration.");
  console.log("Prettier migration completed.");
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
