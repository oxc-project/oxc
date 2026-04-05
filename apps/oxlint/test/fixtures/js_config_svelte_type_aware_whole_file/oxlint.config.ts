import { defineConfig } from "#oxlint";
import svelteParser from "svelte-eslint-parser";
import baseConfig from "./base-config.ts";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  extends: [baseConfig],
  jsPlugins: ["./plugin.ts"],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteParser,
        parserOptions: {
          projectService: true,
          extraFileExtensions: [".svelte"],
          tsconfigRootDir: import.meta.dirname,
        },
      },
      rules: {
        "whole-file-svelte-type-aware/options-visible": "error",
      },
    },
  ],
});
