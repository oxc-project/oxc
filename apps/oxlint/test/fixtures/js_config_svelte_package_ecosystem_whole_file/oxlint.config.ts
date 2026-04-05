import { defineConfig } from "#oxlint";
import sveltePlugin from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";

const tsParser = {
  parseForESLint(code: string) {
    return {
      ast: {
        type: "Program",
        sourceType: "module",
        body: [],
        range: [0, code.length],
        loc: {
          start: { line: 1, column: 0 },
          end: { line: 1, column: code.length },
        },
        comments: [],
        tokens: [],
      },
    };
  },
};

const svelteConfig = {
  preprocess() {
    return null;
  },
};

export default defineConfig({
  categories: {
    correctness: "off",
  },
  extends: [sveltePlugin.configs.recommended],
  overrides: [
    {
      files: ["**/*.svelte"],
      settings: {
        svelte: {
          compileOptions: {
            dev: true,
          },
          kit: {
            files: {
              routes: "src/routes",
            },
          },
        },
      },
      languageOptions: {
        parser: svelteParser,
        parserOptions: {
          parser: tsParser,
          projectService: true,
          extraFileExtensions: [".svelte"],
          svelteConfig,
        },
      },
    },
  ],
});
