import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "bom-plugin",
  },
  rules: {
    bom: {
      create(context) {
        return {
          Program(node) {
            context.report({
              message:
                "\n" +
                `hasBOM: ${context.sourceCode.hasBOM}\n` +
                `sourceText: ${JSON.stringify(context.sourceCode.text)}\n` +
                `Program span: ${node.start}-${node.end}`,
              node,
            });
          },
          DebuggerStatement(node) {
            context.report({
              message: `Debugger statement at ${node.start}-${node.end}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
