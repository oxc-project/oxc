export default {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: {
      create(_context) {
        throw new Error("Whoops!");
      },
    },
  },
};
