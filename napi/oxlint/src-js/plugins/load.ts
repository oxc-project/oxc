import { Context } from './context.js';
import { getErrorMessage } from './utils.js';

import type { Visitor } from './types.ts';

// Linter plugin, comprising multiple rules
interface Plugin {
  meta: {
    name: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

// Linter rule
interface Rule {
  create: (context: Context) => Visitor;
}

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set<string>();

// Rule objects for loaded rules.
// Indexed by `ruleId`, which is passed to `lintFile`.
export const registeredRules: {
  rule: Rule;
  context: Context;
}[] = [];

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param {string} path - Absolute path of plugin file
 * @returns {string} - JSON result
 */
export async function loadPlugin(path: string): Promise<string> {
  try {
    return await loadPluginImpl(path);
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

async function loadPluginImpl(path: string): Promise<string> {
  if (registeredPluginPaths.has(path)) {
    return JSON.stringify({
      Failure: 'This plugin has already been registered',
    });
  }

  const { default: plugin } = (await import(path)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const pluginName = plugin.meta.name;
  const offset = registeredRules.length;
  const ruleNames = [];

  for (const [ruleName, rule] of Object.entries(plugin.rules)) {
    ruleNames.push(ruleName);
    registeredRules.push({
      rule,
      context: new Context(`${pluginName}/${ruleName}`),
    });
  }

  return JSON.stringify({ Success: { name: pluginName, offset, ruleNames } });
}
