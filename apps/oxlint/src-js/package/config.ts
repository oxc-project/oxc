/*
 * `defineConfig` helper and config types.
 */

import type { JsonObject, JsonValue } from "../plugins/json.ts";

export type AllowWarnDeny = "off" | "warn" | "error" | 0 | 1 | 2;

export type DummyRule = AllowWarnDeny | [AllowWarnDeny, ...JsonValue[]];

export type DummyRuleMap = Record<string, DummyRule>;

export type RuleCategories = Record<string, AllowWarnDeny>;
export type OxlintGlobals = Record<
  string,
  "readonly" | "writable" | "off" | "readable" | "writeable" | boolean
>;

export type OxlintEnv = Record<string, boolean>;

export type ExternalPluginEntry = string | { name: string; specifier: string };

export type ExternalPluginsConfig = ExternalPluginEntry[] | null;

export interface OxlintOverride {
  files: string[];
  env?: OxlintEnv;
  globals?: OxlintGlobals;
  plugins?: string[];
  jsPlugins?: ExternalPluginsConfig;
  rules?: DummyRuleMap;
}

export interface OxlintConfig {
  plugins?: string[];
  jsPlugins?: ExternalPluginsConfig;
  categories?: RuleCategories;
  rules?: DummyRuleMap;
  settings?: JsonObject;
  env?: OxlintEnv;
  globals?: OxlintGlobals;
  overrides?: OxlintOverride[];
  ignorePatterns?: string[];
}

/**
 * Define an Oxlint configuration with type inference.
 *
 * @param config - Oxlint configuration
 * @returns Config unchanged
 */
export function defineConfig<T extends OxlintConfig>(config: T): T {
  return config;
}
