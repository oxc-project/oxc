import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "message-id-error-plugin",
  },
  rules: {
    "test-rule": {
      meta: {
        messages: {
          validMessage: "This is a valid message",
        },
      },
      create(context) {
        return {
          DebuggerStatement(node) {
            // Try to use an unknown messageId
            context.report({
              messageId: "unknownMessage",
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
