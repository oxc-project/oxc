import { defineConfig } from "#oxlint";
import svelteParser from "svelte-eslint-parser";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteParser,
      },
      rules: {
        "real-svelte-disable/markup-visible": "error",
      },
    },
  ],
});
