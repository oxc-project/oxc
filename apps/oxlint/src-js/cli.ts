import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.ts";

// Lazy-loaded JS plugin-related functions.
// Using `typeof wrapper` here makes TS check that the function signatures of `loadPlugin` and `loadPluginWrapper`
// are identical. Ditto `lintFile` and `lintFileWrapper`.
let loadPlugin: typeof loadPluginWrapper | null = null;
let loadParser: typeof loadParserWrapper | null = null;
let parseFile: typeof parseFileWrapper | null = null;
let stripFile: typeof stripFileWrapper | null = null;
let setupConfigs: typeof setupConfigsWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;
let lintFileWithCustomAst: typeof lintFileWithCustomAstWrapper | null = null;

/**
 * Load a plugin.
 *
 * Lazy-loads plugins code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param path - Absolute path of plugin file
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @returns Plugin details or error serialized to JSON string
 */
function loadPluginWrapper(
  path: string,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
): Promise<string> {
  if (loadPlugin === null) {
    // Use promises here instead of making `loadPluginWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `loadPluginWrapper`
    return import("./plugins/index.ts").then((mod) => {
      ({
        loadPlugin,
        loadParser,
        parseFile,
        stripFile,
        lintFile,
        lintFileWithCustomAst,
        setupConfigs,
      } = mod);
      return loadPlugin(path, pluginName, pluginNameIsAlias);
    });
  }
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(path, pluginName, pluginNameIsAlias);
}

/**
 * Bootstrap configuration options.
 *
 * Delegates to `setupConfigs`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 * @returns `null` if success, or error message string
 */
function setupConfigsWrapper(optionsJSON: string): string | null {
  debugAssertIsNonNull(setupConfigs);
  return setupConfigs(optionsJSON);
}

/**
 * Lint a file.
 *
 * Delegates to `lintFile`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for file, as JSON
 * @param globalsJSON - Globals for file, as JSON
 * @returns Diagnostics or error serialized to JSON string
 */
function lintFileWrapper(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
): string | null {
  // `lintFileWrapper` is never called without `loadPluginWrapper` being called first,
  // so `lintFile` must be defined here
  debugAssertIsNonNull(lintFile);
  return lintFile(filePath, bufferId, buffer, ruleIds, optionsIds, settingsJSON, globalsJSON);
}

/**
 * Load a custom parser.
 *
 * Lazy-loads parser code on first call, so that overhead is skipped if user doesn't use custom parsers.
 *
 * @param url - Absolute path of parser file as a `file://...` URL
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Parser details or error serialized to JSON string
 */
function loadParserWrapper(url: string, parserOptionsJson: string): Promise<string> {
  if (loadParser === null) {
    // Use promises here instead of making `loadParserWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `loadParserWrapper`
    return import("./plugins/index.ts").then((mod) => {
      ({
        loadPlugin,
        loadParser,
        parseFile,
        stripFile,
        lintFile,
        lintFileWithCustomAst,
        setupConfigs,
      } = mod);
      return loadParser(url, parserOptionsJson);
    });
  }
  debugAssertIsNonNull(loadParser);
  return loadParser(url, parserOptionsJson);
}

/**
 * Lint a file with a pre-parsed AST from a custom parser.
 *
 * Delegates to `lintFileWithCustomAst`, which was lazy-loaded by `loadParserWrapper` or `loadPluginWrapper`.
 *
 * @param filePath - Absolute path of file being linted
 * @param sourceText - Source text of the file
 * @param astJson - Pre-parsed AST as JSON string
 * @param ruleIds - IDs of rules to run on this file
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for file, as JSON
 * @param globalsJSON - Globals for file, as JSON
 * @param parserServicesJson - Parser services from parseForESLint, as JSON string (or "null")
 * @returns Diagnostics or error serialized to JSON string
 */
function lintFileWithCustomAstWrapper(
  filePath: string,
  sourceText: string,
  astJson: string,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  parserServicesJson: string,
): string | null {
  // `lintFileWithCustomAstWrapper` is never called without `loadParserWrapper` or `loadPluginWrapper` being called first,
  // so `lintFileWithCustomAst` must be defined here
  debugAssertIsNonNull(lintFileWithCustomAst);
  return lintFileWithCustomAst(
    filePath,
    sourceText,
    astJson,
    ruleIds,
    optionsIds,
    settingsJSON,
    globalsJSON,
    parserServicesJson,
  );
}

/**
 * Parse a file with a custom parser.
 *
 * Delegates to `parseFile`, which was lazy-loaded by `loadParserWrapper` or `loadPluginWrapper`.
 *
 * @param parserId - ID of the parser to use
 * @param filePath - Absolute path of file being parsed
 * @param sourceText - Source text content
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Parse result or error serialized to JSON string
 */
function parseFileWrapper(
  parserId: number,
  filePath: string,
  sourceText: string,
  parserOptionsJson: string,
): string {
  // `parseFileWrapper` is never called without `loadParserWrapper` being called first,
  // so `parseFile` must be defined here
  debugAssertIsNonNull(parseFile);
  return parseFile(parserId, filePath, sourceText, parserOptionsJson);
}

/**
 * Strip custom syntax from a file using a custom parser.
 *
 * Delegates to `stripFile`, which was lazy-loaded by `loadParserWrapper` or `loadPluginWrapper`.
 *
 * @param parserId - ID of the parser to use
 * @param filePath - Absolute path of file being stripped
 * @param sourceText - Source text content
 * @param parserOptionsJson - Parser options as JSON string
 * @returns Strip result as JSON string, or null if not supported
 */
function stripFileWrapper(
  parserId: number,
  filePath: string,
  sourceText: string,
  parserOptionsJson: string,
): string | null {
  // `stripFileWrapper` is never called without `loadParserWrapper` being called first,
  // so `stripFile` must be defined here
  debugAssertIsNonNull(stripFile);
  return stripFile(parserId, filePath, sourceText, parserOptionsJson);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing callbacks and CLI arguments
// Parser callbacks (loadParserWrapper, parseFileWrapper, lintFileWithCustomAstWrapper, stripFileWrapper)
// are optional but we provide them to enable custom parser support when configured
// via jsParsers in oxlintrc.json
const success = await lint(
  args,
  loadPluginWrapper,
  setupConfigsWrapper,
  lintFileWrapper,
  loadParserWrapper,
  parseFileWrapper,
  lintFileWithCustomAstWrapper,
  stripFileWrapper,
);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
