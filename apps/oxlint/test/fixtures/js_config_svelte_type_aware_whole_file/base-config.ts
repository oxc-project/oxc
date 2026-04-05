import { defineConfig } from "#oxlint";
import svelteConfig from "./svelte.config.ts";
import tsParser from "./ts-parser.ts";

export default defineConfig({
  languageOptions: {
    parserOptions: {
      parser: tsParser,
      svelteConfig,
      tsFlavor: "base-ts-parser",
    },
  },
});
