export default {
  rules: {
    rule: {
      meta: {
        schema: [{ type: "object", properties: { label: { type: "string" } } }],
      },
      create(context) {
        const label = context.options[0]?.label ?? "no options";
        return {
          Program(node) {
            context.report({
              node,
              message: `custom rule diagnostic (${label})`,
            });
          },
        };
      },
    },
  },
};
