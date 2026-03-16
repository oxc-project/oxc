// Basic test for oxlint.config.ts support
import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "no-debugger": "error",
    eqeqeq: "warn",
  },
});
