import { defineConfig } from "#oxlint";

import base from "./base.ts";

export default defineConfig({
  extends: [base],
  rules: {
    "no-debugger": "error",
  },
});
