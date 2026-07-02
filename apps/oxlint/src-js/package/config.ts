/*
 * `defineConfig` helper and config types.
 *
 * Types are generated from npm/oxlint/configuration_schema.json.
 */

import type {
  AllowWarnDeny,
  DummyRule,
  DummyRuleMap,
  ExternalPluginEntry,
  Oxlintrc as FullOxlintrc,
  OxlintEnv,
  OxlintGlobals,
  OxlintOverride,
  RuleCategories,
} from "./config.generated.ts";

type Oxlintrc = Omit<FullOxlintrc, "$schema" | "extends">;

export type {
  AllowWarnDeny,
  DummyRule,
  DummyRuleMap,
  RuleCategories,
  OxlintGlobals,
  OxlintEnv,
  ExternalPluginEntry,
};

export type ExternalPluginsConfig = Exclude<Oxlintrc["jsPlugins"], undefined | null>;

export interface OxlintConfig extends Oxlintrc {
  extends?: OxlintConfig[];
}

export type { OxlintOverride };

interface OxlintConfigContext {
  /**
   * Configure an entire category of rules all at once.
   *
   * Rules enabled or disabled this way will be overwritten by individual rules in the `rules` field.
   */
  categories?: FullOxlintrc["categories"];
  /**
   * Environments enable and disable collections of global variables.
   */
  env?: FullOxlintrc["env"];
  /**
   * Configuration files that this configuration extends.
   */
  extends?: OxlintConfig[];
  /**
   * Enabled or disabled specific global variables.
   */
  globals?: FullOxlintrc["globals"];
  /**
   * Globs to ignore during linting. These are resolved from the configuration file path.
   */
  ignorePatterns?: FullOxlintrc["ignorePatterns"];
  /**
   * JS plugins, allows usage of ESLint plugins with Oxlint.
   */
  jsPlugins?: FullOxlintrc["jsPlugins"];
  /**
   * Oxlint config options.
   */
  options?: FullOxlintrc["options"];
  /**
   * Add, remove, or otherwise reconfigure rules for specific files or groups of files.
   */
  overrides?: FullOxlintrc["overrides"];
  /**
   * Enabled built-in plugins for Oxlint.
   *
   * Setting this field will overwrite the base set of plugins.
   */
  plugins?: FullOxlintrc["plugins"];
  /**
   * Configure linter rules.
   *
   * See Oxlint Rules for the list of rules.
   */
  rules?: FullOxlintrc["rules"];
  /**
   * Plugin-specific configuration for both built-in and custom plugins.
   */
  settings?: FullOxlintrc["settings"];
}

/**
 * Define an Oxlint configuration with type inference.
 *
 * @param config - Oxlint configuration
 * @returns Config unchanged
 */
export function defineConfig<T extends OxlintConfig>(config: T & OxlintConfigContext): T {
  return config;
}
