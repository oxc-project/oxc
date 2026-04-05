import { defineConfig } from "#oxlint";

const svelteStubParser = {
  parseForESLint(code: string) {
    const commentStart = code.indexOf("<!--");
    const commentEnd = code.indexOf("-->") + 3;

    return {
      ast: {
        type: "Program",
        sourceType: "module",
        body: [],
        range: [0, code.length],
        loc: {
          start: { line: 1, column: 0 },
          end: { line: 3, column: 0 },
        },
        comments:
          commentStart >= 0 && commentEnd >= commentStart
            ? [
                {
                  type: "Block",
                  range: [commentStart, commentEnd],
                },
              ]
            : [],
        tokens: [],
      },
      services: {
        isSvelte: true,
      },
      visitorKeys: {
        Program: ["body"],
      },
    };
  },
};

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteStubParser,
      },
      rules: {
        "whole-file-svelte-unused-disable/noop": "error",
      },
    },
  ],
});
