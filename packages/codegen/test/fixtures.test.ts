// Local fixture conformance.
//
// Put a JavaScript or TypeScript reproduction anywhere under `test/fixtures/`.
// This suite infers its language from the extension and checks the normal and source-map printer
// builds against Rust `oxc_codegen`, in both `preserveParens` modes.

import { readdir, readFile } from "node:fs/promises";
import { join as pathJoin, relative as pathRelative } from "node:path";
import { describe, expect, it } from "vitest";

import { checkFixture } from "./utils/common.ts";

import type { Lang } from "./utils/common.ts";

const FIXTURES_DIR_PATH = pathJoin(import.meta.dirname, "fixtures");

const LANG_BY_EXTENSION: Record<string, Lang | undefined> = {
  ".cjs": "js",
  ".cts": "ts",
  ".js": "js",
  ".jsx": "jsx",
  ".mjs": "js",
  ".mts": "ts",
  ".ts": "ts",
  ".tsx": "tsx",
};

const fixturePaths = (await readdir(FIXTURES_DIR_PATH, { recursive: true, withFileTypes: true }))
  .filter((entry) => entry.isFile() && entry.name !== "README.md")
  .map((entry) => pathJoin(pathRelative(FIXTURES_DIR_PATH, entry.parentPath), entry.name))
  .sort();

describe.concurrent("local fixtures", () => {
  it.each(fixturePaths)("%s", async (path) => {
    const sourceText = await readFile(pathJoin(FIXTURES_DIR_PATH, path), "utf8");
    const lang = langOfPath(path);
    if (lang === undefined) throw new Error(`Unsupported fixture extension: ${path}`);

    // Local fixtures are ordinary source files rather than test cases with frontmatter. Let the
    // parser recognize imports and exports while still accepting scripts.
    expect(checkFixture(path, sourceText, lang, "unambiguous")).toBe(true);
  });
});

function langOfPath(path: string): Lang | undefined {
  return LANG_BY_EXTENSION[path.slice(path.lastIndexOf("."))];
}
