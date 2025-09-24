import { dirname, sep } from 'node:path';

const SPAN = { start: 0, end: 0 };

const PARENT_DIR_PATH_LEN = dirname(import.meta.dirname).length + 1;

const relativePath = sep === '/'
  ? path => path.slice(PARENT_DIR_PATH_LEN)
  : path => path.slice(PARENT_DIR_PATH_LEN).replace(/\\/g, '/');

let createOnceCallCount = 0;
const alwaysRunRule = {
  createOnce(context) {
    createOnceCallCount++;

    const topLevelThis = this;

    // Check that these APIs throw here
    const idError = tryCatch(() => context.id);
    const filenameError = tryCatch(() => context.filename);
    const physicalFilenameError = tryCatch(() => context.physicalFilename);
    const optionsError = tryCatch(() => context.options);
    const reportError = tryCatch(() => context.report({ message: 'oh no', node: SPAN }));

    return {
      before() {
        context.report({ message: `createOnce: call count: ${createOnceCallCount}`, node: SPAN });
        context.report({ message: `createOnce: this === rule: ${topLevelThis === alwaysRunRule}`, node: SPAN });
        context.report({ message: `createOnce: id: ${idError?.message}`, node: SPAN });
        context.report({ message: `createOnce: filename: ${filenameError?.message}`, node: SPAN });
        context.report({ message: `createOnce: physicalFilename: ${physicalFilenameError?.message}`, node: SPAN });
        context.report({ message: `createOnce: options: ${optionsError?.message}`, node: SPAN });
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

const skipRunRule = {
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

const beforeOnlyRule = {
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

const afterOnlyRule = {
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

const noHooksRule = {
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

export default {
  meta: {
    name: "create-once-plugin",
  },
  rules: {
    "always-run": alwaysRunRule,
    "skip-run": skipRunRule,
    "before-only": beforeOnlyRule,
    "after-only": afterOnlyRule,
    "no-hooks": noHooksRule,
  },
};

function tryCatch(fn) {
  try {
    fn();
  } catch (err) {
    return err;
  }
  throw new Error('Expected function to throw');
}
