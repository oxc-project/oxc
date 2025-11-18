export default {
  meta: {
    name: 'test-plugin-options',
  },
  rules: {
    'check-options': {
      meta: {
        messages: {
          wrongValue: 'Expected value to be {{expected}}, got {{actual}}',
          noOptions: 'No options provided',
        },
      },
      createOnce(context) {
        // Don't access context.options here - it's not available yet!
        // Options are only available in visitor methods after setupContextForFile is called.

        return {
          before() {
            // Options are now available since setupContextForFile was called
            const options = context.options;

            // Check if options were passed correctly
            if (!options || options.length === 0) {
              context.report({
                messageId: 'noOptions',
                loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 1 } },
              });
              return false;
            }
            return true;
          },

          DebuggerStatement(node) {
            // Options are available in visitor methods
            const options = context.options;

            // First option is a boolean
            const shouldReport = options[0];
            // Second option is an object
            const config = options[1] || {};

            if (shouldReport) {
              context.report({
                messageId: 'wrongValue',
                data: {
                  expected: String(config.expected || 'enabled'),
                  actual: 'disabled',
                },
                node,
              });
            }
          },
        };
      },
    },
  },
};
