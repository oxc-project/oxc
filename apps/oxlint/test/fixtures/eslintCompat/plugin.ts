import { eslintCompatPlugin } from "#oxlint/plugins";

import type { Node, Rule, Visitor, ESTree } from "#oxlint/plugins";

type ESTreeNode = ESTree.Node;

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const createRule: Rule = {
  create(context) {
    context.report({
      message: `create body:\n` + `this === rule: ${this === createRule}`,
      node: SPAN,
    });

    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}":\n` + `filename: ${context.filename}`,
          node,
        });
      },
    };
  },
};

// This aims to test that `createOnce` is called once only, and `before` hook is called once per file.
// i.e. Oxlint calls `createOnce` directly, and not the `create` method that `eslintCompatPlugin` adds to the rule.
let createOnceCallCount = 0;

const createOnceRule: Rule = {
  createOnce(context) {
    createOnceCallCount++;

    // `fileNum` should be different for each file.
    // `identNum` should start at 1 for each file.
    let fileNum = 0,
      identNum: number;
    // Note: Files are processed in unpredictable order, so `files/1.js` may be `fileNum` 1 or 2.
    // Therefore, collect all visits and check them in `after` hook of the 2nd file.
    const visits: { fileNum: number; identNum: number }[] = [];

    // `this` should be the rule object
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
};

// This tests that `after` hook runs after all visit functions, even high-specificity ones matching `Program`.
const createOnceSelectorRule: Rule = {
  createOnce(context) {
    // `fileNum` should be different for each file
    let fileNum = 0;
    // Note: Files are processed in unpredictable order, so `files/1.js` may be `fileNum` 1 or 2.
    // Therefore, collect all visits and check them in `after` hook of the 2nd file.
    const visits: { fileNum: number; selector: string }[] = [];

    return {
      before() {
        fileNum++;
      },
      "*:exit"(node) {
        if (node.type !== "Program") return;

        visits.push({ fileNum, selector: "*" });

        context.report({
          message: `*:exit visit fn:\n` + `filename: ${context.filename}`,
          node,
        });
      },
      "Program:exit"(node) {
        visits.push({ fileNum, selector: "Program" });

        context.report({
          message: `Program:exit visit fn:\n` + `filename: ${context.filename}`,
          node,
        });
      },
      "[body]:exit"(node) {
        visits.push({ fileNum, selector: "[body]" });

        context.report({
          message: `[body]:exit visit fn:\n` + `filename: ${context.filename}`,
          node,
        });
      },
      "[body][body][body]:exit"(node) {
        visits.push({ fileNum, selector: "[body][body][body]" });

        context.report({
          message: `[body][body][body]:exit visit fn:\n` + `filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({
          message: "after hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });

        visits.push({ fileNum, selector: "after" });

        if (fileNum === 2) {
          visits.sort((v1, v2) => v1.fileNum - v2.fileNum);

          const expectedVisits = [
            { fileNum: 1, selector: "*" },
            { fileNum: 1, selector: "Program" },
            { fileNum: 1, selector: "[body]" },
            { fileNum: 1, selector: "[body][body][body]" },
            { fileNum: 1, selector: "after" },
            { fileNum: 2, selector: "*" },
            { fileNum: 2, selector: "Program" },
            { fileNum: 2, selector: "[body]" },
            { fileNum: 2, selector: "[body][body][body]" },
            { fileNum: 2, selector: "after" },
          ];

          if (
            visits.length !== expectedVisits.length ||
            visits.some(
              (v, i) =>
                v.fileNum !== expectedVisits[i].fileNum ||
                v.selector !== expectedVisits[i].selector,
            )
          ) {
            context.report({
              message: `Unexpected visits: ${JSON.stringify(visits)}`,
              node: SPAN,
            });
          }
        }
      },
    };
  },
};

// This tests that `after` hook runs after all CFG event handlers.
// This rules is only run on `files/cfg.js`, to ensure CFG events handlers do not affect behavior of other rules
// which don't use CFG event listeners.
const createOnceCfgRule: Rule = {
  createOnce(context) {
    // Collect all visits and check them in `after` hook
    const visits: { event: string; nodeType?: string }[] = [];

    return {
      onCodePathStart(_codePath: unknown, node: ESTreeNode) {
        visits.push({ event: "onCodePathStart", nodeType: node.type });
      },
      onCodePathEnd(_codePath: unknown, node: ESTreeNode) {
        visits.push({ event: "onCodePathEnd", nodeType: node.type });
      },
      onCodePathSegmentStart(_segment: unknown, node: ESTreeNode) {
        visits.push({ event: "onCodePathSegmentStart", nodeType: node.type });
      },
      onCodePathSegmentEnd(_segment: unknown, node: ESTreeNode) {
        visits.push({ event: "onCodePathSegmentEnd", nodeType: node.type });
      },
      onUnreachableCodePathSegmentStart(_segment: unknown, node: ESTreeNode) {
        visits.push({ event: "onUnreachableCodePathSegmentStart", nodeType: node.type });
      },
      onUnreachableCodePathSegmentEnd(_segment: unknown, node: ESTreeNode) {
        visits.push({ event: "onUnreachableCodePathSegmentEnd", nodeType: node.type });
      },
      onCodePathSegmentLoop(_fromSegment: unknown, _toSegment: unknown, node: ESTreeNode) {
        visits.push({ event: "onCodePathSegmentLoop", nodeType: node.type });
      },
      after() {
        context.report({
          message: "after hook:\n" + `filename: ${context.filename}`,
          node: SPAN,
        });

        visits.push({ event: "after" });

        const expectedVisits: typeof visits = [
          { event: "onCodePathStart", nodeType: "Program" },
          { event: "onCodePathSegmentStart", nodeType: "Program" },
          { event: "onCodePathSegmentEnd", nodeType: "BinaryExpression" },
          { event: "onCodePathSegmentStart", nodeType: "BinaryExpression" },
          { event: "onCodePathSegmentEnd", nodeType: "UpdateExpression" },
          { event: "onCodePathSegmentStart", nodeType: "UpdateExpression" },
          { event: "onCodePathSegmentLoop", nodeType: "BlockStatement" },
          { event: "onCodePathSegmentEnd", nodeType: "BlockStatement" },
          { event: "onCodePathSegmentStart", nodeType: "BlockStatement" },
          { event: "onCodePathSegmentEnd", nodeType: "BlockStatement" },
          { event: "onUnreachableCodePathSegmentStart", nodeType: "BlockStatement" },
          { event: "onUnreachableCodePathSegmentEnd", nodeType: "ForStatement" },
          { event: "onCodePathSegmentStart", nodeType: "ForStatement" },
          { event: "onCodePathSegmentEnd", nodeType: "Program" },
          { event: "onCodePathEnd", nodeType: "Program" },
          { event: "after" },
        ];

        if (
          visits.length !== expectedVisits.length ||
          visits.some(
            (v, i) =>
              v.event !== expectedVisits[i].event || v.nodeType !== expectedVisits[i].nodeType,
          )
        ) {
          context.report({
            message: `Unexpected visits:\n${JSON.stringify(visits, null, 2)}`,
            node: SPAN,
          });
        }
      },
    } as unknown as Visitor; // TODO: Our types don't include CFG event handlers at present
  },
};

// Tests that `before` hook returning `false` disables visiting AST for the file.
const createOnceBeforeFalseRule: Rule = {
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
};

// These 4 rules test that `createOnce` without `before` and `after` hooks works correctly.

const createOnceBeforeOnlyRule: Rule = {
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
};

const createOnceAfterOnlyRule: Rule = {
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
};

const createOnceHooksOnlyRule: Rule = {
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
};

const createOnceNoHooksRule: Rule = {
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
};

export default eslintCompatPlugin({
  meta: {
    name: "eslint-compat-plugin",
  },
  rules: {
    create: createRule,
    "create-once": createOnceRule,
    "create-once-selector": createOnceSelectorRule,
    "create-once-cfg": createOnceCfgRule,
    "create-once-before-false": createOnceBeforeFalseRule,
    "create-once-before-only": createOnceBeforeOnlyRule,
    "create-once-after-only": createOnceAfterOnlyRule,
    "create-once-hooks-only": createOnceHooksOnlyRule,
    "create-once-no-hooks": createOnceNoHooksRule,
  },
});
