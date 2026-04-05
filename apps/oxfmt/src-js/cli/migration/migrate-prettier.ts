/* oxlint-disable no-console */

import { basename, join } from "node:path";
import { access, readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { hasOxfmtrcFile, createBlankOxfmtrcFile, saveOxfmtrcFile, exitWithError } from "./shared";
import { looksLikePreservablePluginSpec, normalizePreservedPluginSpec } from "./plugin_specs";
import type { Options } from "prettier";

/**
 * Run the `--migrate prettier` command to migrate various Prettier's config to `.oxfmtrc.json` file.
 * https://prettier.io/docs/configuration
 */
export async function runMigratePrettier() {
  const cwd = process.cwd();

  if (await hasOxfmtrcFile(cwd)) {
    return exitWithError("Oxfmt configuration file already exists.");
  }

  // XXX: If you statically import `prettier` here,
  // completely unsure why, but Prettier hangs forever when run via `napi`...
  const { resolveConfigFile, resolveConfig } = await import("prettier");

  // NOTE: Prettier's config resolving is based on each file,
  // but ours is based on the project root, typically `cwd`.
  // So we assume the config for a dummy file at the `cwd`.
  const resolvedPrettierConfigPath = await resolveConfigFile(join(cwd, "dummy.js"));
  const prettierConfigPath = resolvedPrettierConfigPath ?? await findFallbackPrettierConfigFile(cwd);
  const usedManualConfigDiscovery = !resolvedPrettierConfigPath && !!prettierConfigPath;

  // No Prettier config found, fallback with `--init` behavior
  if (!prettierConfigPath) {
    console.log("No Prettier configuration file found.");

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

  currentMigrationReport = createMigrationReport();

  const rawPrettierConfig = await resolveRawPrettierConfig(prettierConfigPath);

  if (usedManualConfigDiscovery && !rawPrettierConfig) {
    return exitWithError(`Failed to parse: ${prettierConfigPath}`);
  }

  let prettierConfig;
  let useRawConfigFallback = usedManualConfigDiscovery;
  if (usedManualConfigDiscovery) {
    warnMigration(
      { applyDefaults: true },
      `Prettier did not discover ${basename(prettierConfigPath)} automatically; migrating from the raw config instead.`,
    );
    console.log("Found Prettier configuration at:", prettierConfigPath);
  } else {
    try {
      prettierConfig = await resolveConfig(prettierConfigPath, {
        // Avoid merging `.editorconfig` values
        editorconfig: false,
      });
      console.log("Found Prettier configuration at:", prettierConfigPath);
    } catch {
      if (!rawPrettierConfig) {
        return exitWithError(`Failed to parse: ${prettierConfigPath}`);
      }

      useRawConfigFallback = true;
      warnMigration(
        { applyDefaults: true },
        `Prettier could not fully resolve ${basename(prettierConfigPath)}; migrating from the raw config instead.`,
      );
      console.log("Found Prettier configuration at:", prettierConfigPath);
    }
  }

  const rootPrettierConfig = useRawConfigFallback
    ? rawPrettierConfig
    : {
        ...(prettierConfig ?? {}),
        ...(rawPrettierConfig?.plugins !== undefined ? { plugins: rawPrettierConfig.plugins } : {}),
      };

  // Start with blank, then fill in from `prettierConfig`.
  // NOTE: Some options unsupported by Oxfmt may still be valid when invoking Prettier.
  // However, to avoid inconsistency, we do not enable options that affect Oxfmt.
  const oxfmtrc = await createBlankOxfmtrcFile(cwd);
  migratePrettierOptions(rootPrettierConfig, oxfmtrc, { applyDefaults: true }, cwd);

  const migratedOverrides = migratePrettierOverrides(rawPrettierConfig, cwd);
  if (migratedOverrides) {
    oxfmtrc.overrides = migratedOverrides;
  }

  // Migrate `ignorePatterns` from `.prettierignore`
  const ignores = await resolvePrettierIgnore(cwd);
  if (ignores.length > 0) {
    console.log("Migrated ignore patterns from `.prettierignore`");
  }
  // Keep ignorePatterns at the bottom
  delete oxfmtrc.ignorePatterns;
  oxfmtrc.ignorePatterns = ignores;

  const jsonStr = JSON.stringify(oxfmtrc, null, 2);

  // TODO: Create napi `validateConfig()` and use to ensure validity?

  try {
    await saveOxfmtrcFile(cwd, jsonStr);
    console.log("Created `.oxfmtrc.json`.");
    printMigrationSummary(currentMigrationReport);
  } catch {
    return exitWithError("Failed to create `.oxfmtrc.json`.");
  }
}

// ---

type PrettierConfigObject = Record<string, unknown>;

type MigrationScope = {
  applyDefaults: boolean;
  label?: string;
};

const JSON_LIKE_PRETTIER_CONFIG_BASENAMES = new Set([
  ".prettierrc",
  ".prettierrc.json",
  ".prettierrc.jsonc",
  "prettier.config.json",
  "prettier.config.jsonc",
  "package.json",
]);

const YAML_LIKE_PRETTIER_CONFIG_BASENAMES = new Set([
  ".prettierrc.yaml",
  ".prettierrc.yml",
  "prettier.config.yaml",
  "prettier.config.yml",
]);

const JS_LIKE_PRETTIER_CONFIG_BASENAMES = new Set([
  ".prettierrc.js",
  ".prettierrc.cjs",
  ".prettierrc.mjs",
  "prettier.config.js",
  "prettier.config.cjs",
  "prettier.config.mjs",
]);

const MANUAL_PRETTIER_CONFIG_DISCOVERY_BASENAMES = [
  ...[...JSON_LIKE_PRETTIER_CONFIG_BASENAMES].filter((basename) => basename !== "package.json"),
  ...YAML_LIKE_PRETTIER_CONFIG_BASENAMES,
  ...JS_LIKE_PRETTIER_CONFIG_BASENAMES,
  "package.json",
] as const;

const importedPluginSpecHints = new WeakMap<object, string>();

type MigrationWarning = {
  label?: string;
  message: string;
};

type MigrationReport = {
  inferredPluginSpecs: string[];
  migratedOverrideCount: number;
  migratedPackageJsonPluginScopes: string[];
  migratedTailwindPluginScopes: string[];
  preservedPluginSpecs: string[];
  skippedCustomPluginObjects: number;
  skippedOverrideCount: number;
  warnings: MigrationWarning[];
};

function createMigrationReport(): MigrationReport {
  return {
    inferredPluginSpecs: [],
    migratedOverrideCount: 0,
    migratedPackageJsonPluginScopes: [],
    migratedTailwindPluginScopes: [],
    preservedPluginSpecs: [],
    skippedCustomPluginObjects: 0,
    skippedOverrideCount: 0,
    warnings: [],
  };
}

let currentMigrationReport = createMigrationReport();

function migratePrettierOptions(
  prettierConfig: PrettierConfigObject,
  oxfmtrc: Record<string, unknown>,
  scope: MigrationScope,
  projectDir: string,
): void {
  let hasSortPackageJsonPlugin = false;
  let migratedPlugins: string[] | undefined;

  for (const [key, value] of Object.entries(prettierConfig)) {
    // Handle plugins specially:
    // - `prettier-plugin-tailwindcss` becomes `sortTailwindcss`
    // - `prettier-plugin-packagejson` becomes `sortPackageJson`
    // - other string plugin specs are preserved because Oxfmt can load them directly
    if (key === "plugins" && Array.isArray(value)) {
      const { plugins, usesSortPackageJsonPlugin } = migratePlugins(
        (value as Options["plugins"])!,
        prettierConfig,
        oxfmtrc,
        scope,
        projectDir,
      );
      migratedPlugins = plugins.length > 0 ? plugins : undefined;
      hasSortPackageJsonPlugin = usesSortPackageJsonPlugin;
      continue;
    }

    if (key === "overrides") {
      continue;
    }

    // Oxfmt does not support this, fallback to default
    if (key === "endOfLine" && value === "auto") {
      warnMigration(scope, '"endOfLine: auto" is not supported, skipping...');
      continue;
    }
    // Oxfmt does not support these experimental options yet
    if (key === "experimentalTernaries" || key === "experimentalOperatorPosition") {
      warnMigration(scope, `"${key}" is not supported in JS/TS files yet`);
      continue;
    }

    // Skip Tailwind options - handled separately by migrateTailwindOptions
    if (key.startsWith("tailwind")) {
      continue;
    }

    // Otherwise, copy the value.
    // This may include options that do not affect Oxfmt, like `vueIndentScriptAndStyle`.
    oxfmtrc[key] = value;
  }

  if (migratedPlugins) {
    oxfmtrc.plugins = migratedPlugins;
  }

  if (hasSortPackageJsonPlugin) {
    oxfmtrc.sortPackageJson = {};
    warnMigration(scope, 'Migrated "prettier-plugin-packagejson" to "sortPackageJson"');
  } else if (scope.applyDefaults) {
    // `sortPackageJson` is enabled by default in Oxfmt, but Prettier does not have this.
    // Only enable if `prettier-plugin-packagejson` is used.
    oxfmtrc.sortPackageJson = false;
  }

  if (scope.applyDefaults) {
    // `printWidth` has different default between Prettier and Oxfmt.
    // Oxfmt default is 100, Prettier default is 80.
    if (typeof oxfmtrc.printWidth !== "number") {
      warnMigration(
        scope,
        '"printWidth" is not set in Prettier config, defaulting to 80 (Oxfmt default: 100)',
      );
      oxfmtrc.printWidth = 80;
    }

    // `embeddedLanguageFormatting` is not fully supported for JS-in-XXX yet.
    if (oxfmtrc.embeddedLanguageFormatting !== "off") {
      warnMigration(scope, '"embeddedLanguageFormatting" in JS/TS files is not fully supported yet');
    }
  } else if (oxfmtrc.embeddedLanguageFormatting !== undefined && oxfmtrc.embeddedLanguageFormatting !== "off") {
    warnMigration(scope, '"embeddedLanguageFormatting" in JS/TS files is not fully supported yet');
  }
}

function migratePrettierOverrides(
  rawPrettierConfig: PrettierConfigObject | null,
  projectDir: string,
): Array<Record<string, unknown>> | undefined {
  const overrides = rawPrettierConfig?.overrides;
  if (!Array.isArray(overrides)) {
    return undefined;
  }

  const migratedOverrides: Array<Record<string, unknown>> = [];

  for (const [index, overrideEntry] of overrides.entries()) {
    if (!isRecord(overrideEntry)) {
      currentMigrationReport.skippedOverrideCount += 1;
      warnMigration({ label: `overrides[${index}]`, applyDefaults: false }, "invalid override entry, skipping...");
      continue;
    }

    const files = normalizeOverridePatterns(overrideEntry.files);
    if (!files || files.length === 0) {
      currentMigrationReport.skippedOverrideCount += 1;
      warnMigration(
        { label: `overrides[${index}]`, applyDefaults: false },
        'missing valid "files" patterns, skipping override...',
      );
      continue;
    }

    const migratedOverride: Record<string, unknown> = { files };
    const excludeFiles = normalizeOverridePatterns(overrideEntry.excludeFiles);
    if (excludeFiles && excludeFiles.length > 0) {
      migratedOverride.excludeFiles = excludeFiles;
    }

    const options = isRecord(overrideEntry.options) ? overrideEntry.options : {};
    const migratedOptions: Record<string, unknown> = {};
    migratePrettierOptions(options, migratedOptions, {
      applyDefaults: false,
      label: `overrides[${index}].options`,
    }, projectDir);

    migratedOverride.options = migratedOptions;
    migratedOverrides.push(migratedOverride);
    currentMigrationReport.migratedOverrideCount += 1;
  }

  return migratedOverrides.length > 0 ? migratedOverrides : undefined;
}

function migratePlugins(
  plugins: Options["plugins"],
  prettierConfig: Record<string, unknown>,
  oxfmtrc: Record<string, unknown>,
  scope: MigrationScope,
  projectDir: string,
): { plugins: string[]; usesSortPackageJsonPlugin: boolean } {
  const preservedPlugins: string[] = [];
  let usesSortPackageJsonPlugin = false;

  for (const plugin of plugins ?? []) {
    if (typeof plugin === "string") {
      usesSortPackageJsonPlugin = processMigratablePluginSpec(
        plugin,
        preservedPlugins,
        prettierConfig,
        oxfmtrc,
        scope,
        projectDir,
        usesSortPackageJsonPlugin,
      );
      continue;
    }

    const recognizedPluginSpec = inferRecognizedPluginSpecFromObject(plugin);
    if (recognizedPluginSpec) {
      usesSortPackageJsonPlugin = processMigratablePluginSpec(
        recognizedPluginSpec.spec,
        preservedPlugins,
        prettierConfig,
        oxfmtrc,
        scope,
        projectDir,
        usesSortPackageJsonPlugin,
      );
      pushUnique(currentMigrationReport.inferredPluginSpecs, normalizePreservedPluginSpec(recognizedPluginSpec.spec, projectDir));
      warnMigration(
        scope,
        `plugins: ${describeRecognizedPluginObject(recognizedPluginSpec.reason)}; using "${recognizedPluginSpec.spec}" because plugin objects cannot be stored in .oxfmtrc.json`,
      );
      continue;
    }

    currentMigrationReport.skippedCustomPluginObjects += 1;
    warnMigration(scope, "plugins: custom plugin module is not supported, skipping...");
  }

  return { plugins: preservedPlugins, usesSortPackageJsonPlugin };
}

type RecognizedPluginSpec = {
  spec: string;
  reason: "imported" | "metadata" | "svelte";
};

function processMigratablePluginSpec(
  pluginSpec: string,
  preservedPlugins: string[],
  prettierConfig: Record<string, unknown>,
  oxfmtrc: Record<string, unknown>,
  scope: MigrationScope,
  projectDir: string,
  usesSortPackageJsonPlugin: boolean,
): boolean {
  if (pluginSpec === "prettier-plugin-tailwindcss") {
    recordSpecialPluginMigration(currentMigrationReport.migratedTailwindPluginScopes, scope);
    migrateTailwindOptions(prettierConfig, oxfmtrc, scope);
    return usesSortPackageJsonPlugin;
  }

  if (pluginSpec === "prettier-plugin-packagejson") {
    recordSpecialPluginMigration(currentMigrationReport.migratedPackageJsonPluginScopes, scope);
    return true;
  }

  const normalizedPlugin = normalizePreservedPluginSpec(pluginSpec, projectDir);
  if (!preservedPlugins.includes(normalizedPlugin)) {
    preservedPlugins.push(normalizedPlugin);
    pushUnique(currentMigrationReport.preservedPluginSpecs, normalizedPlugin);
  }

  return usesSortPackageJsonPlugin;
}

function inferRecognizedPluginSpecFromObject(plugin: unknown): RecognizedPluginSpec | undefined {
  const hintedSpec = getImportedPluginSpecHint(plugin);
  if (hintedSpec) {
    return {
      spec: hintedSpec,
      reason: hintedSpec === "prettier-plugin-svelte" ? "svelte" : "imported",
    };
  }

  const unwrappedPlugin = unwrapPluginObject(plugin);
  if (!isRecord(unwrappedPlugin)) {
    return undefined;
  }

  const metadataSpec = inferPluginSpecFromObjectMetadata(unwrappedPlugin);
  if (metadataSpec) {
    return { spec: metadataSpec, reason: "metadata" };
  }

  if (looksLikeSvelteFormatterPlugin(unwrappedPlugin)) {
    return { spec: "prettier-plugin-svelte", reason: "svelte" };
  }

  return undefined;
}

function inferPluginSpecFromObjectMetadata(plugin: Record<string, unknown>): string | undefined {
  const candidates: Array<{ value: unknown; source: "packageName" | "name" }> = [
    { value: plugin.packageName, source: "packageName" },
    { value: isRecord(plugin.meta) ? plugin.meta.packageName : undefined, source: "packageName" },
    { value: plugin.name, source: "name" },
    { value: isRecord(plugin.meta) ? plugin.meta.name : undefined, source: "name" },
  ];

  for (const candidate of candidates) {
    if (typeof candidate.value !== "string") {
      continue;
    }

    const normalizedCandidate = candidate.value.trim();
    if (looksLikePreservablePluginSpec(normalizedCandidate, candidate.source)) {
      return normalizedCandidate;
    }
  }

  return undefined;
}

function describeRecognizedPluginObject(reason: RecognizedPluginSpec["reason"]): string {
  if (reason === "imported") {
    return "recognized an imported formatter plugin object";
  }

  if (reason === "metadata") {
    return "recognized a formatter plugin object via metadata";
  }

  return "recognized a Svelte formatter plugin object";
}

function unwrapPluginObject(plugin: unknown, depth = 0): unknown {
  if (depth >= 2 || !isRecord(plugin)) {
    return plugin;
  }

  if (looksLikePrettierPluginRecord(plugin)) {
    return plugin;
  }

  if ("default" in plugin) {
    return unwrapPluginObject(plugin.default, depth + 1);
  }

  return plugin;
}

function looksLikePrettierPluginRecord(plugin: Record<string, unknown>): boolean {
  return Array.isArray(plugin.languages) || isRecord(plugin.parsers) || isRecord(plugin.printers) || isRecord(plugin.options);
}

function looksLikeSvelteFormatterPlugin(plugin: Record<string, unknown>): boolean {
  const languages = Array.isArray(plugin.languages) ? plugin.languages.filter(isRecord) : [];
  const hasSvelteLanguage = languages.some((language) => {
    const name = typeof language.name === "string" ? language.name : "";
    const parsers = isStringArray(language.parsers) ? language.parsers : [];
    const extensions = isStringArray(language.extensions) ? language.extensions : [];
    const vscodeLanguageIds = isStringArray(language.vscodeLanguageIds) ? language.vscodeLanguageIds : [];

    return (
      name === "svelte"
      || parsers.includes("svelte")
      || extensions.includes(".svelte")
      || vscodeLanguageIds.includes("svelte")
    );
  });

  const parsers = isRecord(plugin.parsers) ? plugin.parsers : undefined;
  const printers = isRecord(plugin.printers) ? plugin.printers : undefined;
  const hasSvelteParser = !!parsers && "svelte" in parsers;
  const hasSveltePrinter = !!printers && "svelte-ast" in printers;

  return hasSvelteLanguage && (hasSvelteParser || hasSveltePrinter);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

// ---

async function resolvePrettierIgnore(cwd: string) {
  const ignores = [];

  try {
    const content = await readFile(join(cwd, ".prettierignore"), "utf8");

    const lines = content.split("\n");
    for (let line of lines) {
      line = line.trim();
      if (line === "" || line.startsWith("#")) {
        continue;
      }
      ignores.push(line);
    }
  } catch {}

  return ignores;
}

async function resolveRawPrettierConfig(
  prettierConfigPath: string,
): Promise<PrettierConfigObject | null> {
  const configBasename = basename(prettierConfigPath);

  if (JSON_LIKE_PRETTIER_CONFIG_BASENAMES.has(configBasename)) {
    return resolveRawJSONPrettierConfig(prettierConfigPath, configBasename);
  }

  if (YAML_LIKE_PRETTIER_CONFIG_BASENAMES.has(configBasename)) {
    return resolveRawYAMLPrettierConfig(prettierConfigPath);
  }

  if (JS_LIKE_PRETTIER_CONFIG_BASENAMES.has(configBasename)) {
    return resolveRawJSImportPrettierConfig(prettierConfigPath);
  }

  return null;
}

async function findFallbackPrettierConfigFile(cwd: string): Promise<string | null> {
  for (const configBasename of MANUAL_PRETTIER_CONFIG_DISCOVERY_BASENAMES) {
    const candidatePath = join(cwd, configBasename);
    if (!(await pathExists(candidatePath))) {
      continue;
    }

    if (configBasename === "package.json") {
      const packageJsonPrettierConfig = await resolveRawJSONPrettierConfig(candidatePath, configBasename);
      if (!packageJsonPrettierConfig) {
        continue;
      }
    }

    return candidatePath;
  }

  return null;
}

async function pathExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function resolveRawJSONPrettierConfig(
  prettierConfigPath: string,
  configBasename: string,
): Promise<PrettierConfigObject | null> {

  try {
    const rawText = await readFile(prettierConfigPath, "utf8");
    const parsed = parseJSONC(rawText);
    if (!isRecord(parsed)) {
      return null;
    }

    if (configBasename === "package.json") {
      return isRecord(parsed.prettier) ? parsed.prettier : null;
    }

    return parsed;
  } catch {
    return null;
  }
}

async function resolveRawYAMLPrettierConfig(
  prettierConfigPath: string,
): Promise<PrettierConfigObject | null> {
  try {
    const rawText = await readFile(prettierConfigPath, "utf8");
    const parsed = await parseYAMLPrettierConfig(rawText);
    return isRecord(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

async function resolveRawJSImportPrettierConfig(
  prettierConfigPath: string,
): Promise<PrettierConfigObject | null> {
  try {
    const sourceText = await readFile(prettierConfigPath, "utf8");
    const moduleUrl = pathToFileURL(prettierConfigPath);
    moduleUrl.searchParams.set("oxfmt-prettier-config", `${Date.now()}`);

    const importedModule = await import(moduleUrl.href);
    const importedConfig = await unwrapImportedPrettierConfig(importedModule);
    if (!isRecord(importedConfig)) {
      return null;
    }

    await collectImportedPluginSpecHints(prettierConfigPath, sourceText, importedConfig);
    return importedConfig;
  } catch {
    return null;
  }
}

async function unwrapImportedPrettierConfig(importedModule: Record<string, unknown>): Promise<unknown> {
  let importedConfig: unknown = importedModule.default ?? importedModule;
  if (typeof importedConfig === "function") {
    importedConfig = importedConfig();
  }
  return await importedConfig;
}


async function collectImportedPluginSpecHints(
  prettierConfigPath: string,
  sourceText: string,
  importedConfig: Record<string, unknown>,
): Promise<void> {
  const pluginObjects = collectPluginObjectsFromConfig(importedConfig);
  if (pluginObjects.length === 0) {
    return;
  }

  for (const pluginSpecifier of extractJSImportSpecifiers(sourceText)) {
    const loadedModule = await loadImportedPluginModule(prettierConfigPath, pluginSpecifier);
    if (!loadedModule) {
      continue;
    }

    const candidateValues = getLoadedPluginCandidateValues(loadedModule);
    for (const pluginObject of pluginObjects) {
      if (matchesLoadedPluginCandidate(pluginObject, candidateValues)) {
        setImportedPluginSpecHint(pluginObject, pluginSpecifier);
      }
    }
  }
}

function collectPluginObjectsFromConfig(
  value: unknown,
  seen = new Set<object>(),
  collected = new Set<object>(),
): object[] {
  if (!value || typeof value !== "object") {
    return [...collected];
  }

  if (seen.has(value)) {
    return [...collected];
  }
  seen.add(value);

  if (Array.isArray(value)) {
    for (const entry of value) {
      collectPluginObjectsFromConfig(entry, seen, collected);
    }
    return [...collected];
  }

  for (const [key, entry] of Object.entries(value)) {
    if (key === "plugins" && Array.isArray(entry)) {
      for (const plugin of entry) {
        if (plugin && typeof plugin === "object") {
          collected.add(plugin);
        }
      }
    }

    collectPluginObjectsFromConfig(entry, seen, collected);
  }

  return [...collected];
}

function extractJSImportSpecifiers(sourceText: string): string[] {
  const specifiers = new Set<string>();
  const specifierPatterns = [
    /\bfrom\s+["']([^"']+)["']/gu,
    /\brequire\(\s*["']([^"']+)["']\s*\)/gu,
    /\bimport\(\s*["']([^"']+)["']\s*\)/gu,
  ];

  for (const pattern of specifierPatterns) {
    for (const match of sourceText.matchAll(pattern)) {
      const specifier = match[1]?.trim();
      if (specifier) {
        specifiers.add(specifier);
      }
    }
  }

  return [...specifiers];
}

async function loadImportedPluginModule(
  prettierConfigPath: string,
  pluginSpecifier: string,
): Promise<unknown | null> {
  const requireFromConfig = createRequire(prettierConfigPath);

  try {
    const resolvedSpecifier = requireFromConfig.resolve(pluginSpecifier);
    try {
      return requireFromConfig(resolvedSpecifier);
    } catch {
      return await import(pathToFileURL(resolvedSpecifier).href);
    }
  } catch {
    try {
      return requireFromConfig(pluginSpecifier);
    } catch {
      return null;
    }
  }
}

function getLoadedPluginCandidateValues(loadedModule: unknown): unknown[] {
  const candidateValues: unknown[] = [loadedModule];
  if (isRecord(loadedModule) && "default" in loadedModule) {
    candidateValues.push(loadedModule.default);
  }

  return candidateValues.flatMap((candidateValue) => {
    const unwrappedCandidate = unwrapPluginObject(candidateValue);
    return candidateValue === unwrappedCandidate
      ? [candidateValue]
      : [candidateValue, unwrappedCandidate];
  });
}

function matchesLoadedPluginCandidate(pluginObject: object, candidateValues: unknown[]): boolean {
  const unwrappedPluginObject = unwrapPluginObject(pluginObject);
  return candidateValues.some((candidateValue) => {
    if (!candidateValue || typeof candidateValue !== "object") {
      return false;
    }

    const unwrappedCandidateValue = unwrapPluginObject(candidateValue);
    return candidateValue === pluginObject
      || candidateValue === unwrappedPluginObject
      || unwrappedCandidateValue === pluginObject
      || unwrappedCandidateValue === unwrappedPluginObject;
  });
}

function setImportedPluginSpecHint(plugin: object, spec: string): void {
  importedPluginSpecHints.set(plugin, spec);

  const unwrappedPlugin = unwrapPluginObject(plugin);
  if (unwrappedPlugin && typeof unwrappedPlugin === "object") {
    importedPluginSpecHints.set(unwrappedPlugin, spec);
  }
}

function getImportedPluginSpecHint(plugin: unknown): string | undefined {
  if (!plugin || typeof plugin !== "object") {
    return undefined;
  }

  const directHint = importedPluginSpecHints.get(plugin);
  if (directHint) {
    return directHint;
  }

  const unwrappedPlugin = unwrapPluginObject(plugin);
  if (unwrappedPlugin && typeof unwrappedPlugin === "object") {
    return importedPluginSpecHints.get(unwrappedPlugin);
  }

  return undefined;
}

function normalizeOverridePatterns(value: unknown): string[] | undefined {
  if (typeof value === "string") {
    return [value];
  }

  if (!Array.isArray(value)) {
    return undefined;
  }

  const patterns = value.filter((item): item is string => typeof item === "string");
  return patterns.length > 0 ? patterns : undefined;
}

async function parseYAMLPrettierConfig(text: string): Promise<unknown> {
  const externalParser = await tryLoadExternalYAMLParser();
  if (externalParser) {
    return externalParser(text);
  }

  return parseSimpleYAML(text);
}

async function tryLoadExternalYAMLParser(): Promise<((text: string) => unknown) | null> {
  const yamlModule = await tryImportModule("yaml");
  const yamlParse = getCallableExport(yamlModule, ["parse"]);
  if (yamlParse) {
    return (text) => yamlParse(text);
  }

  const jsYamlModule = await tryImportModule("js-yaml");
  const jsYamlLoad = getCallableExport(jsYamlModule, ["load"]);
  if (jsYamlLoad) {
    return (text) => jsYamlLoad(text);
  }

  return null;
}

async function tryImportModule(specifier: string): Promise<Record<string, unknown> | null> {
  try {
    const imported = await import(specifier);
    return isRecord(imported) ? imported : null;
  } catch {
    return null;
  }
}

function getCallableExport(
  module: Record<string, unknown> | null,
  exportNames: string[],
): ((text: string) => unknown) | undefined {
  if (!module) {
    return undefined;
  }

  for (const exportName of exportNames) {
    const directExport = module[exportName];
    if (typeof directExport === "function") {
      return directExport as (text: string) => unknown;
    }

    const defaultExport = isRecord(module.default) ? module.default[exportName] : undefined;
    if (typeof defaultExport === "function") {
      return defaultExport as (text: string) => unknown;
    }
  }

  return undefined;
}

function parseSimpleYAML(text: string): unknown {
  const lines = normalizeYAMLLines(text);
  if (lines.length === 0) {
    return {};
  }

  const state = { lines, index: 0 };
  const parsed = parseYAMLBlock(state, 0);
  if (state.index < lines.length) {
    throw new Error("Unexpected trailing YAML content");
  }
  return parsed;
}

type YamlLine = {
  indent: number;
  content: string;
};

type YamlParserState = {
  lines: YamlLine[];
  index: number;
};

function normalizeYAMLLines(text: string): YamlLine[] {
  const normalizedLines: YamlLine[] = [];
  for (const rawLine of text.replace(/^\uFEFF/, "").split(/\r?\n/u)) {
    const uncommentedLine = stripYAMLComment(rawLine);
    const trimmedLine = uncommentedLine.trim();
    if (trimmedLine === "" || trimmedLine === "---" || trimmedLine === "...") {
      continue;
    }

    const indentMatch = uncommentedLine.match(/^ */u);
    const indent = indentMatch ? indentMatch[0].length : 0;
    normalizedLines.push({ indent, content: uncommentedLine.slice(indent) });
  }
  return normalizedLines;
}

function stripYAMLComment(line: string): string {
  let inSingleQuote = false;
  let inDoubleQuote = false;

  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    const previousCharacter = index > 0 ? line[index - 1] : "";

    if (character === "'" && !inDoubleQuote) {
      inSingleQuote = !inSingleQuote;
      continue;
    }

    if (character === '"' && previousCharacter !== "\\" && !inSingleQuote) {
      inDoubleQuote = !inDoubleQuote;
      continue;
    }

    if (character === "#" && !inSingleQuote && !inDoubleQuote) {
      const previous = index === 0 ? "" : line[index - 1];
      if (previous === "" || /\s/u.test(previous)) {
        return line.slice(0, index).trimEnd();
      }
    }
  }

  return line;
}

function parseYAMLBlock(state: YamlParserState, indent: number): unknown {
  const line = state.lines[state.index];
  if (!line) {
    return {};
  }

  if (line.indent < indent) {
    return {};
  }

  if (line.content.startsWith("-")) {
    return parseYAMLSequence(state, indent);
  }

  return parseYAMLMapping(state, indent);
}

function parseYAMLSequence(state: YamlParserState, indent: number): unknown[] {
  const items: unknown[] = [];

  while (state.index < state.lines.length) {
    const line = state.lines[state.index];
    if (!line || line.indent < indent || !line.content.startsWith("-")) {
      break;
    }
    if (line.indent > indent) {
      break;
    }

    const itemContent = line.content.slice(1).trimStart();
    state.index += 1;

    if (itemContent === "") {
      items.push(parseRequiredNestedYAMLBlock(state, indent + 2));
      continue;
    }

    const inlinePair = splitYAMLKeyValue(itemContent);
    if (inlinePair) {
      const objectValue = parseYAMLSequenceItemMapping(state, indent + 2, inlinePair);
      items.push(objectValue);
      continue;
    }

    items.push(parseYAMLScalarOrFlow(itemContent));
  }

  return items;
}

function parseYAMLSequenceItemMapping(
  state: YamlParserState,
  nestedIndent: number,
  firstPair: { key: string; value: string },
): Record<string, unknown> {
  const mapping: Record<string, unknown> = {};
  assignYAMLMappingValue(state, mapping, nestedIndent, firstPair.key, firstPair.value);

  while (state.index < state.lines.length) {
    const line = state.lines[state.index];
    if (!line || line.indent < nestedIndent) {
      break;
    }
    if (line.indent > nestedIndent) {
      throw new Error("Unexpected nested YAML indentation in sequence item");
    }
    if (line.content.startsWith("-")) {
      break;
    }

    const pair = splitYAMLKeyValue(line.content);
    if (!pair) {
      throw new Error("Invalid YAML mapping entry in sequence item");
    }

    state.index += 1;
    assignYAMLMappingValue(state, mapping, nestedIndent, pair.key, pair.value);
  }

  return mapping;
}

function parseYAMLMapping(state: YamlParserState, indent: number): Record<string, unknown> {
  const mapping: Record<string, unknown> = {};

  while (state.index < state.lines.length) {
    const line = state.lines[state.index];
    if (!line || line.indent < indent) {
      break;
    }
    if (line.indent > indent) {
      throw new Error("Unexpected nested YAML indentation");
    }
    if (line.content.startsWith("-")) {
      break;
    }

    const pair = splitYAMLKeyValue(line.content);
    if (!pair) {
      throw new Error("Invalid YAML mapping entry");
    }

    state.index += 1;
    assignYAMLMappingValue(state, mapping, indent, pair.key, pair.value);
  }

  return mapping;
}

function assignYAMLMappingValue(
  state: YamlParserState,
  mapping: Record<string, unknown>,
  indent: number,
  key: string,
  rawValue: string,
): void {
  if (rawValue === "") {
    mapping[key] = parseRequiredNestedYAMLBlock(state, indent + 2);
    return;
  }

  mapping[key] = parseYAMLScalarOrFlow(rawValue);
}

function parseRequiredNestedYAMLBlock(state: YamlParserState, indent: number): unknown {
  const nextLine = state.lines[state.index];
  if (!nextLine || nextLine.indent < indent) {
    return null;
  }
  return parseYAMLBlock(state, indent);
}

function splitYAMLKeyValue(content: string): { key: string; value: string } | null {
  let inSingleQuote = false;
  let inDoubleQuote = false;
  let bracketDepth = 0;
  let braceDepth = 0;

  for (let index = 0; index < content.length; index += 1) {
    const character = content[index];
    const previousCharacter = index > 0 ? content[index - 1] : "";

    if (character === "'" && !inDoubleQuote) {
      inSingleQuote = !inSingleQuote;
      continue;
    }

    if (character === '"' && previousCharacter !== "\\" && !inSingleQuote) {
      inDoubleQuote = !inDoubleQuote;
      continue;
    }

    if (inSingleQuote || inDoubleQuote) {
      continue;
    }

    if (character === "[") bracketDepth += 1;
    else if (character === "]") bracketDepth -= 1;
    else if (character === "{") braceDepth += 1;
    else if (character === "}") braceDepth -= 1;

    if (character === ":" && bracketDepth === 0 && braceDepth === 0) {
      const nextCharacter = index + 1 < content.length ? content[index + 1] : "";
      if (nextCharacter === "" || /\s/u.test(nextCharacter)) {
        const key = content.slice(0, index).trim();
        const value = content.slice(index + 1).trimStart();
        return key !== "" ? { key, value } : null;
      }
    }
  }

  return null;
}

function parseYAMLScalarOrFlow(value: string): unknown {
  const trimmedValue = value.trim();
  if (trimmedValue === "") {
    return "";
  }

  if ((trimmedValue.startsWith("[") && trimmedValue.endsWith("]")) || (trimmedValue.startsWith("{") && trimmedValue.endsWith("}"))) {
    const jsonLike = trimmedValue
      .replace(/([{,]\s*)([A-Za-z0-9_.-]+)\s*:/gu, '$1"$2":')
      .replace(/'([^']*)'/gu, (_, valuePart: string) => JSON.stringify(valuePart));
    try {
      return JSON.parse(jsonLike);
    } catch {
      // fall through to string parsing
    }
  }

  if ((trimmedValue.startsWith('"') && trimmedValue.endsWith('"')) || (trimmedValue.startsWith("'") && trimmedValue.endsWith("'"))) {
    return parseQuotedYAMLScalar(trimmedValue);
  }

  if (trimmedValue === "true") return true;
  if (trimmedValue === "false") return false;
  if (trimmedValue === "null" || trimmedValue === "~") return null;
  if (/^-?(?:0|[1-9]\d*)(?:\.\d+)?$/u.test(trimmedValue)) {
    return Number(trimmedValue);
  }

  return trimmedValue;
}

function parseQuotedYAMLScalar(value: string): string {
  if (value.startsWith('"')) {
    return JSON.parse(value);
  }

  const innerValue = value.slice(1, -1);
  return innerValue.replace(/''/gu, "'");
}

function pushUnique(values: string[], value: string): void {
  if (!values.includes(value)) {
    values.push(value);
  }
}

function formatSummaryList(values: readonly string[]): string {
  return values.length === 0 ? "none" : values.join(", ");
}

function getScopeSummaryLabel(scope: MigrationScope): string {
  return scope.label ?? "<root>";
}

function recordSpecialPluginMigration(targetScopes: string[], scope: MigrationScope): void {
  pushUnique(targetScopes, getScopeSummaryLabel(scope));
}

function printMigrationSummary(report: MigrationReport): void {
  const warningsCount = report.warnings.length;
  const warningSummary = warningsCount === 0
    ? "Migration completed without warnings."
    : `Migration completed with ${warningsCount} warning(s). Review the generated .oxfmtrc.json for approximated or skipped settings.`;

  console.error(warningSummary);
  console.error("Migration summary:");
  console.error(`  - warnings: ${warningsCount}`);
  console.error(`  - overrides migrated: ${report.migratedOverrideCount}`);

  if (report.skippedOverrideCount > 0) {
    console.error(`  - overrides skipped: ${report.skippedOverrideCount}`);
  }
  if (report.preservedPluginSpecs.length > 0) {
    console.error(`  - preserved formatter plugins: ${formatSummaryList(report.preservedPluginSpecs)}`);
  }
  if (report.inferredPluginSpecs.length > 0) {
    console.error(`  - plugin objects converted to string specs: ${formatSummaryList(report.inferredPluginSpecs)}`);
  }
  if (report.migratedTailwindPluginScopes.length > 0) {
    console.error(
      `  - migrated prettier-plugin-tailwindcss in: ${formatSummaryList(report.migratedTailwindPluginScopes)}`,
    );
  }
  if (report.migratedPackageJsonPluginScopes.length > 0) {
    console.error(
      `  - migrated prettier-plugin-packagejson in: ${formatSummaryList(report.migratedPackageJsonPluginScopes)}`,
    );
  }
  if (report.skippedCustomPluginObjects > 0) {
    console.error(`  - unsupported custom plugin objects skipped: ${report.skippedCustomPluginObjects}`);
  }
}

function warnMigration(scope: MigrationScope, message: string): void {
  currentMigrationReport.warnings.push({ label: scope.label, message });
  const label = scope.label ? `${scope.label}: ` : "";
  console.error(`  - ${label}${message}`);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

// https://github.com/fabiospampinato/tiny-jsonc/blob/bb722089210174ec9cb53afcce15245e7ee21b9a/src/index.ts
const stringOrCommentRe = /("(?:\\?[^])*?")|(\/\/.*)|(\/\*[^]*?\*\/)/g;
const stringOrTrailingCommaRe = /("(?:\\?[^])*?")|(,\s*)(?=]|})/g;
function parseJSONC(text: string): unknown {
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

const TAILWIND_OPTION_MAPPING: Record<string, string> = {
  config: "tailwindConfig",
  stylesheet: "tailwindStylesheet",
  functions: "tailwindFunctions",
  attributes: "tailwindAttributes",
  preserveWhitespace: "tailwindPreserveWhitespace",
  preserveDuplicates: "tailwindPreserveDuplicates",
};

/**
 * Migrate prettier-plugin-tailwindcss options to Oxfmt's sortTailwindcss format.
 *
 * Prettier format:
 * ```json
 * {
 *   "plugins": ["prettier-plugin-tailwindcss"],
 *   "tailwindConfig": "./tailwind.config.js",
 *   "tailwindFunctions": ["clsx", "cn"]
 * }
 * ```
 *
 * Oxfmt format:
 * ```json
 * {
 *   "sortTailwindcss": {
 *     "config": "./tailwind.config.js",
 *     "functions": ["clsx", "cn"]
 *   }
 * }
 * ```
 */
function migrateTailwindOptions(
  prettierConfig: Record<string, unknown>,
  oxfmtrc: Record<string, unknown>,
  scope: MigrationScope,
): void {
  // Collect Tailwind options from Prettier config
  const tailwindOptions: Record<string, unknown> = {};
  for (const [oxfmtKey, prettierKey] of Object.entries(TAILWIND_OPTION_MAPPING)) {
    const value = prettierConfig[prettierKey];
    if (value !== undefined) {
      if (
        (prettierKey == "tailwindFunctions" || prettierKey == "tailwindAttributes") &&
        Array.isArray(value)
      ) {
        for (const item of value as string[]) {
          if (typeof item === "string" && item.startsWith("/") && item.endsWith("/")) {
            console.warn(
              `  - ${scope.label ? `${scope.label}: ` : ""}Do not support regex in "${prettierKey}" option yet, skipping: ${item}`,
            );
            continue;
          }
        }
      }
      tailwindOptions[oxfmtKey] = value;
    }
  }

  // Only add sortTailwindcss if plugin is used or options are present
  oxfmtrc.sortTailwindcss = tailwindOptions;
  console.log(
    `${scope.label ? `Migrated ${scope.label} ` : "Migrated "}prettier-plugin-tailwindcss options to sortTailwindcss`,
  );
}
