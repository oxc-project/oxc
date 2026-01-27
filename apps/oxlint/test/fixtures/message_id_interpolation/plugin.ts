import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "interpolation-test",
  },
  rules: {
    "no-var": {
      meta: {
        messages: {
          noData: "Variables should not use var",
          withName: "Variable `{{name}}` should not use var",
          withMultiple: "Variable `{{name}}` of type `{{type}}` should not use var",
          // edge cases
          missingData: "Value is `{{value}}` and name is `{{name}}`",
          withSpaces: "Value with spaces is `{{ value }}` and name is `{{  name  }}`",
        },
      },
      create(context) {
        return {
          VariableDeclaration(node) {
            if (node.kind === "var") {
              const { declarations } = node;
              if (declarations.length > 0) {
                const firstDeclaration = declarations[0];
                if (firstDeclaration.id.type === "Identifier") {
                  const { name } = firstDeclaration.id;

                  if (name === "testWithNoData") {
                    // Test with no placeholders, no data
                    context.report({
                      messageId: "noData",
                      node,
                    });
                  } else if (name === "testWithName") {
                    // Test with single placeholder
                    context.report({
                      messageId: "withName",
                      node,
                      data: { name },
                    });
                  } else if (name === "testWithNameNoData") {
                    // Test with single placeholder, but no data
                    context.report({
                      messageId: "withName",
                      node,
                    });
                  } else if (name === "testWithMultiple") {
                    // Test with multiple placeholders
                    context.report({
                      messageId: "withMultiple",
                      node,
                      data: {
                        name,
                        type: "string",
                      },
                    });
                  } else if (name === "testWithMultipleNoData") {
                    // Test with multiple placeholders, but no data
                    context.report({
                      messageId: "withMultiple",
                      node,
                    });
                  } else if (name === "testWithMissingData") {
                    // Test missing data - placeholder should remain
                    context.report({
                      messageId: "missingData",
                      node,
                      data: {
                        value: "example",
                        // name is missing
                      },
                    });
                  } else if (name === "testWithSpaces") {
                    // Test whitespace in placeholders
                    context.report({
                      messageId: "withSpaces",
                      node,
                      data: {
                        value: "hello",
                        name: "world",
                      },
                    });
                  }
                }
              }
            }
          },
        };
      },
    },
  },
};

export default plugin;
