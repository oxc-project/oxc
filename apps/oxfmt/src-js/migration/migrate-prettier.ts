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
  for (const [key, value] of Object.entries(prettierConfig ?? {})) {
    // Oxfmt does not support this
    if (key === "overrides") {
      console.error(`  - "overrides" is not supported, skipping...`);
      continue;
    }
    // Oxfmt does not yet support plugins
    if (key === "plugins") {
      console.error(`  - "plugins" is not supported yet, skipping...`);
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
  // `embeddedLanguageFormatting` is not fully supported yet and default "off" in Oxfmt.
  // Prettier default is "auto".
  if (oxfmtrc.embeddedLanguageFormatting !== "off") {
    console.error(`  - "embeddedLanguageFormatting" in JS/TS files is not fully supported yet`);
  }

  // Migrate `ignorePatterns` from `.prettierignore`
  const ignores = await resolvePrettierIgnore(cwd);
  if (0 < ignores.length) {
    oxfmtrc.ignorePatterns = ignores;
    console.log("Migrated ignore patterns from `.prettierignore`");
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
