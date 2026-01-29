import type { Plugin } from "#oxlint/plugin";

const MESSAGE_ID_ERROR = "no-var/error";
const messages = {
  [MESSAGE_ID_ERROR]: "Unexpected var, use let or const instead.",
};

const plugin: Plugin = {
  meta: {
    name: "message-id-plugin",
  },
  rules: {
    "no-var": {
      meta: {
        messages,
      },
      create(context) {
        return {
          VariableDeclaration(node) {
            if (node.kind === "var") {
              const decl = node.declarations[0];
              const varName = decl.id.type === "Identifier" ? decl.id.name : "<destructured>";

              if (varName === "reportUsingNode") {
                context.report({
                  messageId: MESSAGE_ID_ERROR,
                  node,
                });
              } else if (varName === "reportUsingRange") {
                context.report({
                  messageId: MESSAGE_ID_ERROR,
                  loc: node.loc,
                });
              }
            }
          },
        };
      },
    },
  },
};

export default plugin;
