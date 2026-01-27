import { defineRule } from "#oxlint/plugin";

import type { Node } from "#oxlint/plugin";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const createRule = defineRule({
  create(context) {
    context.report({ message: `create body:\nthis === rule: ${this === createRule}`, node: SPAN });

    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\nfilename: ${context.filename}`,
          node,
        });
      },
    };
  },
});

// This aims to test that `createOnce` is called once only, and `before` hook is called once per file.
// i.e. Oxlint calls `createOnce` directly, and not the `create` method that `defineRule` adds to the rule.
let createOnceCallCount = 0;

const createOnceRule = defineRule({
  createOnce(context) {
    createOnceCallCount++;

    // `fileNum` should be different for each file.
    // `identNum` should start at 1 for each file.
    let fileNum = 0,
      identNum: number;
    // Note: Files are processed in unpredictable order, so `files/1.js` may be `fileNum` 1 or 2.
    // Therefore, collect all visits and check them in `after` hook of the 2nd file.
    const visits: { fileNum: number; identNum: number }[] = [];

    // `this` should be the rule object returned by `defineRule`
    // oxlint-disable-next-line typescript-eslint/no-this-alias
    const topLevelThis = this;

    return {
      before() {
        fileNum++;
        identNum = 0;

        context.report({
          message:
            "before hook:\n" +
            `createOnce call count: ${createOnceCallCount}\n` +
            `this === rule: ${topLevelThis === createOnceRule}\n` +
            `filename: ${context.filename}`,
          node: SPAN,
        });
      },
      Identifier(node) {
        identNum++;
        visits.push({ fileNum, identNum });

        context.report({
          message:
            `ident visit fn "${node.name}":\n` +
            `identNum: ${identNum}\n` +
            `filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({
          message: "after hook:\n" + `identNum: ${identNum}\n` + `filename: ${context.filename}`,
          node: SPAN,
        });

        if (fileNum === 2) {
          visits.sort((v1, v2) => v1.fileNum - v2.fileNum);

          const expectedVisits = [
            { fileNum: 1, identNum: 1 },
            { fileNum: 1, identNum: 2 },
            { fileNum: 2, identNum: 1 },
            { fileNum: 2, identNum: 2 },
          ];

          if (
            visits.length !== expectedVisits.length ||
            visits.some(
              (v, i) =>
                v.fileNum !== expectedVisits[i].fileNum ||
                v.identNum !== expectedVisits[i].identNum,
            )
          ) {
            context.report({ message: `Unexpected visits: ${JSON.stringify(visits)}`, node: SPAN });
          }
        }
      },
    };
  },
});

// Tests that `before` hook returning `false` disables visiting AST for the file.
const createOnceBeforeFalseRule = defineRule({
  createOnce(context) {
    return {
      before() {
        context.report({
          message: "before hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });

        // Only visit AST for `files/2.js`
        return context.filename.endsWith("2.js");
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\n` + `filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({
          message: "after hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });
      },
    };
  },
});

// These 4 rules test that `createOnce` without `before` and `after` hooks works correctly.

const createOnceBeforeOnlyRule = defineRule({
  createOnce(context) {
    return {
      before() {
        context.report({
          message: "before hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\n` + `filename: ${context.filename}`,
          node,
        });
      },
    };
  },
});

const createOnceAfterOnlyRule = defineRule({
  createOnce(context) {
    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\n` + `filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({
          message: "after hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });
      },
    };
  },
});

const createOnceHooksOnlyRule = defineRule({
  createOnce(context) {
    return {
      // Neither hook should be called, because no AST node visitor functions
      before() {
        context.report({
          message: "before hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });
      },
      after() {
        context.report({
          message: "after hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });
      },
    };
  },
});

const createOnceNoHooksRule = defineRule({
  createOnce(context) {
    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\n` + `filename: ${context.filename}`,
          node,
        });
      },
    };
  },
});

export default {
  meta: {
    name: "define-rule-plugin",
  },
  rules: {
    create: createRule,
    "create-once": createOnceRule,
    "create-once-before-false": createOnceBeforeFalseRule,
    "create-once-before-only": createOnceBeforeOnlyRule,
    "create-once-after-only": createOnceAfterOnlyRule,
    "create-once-hooks-only": createOnceHooksOnlyRule,
    "create-once-no-hooks": createOnceNoHooksRule,
  },
};
