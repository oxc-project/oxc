export default {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: {
      create(_context) {
        return {
          Identifier(_node) {
            throw new Error("Whoops!");
          },
        };
      },
    },
  },
};
