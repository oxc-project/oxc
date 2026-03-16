import assert from "node:assert";

import type { Plugin, Rule } from "#oxlint/plugins";

// Aim of this test is:
//
// 1. Check that errors thrown during visitor compilation are handled correctly and shown to user as diagnostics.
// 2. Check that global state is reset after an error during visiting a file, so it's in correct initial state
//    when Oxlint starts linting the next file.
//
// The 2nd is tricky to test because usually the order Oxlint lints files in is non-deterministic.
// To make this test deterministic, we run it with `oxlint --threads 1`
// (`options.json` file for this fixture contains `"singleThread": true`).
// This guarantees that `1.js` is linted before `2.js`.
//
// This rule throws an error during visitor compilation when linting 1st file. If global state is not reset properly
// before linting 2nd file, `Identifier` visitor for 1st file will not be cleaned up and will fire on 2nd file.

let fileIndex = 0;

const rule: Rule = {
  create(context) {
    const isFirstFileLinted = fileIndex === 0;
    fileIndex++;

    // Check the order files get linted in is what we expect
    const isFirstFile = context.filename.endsWith("1.js");
    assert(isFirstFile === isFirstFileLinted);

    if (isFirstFile) {
      return {
        Identifier(node) {
          context.report({
            message: `Identifier in 1st file: ${node.name}`,
            node,
          });
        },

        // Illegal value, causes error during visitor compilation
        Program: 123 as any,
      };
    }

    return {
      Identifier(node) {
        context.report({
          message: `Identifier in 2nd file: ${node.name}`,
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
