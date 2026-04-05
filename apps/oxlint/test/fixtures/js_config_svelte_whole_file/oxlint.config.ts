import { defineConfig } from "#oxlint";

const svelteStubParser = {
  parseForESLint(code: string, options?: Record<string, unknown>) {
    return {
      ast: {
        type: "Program",
        sourceType: "module",
        body: [],
        range: [0, code.length],
        loc: {
          start: { line: 1, column: 0 },
          end: { line: 5, column: 0 },
        },
        comments: [],
        tokens: [],
      },
      services: {
        isSvelte: true,
        filePath: options?.filePath ?? null,
        flavor: options?.flavor ?? null,
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
        parserOptions: {
          flavor: "svelte-stub",
        },
      },
      rules: {
        "whole-file-svelte/markup-visible": "error",
      },
    },
  ],
});
