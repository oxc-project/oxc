import { defineConfig } from "#oxlint";

const options = {
  denyWarnings: true,
};

export default defineConfig({
  categories: { correctness: "off" },
  rules: {
    "no-debugger": "warn",
  },
  options,
});
