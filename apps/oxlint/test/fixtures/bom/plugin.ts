import assert from "node:assert";

import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "bom-plugin",
  },
  rules: {
    bom: {
      create(context) {
        // Check file has not been formatted by accident.
        // We want the fixture files not to have trailing whitespace to check diagnostics at very end of file.
        const sourceText = context.sourceCode.text;
        assert(sourceText.endsWith(";"), "Fixture file has been formatted");

        return {
          Program(node) {
            context.report({
              message:
                "\n" +
                `hasBOM: ${context.sourceCode.hasBOM}\n` +
                `sourceText: ${JSON.stringify(sourceText)}\n` +
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
