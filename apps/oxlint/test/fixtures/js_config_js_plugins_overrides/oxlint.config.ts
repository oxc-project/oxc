import { defineConfig } from "#oxlint";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  overrides: [
    {
      files: ["files/**/*.js"],
      jsPlugins: ["./plugin.ts"],
      rules: {
        "basic-custom-plugin/no-debugger": "error",
      },
    },
  ],
});
