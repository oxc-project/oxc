import { sep } from 'node:path';

import type { Node, Plugin, Rule } from '../../../dist/index.js';

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const DIR_PATH_LEN = import.meta.dirname.length + 1;

const relativePath = sep === '/'
  ? (path: string) => path.slice(DIR_PATH_LEN)
  : (path: string) => path.slice(DIR_PATH_LEN).replace(/\\/g, '/');

let createOnceCallCount = 0;

const alwaysRunRule: Rule = {
  createOnce(context) {
    createOnceCallCount++;

    // `this` should be the rule object
    // oxlint-disable-next-line typescript-eslint/no-this-alias
    const topLevelThis = this;

    // Check that these APIs throw here
    const idError = tryCatch(() => context.id);
    const filenameError = tryCatch(() => context.filename);
    const physicalFilenameError = tryCatch(() => context.physicalFilename);
    const optionsError = tryCatch(() => context.options);
    const sourceCodeError = tryCatch(() => context.sourceCode);
    const reportError = tryCatch(() => context.report({ message: 'oh no', node: SPAN }));

    return {
      before() {
        context.report({ message: `createOnce: call count: ${createOnceCallCount}`, node: SPAN });
        context.report({ message: `createOnce: this === rule: ${topLevelThis === alwaysRunRule}`, node: SPAN });
        context.report({ message: `createOnce: id: ${idError?.message}`, node: SPAN });
        context.report({ message: `createOnce: filename: ${filenameError?.message}`, node: SPAN });
        context.report({ message: `createOnce: physicalFilename: ${physicalFilenameError?.message}`, node: SPAN });
        context.report({ message: `createOnce: options: ${optionsError?.message}`, node: SPAN });
        context.report({ message: `createOnce: sourceCode: ${sourceCodeError?.message}`, node: SPAN });
        context.report({ message: `createOnce: report: ${reportError?.message}`, node: SPAN });

        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${relativePath(context.filename)}`, node: SPAN });
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${relativePath(context.filename)}`,
          node,
        });
      },
      after() {
        context.report({ message: `after hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `after hook: filename: ${relativePath(context.filename)}`, node: SPAN });
      },
    };
  },
};

const skipRunRule: Rule = {
  createOnce(context) {
    return {
      before() {
        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${relativePath(context.filename)}`, node: SPAN });
        // Skip running this rule
        return false;
      },
      Identifier(node) {
        context.report({ message: `ident visit fn "${node.name}": should not be output`, node });
      },
      after() {
        context.report({ message: 'after hook: should not be output', node: SPAN });
      },
    };
  },
};

const beforeOnlyRule: Rule = {
  createOnce(context) {
    return {
      before() {
        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${relativePath(context.filename)}`, node: SPAN });
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${relativePath(context.filename)}`,
          node,
        });
      },
    };
  },
};

const afterOnlyRule: Rule = {
  createOnce(context) {
    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${relativePath(context.filename)}`,
          node,
        });
      },
      after() {
        context.report({ message: `after hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `after hook: filename: ${relativePath(context.filename)}`, node: SPAN });
      },
    };
  },
};

const hooksOnlyRule: Rule = {
  createOnce(context) {
    return {
      // Neither hook should be called, because no AST node visitor functions
      before() {
        context.report({ message: 'before hook: should not be output', node: SPAN });
      },
      after() {
        context.report({ message: 'after hook: should not be output', node: SPAN });
      },
    };
  },
};

const noHooksRule: Rule = {
  createOnce(context) {
    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${relativePath(context.filename)}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'create-once-plugin',
  },
  rules: {
    'always-run': alwaysRunRule,
    'skip-run': skipRunRule,
    'before-only': beforeOnlyRule,
    'after-only': afterOnlyRule,
    'only-hooks': hooksOnlyRule,
    'no-hooks': noHooksRule,
  },
};

export default plugin;

function tryCatch(fn: () => unknown) {
  try {
    fn();
  } catch (err) {
    return err;
  }
  throw new Error('Expected function to throw');
}
