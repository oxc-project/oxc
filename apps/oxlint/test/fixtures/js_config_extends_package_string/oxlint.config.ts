import { defineConfig } from "#oxlint";

export default defineConfig({
  extends: ["oxlint-config-base"],
  rules: {
    "no-debugger": "error",
  },
});
