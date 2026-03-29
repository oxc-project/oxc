/*
 * `definePlugin` and `defineRule` functions.
 */

import type { Plugin, Rule } from "../plugins/load.ts";

/**
 * Define a plugin.
 *
 * No-op function, just to provide type safety. Input is passed through unchanged.
 *
 * @param plugin - Plugin to define
 * @returns Same plugin as passed in
 */
export function definePlugin(plugin: Plugin): Plugin {
  return plugin;
}

/**
 * Define a rule.
 *
 * No-op function, just to provide type safety. Input is passed through unchanged.
 *
 * @param rule - Rule to define
 * @returns Same rule as passed in
 */
export function defineRule(rule: Rule): Rule {
  return rule;
}
