import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "interpolation-test",
  },
  rules: {
    "no-var": {
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
                      message: "Variables should not use var",
                      node,
                    });
                  } else if (name === "testWithName") {
                    // Test with single placeholder
                    context.report({
                      message: "Variable `{{name}}` should not use var",
                      node,
                      data: { name },
                    });
                  } else if (name === "testWithNameNoData") {
                    // Test with single placeholder, but no data
                    context.report({
                      message: "Variable `{{name}}` should not use var",
                      node,
                    });
                  } else if (name === "testWithMultiple") {
                    // Test with multiple placeholders
                    context.report({
                      message: "Variable `{{name}}` of type `{{type}}` should not use var",
                      node,
                      data: {
                        name,
                        type: "string",
                      },
                    });
                  } else if (name === "testWithMultipleNoData") {
                    // Test with multiple placeholders, but no data
                    context.report({
                      message: "Variable `{{name}}` of type `{{type}}` should not use var",
                      node,
                    });
                  } else if (name === "testWithMissingData") {
                    // Test missing data - placeholder should remain
                    context.report({
                      message: "Value is `{{value}}` and name is `{{name}}`",
                      node,
                      data: {
                        value: "example",
                        // name is missing
                      },
                    });
                  } else if (name === "testWithSpaces") {
                    // Test whitespace in placeholders
                    context.report({
                      message: "Value with spaces is `{{ value }}` and name is `{{  name  }}`",
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
