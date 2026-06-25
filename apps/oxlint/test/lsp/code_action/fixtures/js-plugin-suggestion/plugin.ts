import type { Plugin, Rule, Suggestion } from "#oxlint/plugins";

const rule: Rule = {
  meta: {
    hasSuggestions: true,
  },
  create(context) {
    return {
      DebuggerStatement(node) {
        const suggestion: Suggestion = {
          desc: "Do it",
          fix(fixer) {
            return fixer.removeRange([node.start, node.end]);
          },
        };

        context.report({
          message: "Remove debugger statement",
          node,
          suggest: [suggestion],
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "suggestions-plugin",
  },
  rules: {
    suggestions: rule,
  },
};

export default plugin;
