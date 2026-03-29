// Basic test for oxlint.config.ts support
import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "typescript/no-floating-promises": "error",
  },
  categories: { correctness: "off" },
  options: {
    typeAware: true,
  },
});
