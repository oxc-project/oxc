import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "help-plugin",
  },
  rules: {
    "no-debugger": {
      meta: {
        messages: {
          unexpected: "Unexpected debugger statement.",
        },
      },
      create(context) {
        return {
          DebuggerStatement(node) {
            if (context.filename.endsWith("invalid.js")) {
              context.report({
                node,
                messageId: "unexpected",
                help: 123 as never,
              });
              return;
            }

            context.report({
              node,
              messageId: "unexpected",
              help: "Remove the debugger statement before committing your code.",
            });
            context.report({ node, message: "Omitted help." });
            context.report({ node, message: "Null help.", help: null });
            context.report({ node, message: "Undefined help.", help: undefined });
          },
        };
      },
    },
  },
};

export default plugin;
