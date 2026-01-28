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

export type OxlintConfig = Oxlintrc;

export type { OxlintOverride };

/**
 * Define an Oxlint configuration with type inference.
 *
 * @param config - Oxlint configuration
 * @returns Config unchanged
 */
export function defineConfig<T extends OxlintConfig>(config: T): T {
  return config;
}
