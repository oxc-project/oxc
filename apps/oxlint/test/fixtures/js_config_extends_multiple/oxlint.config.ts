import { defineConfig } from "#oxlint";

import baseA from "./base_a.ts";
import baseB from "./base_b.ts";

export default defineConfig({
  extends: [baseA, baseB],
  rules: {
    "no-debugger": "error",
  },
});
