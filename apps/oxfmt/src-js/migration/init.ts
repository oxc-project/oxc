/* oxlint-disable no-console */

import { stat, writeFile } from "node:fs/promises";

async function isFile(path: string) {
  try {
    const stats = await stat(path);
    return stats.isFile();
  } catch {
    return false;
  }
}

/**
 * Run the `--init` command to scaffold a default `.oxfmtrc.json` file.
 */
export async function runInit() {
  // Check if config file already exists
  if ((await isFile(".oxfmtrc.json")) || (await isFile(".oxfmtrc.jsonc"))) {
    console.error("Configuration file already exists.");
    process.exitCode = 1;
    return;
  }

  // Build config object
  const schemaPath = "./node_modules/oxfmt/configuration_schema.json";

  const config: Record<string, unknown> = {
    // Add `$schema` field at the top if schema file exists in `node_modules`
    $schema: schemaPath,
    // `ignorePatterns` is included to make visible and preferred over `.prettierignore`
    ignorePatterns: [],
  };

  // Remove if this command is run with e.g. `npx`
  // NOTE: To keep `$schema` field at the top, we delete it here instead of defining conditionally above
  if (!(await isFile(schemaPath))) {
    delete config.$schema;
  }

  try {
    const jsonStr = JSON.stringify(config, null, 2);

    // TODO: Call napi `validateConfig()` to ensure validity

    await writeFile(".oxfmtrc.json", jsonStr + "\n");
    console.log("Created `.oxfmtrc.json`.");
  } catch {
    console.error("Failed to write `.oxfmtrc.json`.");
    process.exitCode = 1;
  }
}
