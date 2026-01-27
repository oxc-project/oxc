import type { Node, Plugin, Rule } from "#oxlint/plugin";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

let createOnceCallCount = 0;

const alwaysRunRule: Rule = {
  createOnce(context) {
    createOnceCallCount++;

    // `this` should be the rule object
    // oxlint-disable-next-line typescript-eslint/no-this-alias
    const topLevelThis = this;

    // Check that these APIs throw here
    const idError = tryCatch(() => context.id);
    const cwdError = tryCatch(() => context.cwd);
    const getCwdError = tryCatch(() => context.getCwd());
    const filenameError = tryCatch(() => context.filename);
    const getFilenameError = tryCatch(() => context.getFilename());
    const physicalFilenameError = tryCatch(() => context.physicalFilename);
    const getPhysicalFilenameError = tryCatch(() => context.getPhysicalFilename());
    const optionsError = tryCatch(() => context.options);
    const sourceCodeError = tryCatch(() => context.sourceCode);
    const getSourceCodeError = tryCatch(() => context.getSourceCode());
    const settingsError = tryCatch(() => context.settings);
    const parserOptionsError = tryCatch(() => context.parserOptions);
    const reportError = tryCatch(() => context.report({ message: "oh no", node: SPAN }));

    return {
      before() {
        context.report({ message: `createOnce: call count: ${createOnceCallCount}`, node: SPAN });
        context.report({
          message: `createOnce: this === rule: ${topLevelThis === alwaysRunRule}`,
          node: SPAN,
        });
        context.report({ message: `createOnce: cwd error: ${cwdError?.message}`, node: SPAN });
        context.report({
          message: `createOnce: getCwd() error: ${getCwdError?.message}`,
          node: SPAN,
        });
        context.report({ message: `createOnce: id error: ${idError?.message}`, node: SPAN });
        context.report({
          message: `createOnce: filename error: ${filenameError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: getFilename() error: ${getFilenameError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: physicalFilename error: ${physicalFilenameError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: getPhysicalFilename() error: ${getPhysicalFilenameError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: options error: ${optionsError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: sourceCode error: ${sourceCodeError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: getSourceCode() error: ${getSourceCodeError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: settings error: ${settingsError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: parserOptions error: ${parserOptionsError?.message}`,
          node: SPAN,
        });
        context.report({
          message: `createOnce: report error: ${reportError?.message}`,
          node: SPAN,
        });

        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${context.filename}`, node: SPAN });
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({ message: `after hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `after hook: filename: ${context.filename}`, node: SPAN });
      },
    };
  },
};

const skipRunRule: Rule = {
  createOnce(context) {
    return {
      before() {
        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${context.filename}`, node: SPAN });
        // Skip running this rule
        return false;
      },
      Identifier(node) {
        context.report({ message: `ident visit fn "${node.name}": should not be output`, node });
      },
      after() {
        context.report({ message: "after hook: should not be output", node: SPAN });
      },
    };
  },
};

const beforeOnlyRule: Rule = {
  createOnce(context) {
    return {
      before() {
        context.report({ message: `before hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `before hook: filename: ${context.filename}`, node: SPAN });
      },
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${context.filename}`,
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
          message: `ident visit fn "${node.name}": filename: ${context.filename}`,
          node,
        });
      },
      after() {
        context.report({ message: `after hook: id: ${context.id}`, node: SPAN });
        context.report({ message: `after hook: filename: ${context.filename}`, node: SPAN });
      },
    };
  },
};

const hooksOnlyRule: Rule = {
  createOnce(context) {
    return {
      // Neither hook should be called, because no AST node visitor functions
      before() {
        context.report({ message: "before hook: should not be output", node: SPAN });
      },
      after() {
        context.report({ message: "after hook: should not be output", node: SPAN });
      },
    };
  },
};

const noHooksRule: Rule = {
  createOnce(context) {
    return {
      Identifier(node) {
        context.report({
          message: `ident visit fn "${node.name}": filename: ${context.filename}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "create-once-plugin",
  },
  rules: {
    "always-run": alwaysRunRule,
    "skip-run": skipRunRule,
    "before-only": beforeOnlyRule,
    "after-only": afterOnlyRule,
    "only-hooks": hooksOnlyRule,
    "no-hooks": noHooksRule,
  },
};

export default plugin;

function tryCatch(fn: () => unknown) {
  try {
    fn();
  } catch (err) {
    return err;
  }
  throw new Error("Expected function to throw");
}
