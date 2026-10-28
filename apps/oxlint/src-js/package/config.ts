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
  Oxlintrc,
  OxlintEnv,
  OxlintGlobals,
  OxlintOverride,
  RuleCategories,
} from "./config.generated.ts";

export type {
  AllowWarnDeny,
  DummyRule,
  DummyRuleMap,
  RuleCategories,
  Oxlintrc,
  OxlintGlobals,
  OxlintOverride,
  OxlintEnv,
  ExternalPluginEntry,
};

export type ExternalPluginsConfig = Exclude<Oxlintrc["jsPlugins"], undefined | null>;

export interface OxlintConfig extends Omit<Oxlintrc, "$schema" | "extends"> {
  /**
   * Configuration objects to extend.
   *
   * Import configurations and pass them directly instead of using file paths.
   */
  extends?: OxlintConfig[];
}

/**
 * Define an Oxlint configuration with type inference.
 *
 * @param config - Oxlint configuration
 * @returns Config unchanged
 */
export function defineConfig<T extends OxlintConfig>(config: T & OxlintConfig): T {
  return config;
}
