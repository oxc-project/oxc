import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "no-debugger": "off",
  },
  overrides: [
    {
      files: ["files/**/*.js"],
      rules: {
        "no-debugger": "error",
      },
    },
  ],
});
