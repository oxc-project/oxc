export default {
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
