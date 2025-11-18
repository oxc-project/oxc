import { pathToFileURL } from 'node:url';

import { createContext } from './context.js';
import { getErrorMessage } from './utils.js';

import type { Writable } from 'type-fest';
import type { Context } from './context.ts';
import type { JsonValue } from './json.ts';
import type { RuleMeta } from './rule_meta.ts';
import type { AfterHook, BeforeHook, Visitor, VisitorWithHooks } from './types.ts';

const ObjectKeys = Object.keys;

/**
 * Linter plugin, comprising multiple rules
 */
export interface Plugin {
  meta?: {
    name?: string;
  };
  rules: {
    [key: string]: Rule;
  };
}

/**
 * Linter rule.
 *
 * `Rule` can have either `create` method, or `createOnce` method.
 * If `createOnce` method is present, `create` is ignored.
 *
 * If defining the rule with `createOnce`, and you want the rule to work with ESLint too,
 * you need to wrap the rule with `defineRule`.
 */
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

/**
 * Options for a rule on a file.
 */
export type Options = JsonValue[];

/**
 * Linter rule, context object, and other details of rule.
 * If `rule` has a `createOnce` method, the visitor it returns is stored in `visitor` property.
 */
export type RuleDetails = CreateRuleDetails | CreateOnceRuleDetails;

interface RuleDetailsBase {
  // Static properties of the rule
  readonly context: Readonly<Context>;
  readonly isFixable: boolean;
  readonly messages: Readonly<Record<string, string>> | null;
  // Updated for each file
  ruleIndex: number;
  options: Readonly<Options>;
}

interface CreateRuleDetails extends RuleDetailsBase {
  readonly rule: CreateRule;
  readonly visitor: null;
  readonly beforeHook: null;
  readonly afterHook: null;
}

interface CreateOnceRuleDetails extends RuleDetailsBase {
  readonly rule: CreateOnceRule;
  readonly visitor: Visitor;
  readonly beforeHook: BeforeHook | null;
  readonly afterHook: AfterHook | null;
}

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set<string>();

// Rule objects for loaded rules.
// Indexed by `ruleId`, which is passed to `lintFile`.
export const registeredRules: RuleDetails[] = [];

// `before` hook which makes rule never run.
const neverRunBeforeHook: BeforeHook = () => false;

// Default rule options
const DEFAULT_OPTIONS: Readonly<Options> = Object.freeze([]);

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
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions containing try/catch.
 *
 * @param path - Absolute path of plugin file
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns Plugin details or error serialized to JSON string
 */
export async function loadPlugin(path: string, packageName: string | null): Promise<string> {
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
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns - Plugin details
 * @throws {Error} If plugin has already been registered
 * @throws {Error} If plugin has no name
 * @throws {TypeError} If one of plugin's rules is malformed, or its `createOnce` method returns invalid visitor
 * @throws {TypeError} if `plugin.meta.name` is not a string
 * @throws {*} If plugin throws an error during import
 */
async function loadPluginImpl(path: string, packageName: string | null): Promise<PluginDetails> {
  if (registeredPluginPaths.has(path)) {
    throw new Error('This plugin has already been registered. This is a bug in Oxlint. Please report it.');
  }

  const { default: plugin } = (await import(pathToFileURL(path).href)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules

  const pluginName = getPluginName(plugin, packageName);

  const offset = registeredRules.length;
  const { rules } = plugin;
  const ruleNames = ObjectKeys(rules);
  const ruleNamesLen = ruleNames.length;

  for (let i = 0; i < ruleNamesLen; i++) {
    const ruleName = ruleNames[i],
      rule = rules[ruleName];

    // Validate `rule.meta` and convert to vars with standardized shape
    let isFixable = false,
      messages: Record<string, string> | null = null;
    const ruleMeta = rule.meta;
    if (ruleMeta != null) {
      if (typeof ruleMeta !== 'object') throw new TypeError('Invalid `rule.meta`');

      const { fixable } = ruleMeta;
      if (fixable != null) {
        if (fixable !== 'code' && fixable !== 'whitespace') throw new TypeError('Invalid `rule.meta.fixable`');
        isFixable = true;
      }

      // Extract messages for messageId support
      const inputMessages = ruleMeta.messages;
      if (inputMessages != null) {
        if (typeof inputMessages !== 'object') {
          throw new TypeError('`rule.meta.messages` must be an object if provided');
        }
        messages = inputMessages;
      }
    }

    // Create `RuleDetails` object for rule.
    const ruleDetails: RuleDetails = {
      rule: rule as CreateRule, // Could also be `CreateOnceRule`, but just to satisfy type checker
      context: null as Readonly<Context>, // Filled in below
      isFixable,
      messages,
      ruleIndex: 0,
      options: DEFAULT_OPTIONS,
      visitor: null,
      beforeHook: null,
      afterHook: null,
    };

    // Create `Context` object for rule. This will be re-used for every file.
    const context = createContext(`${pluginName}/${ruleName}`, ruleDetails);
    (ruleDetails as Writable<RuleDetails>).context = context;

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

      (ruleDetails as Writable<CreateOnceRuleDetails>).visitor = visitor;
      (ruleDetails as Writable<CreateOnceRuleDetails>).beforeHook = beforeHook;
      (ruleDetails as Writable<CreateOnceRuleDetails>).afterHook = afterHook;
    }

    registeredRules.push(ruleDetails);
  }

  return { name: pluginName, offset, ruleNames };
}

/**
 * Get plugin name.
 * - If `plugin.meta.name` is defined, return it.
 * - Otherwise, fall back to `packageName`, if defined.
 * - If neither is defined, throw an error.
 *
 * @param plugin - Plugin object
 * @param packageName - Package name from `package.json`
 * @returns Plugin name
 * @throws {TypeError} If `plugin.meta.name` is not a string
 * @throws {Error} If neither `plugin.meta.name` nor `packageName` are defined
 */
function getPluginName(plugin: Plugin, packageName: string | null): string {
  const pluginMeta = plugin.meta;
  if (pluginMeta != null) {
    const pluginMetaName = pluginMeta.name;
    if (pluginMetaName != null) {
      if (typeof pluginMetaName !== 'string') throw new TypeError('`plugin.meta.name` must be a string if defined');
      return pluginMetaName;
    }
  }

  if (packageName !== null) return packageName;

  throw new Error(
    'Plugin must either define `meta.name`, or be loaded from an NPM package with a `name` field in `package.json`',
  );
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
