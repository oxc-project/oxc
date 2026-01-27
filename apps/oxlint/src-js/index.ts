import { definePlugin as _definePlugin, defineRule as _defineRule } from "./package/define.ts";
import { RuleTester as _RuleTester } from "./package/rule_tester.ts";

/**
 * @deprecated Import from `oxlint/plugin` instead
 */
export const definePlugin = _definePlugin;

/**
 * @deprecated Import from `oxlint/plugin` instead
 */
export const defineRule = _defineRule;

/**
 * @deprecated Import from `oxlint/rule-tester` instead
 */
export const RuleTester = _RuleTester;
