import { defineConfig } from "#oxlint";

const warningLimit = 0;
const options = {
  maxWarnings: warningLimit,
};

export default defineConfig({
  categories: { correctness: "off" },
  rules: {
    "no-debugger": "warn",
  },
  options,
});
