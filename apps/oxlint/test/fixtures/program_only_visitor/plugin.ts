import type { Context, Plugin } from "#oxlint/plugins";

// All rules in this plugin only visit `Program`, so Oxlint calls their visitors on the root node
// without walking the rest of the AST. These rules check that visitors still behave the same as
// when the AST is walked: called with the root node, enter before exit, empty ancestors,
// and full access to the AST, tokens and comments via `context.sourceCode`.

let eventCount = 0;

function describe(context: Context, node: any, event: string): string {
  const { sourceCode } = context;
  return (
    `${event} #${eventCount++}\n`
    + `node is sourceCode.ast: ${node === sourceCode.ast}\n`
    + `parent: ${node.parent}\n`
    + `ancestors: ${sourceCode.getAncestors(node).length}\n`
    + `body: ${node.body.map((stmt: any) => stmt.type).join(", ")}\n`
    + `comments: ${sourceCode.getAllComments().length}\n`
    + `tokens: ${sourceCode.ast.tokens.length}`
  );
}

const plugin: Plugin = {
  meta: {
    name: "program-only",
  },
  rules: {
    "enter-and-exit": {
      create(context) {
        return {
          Program(node) {
            context.report({ message: describe(context, node, "Program enter"), node });
          },
          "Program:exit"(node) {
            context.report({ message: describe(context, node, "Program exit"), node });
          },
        };
      },
    },

    "exit-only": {
      create(context) {
        return {
          "Program:exit"(node) {
            context.report({ message: describe(context, node, "Program:exit only"), node });
          },
        };
      },
    },

    // Attribute selector on `Program` is still matched against the root node
    selector: {
      create(context) {
        return {
          "Program[sourceType='module']"(node) {
            context.report({
              message: describe(context, node, "Program[sourceType='module']"),
              node,
            });
          },
          "Program[sourceType='script']"(node) {
            context.report({ message: "Should not match", node });
          },
        };
      },
    },

    // `before` and `after` hooks run around the `Program` visit
    "create-once": {
      createOnce(context) {
        let visited = false;
        return {
          before() {
            visited = false;
          },
          Program(node) {
            visited = true;
            context.report({ message: describe(context, node, "createOnce Program"), node });
          },
          after() {
            context.report({
              message: `after hook #${eventCount++}: visited Program: ${visited}`,
              loc: { line: 1, column: 0 },
            });
          },
        };
      },
    },
  },
};

export default plugin;
