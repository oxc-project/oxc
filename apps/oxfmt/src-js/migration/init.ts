/* oxlint-disable no-console */

import {
  hasOxfmtrcFile,
  createBlankOxfmtrcFile,
  saveOxfmtrcFile,
  exitWithError,
} from "./shared.js";

/**
 * Run the `--init` command to scaffold a default `.oxfmtrc.json` file.
 */
export async function runInit() {
  const cwd = process.cwd();

  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt configuration file already exists.");
  }

  // Create blank config
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);
  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  try {
    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json`.");
  } catch {
    return exitWithError("Failed to create `.oxfmtrc.json`.");
  }
}
