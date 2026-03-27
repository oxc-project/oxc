import type { Plugin, Rule } from "#oxlint/plugins";

const rule: Rule = {
  create(context) {
    return {
      Program(node) {
        context.report({
          message: `This is a ${Array.from({ length: 240 }, () => "very").join(" ")} long message`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "plugin",
  },
  rules: {
    "long-message": rule,
  },
};

export default plugin;
