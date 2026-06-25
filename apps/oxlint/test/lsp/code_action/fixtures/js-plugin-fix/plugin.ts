import type { Diagnostic, Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "fixes-plugin",
  },
  rules: {
    fixes: {
      meta: {
        fixable: "code",
      },
      create(context) {
        return {
          DebuggerStatement(node) {
            const report: Diagnostic = {
              message: "Remove debugger statement",
              node,
              fix(fixer) {
                return fixer.removeRange([node.start, node.end]);
              },
            };
            context.report(report);
          },
        };
      },
    },
  },
};

export default plugin;
