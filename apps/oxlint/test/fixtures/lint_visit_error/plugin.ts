import assert from "node:assert";

import type { Plugin, Rule } from "#oxlint/plugin";

// Aim of this test is:
//
// 1. Check that errors thrown during AST visitation are handled correctly and shown to user as diagnostics.
// 2. Check that global state is reset after an error during visiting a file, so it's in correct initial state
//    when Oxlint starts linting the next file.
//
// The 2nd is tricky to test because usually the order Oxlint lints files in is non-deterministic.
// To make this test deterministic, we run it with `oxlint --threads 1`
// (`options.json` file for this fixture contains `"singleThread": true`).
// This guarantees that `1.js` is linted before `2.js`.
//
// This rule throws an error during AST visitation when linting 1st file. If global state is not reset properly
// before linting 2nd file, debug assertions will throw an error on 2nd file.
//
// In particular, we want to check that `ancestors` is cleared after an error while visiting a file.
// If `ancestors` is cleared after that error, then `Program Program` visitor will not be called.
// If it *is* called, then we have 2 `Program` nodes in `ancestors`, which is wrong
// - some nodes from 1st file's AST remain in `ancestors` when we start walking AST of 2nd file.
//
// Actually `debugAssert(ancestors.length === 0)` before calling `walkProgram` in `src-js/plugins/lint.ts` should
// throw in this case before the AST even gets walked, so the `"Program Program"` visitor should not be called anyway.
// But including it just to make sure.

let fileIndex = 0;

const rule: Rule = {
  create(context) {
    const isFirstFileLinted = fileIndex === 0;
    fileIndex++;

    // Check the order files get linted in is what we expect
    const isFirstFile = context.filename.endsWith("1.js");
    assert(isFirstFile === isFirstFileLinted);

    return {
      Identifier(node) {
        if (isFirstFile) throw new Error(`Identifier in 1st file: ${node.name}`);

        context.report({
          message: `Identifier in 2nd file: ${node.name}`,
          node,
        });
      },

      // This visitor should not be called.
      // If it is, we have 2 `Program` nodes in `ancestors`.
      "Program Program"(node) {
        context.report({
          message: "Ancestors has not been cleared after error",
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: rule,
  },
};

export default plugin;
