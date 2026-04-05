import { defineConfig } from "#oxlint";

export default defineConfig({
  extends: ["./base.json"],
  rules: {
    "no-debugger": "error",
  },
});
