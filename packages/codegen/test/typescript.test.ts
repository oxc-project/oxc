// TypeScript conformance.
//
// Tests this package prints every TypeScript fixture exactly as Rust `oxc_codegen` prints it.
//
// A TypeScript test file can hold several units, separated by `// @filename:` directives,
// each with its own language and module setting. `makeUnitsFromTest` splits them out,
// and every unit is checked.

import { readdir, readFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { describe, it } from "vitest";

import { checkFixture, langOf, TS_CASES_DIR_PATH } from "./utils/common.ts";
import { makeUnitsFromTest } from "./utils/typescript-make-units-from-test.ts";

// The two directories which hold the test cases.
// The rest of `tests/cases` is editor scenarios and multi-file projects, which are not
// standalone sources. This is the same pair `tasks/coverage` takes its TypeScript suite from.
const CASE_DIR_NAMES = ["compiler", "conformance"];

// Every extension `makeUnitsFromTest` knows how to give a source type to
const EXTENSION_REGEX = /\.([cm]?[jt]sx?)$/;

const fixturePaths = (
  await Promise.all(
    CASE_DIR_NAMES.map(async (dirName) => {
      const paths = await readdir(pathJoin(TS_CASES_DIR_PATH, dirName), { recursive: true });
      return paths.filter((path) => EXTENSION_REGEX.test(path)).map((path) => `${dirName}/${path}`);
    }),
  )
)
  .flat()
  .sort();

// `oxc-parser` deserializes the AST with recursive JS functions, and these fixtures nest binary
// expressions deeply enough to overflow the stack.
// They sit right on the limit, so they fail intermittently rather than every time.
// That is a limit of the parser rather than of the printer.
//
// `napi/parser`'s tests never meet them. Their TypeScript fixture list comes from the `estree-conformance` submodule,
// which holds no recording for either file.
const SKIPPED_PATHS = new Set([
  "compiler/binderBinaryExpressionStress.ts",
  "compiler/binderBinaryExpressionStressJs.ts",
]);

describe.concurrent("TypeScript", () => {
  // oxlint-disable-next-line vitest/expect-expect
  it.for(fixturePaths)("%s", async (path, ctx) => {
    if (SKIPPED_PATHS.has(path)) ctx.skip();

    let sourceText = await readFile(pathJoin(TS_CASES_DIR_PATH, path), "utf8");
    // Trim off UTF-8 BOM
    if (sourceText.charCodeAt(0) === 0xfeff) sourceText = sourceText.slice(1);

    const { tests } = makeUnitsFromTest(path, sourceText);

    let checked = 0;
    for (const { name, content, sourceType } of tests) {
      const lang = langOf(sourceType.typescript, sourceType.jsx);

      // Always a TS-shaped AST, even for the `.js` units. The TypeScript suite has fixtures
      // which put TS syntax in a `.js` file on purpose, and Oxc's Rust parser keeps and prints it,
      // so the JS side has to be able to see it too.
      const checkedUnit = checkFixture(
        name,
        content,
        lang,
        sourceType.module ? "module" : "unambiguous",
        "ts",
      );
      if (checkedUnit) checked++;
    }

    // Every unit failed to parse, so the fixture contributed nothing to check
    if (checked === 0) ctx.skip();
  });
});
