// Acorn-JSX conformance.
//
// Tests this package prints every JSX fixture exactly as Rust `oxc_codegen` prints it.

import { readdir, readFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";

import { checkFixture, JSX_DIR_PATH } from "./utils/common.ts";

// The directory holds the recorded ASTs and tokens alongside the sources.
// Only the sources matter here - both printers work from a fresh parse.
const fixturePaths = (await readdir(JSX_DIR_PATH, { recursive: true }))
  .filter((path) => path.endsWith(".jsx"))
  .sort();

describe.concurrent("JSX", () => {
  // oxlint-disable-next-line vitest/expect-expect
  it.for(fixturePaths)("%s", async (path, ctx) => {
    const sourceText = await readFile(pathJoin(JSX_DIR_PATH, path), "utf8");
    if (!checkFixture(path, sourceText, "jsx", "unambiguous")) ctx.skip();
  });
});
