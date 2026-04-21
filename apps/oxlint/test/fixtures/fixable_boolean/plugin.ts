import type { Plugin } from "#oxlint/plugins";

// Test backward compatibility for `meta.fixable: true` and `meta.fixable: false`
// which some ESLint plugins use instead of "code" or "whitespace"
const plugin: Plugin = {
  meta: {
    name: "fixable-boolean-plugin",
  },
  rules: {
    // Rule with fixable: true (backward compatibility)
    "no-debugger-true": {
      meta: {
        fixable: true as any,
      },
      create(context) {
        return {
          DebuggerStatement(node) {
            context.report({
              message: "Debugger with fixable: true",
              node,
              fix(fixer) {
                return fixer.remove(node);
              },
            });
          },
        };
      },
    },
    // Rule with fixable: false (backward compatibility)
    "no-console-false": {
      meta: {
        fixable: false as any,
      },
      create(context) {
        return {
          CallExpression(node) {
            if (
              node.callee.type === "MemberExpression" &&
              node.callee.object.type === "Identifier" &&
              node.callee.object.name === "console"
            ) {
              context.report({
                message: "Console with fixable: false",
                node,
              });
            }
          },
        };
      },
    },
  },
};

export default plugin;
