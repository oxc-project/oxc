export default {
  meta: {
    name: "fixes-plugin",
  },
  rules: {
    "fixes": {
      meta: {
        fixable: "code",
      },
      create(context) {
        let debuggerCount = 0;
        return {
          DebuggerStatement(node) {
            debuggerCount++;

            let thisIsReport;
            const report = {
              message: "Remove debugger statement",
              node,
              fix(fixer) {
                thisIsReport = this === report;
                if (debuggerCount === 1) return fixer.remove(node);
                return fixer.removeRange([node.start, node.end]);
              },
            };
            context.report(report);

            if (!thisIsReport) context.report({ message: `this in fix function is not report object`, node });
          },
          Identifier(node) {
            switch (node.name) {
              case "a":
                return context.report({
                  message: 'Replace "a" with "daddy"',
                  node,
                  fix(fixer) {
                    return fixer.replaceText(node, "daddy");
                  },
                });
              case "b":
                return context.report({
                  message: 'Replace "b" with "abacus"',
                  node,
                  fix(fixer) {
                    return fixer.replaceTextRange([node.start, node.end], "abacus");
                  },
                });
              case "c":
                return context.report({
                  message: 'Prefix "c" with "magi"',
                  node,
                  fix(fixer) {
                    return fixer.insertTextBefore(node, "magi");
                  },
                });
              case "d":
                return context.report({
                  message: 'Prefix "d" with "damne"',
                  node,
                  fix(fixer) {
                    return fixer.insertTextBeforeRange([node.start, node.end], "damne");
                  },
                });
              case "e":
                return context.report({
                  message: 'Postfix "e" with "lephant"',
                  node,
                  fix(fixer) {
                    return fixer.insertTextAfter(node, "lephant");
                  },
                });
              case "f":
                return context.report({
                  message: 'Postfix "f" with "eck"',
                  node,
                  fix(fixer) {
                    return fixer.insertTextAfterRange([node.start, node.end], "eck");
                  },
                });
              case "g":
                return context.report({
                  message: 'Replace "g" with "numpty"',
                  node,
                  fix(fixer) {
                    // Fixes can be in any order
                    return [
                      fixer.insertTextAfter(node, "ty"),
                      fixer.replaceText(node, "mp"),
                      fixer.insertTextBefore(node, "nu"),
                    ];
                  },
                });
              case "h":
                return context.report({
                  message: 'Replace "h" with "dangermouse"',
                  node,
                  fix(fixer) {
                    // Fixes can be in any order
                    const range = [node.start, node.end];
                    return [
                      fixer.replaceTextRange(range, "er"),
                      fixer.insertTextAfterRange(range, "mouse"),
                      fixer.insertTextBeforeRange(range, "dang"),
                    ];
                  },
                });
              case "i":
                return context.report({
                  message: 'Replace "i" with "granular"',
                  node,
                  // `fix` can be a generator function
                  *fix(fixer) {
                    yield fixer.insertTextBefore(node, "gra");
                    yield fixer.replaceText(node, "nu");
                    yield fixer.insertTextAfter(node, "lar");
                  },
                });
              case "j":
                return context.report({
                  message: 'Replace "j" with "cowabunga"',
                  node,
                  // `fix` can be a generator function
                  *fix(fixer) {
                    // Fixes can be in any order
                    const range = [node.start, node.end];
                    yield fixer.insertTextAfterRange(range, "bunga");
                    yield fixer.replaceTextRange(range, "a");
                    yield fixer.insertTextBeforeRange(range, "cow");
                  },
                });
            }
          },
        };
      },
    },
  },
};
