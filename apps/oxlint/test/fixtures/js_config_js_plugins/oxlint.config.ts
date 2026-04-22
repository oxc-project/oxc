import { defineConfig } from "#oxlint";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  rules: {
    "basic-custom-plugin/no-debugger": "error",
  },
});
