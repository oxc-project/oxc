/*
 * Oxlint JS plugin which provides built-in ESLint rules.
 */

import assert from "node:assert";
import { builtinRules } from "eslint/use-at-your-own-risk";

import type { Rule } from "eslint";

// ESLint rules to export in this plugin
const RULES = ["prefer-const"];

const rules: Record<string, Rule.RuleModule> = {};
for (const ruleName of RULES) {
  const rule = builtinRules.get(ruleName);
  assert(rule != null, `Cannot find rule '${ruleName}'`);
  rules[ruleName] = rule;
}

export default {
  meta: {
    name: "eslint-js",
  },
  rules,
};
