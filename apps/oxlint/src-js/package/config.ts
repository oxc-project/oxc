/*
 * `defineConfig` helper and config types.
 *
 * Types are generated from npm/oxlint/configuration_schema.json.
 */

import type { ParserLike } from "./parser.ts";
import type {
  AllowWarnDeny,
  DummyRule,
  DummyRuleMap,
  ExternalPluginEntry,
  Oxlintrc as FullOxlintrc,
  OxlintEnv,
  OxlintGlobals,
  OxlintOverride as GeneratedOxlintOverride,
  RuleCategories,
  RuleCategoryConfig,
} from "./config.generated.ts";

type Oxlintrc = Omit<FullOxlintrc, "$schema" | "extends" | "overrides">;

export type {
  AllowWarnDeny,
  DummyRule,
  DummyRuleMap,
  RuleCategories,
  RuleCategoryConfig,
  OxlintGlobals,
  OxlintEnv,
  ExternalPluginEntry,
};

export interface OxlintLanguageOptions {
  parser?: Readonly<ParserLike>;
  parserOptions?: Record<string, unknown>;
}

export interface OxlintOverride extends GeneratedOxlintOverride {
  languageOptions?: OxlintLanguageOptions;
}

export interface OxlintFlatCompatPlugin {
  meta?: {
    name?: string;
  };
  rules?: Record<string, unknown>;
}

export interface OxlintFlatCompatConfig {
  name?: string;
  files?: string[];
  ignores?: string[];
  processor?: string;
  plugins?: Record<string, OxlintFlatCompatPlugin>;
  settings?: FullOxlintrc["settings"];
  rules?: DummyRuleMap;
  languageOptions?: OxlintLanguageOptions;
}

export type OxlintExtendsEntry =
  | OxlintConfig
  | OxlintFlatCompatConfig
  | string
  | OxlintExtendsEntry[];

export type ExternalPluginsConfig = Exclude<Oxlintrc["jsPlugins"], undefined | null>;

export interface OxlintConfig extends Oxlintrc {
  extends?: OxlintExtendsEntry[];
  languageOptions?: OxlintLanguageOptions;
  overrides?: OxlintOverride[];
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
