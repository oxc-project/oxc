// Test that overrides work in oxlint.config.ts
import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "no-debugger": "off",
  },
  overrides: [
    {
      files: ["*.ts"],
      rules: {
        "no-debugger": "error",
      },
    },
  ],
});
