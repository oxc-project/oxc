import { Context } from './context.js';
import { getErrorMessage } from './utils.js';

import type { AfterHook, BeforeHook, RuleMeta, Visitor, VisitorWithHooks } from './types.ts';

const ObjectKeys = Object.keys;

// Linter plugin, comprising multiple rules
export interface Plugin {
  meta: {
    name: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

// Linter rule.
// `Rule` can have either `create` method, or `createOnce` method.
// If `createOnce` method is present, `create` is ignored.
export type Rule = CreateRule | CreateOnceRule;

interface CreateRule {
  meta?: RuleMeta;
  create: (context: Context) => Visitor;
}

export interface CreateOnceRule {
  meta?: RuleMeta;
  create?: (context: Context) => Visitor;
  createOnce: (context: Context) => VisitorWithHooks;
}

// Linter rule and context object.
// If `rule` has a `createOnce` method, the visitor it returns is stored in `visitor`.
type RuleAndContext = CreateRuleAndContext | CreateOnceRuleAndContext;

interface CreateRuleAndContext {
  rule: CreateRule;
  context: Context;
  visitor: null;
  beforeHook: null;
  afterHook: null;
}

interface CreateOnceRuleAndContext {
  rule: CreateOnceRule;
  context: Context;
  visitor: Visitor;
  beforeHook: BeforeHook | null;
  afterHook: AfterHook | null;
}

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set<string>();

// Rule objects for loaded rules.
// Indexed by `ruleId`, which is passed to `lintFile`.
export const registeredRules: RuleAndContext[] = [];

// Plugin details returned to Rust
interface PluginDetails {
  // Plugin name
  name: string;
  // Index of first rule of this plugin within `registeredRules`
  offset: number;
  // Names of rules within this plugin, in same order as in `registeredRules`
  ruleNames: string[];
}

// Default rule metadata, used if `rule.meta` property is empty.
const emptyRuleMeta: RuleMeta = {};

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param path - Absolute path of plugin file
 * @returns JSON result
 */
export async function loadPlugin(path: string): Promise<string> {
  try {
    const res = await loadPluginImpl(path);
    return JSON.stringify({ Success: res });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Load a plugin.
 *
 * @param path - Absolute path of plugin file
 * @returns - Plugin details
 * @throws {Error} If plugin has already been registered
 * @throws {TypeError} If one of plugin's rules is malformed or its `createOnce` method returns invalid visitor
 * @throws {*} If plugin throws an error during import
 */
async function loadPluginImpl(path: string): Promise<PluginDetails> {
  if (registeredPluginPaths.has(path)) {
    throw new Error('This plugin has already been registered. This is a bug in Oxlint. Please report it.');
  }

  const { default: plugin } = (await import(path)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const pluginName = plugin.meta.name;
  const offset = registeredRules.length;
  const { rules } = plugin;
  const ruleNames = ObjectKeys(rules);
  const ruleNamesLen = ruleNames.length;

  for (let i = 0; i < ruleNamesLen; i++) {
    const ruleName = ruleNames[i],
      rule = rules[ruleName];

    // Validate `rule.meta` and convert to object with standardized shape
    // (all properties defined with default values if not supplied)
    let ruleMeta = rule.meta;
    if (ruleMeta == null) {
      ruleMeta = emptyRuleMeta;
    } else {
      if (typeof ruleMeta !== 'object') throw new TypeError('Invalid `meta`');
      // TODO: Validate and conform individual properties of `meta` once they're supported
      ruleMeta = emptyRuleMeta;
    }

    // Create `Context` object for rule. This will be re-used for every file.
    // It's updated with file-specific data before linting each file with `setupContextForFile`.
    const context = new Context(`${pluginName}/${ruleName}`, ruleMeta);

    let ruleAndContext;
    if ('createOnce' in rule) {
      // TODO: Compile visitor object to array here, instead of repeating compilation on each file
      let visitorWithHooks = rule.createOnce(context);
      if (typeof visitorWithHooks !== 'object' || visitorWithHooks === null) {
        throw new TypeError('`createOnce` must return an object');
      }

      let { before: beforeHook, after: afterHook, ...visitor } = visitorWithHooks;
      beforeHook = conformHookFn(beforeHook, 'before');
      afterHook = conformHookFn(afterHook, 'after');

      ruleAndContext = { rule, context, visitor, beforeHook, afterHook };
    } else {
      ruleAndContext = { rule, context, visitor: null, beforeHook: null, afterHook: null };
    }

    registeredRules.push(ruleAndContext);
  }

  return { name: pluginName, offset, ruleNames };
}

/**
 * Validate and conform `before` / `after` hook function.
 * @param hookFn - Hook function, or `null` / `undefined`
 * @param hookName - Name of the hook
 * @returns Hook function, or null
 * @throws {TypeError} If `hookFn` is not a function, `null`, or `undefined`
 */
function conformHookFn<H>(hookFn: H | null | undefined, hookName: string): H | null {
  if (hookFn == null) return null;
  if (typeof hookFn !== 'function') throw new TypeError(`\`${hookName}\` hook must be a function if provided`);
  return hookFn;
}
