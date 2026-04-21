import { defineConfig } from "#oxlint";

console.log("stdout pollution from config");
export default defineConfig({
  categories: {
    correctness: "off",
  },
  rules: {
    "no-debugger": "error",
  },
});
