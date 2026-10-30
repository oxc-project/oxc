export default {
  meta: { name: "eslint-plugin-custom" },
  rules: {
    rule: {
      create(context) {
        return {
          Program(node) {
            context.report({
              node,
              message: "custom rule diagnostic",
            });
          },
        };
      },
    },
  },
};
