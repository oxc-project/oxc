import assert from "node:assert";

import type { Plugin, Rule, ESTree } from "#oxlint/plugin";

type Node = ESTree.Node;

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
// In particular, we want to check that `steps` used when creating CFG is cleared after an error while visiting a file.
// If `steps` is cleared after that error, then `onCodePathStart` visitor will be called only once.
// If it's not cleared then we'll get extra calls to `Identifier` and `onCodePathStart` visitors,
// because the steps from the 1st file are replayed again for the 2nd file.
//
// Actually `debugAssert(steps.length === 0)` at start of `prepareSteps` in `src-js/plugins/cfg.ts` should
// throw in this case before the AST even gets walked, but we have 2 ways to spot this error.

let fileIndex = 0;

const rule: Rule = {
  // @ts-expect-error - TODO: Make the types for CFG events work
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
      onCodePathStart(codePath: any, node: Node) {
        context.report({
          message: `onCodePathStart: ${node.type}`,
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
