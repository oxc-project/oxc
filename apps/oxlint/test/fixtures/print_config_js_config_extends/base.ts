import { defineConfig } from "#oxlint";

export default defineConfig({
  plugins: ["import"],
  rules: {
    "import/no-unassigned-import": "error",
  },
  overrides: [
    {
      files: ["**/*.test.ts"],
      rules: {
        "no-debugger": "error",
      },
    },
  ],
});
