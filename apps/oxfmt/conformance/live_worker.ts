// Runs one `run_live.ts` comparison (a fixture under one option set) in a `tinypool` worker,
// Prettier is CPU-bound JS, so the main thread alone serializes it.

import { readFileSync } from "node:fs";
import prettier from "prettier";
import * as astroPlugin from "prettier-plugin-astro";
import * as sveltePlugin from "prettier-plugin-svelte";
import { format } from "../dist/index.js";

export type LiveTask = {
  name: string;
  fullPath: string;
  isProject: boolean;
  options: Record<string, unknown>;
};

/** `null` when both agree, otherwise both outputs (`"ERROR"` for a failed one). */
export default async function run({ name, fullPath, isProject, options }: LiveTask) {
  let input = readFileSync(fullPath, "utf8");
  let expected = await formatWithPrettier(name, input, options);
  // A project hands over Prettier's output, unless Prettier cannot read it back either
  if (isProject && expected !== null) {
    const again = await formatWithPrettier(name, expected, options);
    if (again !== null) [input, expected] = [expected, again];
  }
  const prettierResult = expected ?? "ERROR";
  const oxfmtResult = await formatWithOxfmt(name, input, options);

  // Both failing to parse is not a pass, the fixture exercised nothing
  if (oxfmtResult === prettierResult && oxfmtResult !== "ERROR") return null;
  return { name, oxfmt: oxfmtResult, prettier: prettierResult };
}

/** `null` when Prettier fails. */
async function formatWithPrettier(
  fileName: string,
  content: string,
  options: Record<string, unknown>,
): Promise<string | null> {
  try {
    return await prettier.format(content, {
      ...options,
      filepath: fileName,
      plugins: [sveltePlugin, astroPlugin],
    });
  } catch {
    return null;
  }
}

async function formatWithOxfmt(
  fileName: string,
  content: string,
  options: Record<string, unknown>,
): Promise<string> {
  const res = await format(fileName, content, options);
  return res.errors.length === 0 ? res.code : "ERROR";
}
