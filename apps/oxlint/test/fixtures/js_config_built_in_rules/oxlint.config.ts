import { defineConfig } from "#oxlint";
import { allRules } from "#oxlint/built-in-rules";

const noDebuggerRule = allRules.find(
  (rule) =>
    rule.plugin === "eslint" && rule.rule === "no-debugger" && rule.category === "correctness",
);

if (!noDebuggerRule) {
  throw new Error("Expected eslint/no-debugger rule metadata to exist.");
}

export default defineConfig({
  rules: {
    [noDebuggerRule.rule]: "error",
  },
});
