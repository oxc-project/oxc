import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "basic-custom-plugin",
  },
  rules: {
    "rule-point": {
      create(_context) {
        return {
          Program(_program) {
            _context.report({
              message: "oops",
              loc: {
                start: { line: 0, column: 0 },
              },
            });
          },
        };
      },
    },
    "rule-wide": {
      create(_context) {
        return {
          Program(_program) {
            _context.report({
              message: "oops (wide)",
              loc: {
                start: { line: 1, column: 0 },
                end: { line: 1, column: 1 },
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
