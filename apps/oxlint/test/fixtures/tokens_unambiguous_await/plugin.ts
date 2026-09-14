import assert from "node:assert/strict";

import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: { name: "tokens-unambiguous-await" },
  rules: {
    check: {
      create(context) {
        return {
          Program(node) {
            // The .js file reparses two script statements as nested awaits.
            // The .mjs control parses the same source directly as a module.
            const { sourceCode } = context;
            const expected = [
              ["Identifier", "await", [0, 5]],
              ["Identifier", "await", [6, 11]],
              ["RegularExpression", "/x/u", [12, 16]],
              ["Punctuator", ";", [16, 17]],
              ["Keyword", "export", [18, 24]],
              ["Punctuator", "{", [25, 26]],
              ["Punctuator", "}", [26, 27]],
              ["Punctuator", ";", [27, 28]],
            ];

            for (const tokens of [node.tokens, sourceCode.getTokens(node)]) {
              assert.deepEqual(
                tokens.map(({ type, value, range }) => [type, value, range]),
                expected,
              );
            }
          },
        };
      },
    },
  },
};

export default plugin;
