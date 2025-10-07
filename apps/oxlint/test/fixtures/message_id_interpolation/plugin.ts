import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'interpolation-test',
  },
  rules: {
    'no-var': {
      meta: {
        messages: {
          noData: 'Variables should not use var',
          withName: 'Variable {{name}} should not use var',
          withMultiple: 'Variable {{name}} of type {{type}} should not use var',
          // edge cases
          missingData: 'Value is {{value}} and name is {{name}}',
          withSpaces: 'Value with spaces: {{ value }} and name: {{  name  }}',
        },
      },
      create(context) {
        return {
          VariableDeclaration(node) {
            if (node.kind === 'var') {
              const declarations = node.declarations;
              if (declarations.length > 0) {
                const firstDeclaration = declarations[0];
                if (firstDeclaration.id.type === 'Identifier') {
                  const name = firstDeclaration.id.name;

                  // Test with single placeholder
                  if (name === 'testWithNoData') {
                    context.report({
                      messageId: 'withName',
                      node,
                    });
                  } // Test with multiple placeholders
                  else if (name === 'testWithName') {
                    context.report({
                      messageId: 'withMultiple',
                      node,
                      data: {
                        name,
                        type: 'string',
                      },
                    });
                  } // Test without data
                  else if (name === 'testWithMultiple') {
                    context.report({
                      messageId: 'withMultiple',
                      node,
                      data: {
                        name,
                        type: 'number',
                      },
                    });
                  } else if (name === 'testWithMissingData') {
                    // Test missing data - placeholder should remain
                    context.report({
                      messageId: 'missingData',
                      node,
                      data: {
                        value: 'example',
                        // name is missing
                      },
                    });
                  } else if (name === 'testWithSpaces') {
                    // Test whitespace in placeholders
                    context.report({
                      messageId: 'withSpaces',
                      node,
                      data: {
                        value: 'hello',
                        name: 'world',
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
