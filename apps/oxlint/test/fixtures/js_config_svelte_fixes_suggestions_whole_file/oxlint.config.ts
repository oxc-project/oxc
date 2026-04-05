import { defineConfig } from "#oxlint";
import sveltePlugin from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  extends: [sveltePlugin.configs.recommended],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteParser,
      },
    },
  ],
});
