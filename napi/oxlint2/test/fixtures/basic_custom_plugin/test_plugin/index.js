export default {
  meta: {
    name: "basic-custom-plugin",
  },
  rules: {
    "no-debugger": (_context) => {
      return {
        DebuggerStatement(_debuggerStatement) {
          throw new Error("unimplemented");
        },
      };
    },
  },
};
