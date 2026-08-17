// Cheap corpus rule: the same `no-debugger` JS plugin the e2e fixtures use.
// One visitor key, no per-node work, so a run over the corpus is parse-bound. This is the
// control for "does routing through workers tax runs where JS does almost nothing?".

const plugin = {
  meta: { name: "bench" },
  rules: {
    "no-debugger": {
      create(context) {
        return {
          DebuggerStatement(node) {
            context.report({ message: "Unexpected debugger statement", node });
          },
        };
      },
    },
  },
};

export default plugin;
