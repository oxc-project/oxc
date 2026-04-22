import { defineRule } from "#oxlint/plugins";

const createRule = defineRule({
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
});

const createOnceRule = defineRule({
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
  },
};
