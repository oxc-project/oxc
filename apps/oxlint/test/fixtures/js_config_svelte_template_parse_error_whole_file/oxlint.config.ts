import { defineConfig } from "#oxlint";
import sveltePlugin from "eslint-plugin-svelte";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  extends: [sveltePlugin.configs.recommended],
});
