import assert from "node:assert";

import type { Node, Plugin, Rule, Suggestion } from "#oxlint/plugins";

const rule: Rule = {
  meta: {
    hasSuggestions: true,
  },
  create(context) {
    // Check file has not been formatted by accident.
    // We want the fixture files not to have trailing whitespace to check suggestions at very end of file.
    const sourceText = context.sourceCode.text;
    assert(!sourceText.endsWith("\n"), "Fixture has been formatted");

    let debuggerCount = 0;
    return {
      DebuggerStatement(node) {
        debuggerCount++;

        let thisIsSuggestion;

        const suggestion: Suggestion = {
          desc: "Do it",
          fix(fixer) {
            thisIsSuggestion = this === suggestion;
            if (debuggerCount === 1) return fixer.remove(node);
            return fixer.removeRange([node.start, node.end]);
          },
        };

        context.report({
          message: "Remove debugger statement",
          node,
          suggest: [suggestion],
        });

        if (!thisIsSuggestion) {
          context.report({ message: `this in fix function is not suggestion object`, node });
        }
      },
      Identifier(node) {
        switch (node.name) {
          case "a":
            return context.report({
              message: 'Replace "a" with "daddy"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.replaceText(node, "daddy");
                  },
                },
              ],
            });

          case "b":
            return context.report({
              message: 'Replace "b" with "abacus"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.replaceTextRange([node.start, node.end], "abacus");
                  },
                },
              ],
            });

          case "c":
            return context.report({
              message: 'Prefix "c" with "magi"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.insertTextBefore(node, "magi");
                  },
                },
              ],
            });

          case "d":
            return context.report({
              message: 'Prefix "d" with "damne"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.insertTextBeforeRange([node.start, node.end], "damne");
                  },
                },
              ],
            });

          case "e":
            return context.report({
              message: 'Postfix "e" with "lephant"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.insertTextAfter(node, "lephant");
                  },
                },
              ],
            });

          case "f":
            return context.report({
              message: 'Postfix "f" with "eck"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.insertTextAfterRange([node.start, node.end], "eck");
                  },
                },
              ],
            });

          case "g":
            return context.report({
              message: 'Replace "g" with "rage"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    // Add text before and after the node only, to test combining fixes that include original source.
                    // Fixes can be in any order.
                    return [
                      // Test that any object with `range` property works
                      fixer.insertTextAfter({ range: [node.start, node.end] }, "e"),
                      fixer.insertTextBefore(node, "ra"),
                    ];
                  },
                },
              ],
            });

          case "h":
            return context.report({
              message: 'Replace "h" with "dangermouse"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    // Fixes can be in any order
                    const { range } = node;
                    return [
                      fixer.replaceTextRange(range, "er"),
                      fixer.insertTextAfterRange(range, "mouse"),
                      fixer.insertTextBeforeRange(range, "dang"),
                    ];
                  },
                },
              ],
            });

          case "i":
            return context.report({
              message: 'Replace "i" with "granular"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  // `fix` can be a generator function
                  *fix(fixer) {
                    yield fixer.insertTextBefore(node, "gra");
                    yield fixer.replaceText(node, "nu");
                    // Test that any object with `range` property works
                    yield fixer.insertTextAfter({ range: [node.start, node.end] } as Node, "lar");
                  },
                },
              ],
            });

          case "j":
            return context.report({
              message: 'Replace "j" with "cowabunga"',
              node,
              suggest: [
                {
                  desc: "Do it",
                  // `fix` can be a generator function
                  *fix(fixer) {
                    // Fixes can be in any order
                    const { range } = node;
                    yield fixer.insertTextAfterRange(range, "bunga");
                    yield fixer.replaceTextRange(range, "a");
                    yield fixer.insertTextBeforeRange(range, "cow");
                  },
                },
              ],
            });

          case "k":
            return context.report({
              message: 'Replace "k" with "kaboom"',
              node,
              // `--fix-suggestions` will apply only the first suggestion
              suggest: [
                {
                  desc: "Do it",
                  fix(fixer) {
                    return fixer.insertTextAfter(node, "aboom");
                  },
                },
                {
                  desc: "Do something else",
                  fix(fixer) {
                    return fixer.insertTextBefore(node, "prefix1");
                  },
                },
                {
                  desc: "Do another thing",
                  fix(fixer) {
                    return fixer.insertTextBefore(node, "prefix2");
                  },
                },
              ],
            });
        }
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
