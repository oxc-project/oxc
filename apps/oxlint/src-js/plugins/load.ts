import { createContext } from "./context.ts";
import { deepFreezeJsonArray } from "./json.ts";
import { compileSchema, DEFAULT_OPTIONS } from "./options.ts";
import { getErrorMessage } from "../utils/utils.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Writable } from "type-fest";
import type { Context } from "./context.ts";
import type { Options, SchemaValidator } from "./options.ts";
import type { RuleMeta } from "./rule_meta.ts";
import type { AfterHook, BeforeHook, Visitor, VisitorWithHooks } from "./types.ts";
import type { SetNullable } from "../utils/types.ts";

/**
 * Linter plugin, comprising multiple rules
 */
export interface Plugin {
  meta?: {
    name?: string;
  };
  rules: Record<string, Rule>;
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
  readonly fullName: string;
  readonly context: Readonly<Context>;
  readonly isFixable: boolean;
  readonly messages: Readonly<Record<string, string>> | null;
  readonly defaultOptions: Readonly<Options>;
  // Function to validate options against schema.
  // `false` means validation is disabled. Rule accepts any options.
  // `null` means no validator provided. Rule does not accept any options.
  readonly optionsSchemaValidator: SchemaValidator | false | null;
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
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @returns Plugin details or error serialized to JSON string
 */
export async function loadPlugin(
  url: string,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
): Promise<string> {
  try {
    if (DEBUG) {
      if (registeredPluginUrls.has(url)) throw new Error("This plugin has already been registered");
      registeredPluginUrls.add(url);
    }

    const plugin = (await import(url)).default as Plugin;
    const res = registerPlugin(plugin, pluginName, pluginNameIsAlias);
    return JSON.stringify({ Success: res });
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

/**
 * Register a plugin.
 *
 * @param plugin - Plugin
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @returns - Plugin details
 * @throws {Error} If `plugin.meta.name` is `null` / `undefined` and `packageName` not provided
 * @throws {TypeError} If one of plugin's rules is malformed, or its `createOnce` method returns invalid visitor
 * @throws {TypeError} If `plugin.meta.name` is not a string
 */
export function registerPlugin(
  plugin: Plugin,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
): PluginDetails {
  // TODO: Use a validation library to assert the shape of the plugin, and of rules

  pluginName = getPluginName(plugin, pluginName, pluginNameIsAlias);

  const offset = registeredRules.length;
  const { rules } = plugin;
  const ruleNames = Object.keys(rules);
  const ruleNamesLen = ruleNames.length;

  for (let i = 0; i < ruleNamesLen; i++) {
    const ruleName = ruleNames[i],
      rule = rules[ruleName];

    const fullRuleName = `${pluginName}/${ruleName}`;

    // Validate `rule.meta` and convert to vars with standardized shape
    let isFixable = false,
      messages: Record<string, string> | null = null,
      defaultOptions: Readonly<Options> = DEFAULT_OPTIONS,
      schemaValidator: SchemaValidator | false | null = null;
    const ruleMeta = rule.meta;
    if (ruleMeta != null) {
      if (typeof ruleMeta !== "object") throw new TypeError("Invalid `rule.meta`");

      const { fixable } = ruleMeta;
      if (fixable != null) {
        if (fixable !== "code" && fixable !== "whitespace") {
          throw new TypeError("Invalid `rule.meta.fixable`");
        }
        isFixable = true;
      }

      // If `schema` provided, compile schema to validator for applying schema defaults to options
      schemaValidator = compileSchema(ruleMeta.schema);

      // Get default options
      const inputDefaultOptions = ruleMeta.defaultOptions;
      if (inputDefaultOptions != null) {
        if (!Array.isArray(inputDefaultOptions)) {
          throw new TypeError("`rule.meta.defaultOptions` must be an array if provided");
        }

        // ESLint treats empty `defaultOptions` the same as no `defaultOptions`,
        // and does not validate against schema
        if (inputDefaultOptions.length !== 0) {
          defaultOptions = conformDefaultOptions(inputDefaultOptions);

          // Validate default options against schema, if schema was provided.
          // This also applies any defaults from schema.
          // `schemaValidator` can mutate original `rule.meta.defaultOptions` object in place,
          // but that's what ESLint does, so it's OK.
          if (schemaValidator === null) {
            throw new Error(
              `Rule ${fullRuleName}:\n` +
                "Rules which accept options must provide a schema as `rule.meta.schema`, " +
                "or disable schema validation with `rule.meta.schema: false` (not recommended).",
            );
          }
          // Note: `defaultOptions` is not frozen yet
          if (schemaValidator !== false) {
            schemaValidator(defaultOptions as Writable<typeof defaultOptions>, fullRuleName);
          }

          deepFreezeJsonArray(defaultOptions as Writable<typeof defaultOptions>);
        }
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
      fullName: fullRuleName,
      rule: rule as CreateRule, // Could also be `CreateOnceRule`, but just to satisfy type checker
      context: null!, // Filled in below
      isFixable,
      messages,
      defaultOptions,
      optionsSchemaValidator: schemaValidator,
      ruleIndex: 0,
      options: null,
      visitor: null,
      beforeHook: null,
      afterHook: null,
    };

    // Create `Context` object for rule. This will be re-used for every file.
    const context = createContext(ruleDetails);
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
      if (Object.keys(visitor).length === 0) {
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
 *
 * - Plugin is named with an alias in config, return the alias.
 * - If `plugin.meta.name` is defined, return it.
 * - Otherwise, fall back to `packageName`, if defined.
 * - If neither is defined, throw an error.
 *
 * @param plugin - Plugin object
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @returns Plugin name
 * @throws {TypeError} If `plugin.meta.name` is not a string
 * @throws {Error} If neither `plugin.meta.name` nor `packageName` are defined
 */
function getPluginName(
  plugin: Plugin,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
): string {
  // If plugin is defined with an alias in config, that takes priority
  if (pluginNameIsAlias) {
    debugAssertIsNonNull(pluginName);
    return pluginName;
  }

  // If plugin defines its own name, that takes priority over package name.
  // Normalize plugin name.
  const pluginMetaName = plugin.meta?.name;
  if (pluginMetaName != null) {
    if (typeof pluginMetaName !== "string") {
      throw new TypeError("`plugin.meta.name` must be a string if defined");
    }
    return normalizePluginName(pluginMetaName);
  }

  // Fallback to package name (which is already normalized on Rust side)
  if (pluginName !== null) return pluginName;

  throw new Error(
    "Plugin must either define `meta.name`, be loaded from an NPM package with a `name` field in `package.json`, " +
      "or be given an alias in config",
  );
}

/**
 * Normalize plugin name by stripping common ESLint plugin prefixes and suffixes.
 *
 * This handles the various naming conventions used in the ESLint ecosystem:
 * - `eslint-plugin-foo` -> `foo`
 * - `@scope/eslint-plugin` -> `@scope`
 * - `@scope/eslint-plugin-foo` -> `@scope/foo`
 *
 * This logic is replicated on Rust side in `normalize_plugin_name` in `crates/oxc_linter/src/config/plugins.rs`.
 * The 2 implementations must be kept in sync.
 *
 * @param name - Plugin name defined by plugin
 * @returns Normalized plugin name
 */
function normalizePluginName(name: string): string {
  const slashIndex = name.indexOf("/");

  // If no slash, it's a non-scoped package. Trim off `eslint-plugin-` prefix.
  if (slashIndex === -1) {
    return name.startsWith("eslint-plugin-") ? name.slice("eslint-plugin-".length) : name;
  }

  const scope = name.slice(0, slashIndex),
    rest = name.slice(slashIndex + 1);

  // `@scope/eslint-plugin` -> `@scope`
  if (rest === "eslint-plugin") return scope;
  // `@scope/eslint-plugin-foo` -> `@scope/foo`
  if (rest.startsWith("eslint-plugin-")) return `${scope}/${rest.slice("eslint-plugin-".length)}`;

  // No normalization needed
  return name;
}

/**
 * Serialize default options to JSON and deserialize again.
 *
 * This is the simplest way to make sure that `defaultOptions` does not contain any `undefined` values,
 * or circular references. It may also be the fastest, as `JSON.parse` and `JSON.stringify` are native code.
 * If we move to doing options merging on Rust side, we'll need to convert to JSON anyway.
 *
 * Special handling for `Infinity` / `-Infinity` values, to ensure they survive the round trip.
 * Without this, they would be converted to `null`.
 *
 * @param defaultOptions - Default options array
 * @returns Conformed default options array
 */
function conformDefaultOptions(defaultOptions: Options): Options {
  let json,
    containsInfinity = false;
  try {
    json = JSON.stringify(defaultOptions, (key, value) => {
      if (value === Infinity || value === -Infinity) {
        containsInfinity = true;
        return value === Infinity ? POS_INFINITY_PLACEHOLDER : NEG_INFINITY_PLACEHOLDER;
      }
      return value;
    });
  } catch (err) {
    throw new Error(
      `\`rule.meta.defaultOptions\` must be JSON-serializable: ${getErrorMessage(err)}`,
    );
  }

  if (containsInfinity) {
    const plainJson = JSON.stringify(defaultOptions);
    if (
      plainJson.includes(POS_INFINITY_PLACEHOLDER) ||
      plainJson.includes(NEG_INFINITY_PLACEHOLDER)
    ) {
      throw new Error(
        `\`rule.meta.defaultOptions\` cannot contain the strings "${POS_INFINITY_PLACEHOLDER}" or "${NEG_INFINITY_PLACEHOLDER}"`,
      );
    }

    // `JSON.parse` will convert these back to `Infinity` / `-Infinity`
    json = json
      .replaceAll(POS_INFINITY_PLACEHOLDER_STR, "1e+400")
      .replaceAll(NEG_INFINITY_PLACEHOLDER_STR, "-1e+400");
  }

  return JSON.parse(json);
}

const POS_INFINITY_PLACEHOLDER = "$_$_$_POS_INFINITY_$_$_$";
const NEG_INFINITY_PLACEHOLDER = "$_$_$_NEG_INFINITY_$_$_$";
const POS_INFINITY_PLACEHOLDER_STR = JSON.stringify(POS_INFINITY_PLACEHOLDER);
const NEG_INFINITY_PLACEHOLDER_STR = JSON.stringify(NEG_INFINITY_PLACEHOLDER);

/**
 * Validate and conform `before` / `after` hook function.
 * @param hookFn - Hook function, or `null` / `undefined`
 * @param hookName - Name of the hook
 * @returns Hook function, or null
 * @throws {TypeError} If `hookFn` is not a function, `null`, or `undefined`
 */
function conformHookFn<H>(hookFn: H | null | undefined, hookName: string): H | null {
  if (hookFn == null) return null;
  if (typeof hookFn !== "function") {
    throw new TypeError(`\`${hookName}\` hook must be a function if provided`);
  }
  return hookFn;
}
