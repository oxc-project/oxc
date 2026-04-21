import { defineConfig } from "#oxlint";

const options = {
  typeAware: true,
  typeCheck: true,
};

export default defineConfig({
  categories: { correctness: "off" },
  rules: {
    "typescript/no-floating-promises": "error",
  },
  options,
});
