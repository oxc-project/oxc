import { pathToFileURL } from 'node:url';

import { Context } from './context.js';
import { getErrorMessage } from './utils.js';

import type { AfterHook, BeforeHook, RuleMeta, Visitor, VisitorWithHooks } from './types.ts';

const ObjectKeys = Object.keys;

// Linter plugin, comprising multiple rules
export interface Plugin {
  meta?: {
    name?: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

// Linter rule.
// `Rule` can have either `create` method, or `createOnce` method.
// If `createOnce` method is present, `create` is ignored.
export type Rule = CreateRule | CreateOnceRule;

export interface CreateRule {
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

// `before` hook which makes rule never run.
const neverRunBeforeHook: BeforeHook = () => false;

// Plugin details returned to Rust
interface PluginDetails {
  // Plugin name
  name: string;
  // Index of first rule of this plugin within `registeredRules`
  offset: number;
  // Names of rules within this plugin, in same order as in `registeredRules`
  ruleNames: string[];
}

/**
 * Load a plugin.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param path - Absolute path of plugin file
 * @param packageName - Optional package name from package.json (fallback if plugin.meta.name is missing)
 * @returns JSON result
 */
export async function loadPlugin(path: string, packageName?: string): Promise<string> {
  try {
    const res = await loadPluginImpl(path, packageName);
    return JSON.stringify({ Success: res });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Load a plugin.
 *
 * @param path - Absolute path of plugin file
 * @param packageName - Optional package name from package.json (fallback if plugin.meta.name is missing)
 * @returns - Plugin details
 * @throws {Error} If plugin has already been registered
 * @throws {TypeError} If one of plugin's rules is malformed or its `createOnce` method returns invalid visitor
 * @throws {*} If plugin throws an error during import
 */
async function loadPluginImpl(path: string, packageName?: string): Promise<PluginDetails> {
  if (registeredPluginPaths.has(path)) {
    throw new Error('This plugin has already been registered. This is a bug in Oxlint. Please report it.');
  }

  const { default: plugin } = (await import(pathToFileURL(path).href)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  // Get plugin name from plugin.meta.name, or fall back to package name from package.json
  const pluginName = plugin.meta?.name ?? packageName;
  if (!pluginName) {
    throw new TypeError(
      'Plugin must have either meta.name or be loaded from an npm package with a name field in package.json',
    );
  }
  const offset = registeredRules.length;
  const { rules } = plugin;
  const ruleNames = ObjectKeys(rules);
  const ruleNamesLen = ruleNames.length;

  for (let i = 0; i < ruleNamesLen; i++) {
    const ruleName = ruleNames[i],
      rule = rules[ruleName];

    // Validate `rule.meta` and convert to vars with standardized shape
    let isFixable = false;
    let messages: Record<string, string> | null = null;
    let ruleMeta = rule.meta;
    if (ruleMeta != null) {
      if (typeof ruleMeta !== 'object') throw new TypeError('Invalid `meta`');

      const { fixable } = ruleMeta;
      if (fixable != null) {
        if (fixable !== 'code' && fixable !== 'whitespace') throw new TypeError('Invalid `meta.fixable`');
        isFixable = true;
      }

      // Extract messages for messageId support
      const inputMessages = ruleMeta.messages;
      if (inputMessages != null) {
        if (typeof inputMessages !== 'object') throw new TypeError('`meta.messages` must be an object if provided');
        messages = inputMessages;
      }
    }

    // Create `Context` object for rule. This will be re-used for every file.
    // It's updated with file-specific data before linting each file with `setupContextForFile`.
    const context = new Context(`${pluginName}/${ruleName}`, isFixable, messages);

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

      // If empty visitor, make this rule never run by substituting a `before` hook which always returns `false`.
      // This means the original `before` hook won't run either.
      //
      // Reason for doing this is:
      // In future, we may do a check on Rust side whether AST contains any nodes which rules act on,
      // and if not, skip calling into JS entirely. In that case, the `before` hook won't get called.
      // We can't emulate that behavior exactly, but we can at least emulate it in this simple case,
      // and prevent users defining rules with *only* a `before` hook, which they expect to run on every file.
      if (ObjectKeys(visitor).length === 0) {
        beforeHook = neverRunBeforeHook;
        afterHook = null;
      }

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
