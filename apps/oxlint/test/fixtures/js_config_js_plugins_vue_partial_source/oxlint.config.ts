import { defineConfig } from "#oxlint";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  rules: {
    "empty-lines-plugin/no-multiple-empty-lines": ["error", { max: 0, maxBOF: 0 }],
  },
});
