import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "legacy-report-plugin",
  },
  rules: {
    "legacy-report": {
      create(context) {
        return {
          DebuggerStatement(node) {
            // @ts-expect-error ESLint legacy report signature still used by external plugins.
            context.report(node as never, "Unexpected Debugger Statement" as never);
          },
        };
      },
    },
  },
};

export default plugin;
