/* oxlint-disable no-console */

import { join } from "node:path";
import { readFile } from "node:fs/promises";
import { hasOxfmtrcFile, createBlankOxfmtrcFile, saveOxfmtrcFile, exitWithError } from "./shared";

interface BiomeConfig {
  formatter?: BiomeFormatterConfig;
  javascript?: {
    formatter?: BiomeJsFormatterConfig;
  };
  files?: {
    includes?: string[];
  };
  overrides?: BiomeOverride[];
}

interface BiomeFormatterConfig {
  enabled?: boolean;
  indentStyle?: "tab" | "space";
  indentWidth?: number;
  lineWidth?: number;
  lineEnding?: "lf" | "crlf" | "cr";
  attributePosition?: "auto" | "multiline";
  bracketSpacing?: boolean;
}

interface BiomeJsFormatterConfig extends BiomeFormatterConfig {
  quoteStyle?: "single" | "double";
  jsxQuoteStyle?: "single" | "double";
  quoteProperties?: "asNeeded" | "preserve";
  trailingCommas?: "all" | "es5" | "none";
  semicolons?: "always" | "asNeeded";
  arrowParentheses?: "always" | "asNeeded";
  bracketSameLine?: boolean;
}

interface BiomeOverride {
  includes?: string[];
  formatter?: BiomeFormatterConfig;
  javascript?: {
    formatter?: BiomeJsFormatterConfig;
  };
}

const BIOME_DEFAULTS = {
  lineWidth: 80,
  indentStyle: "tab",
  indentWidth: 2,
  lineEnding: "lf",
  attributePosition: "auto",
  bracketSpacing: true,
  quoteStyle: "double",
  jsxQuoteStyle: "double",
  quoteProperties: "asNeeded",
  trailingCommas: "all",
  semicolons: "always",
  arrowParentheses: "always",
  bracketSameLine: false,
} as const;

/**
 * Run the `--migrate biome` command to migrate Biome's config to `.oxfmtrc.json` file.
 * https://biomejs.dev/reference/configuration/
 */
export async function runMigrateBiome() {
  const cwd = process.cwd();

  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt configuration file already exists.");
  }

  const biomeConfigPath = await resolveBiomeConfigFile(cwd);

  // No Biome config found, fallback with `--init` behavior
  if (!biomeConfigPath) {
    console.log("No Biome configuration file found.");

    const oxfmtrc = await createBlankOxfmtrcFile(cwd);
    const jsonStr = JSON.stringify(oxfmtrc, null, 2);

    // TODO: Create napi `validateConfig()` and use to ensure validity?

    try {
      await saveOxfmtrcFile(cwd, jsonStr);
      console.log("Created `.oxfmtrc.json` instead.");
    } catch {
      exitWithError("Failed to create `.oxfmtrc.json`.");
    }

    return;
  }

  let biomeConfig: BiomeConfig;
  try {
    const content = await readFile(biomeConfigPath, "utf8");
    // Biome supports JSONC (JSON with comments)
    biomeConfig = parseJSONC(content);
    console.log("Found Biome configuration at:", biomeConfigPath);
  } catch {
    return exitWithError(`Failed to parse: ${biomeConfigPath}`);
  }

  // Start with blank, then fill in from `biomeConfig`.
  // NOTE: Biome has two levels of formatter config:
  // - `formatter.*` for global options
  // - `javascript.formatter.*` for JS/TS specific options (takes precedence)
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);
  const formatterConfig = biomeConfig.formatter ?? {};
  const jsFormatterConfig = biomeConfig.javascript?.formatter ?? {};

  migrateIndentStyle(formatterConfig, jsFormatterConfig, oxfmtrc);
  migrateIndentWidth(formatterConfig, jsFormatterConfig, oxfmtrc);
  migrateLineWidth(formatterConfig, jsFormatterConfig, oxfmtrc);
  migrateQuoteStyle(jsFormatterConfig, oxfmtrc);
  migrateJsxQuoteStyle(jsFormatterConfig, oxfmtrc);
  migrateQuoteProperties(jsFormatterConfig, oxfmtrc);
  migrateTrailingCommas(jsFormatterConfig, oxfmtrc);
  migrateSemicolons(jsFormatterConfig, oxfmtrc);
  migrateArrowParentheses(jsFormatterConfig, oxfmtrc);
  migrateBracketSameLine(formatterConfig, jsFormatterConfig, oxfmtrc);
  migrateBracketSpacing(formatterConfig, jsFormatterConfig, oxfmtrc);
  migrateAttributePosition(formatterConfig, jsFormatterConfig, oxfmtrc);

  // Migrate ignore patterns from `files.includes` negated patterns
  const ignores = extractIgnorePatterns(biomeConfig);
  if (ignores.length > 0) {
    console.log("Migrated ignore patterns from Biome config");
  }
  // Keep ignorePatterns at the bottom
  delete oxfmtrc.ignorePatterns;
  oxfmtrc.ignorePatterns = ignores;

  // TODO: Oxfmt now supports `overrides`,
  // but automatic migration is complex due to different config structures.
  if (biomeConfig.overrides && biomeConfig.overrides.length > 0) {
    console.warn(`  - "overrides" cannot be migrated automatically yet`);
  }

  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  try {
    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json`.");
  } catch {
    return exitWithError("Failed to create `.oxfmtrc.json`.");
  }
}

// ---

async function resolveBiomeConfigFile(cwd: string): Promise<string | null> {
  // Biome supports both `biome.json` and `biome.jsonc`.
  // If both exist, `biome.json` takes priority.
  const candidates = ["biome.json", "biome.jsonc"];

  for (const filename of candidates) {
    const filepath = join(cwd, filename);
    try {
      // oxlint-disable-next-line no-await-in-loop -- sequential check is intentional
      await readFile(filepath, "utf8");
      return filepath;
    } catch {}
  }

  return null;
}

// https://github.com/fabiospampinato/tiny-jsonc/blob/bb722089210174ec9cb53afcce15245e7ee21b9a/src/index.ts
const stringOrCommentRe = /("(?:\\?[^])*?")|(\/\/.*)|(\/\*[^]*?\*\/)/g;
const stringOrTrailingCommaRe = /("(?:\\?[^])*?")|(,\s*)(?=]|})/g;
function parseJSONC(text: string) {
  text = String(text); // To be extra safe
  try {
    // Fast path for valid JSON
    return JSON.parse(text);
  } catch {
    // Slow path for JSONC and invalid inputs
    return JSON.parse(text.replace(stringOrCommentRe, "$1").replace(stringOrTrailingCommaRe, "$1"));
  }
}

// ---

// `indentStyle` -> `useTabs`
function migrateIndentStyle(
  formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.indentStyle ?? formatterConfig.indentStyle;
  if (value !== undefined) {
    oxfmtrc.useTabs = value === "tab";
  } else {
    // Biome default is "tab"
    oxfmtrc.useTabs = BIOME_DEFAULTS.indentStyle === "tab";
  }
}

// `indentWidth` -> `tabWidth`
function migrateIndentWidth(
  formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.indentWidth ?? formatterConfig.indentWidth;
  if (value !== undefined) {
    oxfmtrc.tabWidth = value;
  } else {
    oxfmtrc.tabWidth = BIOME_DEFAULTS.indentWidth;
  }
}

// `lineWidth` -> `printWidth`
function migrateLineWidth(
  formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.lineWidth ?? formatterConfig.lineWidth;
  if (value !== undefined) {
    oxfmtrc.printWidth = value;
  } else {
    // Biome default is 80, Oxfmt default is 100
    oxfmtrc.printWidth = BIOME_DEFAULTS.lineWidth;
  }
}

// `quoteStyle` -> `singleQuote`
function migrateQuoteStyle(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.quoteStyle;
  if (value !== undefined) {
    oxfmtrc.singleQuote = value === "single";
  } else {
    // Biome default is "double"
    oxfmtrc.singleQuote = false;
  }
}

// `jsxQuoteStyle` -> `jsxSingleQuote`
function migrateJsxQuoteStyle(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.jsxQuoteStyle;
  if (value !== undefined) {
    oxfmtrc.jsxSingleQuote = value === "single";
  } else {
    // Biome default is "double"
    oxfmtrc.jsxSingleQuote = false;
  }
}

// `quoteProperties` -> `quoteProps`
// Biome uses "asNeeded", Oxfmt uses "as-needed"
function migrateQuoteProperties(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.quoteProperties;
  if (value !== undefined) {
    if (value === "asNeeded") {
      oxfmtrc.quoteProps = "as-needed";
    } else if (value === "preserve") {
      oxfmtrc.quoteProps = "preserve";
    }
  } else {
    oxfmtrc.quoteProps = "as-needed";
  }
}

// `trailingCommas` -> `trailingComma`
function migrateTrailingCommas(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.trailingCommas;
  if (value !== undefined) {
    oxfmtrc.trailingComma = value;
  } else {
    oxfmtrc.trailingComma = BIOME_DEFAULTS.trailingCommas;
  }
}

// `semicolons` -> `semi`
function migrateSemicolons(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.semicolons;
  if (value !== undefined) {
    oxfmtrc.semi = value === "always";
  } else {
    // Biome default is "always"
    oxfmtrc.semi = BIOME_DEFAULTS.semicolons === "always";
  }
}

// `arrowParentheses` -> `arrowParens`
// Biome uses "asNeeded", Oxfmt uses "avoid"
function migrateArrowParentheses(
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.arrowParentheses;
  if (value !== undefined) {
    if (value === "always") {
      oxfmtrc.arrowParens = "always";
    } else if (value === "asNeeded") {
      oxfmtrc.arrowParens = "avoid";
    }
  } else {
    // Biome default is "always"
    oxfmtrc.arrowParens = BIOME_DEFAULTS.arrowParentheses === "always" ? "always" : "avoid";
  }
}

// `bracketSameLine`
function migrateBracketSameLine(
  _formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.bracketSameLine;
  if (value !== undefined) {
    oxfmtrc.bracketSameLine = value;
  } else {
    oxfmtrc.bracketSameLine = BIOME_DEFAULTS.bracketSameLine;
  }
}

// `bracketSpacing`
function migrateBracketSpacing(
  formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.bracketSpacing ?? formatterConfig.bracketSpacing;
  if (value !== undefined) {
    oxfmtrc.bracketSpacing = value;
  } else {
    oxfmtrc.bracketSpacing = BIOME_DEFAULTS.bracketSpacing;
  }
}

// `attributePosition` -> `singleAttributePerLine`
function migrateAttributePosition(
  formatterConfig: BiomeFormatterConfig,
  jsFormatterConfig: BiomeJsFormatterConfig,
  oxfmtrc: Record<string, unknown>,
): void {
  const value = jsFormatterConfig.attributePosition ?? formatterConfig.attributePosition;
  if (value !== undefined) {
    if (value === "multiline") {
      oxfmtrc.singleAttributePerLine = true;
    } else {
      oxfmtrc.singleAttributePerLine = false;
    }
  }
}

// ---

function extractIgnorePatterns(biomeConfig: BiomeConfig): string[] {
  // In Biome, patterns starting with `!` (but not `!!`) are used to exclude files.
  // These are converted to Oxfmt's `ignorePatterns` format.
  const ignores: string[] = [];

  if (biomeConfig.files?.includes) {
    for (const pattern of biomeConfig.files.includes) {
      if (pattern.startsWith("!") && !pattern.startsWith("!!")) {
        ignores.push(pattern.slice(1));
      }
    }
  }

  return ignores;
}
