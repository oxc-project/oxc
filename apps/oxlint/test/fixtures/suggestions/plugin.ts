import { sep as pathSep } from "node:path";
import assert from "node:assert";

import type { Node, Plugin, Rule, Suggestion } from "#oxlint/plugins";

const U32_MAX_PLUS_ONE = 2 ** 32;

const rule: Rule = {
  meta: {
    hasSuggestions: true,
  },
  create(context) {
    // Check file has not been formatted by accident.
    // We want the fixture files not to have trailing whitespace to check suggestions at very end of file.
    const sourceText = context.sourceCode.text;
    assert(!sourceText.endsWith("\n"), "Fixture file has been formatted");

    const path = context.filename;
    const filename = path.slice(path.lastIndexOf(pathSep) + 1);

    let debuggerCount = 0;

    return {
      Program(node) {
        // Remove BOM in `bom_remove.js` and `bom_remove2.js`.
        // ESLint's `unicode-bom` rule returns fixes with `range: [-1, 0]` to remove BOM.
        // Check behavior is correct for both `fix` returning a single fix and returning multiple fixes.
        if (filename === "bom_remove.js") {
          context.report({
            message: "Remove BOM",
            node,
            suggest: [
              {
                desc: "Do it",
                fix(fixer) {
                  return fixer.removeRange([-1, 0]);
                },
              },
            ],
          });
        } else if (filename === "bom_remove2.js") {
          context.report({
            message: "Remove BOM multiple",
            node,
            suggest: [
              {
                desc: "Do it",
                fix(fixer) {
                  return [fixer.removeRange([0, 0]), fixer.removeRange([-1, 0])];
                },
              },
            ],
          });
        }
      },

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

          // Test behavior when `fix` returns a fix with illegal range.
          // Check behavior is correct for both `fix` returning a single fix and returning multiple fixes.
          case "x":
            switch (filename) {
              case "range_start_after_end.js":
                context.report({
                  message: "start after end",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([3, 2]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "start after end multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [fixer.removeRange([0, 0]), fixer.removeRange([3, 2])];
                      },
                    },
                  ],
                });
                break;

              case "range_end_out_of_bounds.js":
                context.report({
                  message: "end out of bounds",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([0, sourceText.length + 1]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "end out of bounds multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [
                          fixer.removeRange([0, 0]),
                          fixer.removeRange([0, sourceText.length + 1]),
                        ];
                      },
                    },
                  ],
                });
                break;

              case "range_start_too_large.js":
                context.report({
                  message: "start too large",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([U32_MAX_PLUS_ONE, 0]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "start too large multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [
                          fixer.removeRange([0, 0]),
                          fixer.removeRange([U32_MAX_PLUS_ONE, 0]),
                        ];
                      },
                    },
                  ],
                });
                break;

              case "range_end_too_large.js":
                context.report({
                  message: "end too large",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([0, U32_MAX_PLUS_ONE]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "end too large multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [
                          fixer.removeRange([0, 0]),
                          fixer.removeRange([0, U32_MAX_PLUS_ONE]),
                        ];
                      },
                    },
                  ],
                });
                break;

              case "range_start_negative.js":
                context.report({
                  message: "start negative",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([-10, 0]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "start negative multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [fixer.removeRange([0, 0]), fixer.removeRange([-10, 0])];
                      },
                    },
                  ],
                });
                break;

              case "range_end_negative.js":
                context.report({
                  message: "end negative",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return fixer.removeRange([0, -10]);
                      },
                    },
                  ],
                });
                context.report({
                  message: "end negative multiple",
                  node,
                  suggest: [
                    {
                      desc: "Do it",
                      fix(fixer) {
                        return [fixer.removeRange([0, 0]), fixer.removeRange([0, -10])];
                      },
                    },
                  ],
                });
                break;
            }
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
