export default {
  meta: {
    name: "basic-custom-plugin",
  },
  rules: {
    "no-debugger": (context) => {
      // TODO: move this call into `DebuggerStatement`, once we are walking the ast.
      context.report({
        message: "Unexpected Debugger Statement",
        node: { start: 0, end: 0 },
      });
      return {
        DebuggerStatement(_debuggerStatement) {},
      };
    },
  },
};
