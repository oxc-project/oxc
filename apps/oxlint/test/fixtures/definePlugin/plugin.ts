import { definePlugin } from "#oxlint/plugins";

import type { Rule } from "#oxlint/plugins";

const createRule: Rule = {
  create(context) {
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

const createOnceRule: Rule = {
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

export default definePlugin({
  meta: {
    name: "define-plugin-plugin",
  },
  rules: {
    create: createRule,
    "create-once": createOnceRule,
  },
});
