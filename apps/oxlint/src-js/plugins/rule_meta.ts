import type { Options, RuleOptionsSchema } from "./options.ts";

/**
 * Rule metadata.
 * `meta` property of `Rule`.
 */
export interface RuleMeta {
  /**
   * Type of rule.
   *
   * - `problem`: The rule is identifying code that either will cause an error or may cause a confusing behavior.
   *   Developers should consider this a high priority to resolve.
   * - `suggestion`: The rule is identifying something that could be done in a better way but no errors will occur
   *   if the code isn’t changed.
   * - `layout`: The rule cares primarily about whitespace, semicolons, commas, and parentheses, all the parts
   *   of the program that determine how the code looks rather than how it executes.
   *   These rules work on parts of the code that aren’t specified in the AST.
   */
  type?: "problem" | "suggestion" | "layout";
  /**
   * Rule documentation.
   */
  docs?: RuleDocs;
  /**
   * Templates for error/warning messages.
   */
  messages?: Record<string, string>;
  /**
   * Type of fixes that the rule provides.
   * Must be `'code'` or `'whitespace'` if the rule provides fixes.
   */
  fixable?: "code" | "whitespace";
  /**
   * Specifies whether rule can return suggestions.
   * Must be `true` if the rule provides suggestions.
   * @default false
   */
  hasSuggestions?: boolean;
  /**
   * Shape of options for the rule.
   * Mandatory if the rule has options.
   */
  schema?: RuleOptionsSchema;
  /**
   * Default options for the rule.
   * If present, any user-provided options in their config will be merged on top of them recursively.
   */
  // TODO: Use this to alter options passed to rules.
  defaultOptions?: Options;
  /**
   * Indicates whether the rule has been deprecated, and info about the deprecation and possible replacements.
   */
  deprecated?: boolean | RuleDeprecatedInfo;
  /**
   * Information about available replacements for the rule.
   * This may be an empty array to explicitly state there is no replacement.
   * @deprecated Use `deprecated.replacedBy` instead.
   */
  replacedBy?: RuleReplacedByInfo[];
}

/**
 * Rule documentation.
 * `docs` property of `RuleMeta`.
 *
 * Often used for documentation generation and tooling.
 */
export interface RuleDocs {
  /**
   * Short description of the rule.
   */
  description?: string;
  /**
   * Typically a boolean, representing whether the rule is enabled by the recommended config.
   */
  recommended?: unknown;
  /**
   * URL for rule documentation.
   */
  url?: string;
  /**
   * Other arbitrary user-defined properties.
   */
  [key: string]: unknown;
}

/**
 * Info about deprecation of a rule, and possible replacements.
 * `deprecated` property of `RuleMeta`.
 */
// Note: ESLint docs specifically say "Every property is optional."
export interface RuleDeprecatedInfo {
  /**
   * General message presentable to the user. May contain why this rule is deprecated or how to replace the rule.
   */
  message?: string;
  /**
   * URL with more information about this rule deprecation.
   */
  url?: string;
  /**
   * Information about available replacements for the rule.
   * This may be an empty array to explicitly state there is no replacement.
   */
  replacedBy?: RuleReplacedByInfo[];
  /**
   * Version (as semver string) deprecating the rule.
   */
  deprecatedSince?: string;
  /**
   * Version (as semver string) likely to remove the rule.
   * e.g. the next major version.
   *
   * The special value `null` means the rule will no longer be changed, but will be kept available indefinitely.
   */
  availableUntil?: string | null;
}

/**
 * Info about a possible replacement for a rule.
 */
// Note: ESLint docs specifically say "Every property is optional."
export interface RuleReplacedByInfo {
  /**
   * A general message about this rule replacement.
   */
  message?: string;
  /**
   * A URL with more information about this rule replacement.
   */
  url?: string;
  /**
   * Which plugin has the replacement rule.
   *
   * The `name` property should be the package name, and should be:
   * - `"oxlint"` if the replacement is an Oxlint core rule.
   * - `"eslint"` if the replacement is an ESLint core rule.
   *
   * This property should be omitted if the replacement rule is in the same plugin.
   */
  plugin?: RuleReplacedByExternalSpecifier;
  /**
   * Name of replacement rule.
   * May be omitted if the plugin only contains a single rule, or has the same name as the rule.
   */
  rule?: RuleReplacedByExternalSpecifier;
}

/**
 * Details about a plugin or rule that replaces a deprecated rule.
 */
// Note: ESLint docs specifically say "Every property is optional."
export interface RuleReplacedByExternalSpecifier {
  /**
   * For a plugin, the package name.
   * For a rule, the rule name.
   */
  name?: string;
  /**
   * URL pointing to documentation for the plugin / rule.
   */
  url?: string;
}
