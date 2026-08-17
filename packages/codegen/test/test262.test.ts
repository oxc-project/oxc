// Test262 conformance.
//
// Tests this package prints every Test262 fixture exactly as Rust `oxc_codegen` prints it.

import { readdir, readFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";

import { checkFixture, TEST262_DIR_PATH, test262SourceType } from "./utils/common.ts";

// Test262 keeps its fixtures' shared helpers in `harness`, which are not test cases
const fixturePaths = (await readdir(TEST262_DIR_PATH, { recursive: true }))
  .filter((path) => path.endsWith(".js") && !path.startsWith("harness/"))
  .sort();

describe.concurrent("test262", () => {
  for (const path of fixturePaths) {
    // oxlint-disable-next-line vitest/valid-title, vitest/expect-expect
    it(path, async (ctx) => {
      const sourceText = await readFile(pathJoin(TEST262_DIR_PATH, path), "utf8");
      if (!checkFixture(path, sourceText, "js", test262SourceType(sourceText))) ctx.skip();
    });
  }
});
