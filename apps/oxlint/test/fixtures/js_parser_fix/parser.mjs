// Toy custom parser, wrapping `@typescript-eslint/parser`'s `parseForESLint`.
// Proves fixes work for files parsed by a custom parser.

import { parseForESLint } from "@typescript-eslint/parser";

export default {
  parseForESLint(code, options) {
    return parseForESLint(code, options);
  },
};
