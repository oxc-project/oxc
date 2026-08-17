// Kills the worker isolate mid-`lintFile`, to check that Rust threads waiting on that worker are
// released with an error instead of blocking forever.
//
// `process.exit` inside a worker thread tears down that isolate, so the queued `lintFile` callback
// never runs and the channel it would have completed is left hanging.
export default {
  meta: {
    name: "crash-plugin",
  },
  rules: {
    "crash-on-debugger": {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: "Unexpected Debugger Statement",
              node: debuggerStatement,
            });
            process.exit(7);
          },
        };
      },
    },
  },
};
