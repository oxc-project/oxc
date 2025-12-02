import { createContext } from "./context.ts";
import { deepFreezeJsonArray } from "./json.ts";
import { DEFAULT_OPTIONS } from "./options.ts";
import { getErrorMessage } from "../utils/utils.ts";

import type { Writable } from "type-fest";
import type { Context } from "./context.ts";
import type { Options } from "./options.ts";
import type { RuleMeta } from "./rule_meta.ts";
import type { AfterHook, BeforeHook, Visitor, VisitorWithHooks } from "./types.ts";
import type { SetNullable } from "../utils/types.ts";

const ObjectKeys = Object.keys,
  { isArray } = Array;

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
 * Linter rule, context object, and other details of rule.
 * If `rule` has a `createOnce` method, the visitor it returns is stored in `visitor` property.
 */
export type RuleDetails = CreateRuleDetails | CreateOnceRuleDetails;

interface RuleDetailsBase {
  // Static properties of the rule
  readonly context: Readonly<Context>;
  readonly isFixable: boolean;
  readonly messages: Readonly<Record<string, string>> | null;
  readonly defaultOptions: Readonly<Options>;
  // Updated for each file
  ruleIndex: number;
  options: Readonly<Options> | null; // Initially `null`, set to options object before linting a file
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
const registeredPluginUrls = new Set<string>();

// Rule objects for loaded rules.
// Indexed by `ruleId`, which is passed to `lintFile`.
export const registeredRules: RuleDetails[] = [];

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
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions containing try/catch.
 *
 * @param url - Absolute path of plugin file as a `file://...` URL
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns Plugin details or error serialized to JSON string
 */
export async function loadPlugin(url: string, packageName: string | null): Promise<string> {
  try {
    if (DEBUG) {
      if (registeredPluginUrls.has(url)) throw new Error("This plugin has already been registered");
      registeredPluginUrls.add(url);
    }

    const plugin = (await import(url)).default as Plugin;
    const res = registerPlugin(plugin, packageName);
    return JSON.stringify({ Success: res });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Register a plugin.
 *
 * @param plugin - Plugin
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns - Plugin details
 * @throws {Error} If `plugin.meta.name` is `null` / `undefined` and `packageName` not provided
 * @throws {TypeError} If one of plugin's rules is malformed, or its `createOnce` method returns invalid visitor
 * @throws {TypeError} If `plugin.meta.name` is not a string
 */
export function registerPlugin(plugin: Plugin, packageName: string | null): PluginDetails {
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
      messages: Record<string, string> | null = null,
      defaultOptions: Readonly<Options> = DEFAULT_OPTIONS;
    const ruleMeta = rule.meta;
    if (ruleMeta != null) {
      if (typeof ruleMeta !== "object") throw new TypeError("Invalid `rule.meta`");

      const { fixable } = ruleMeta;
      if (fixable != null) {
        if (fixable !== "code" && fixable !== "whitespace")
          throw new TypeError("Invalid `rule.meta.fixable`");
        isFixable = true;
      }

      const inputDefaultOptions = ruleMeta.defaultOptions;
      if (inputDefaultOptions != null) {
        // TODO: Validate is JSON-serializable, and validate against provided options schema
        if (!isArray(inputDefaultOptions)) {
          throw new TypeError("`rule.meta.defaultOptions` must be an array if provided");
        }
        deepFreezeJsonArray(inputDefaultOptions);
        defaultOptions = inputDefaultOptions;
      }

      // Extract messages for messageId support
      const inputMessages = ruleMeta.messages;
      if (inputMessages != null) {
        if (typeof inputMessages !== "object") {
          throw new TypeError("`rule.meta.messages` must be an object if provided");
        }
        messages = inputMessages;
      }
    }

    // Create `RuleDetails` object for rule.
    const ruleDetails: RuleDetails = {
      rule: rule as CreateRule, // Could also be `CreateOnceRule`, but just to satisfy type checker
      context: null!, // Filled in below
      isFixable,
      messages,
      defaultOptions,
      ruleIndex: 0,
      options: null,
      visitor: null,
      beforeHook: null,
      afterHook: null,
    };

    // Create `Context` object for rule. This will be re-used for every file.
    const context = createContext(`${pluginName}/${ruleName}`, ruleDetails);
    (ruleDetails as Writable<RuleDetails>).context = context;

    if ("createOnce" in rule) {
      // TODO: Compile visitor object to array here, instead of repeating compilation on each file
      const visitorWithHooks = rule.createOnce(context) as SetNullable<
        VisitorWithHooks,
        "before" | "after"
      >;
      if (typeof visitorWithHooks !== "object" || visitorWithHooks === null) {
        throw new TypeError("`createOnce` must return an object");
      }

      let { before: beforeHook, after: afterHook, ...visitor } = visitorWithHooks;
      beforeHook = conformHookFn(beforeHook, "before");
      afterHook = conformHookFn(afterHook, "after");

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

      (ruleDetails as unknown as Writable<CreateOnceRuleDetails>).visitor = visitor;
      (ruleDetails as unknown as Writable<CreateOnceRuleDetails>).beforeHook = beforeHook;
      (ruleDetails as unknown as Writable<CreateOnceRuleDetails>).afterHook = afterHook;
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
      if (typeof pluginMetaName !== "string")
        throw new TypeError("`plugin.meta.name` must be a string if defined");
      return pluginMetaName;
    }
  }

  if (packageName !== null) return packageName;

  throw new Error(
    "Plugin must either define `meta.name`, or be loaded from an NPM package with a `name` field in `package.json`",
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
  if (typeof hookFn !== "function")
    throw new TypeError(`\`${hookName}\` hook must be a function if provided`);
  return hookFn;
}
