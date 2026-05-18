import { defineConfig } from "#oxlint";

import base from "./base.ts";

export default defineConfig({
  categories: {
    correctness: "off",
  },
  extends: [base],
});
