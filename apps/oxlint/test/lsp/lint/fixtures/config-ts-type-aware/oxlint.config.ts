import { defineConfig } from "#oxlint";

export default defineConfig({
  options: {
    typeAware: true,
  },
  rules: {
    "typescript/no-floating-promises": "error",
  },
});
