import type { Plugin } from "../../../dist/index.js";

function trapReport(_context: any) {
  return function (_obj: any) {};
}

const baseRule = {
  create(context: any) {
    // Base rule tries to access sourceCode
    const _sourceCode = context.sourceCode;
    return {
      Identifier(node: any) {
        context.report({
          node,
          message: "Base rule found identifier",
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "wrapped-context",
  },
  rules: {
    "wrapped-rule": {
      create(context) {
        const contextForBaseRule = Object.create(context, {
          report: {
            value: trapReport(context),
            writable: false,
          },
        });

        return baseRule.create(contextForBaseRule);
      },
    },
  },
};

export default plugin;
