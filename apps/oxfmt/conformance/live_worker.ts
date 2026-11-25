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

/** The comparison with Prettier, `null` when both agree, otherwise both outputs (`"ERROR"` for a failed one);
 * and for a project, whether oxfmt's output of the file as written is no fixpoint (`null` when it is, `true` when the second pass errored). */
export default async function run({ name, fullPath, isProject, options }: LiveTask) {
  const source = readFileSync(fullPath, "utf8");
  let input = source;
  let expected = await formatWithPrettier(name, input, options);
  // A project hands over Prettier's output, unless Prettier cannot read it back either
  if (isProject && expected !== null) {
    const again = await formatWithPrettier(name, expected, options);
    if (again !== null) [input, expected] = [expected, again];
  }
  const prettierResult = expected ?? "ERROR";
  const oxfmtResult = await formatWithOxfmt(name, input, options);

  // Both failing to parse is not a pass, the fixture exercised nothing
  const failure =
    oxfmtResult === prettierResult && oxfmtResult !== "ERROR"
      ? null
      : { name, oxfmt: oxfmtResult, prettier: prettierResult };
  // A project's file as written, which Prettier's output does not show (the shapes Prettier normalizes away)
  const nonIdempotent = isProject ? await nonIdempotentOf(name, source, options) : null;
  return { failure, nonIdempotent };
}

/** `null` when oxfmt's output of `content` formats to itself (or `content` fails), `true` when the second pass errored. */
async function nonIdempotentOf(
  fileName: string,
  content: string,
  options: Record<string, unknown>,
): Promise<boolean | null> {
  const first = await formatWithOxfmt(fileName, content, options);
  if (first === "ERROR") return null;
  const second = await formatWithOxfmt(fileName, first, options);
  return second === first ? null : second === "ERROR";
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
