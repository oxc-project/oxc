export default {
  meta: { name: "@scope/custom" },
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
