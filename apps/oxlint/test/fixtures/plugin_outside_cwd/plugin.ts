import type { Plugin } from "#oxlint/plugins";

// This test checks that a plugin which is outside the current working directory can be loaded.
// `cwd` is set to `files/subdirectory` in `options.json`, and the plugin is outside that directory.

const plugin: Plugin = {
  meta: {
    name: "basic-custom-plugin",
  },
  rules: {
    "no-debugger": {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: "Unexpected Debugger Statement",
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};

export default plugin;
