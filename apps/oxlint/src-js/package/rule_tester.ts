/*
 * `RuleTester` class.
 *
 * Heavily based on ESLint's `RuleTester`, but without the complications of configs.
 * Has the same user-facing API as ESLint's version.
 * Code: https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/rule-tester/rule-tester.js
 * License (MIT): https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/LICENSE
 */

import { default as assert, AssertionError } from "node:assert";
import util from "node:util";
import stringify from "json-stable-stringify-without-jsonify";
import { registerPlugin, registeredRules } from "../plugins/load.ts";
import { lintFileImpl, resetFile } from "../plugins/lint.ts";
import { getLineColumnFromOffset, getNodeByRangeIndex } from "../plugins/location.ts";
import {
  allOptions,
  initAllOptions,
  mergeOptions,
  DEFAULT_OPTIONS_ID,
} from "../plugins/options.ts";
import { diagnostics, replacePlaceholders, PLACEHOLDER_REGEX } from "../plugins/report.ts";
import { parse } from "./parse.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { RequireAtLeastOne } from "type-fest";
import type { Plugin, Rule } from "../plugins/load.ts";
import type { Options } from "../plugins/options.ts";
import type { DiagnosticData, Suggestion } from "../plugins/report.ts";
import type { ParseOptions } from "./parse.ts";

const { hasOwn } = Object,
  { isArray } = Array;

// ------------------------------------------------------------------------------
// `describe` and `it` functions
// ------------------------------------------------------------------------------

type DescribeFn = (text: string, fn: () => void) => void;
type ItFn = ((text: string, fn: () => void) => void) & { only?: ItFn };

/**
 * Default `describe` function, if `describe` doesn't exist as a global.
 * @param text - Description of the test case
 * @param method - Test case logic
 * @returns Returned value of `method`
 */
function defaultDescribe<R>(text: string, method: () => R): R {
  return method.call(this);
}

const globalObj = globalThis as { describe?: DescribeFn; it?: ItFn };

// `describe` function. Can be overwritten via `RuleTester.describe` setter.
let describe: DescribeFn =
  typeof globalObj.describe === "function" ? globalObj.describe : defaultDescribe;

/**
 * Default `it` function, if `it` doesn't exist as a global.
 * @param text - Description of the test case
 * @param method - Test case logic
 * @throws {Error} Any error upon execution of `method`
 * @returns Returned value of `method`
 */
function defaultIt<R>(text: string, method: () => R): R {
  try {
    return method.call(this);
  } catch (err) {
    if (err instanceof AssertionError) {
      err.message += ` (${util.inspect(err.actual)} ${err.operator} ${util.inspect(err.expected)})`;
    }
    throw err;
  }
}

// `it` function. Can be overwritten via `RuleTester.it` setter.
let it: ItFn = typeof globalObj.it === "function" ? globalObj.it : defaultIt;

// `it.only` function. Can be overwritten via `RuleTester.it` or `RuleTester.itOnly` setters.
let itOnly: ItFn | null =
  it !== defaultIt && typeof it.only === "function" ? Function.bind.call(it.only, it) : null;

/**
 * Get `it` function.
 * @param only - `true` if `it.only` should be used
 * @throws {Error} If `it.only` is not available
 * @returns `it` or `it.only` function
 */
function getIt(only?: boolean): ItFn {
  return only ? getItOnly() : it;
}

/**
 * Get `it.only` function.
 * @throws {Error} If `it.only` is not available
 * @returns `it.only` function
 */
function getItOnly(): ItFn {
  if (itOnly === null) {
    throw new Error(
      "To use `only`, use `RuleTester` with a test framework that provides `it.only()` like Mocha, " +
        "or provide a custom `it.only` function by assigning it to `RuleTester.itOnly`",
    );
  }
  return itOnly;
}

// ------------------------------------------------------------------------------
// Config
// ------------------------------------------------------------------------------

/**
 * Configuration for `RuleTester`.
 */
interface Config {
  /**
   * ESLint compatibility mode.
   * If `true`, column offsets in diagnostics are incremented by 1, to match ESLint's behavior.
   */
  eslintCompat?: boolean;
  languageOptions?: LanguageOptions;
}

/**
 * Language options config.
 */
interface LanguageOptions {
  ecmaVersion?: number | "latest";
  sourceType?: SourceType;
  globals?: Record<
    string,
    boolean | "true" | "writable" | "writeable" | "false" | "readonly" | "readable" | "off" | null
  >;
  parser?: {
    parse?: (code: string, options?: Record<string, unknown>) => unknown;
    parseForESLint?: (code: string, options?: Record<string, unknown>) => unknown;
  };
  parserOptions?: ParserOptions;
}

/**
 * Source type.
 *
 * - `'unambiguous'` is not supported in ESLint compatibility mode.
 * - `'commonjs'` is only supported in ESLint compatibility mode.
 */
type SourceType = "script" | "module" | "unambiguous" | "commonjs";

/**
 * Parser options config.
 */
interface ParserOptions {
  ecmaFeatures?: EcmaFeatures;
  /**
   * Language variant to parse file as.
   */
  lang?: Language;
  /**
   * `true` to ignore non-fatal parsing errors.
   */
  ignoreNonFatalErrors?: boolean;
}

/**
 * ECMA features config.
 */
interface EcmaFeatures {
  /**
   * `true` to enable JSX parsing.
   *
   * `parserOptions.lang` takes priority over this option, if `lang` is specified.
   */
  jsx?: boolean;
}

/**
 * Parser language.
 */
type Language = "js" | "jsx" | "ts" | "tsx" | "dts";

// `RuleTester` uses this config as its default. Can be overwritten via `RuleTester.setDefaultConfig()`.
let sharedConfig: Config = {};

// ------------------------------------------------------------------------------
// Test cases
// ------------------------------------------------------------------------------

// List of keys that `ValidTestCase` or `InvalidTestCase` can have.
// Must be kept in sync with properties of `ValidTestCase` and `InvalidTestCase` interfaces.
const TEST_CASE_PROP_KEYS = new Set([
  "code",
  "name",
  "only",
  "filename",
  "options",
  "before",
  "after",
  "output",
  "errors",
  // Not a valid key for `TestCase` interface, but present here to prevent prototype pollution in `createConfigForRun`
  "__proto__",
]);

/**
 * Test case.
 */
interface TestCase extends Config {
  code: string;
  name?: string;
  only?: boolean;
  filename?: string;
  options?: Options;
  before?: (this: this) => void;
  after?: (this: this) => void;
}

/**
 * Test case for valid code.
 */
interface ValidTestCase extends TestCase {}

/**
 * Test case for invalid code.
 */
interface InvalidTestCase extends TestCase {
  output?: string | null;
  errors: number | ErrorEntry[];
}

type ErrorEntry = Error | string | RegExp;

/**
 * Expected error.
 */
type Error = RequireAtLeastOne<ErrorBase, "message" | "messageId">;

interface ErrorBase {
  message?: string | RegExp;
  messageId?: string;
  data?: DiagnosticData;
  line?: number;
  column?: number;
  endLine?: number;
  endColumn?: number;
}

/**
 * Test cases for a rule.
 */
interface TestCases {
  valid: (ValidTestCase | string)[];
  invalid: InvalidTestCase[];
}

/**
 * Diagnostic included in assertion errors.
 *
 * This matches what ESLint's includes in errors it emits.
 * `severity` field is pointless, as it's always `1`, but ESLint includes it, so we do too.
 */
interface Diagnostic {
  ruleId: string;
  message: string;
  messageId: string | null;
  severity: 1;
  nodeType: string | null;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  suggestions: Suggestion[] | null;
}

// Default path (without extension) for test cases if not provided
const DEFAULT_FILENAME_BASE = "file";

// ------------------------------------------------------------------------------
// `RuleTester` class
// ------------------------------------------------------------------------------

/**
 * Utility class for testing rules.
 */
export class RuleTester {
  #config: Config | null;

  /**
   * Creates a new instance of RuleTester.
   * @param config? - Extra configuration for the tester (optional)
   */
  constructor(config?: Config | null) {
    if (config === undefined) {
      config = null;
    } else if (config !== null && typeof config !== "object") {
      throw new TypeError("`config` must be an object if provided");
    }

    this.#config = config;
  }

  /**
   * Set the configuration to use for all future tests.
   * @param config - The configuration to use
   * @throws {TypeError} If `config` is not an object
   */
  static setDefaultConfig(config: Config): void {
    if (typeof config !== "object" || config === null) {
      throw new TypeError("`config` must be an object");
    }
    sharedConfig = config;
  }

  /**
   * Get the current configuration used for all tests.
   * @returns The current configuration
   */
  static getDefaultConfig(): Config {
    return sharedConfig;
  }

  /**
   * Reset the configuration to the initial configuration of the tester removing
   * any changes made until now.
   * @returns {void}
   */
  static resetDefaultConfig() {
    sharedConfig = {};
  }

  // Getters/setters for `describe` and `it` functions

  static get describe(): DescribeFn {
    return describe;
  }

  static set describe(value: DescribeFn) {
    describe = value;
  }

  static get it(): ItFn {
    return it;
  }

  static set it(value: ItFn) {
    it = value;
    if (typeof it.only === "function") {
      itOnly = Function.bind.call(it.only, it);
    } else {
      itOnly = null;
    }
  }

  static get itOnly(): ItFn {
    return getItOnly();
  }

  static set itOnly(value: ItFn) {
    itOnly = value;
  }

  /**
   * Add the `only` property to a test to run it in isolation.
   * @param item - A single test to run by itself
   * @returns The test with `only` set
   */
  static only(item: string | TestCase): TestCase {
    if (typeof item === "string") return { code: item, only: true };
    return { ...item, only: true };
  }

  /**
   * Adds a new rule test to execute.
   * @param ruleName - Name of the rule to run
   * @param rule - Rule to test
   * @param tests - Collection of tests to run
   * @throws {TypeError|Error} If `rule` is not an object with a `create` method,
   *   or if non-object `test`, or if a required scenario of the given type is missing
   */
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    // Create plugin for the rule
    const plugin: Plugin = {
      meta: { name: "rule-to-test" },
      rules: { [ruleName]: rule },
    };

    const config = createConfigForRun(this.#config);

    describe(ruleName, () => {
      if (tests.valid.length > 0) {
        describe("valid", () => {
          const seenTestCases = new Set<string>();
          for (let test of tests.valid) {
            if (typeof test === "string") test = { code: test };

            const it = getIt(test.only);
            it(getTestName(test), () => {
              runValidTestCase(test, plugin, config, seenTestCases);
            });
          }
        });
      }

      if (tests.invalid.length > 0) {
        describe("invalid", () => {
          const seenTestCases = new Set<string>();
          for (const test of tests.invalid) {
            const it = getIt(test.only);
            it(getTestName(test), () => {
              runInvalidTestCase(test, plugin, config, seenTestCases);
            });
          }
        });
      }
    });
  }
}

// In debug builds only, we provide a hook to modify test cases before they're run.
// Hook can be registered by calling `RuleTester.registerModifyTestCaseHook`.
// This is used in conformance tester.
let modifyTestCase: ((test: TestCase) => void) | null = null;

if (DEBUG) {
  (RuleTester as any).registerModifyTestCaseHook = (alter: (test: TestCase) => void) => {
    modifyTestCase = alter;
  };
}

/**
 * Run valid test case.
 * @param test - Valid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case fails
 */
function runValidTestCase(
  test: ValidTestCase,
  plugin: Plugin,
  config: Config,
  seenTestCases: Set<string>,
): void {
  try {
    runBeforeHook(test);
    assertValidTestCaseIsWellFormed(test, seenTestCases);
    assertValidTestCasePasses(test, plugin, config);
  } finally {
    runAfterHook(test);
  }
}

/**
 * Assert that valid test case passes.
 * @param test - Valid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @throws {AssertionError} If the test case fails
 */
function assertValidTestCasePasses(test: ValidTestCase, plugin: Plugin, config: Config): void {
  test = mergeConfigIntoTestCase(test, config);

  const diagnostics = lint(test, plugin);
  assertErrorCountIsCorrect(diagnostics, 0);
}

/**
 * Run invalid test case.
 * @param test - Invalid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case fails
 */
function runInvalidTestCase(
  test: InvalidTestCase,
  plugin: Plugin,
  config: Config,
  seenTestCases: Set<string>,
): void {
  const ruleName = Object.keys(plugin.rules)[0];
  try {
    runBeforeHook(test);
    assertInvalidTestCaseIsWellFormed(test, seenTestCases, ruleName);
    assertInvalidTestCasePasses(test, plugin, config);
  } finally {
    runAfterHook(test);
  }
}

/**
 * Assert that invalid test case passes.
 * @param test - Invalid test case
 * @param plugin - Plugin containing rule being tested
 * @param config - Config from `RuleTester` instance
 * @throws {AssertionError} If the test case fails
 */
function assertInvalidTestCasePasses(test: InvalidTestCase, plugin: Plugin, config: Config): void {
  test = mergeConfigIntoTestCase(test, config);

  const diagnostics = lint(test, plugin);

  const { errors } = test;
  if (typeof errors === "number") {
    // If `errors` is a number, it's expected error count
    assertErrorCountIsCorrect(diagnostics, errors);
  } else {
    // `errors` is an array of error objects
    assertErrorCountIsCorrect(diagnostics, errors.length);

    const rule = Object.values(plugin.rules)[0],
      messages = rule.meta?.messages ?? null;

    for (let errorIndex = 0; errorIndex < errors.length; errorIndex++) {
      const error: ErrorEntry = errors[errorIndex]!,
        diagnostic = diagnostics[errorIndex]!;
      if (typeof error === "string" || error instanceof RegExp) {
        // `error` is a string or `RegExp` - it's expected message
        assertMessageMatches(diagnostic.message, error);
        assert(
          diagnostic.suggestions === null,
          `Error at index ${errorIndex} has suggestions. Please convert the test error into an object ` +
            "and specify `suggestions` property on it to test suggestions",
        );
      } else {
        // `error` is an error object
        assertInvalidTestCaseMessageIsCorrect(diagnostic, error, messages);
        assertInvalidTestCaseLocationIsCorrect(diagnostic, error, test);

        // TODO: Test suggestions
      }
    }
  }

  // TODO: Test output after fixes
}

/**
 * Assert that message reported by rule under test matches the expected message.
 * @param diagnostic - Diagnostic emitted by rule under test
 * @param error - Error object from test case
 * @param messages - Messages from rule under test
 * @throws {AssertionError} If `message` / `messageId` is not correct
 */
function assertInvalidTestCaseMessageIsCorrect(
  diagnostic: Diagnostic,
  error: Error,
  messages: Record<string, string> | null,
): void {
  // Check `message` property
  if (hasOwn(error, "message")) {
    // Check `message` property
    assert(
      !hasOwn(error, "messageId"),
      "Error should not specify both `message` and a `messageId`",
    );
    assert(!hasOwn(error, "data"), "Error should not specify both `data` and `message`");
    assertMessageMatches(diagnostic.message, error.message!);
    return;
  }

  assert(hasOwn(error, "messageId"), "Test error must specify either a `messageId` or `message`");

  // Check `messageId` property
  assert(
    messages !== null,
    "Error can not use `messageId` if rule under test doesn't define `meta.messages`",
  );

  const messageId: string = error.messageId!;
  if (!hasOwn(messages, messageId)) {
    const legalMessageIds = `[${Object.keys(messages)
      .map((key) => `'${key}'`)
      .join(", ")}]`;
    assert.fail(`Invalid messageId '${messageId}'. Expected one of ${legalMessageIds}`);
  }

  assert.strictEqual(
    diagnostic.messageId,
    messageId,
    `messageId '${diagnostic.messageId}' does not match expected messageId '${messageId}'`,
  );

  const reportedMessage = diagnostic.message;
  const ruleMessage = messages[messageId];

  const unsubstitutedPlaceholders = getUnsubstitutedMessagePlaceholders(
    reportedMessage,
    ruleMessage,
    error.data,
  );
  if (unsubstitutedPlaceholders.length !== 0) {
    assert.fail(
      "The reported message has " +
        (unsubstitutedPlaceholders.length > 1
          ? `unsubstituted placeholders: ${unsubstitutedPlaceholders.map((name) => `'${name}'`).join(", ")}`
          : `an unsubstituted placeholder '${unsubstitutedPlaceholders[0]}'`) +
        `. Please provide the missing ${unsubstitutedPlaceholders.length > 1 ? "values" : "value"} ` +
        "via the `data` property on the error object.",
    );
  }

  if (hasOwn(error, "data")) {
    // If data was provided, then directly compare the returned message to a synthetic
    // interpolated message using the same message ID and data provided in the test
    const rehydratedMessage = replacePlaceholders(ruleMessage, error.data!);

    assert.strictEqual(
      reportedMessage,
      rehydratedMessage,
      `Hydrated message "${rehydratedMessage}" does not match "${reportedMessage}"`,
    );
  }
}

/**
 * Assert that location reported by rule under test matches the expected location.
 * @param diagnostic - Diagnostic emitted by rule under test
 * @param error - Error object from test case
 * @param config - Config for this test case
 * @throws {AssertionError} If diagnostic's location does not match expected location
 */
function assertInvalidTestCaseLocationIsCorrect(
  diagnostic: Diagnostic,
  error: Error,
  test: TestCase,
) {
  interface Location {
    line?: number;
    column?: number;
    endLine?: number;
    endColumn?: number;
  }

  const actualLocation: Location = {};
  const expectedLocation: Location = {};

  const columnOffset = test.eslintCompat === true ? 1 : 0;

  if (hasOwn(error, "line")) {
    actualLocation.line = diagnostic.line;
    expectedLocation.line = error.line;
  }

  if (hasOwn(error, "column")) {
    actualLocation.column = diagnostic.column + columnOffset;
    expectedLocation.column = error.column;
  }

  if (hasOwn(error, "endLine")) {
    actualLocation.endLine = diagnostic.endLine;
    expectedLocation.endLine = error.endLine;
  }

  if (hasOwn(error, "endColumn")) {
    actualLocation.endColumn = diagnostic.endColumn + columnOffset;
    expectedLocation.endColumn = error.endColumn;
  }

  if (Object.keys(expectedLocation).length > 0) {
    assert.deepStrictEqual(
      actualLocation,
      expectedLocation,
      "Actual error location does not match expected error location.",
    );
  }
}

/**
 * Assert that the number of errors reported for test case is as expected.
 * @param diagnostics - Diagnostics reported by the rule under test
 * @param expectedErrorCount - Expected number of diagnistics
 * @throws {AssertionError} If the number of diagnostics is not as expected
 */
function assertErrorCountIsCorrect(diagnostics: Diagnostic[], expectedErrorCount: number): void {
  if (diagnostics.length === expectedErrorCount) return;

  assert.strictEqual(
    diagnostics.length,
    expectedErrorCount,
    util.format(
      "Should have %s error%s but had %d: %s",
      expectedErrorCount === 0 ? "no" : expectedErrorCount,
      expectedErrorCount === 1 ? "" : "s",
      diagnostics.length,
      util.inspect(diagnostics),
    ),
  );
}

/**
 * Assert that message is matched by matcher.
 * Matcher can be a string or a regular expression.
 * @param message - Message
 * @param matcher - Matcher
 * @throws {AssertionError} If message does not match
 */
function assertMessageMatches(message: string, matcher: string | RegExp) {
  if (typeof matcher === "string") {
    assert.strictEqual(message, matcher);
  } else {
    assert(matcher.test(message), `Expected '${message}' to match ${matcher}`);
  }
}

/**
 * Get placeholders in the reported messages but only includes the placeholders available in the raw message
 * and not in the provided data.
 * @param message - Reported message
 * @param raw - Raw message specified in the rule's `meta.messages`
 * @param data - Data from the test case's error object
 * @returns Missing placeholder names
 */
function getUnsubstitutedMessagePlaceholders(
  message: string,
  raw: string,
  data?: DiagnosticData,
): string[] {
  const unsubstituted = getMessagePlaceholders(message);
  if (unsubstituted.length === 0) return [];

  // Remove false positives by only counting placeholders in the raw message,
  // which were not provided in the data matcher or added with a data property
  const known = getMessagePlaceholders(raw);
  const provided = data === undefined ? [] : Object.keys(data);
  return unsubstituted.filter((name) => known.includes(name) && !provided.includes(name));
}

/**
 * Extract names of `{{ name }}` placeholders from a message.
 * @param message - Message
 * @returns Array of placeholder names
 */
function getMessagePlaceholders(message: string): string[] {
  return Array.from(message.matchAll(PLACEHOLDER_REGEX), ([, name]) => name.trim());
}

// In debug builds, wrap `runValidTestCase` and `runInvalidTestCase` to add test case to error object.
// This is used in conformance tests.
type RunFunction<T> = (test: T, plugin: Plugin, config: Config, seenTestCases: Set<string>) => void;

function wrapRunTestCaseFunction<T extends ValidTestCase | InvalidTestCase>(
  run: RunFunction<T>,
): RunFunction<T> {
  return function (test, plugin, config, seenTestCases) {
    try {
      run(test, plugin, config, seenTestCases);
    } catch (err) {
      // oxlint-disable-next-line no-ex-assign
      if (typeof err !== "object" || err === null) err = new Error("Unknown error");
      err.__testCase = test;
      throw err;
    }
  };
}

if (DEBUG) {
  // oxlint-disable-next-line no-func-assign
  (runValidTestCase as any) = wrapRunTestCaseFunction(runValidTestCase);
  // oxlint-disable-next-line no-func-assign
  (runInvalidTestCase as any) = wrapRunTestCaseFunction(runInvalidTestCase);
}

/**
 * Create config for a test run.
 * Merges config from `RuleTester` instance on top of shared config.
 * Removes properties which are not allowed in `Config`s, as they can only be properties of `TestCase`.
 *
 * @param config - Config from `RuleTester` instance
 * @returns Merged config
 */
function createConfigForRun(config: Config | null): Config {
  const merged: Config = {};
  addConfigPropsFrom(sharedConfig, merged);
  if (config !== null) addConfigPropsFrom(config, merged);
  return merged;
}

function addConfigPropsFrom(config: Config, merged: Config): void {
  // Note: `TEST_CASE_PROP_KEYS` includes `"__proto__"`, so using assignment `merged[key] = ...`
  // cannot set prototype of `merged`, instead of setting a property
  for (const key of Object.keys(config) as (keyof Config)[]) {
    if (TEST_CASE_PROP_KEYS.has(key)) continue;
    if (key === "languageOptions") {
      merged.languageOptions = mergeLanguageOptions(config.languageOptions, merged.languageOptions);
    } else {
      (merged as Record<string, unknown>)[key] = config[key];
    }
  }
}

/**
 * Create config for a test case.
 * Merges properties of test case on top of config from `RuleTester` instance.
 *
 * @param test - Test case
 * @param config - Config from `RuleTester` instance / shared config
 * @returns Merged config
 */
function mergeConfigIntoTestCase<T extends ValidTestCase | InvalidTestCase>(
  test: T,
  config: Config,
): T {
  // `config` has already been cleansed of properties which are exclusive to `TestCase`,
  // so no danger here of `config` having a property called e.g. `errors` which would affect the test case
  const merged = {
    ...config,
    ...test,
    languageOptions: mergeLanguageOptions(test.languageOptions, config.languageOptions),
  };

  // Call hook to modify test case before it is run.
  // `modifyTestCase` is only available in debug builds - it's only for conformance testing.
  if (DEBUG && modifyTestCase !== null) modifyTestCase(merged);

  return merged;
}

/**
 * Merge language options from test case / config onto language options from base config.
 * @param localLanguageOptions - Language options from test case / config
 * @param baseLanguageOptions - Language options from base config
 * @returns Merged language options, or `undefined` if neither has language options
 */
function mergeLanguageOptions(
  localLanguageOptions?: LanguageOptions | null,
  baseLanguageOptions?: LanguageOptions | null,
): LanguageOptions | undefined {
  if (localLanguageOptions == null) return baseLanguageOptions ?? undefined;
  if (baseLanguageOptions == null) return localLanguageOptions;

  return {
    ...baseLanguageOptions,
    ...localLanguageOptions,
    parserOptions: mergeParserOptions(
      localLanguageOptions.parserOptions,
      baseLanguageOptions.parserOptions,
    ),
  };
}

/**
 * Merge parser options from test case / config onto language options from base config.
 * @param localParserOptions - Parser options from test case / config
 * @param baseParserOptions - Parser options from base config
 * @returns Merged parser options, or `undefined` if neither has parser options
 */
function mergeParserOptions(
  localParserOptions?: ParserOptions | null,
  baseParserOptions?: ParserOptions | null,
): ParserOptions | undefined {
  if (localParserOptions == null) return baseParserOptions ?? undefined;
  if (baseParserOptions == null) return localParserOptions;

  return {
    ...baseParserOptions,
    ...localParserOptions,
    ecmaFeatures: mergeEcmaFeatures(
      localParserOptions.ecmaFeatures,
      baseParserOptions.ecmaFeatures,
    ),
  };
}

/**
 * Merge ecma features from test case / config onto ecma features from base config.
 * @param localEcmaFeatures - Ecma features from test case / config
 * @param baseEcmaFeatures - Ecma features from base config
 * @returns Merged ecma features, or `undefined` if neither has ecma features
 */
function mergeEcmaFeatures(
  localEcmaFeatures?: EcmaFeatures | null,
  baseEcmaFeatures?: EcmaFeatures | null,
): EcmaFeatures | undefined {
  if (localEcmaFeatures == null) return baseEcmaFeatures ?? undefined;
  if (baseEcmaFeatures == null) return localEcmaFeatures;
  return { ...baseEcmaFeatures, ...localEcmaFeatures };
}

/**
 * Lint a test case.
 * @param test - Test case
 * @param plugin - Plugin containing rule being tested
 * @returns Array of diagnostics
 */
function lint(test: TestCase, plugin: Plugin): Diagnostic[] {
  // Get parse options
  const parseOptions = getParseOptions(test);

  // Determine filename.
  // If not provided, use default filename based on `parseOptions.lang`.
  let { filename } = test;
  if (filename == null) {
    let ext: string | undefined = parseOptions.lang;
    if (ext == null) {
      ext = "js";
    } else if (ext === "dts") {
      ext = "d.ts";
    }
    filename = `${DEFAULT_FILENAME_BASE}.${ext}`;
  }

  // Initialize `allOptions` if not already initialized
  if (allOptions === null) initAllOptions();
  debugAssertIsNonNull(allOptions);

  try {
    registerPlugin(plugin, null);

    // Get options.
    // * If no options provided, use default options for the rule with `optionsId: DEFAULT_OPTIONS_ID`.
    // * If options provided, merge them with default options for the rule.
    //   Push merged options to `allOptions`, and use `optionsId: 1` (the index within `allOptions`).
    debugAssert(allOptions.length === 1);

    let optionsId = DEFAULT_OPTIONS_ID;
    const testOptions = test.options;
    if (testOptions != null) {
      const { defaultOptions } = registeredRules[0];
      allOptions.push(mergeOptions(testOptions, defaultOptions));
      optionsId = 1;
    }

    // Parse file into buffer
    parse(filename, test.code, parseOptions);

    // Lint file.
    // Buffer is stored already, at index 0. No need to pass it.
    const settingsJSON = "{}"; // TODO
    const globalsJSON = "{}"; // TODO
    lintFileImpl(filename, 0, null, [0], [optionsId], settingsJSON, globalsJSON);

    // Return diagnostics
    const ruleId = `${plugin.meta!.name!}/${Object.keys(plugin.rules)[0]}`;

    return diagnostics.map((diagnostic) => {
      const { line, column } = getLineColumnFromOffset(diagnostic.start),
        { line: endLine, column: endColumn } = getLineColumnFromOffset(diagnostic.end);
      const node = getNodeByRangeIndex(diagnostic.start);
      return {
        ruleId,
        message: diagnostic.message,
        messageId: diagnostic.messageId,
        severity: 1,
        nodeType: node === null ? null : node.type,
        line,
        column,
        endLine,
        endColumn,
        suggestions: null, // TODO
      };
    });
  } finally {
    // Reset state
    registeredRules.length = 0;
    allOptions.length = 1;
    diagnostics.length = 0;
    resetFile();
  }
}

/**
 * Get parse options for a test case.
 * @param test - Test case
 * @returns Parse options
 */
function getParseOptions(test: TestCase): ParseOptions {
  const parseOptions: ParseOptions = {};

  const { languageOptions } = test;
  if (languageOptions != null) {
    // Handle `languageOptions.sourceType`
    let { sourceType } = languageOptions;
    if (sourceType != null) {
      if (test.eslintCompat === true) {
        // ESLint compatibility mode.
        // `unambiguous` is disallowed. Treat `commonjs` as `script`.
        if (sourceType === "commonjs") {
          sourceType = "script";
        } else if (sourceType === "unambiguous") {
          throw new Error(
            "'unambiguous' source type is not supported in ESLint compatibility mode.\n" +
              "Disable ESLint compatibility mode by setting `eslintCompat` to `false` in the config / test case.",
          );
        }
      } else {
        // Not ESLint compatibility mode.
        // `commonjs` is disallowed.
        if (sourceType === "commonjs") {
          throw new Error(
            "'commonjs' source type is only supported in ESLint compatibility mode.\n" +
              "Enable ESLint compatibility mode by setting `eslintCompat` to `true` in the config / test case.",
          );
        }
      }

      parseOptions.sourceType = sourceType;
    }

    // Handle `languageOptions.parserOptions`
    const { parserOptions } = languageOptions;
    if (parserOptions != null) {
      // Handle `parserOptions.ignoreNonFatalErrors`
      if (parserOptions.ignoreNonFatalErrors === true) parseOptions.ignoreNonFatalErrors = true;

      // Handle `parserOptions.lang`
      const { lang } = parserOptions;
      if (lang != null) {
        parseOptions.lang = lang;
      } else if (parserOptions.ecmaFeatures?.jsx === true) {
        parseOptions.lang = "jsx";
      }
    }
  }

  return parseOptions;
}

// Regex to match other control characters (except tab, newline, carriage return)
// eslint-disable-next-line no-control-regex
const CONTROL_CHAR_REGEX = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/gu;

/**
 * Get name of test case.
 * Control characters in name are replaced with `\u00xx` form.
 * @param test - Test case
 * @returns Name of test case
 */
function getTestName(test: TestCase): string {
  const name = test.name || test.code;

  if (typeof name !== "string") return "";

  return name.replace(
    CONTROL_CHAR_REGEX,
    (c) => `\\u${c.codePointAt(0)!.toString(16).padStart(4, "0")}`,
  );
}

/**
 * Runs before hook on the given test case.
 * @param test - Test to run the hook on
 * @throws {Error} - If the hook is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runBeforeHook(test: TestCase): void {
  if (hasOwn(test, "before")) runHook(test, test.before, "before");
}

/**
 * Runs after hook on the given test case.
 * @param test - Test to run the hook on
 * @throws {Error} - If the hook is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runAfterHook(test: TestCase): void {
  if (hasOwn(test, "after")) runHook(test, test.after, "after");
}

/**
 * Runs a hook on the given test case.
 * @param test - Test to run the hook on
 * @param hook - Hook function
 * @param name - Name of the hook
 * @throws {Error} - If the property is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runHook<T extends TestCase>(
  test: T,
  hook: ((this: T) => void) | undefined,
  name: string,
): void {
  assert.strictEqual(
    typeof hook,
    "function",
    `Optional test case property \`${name}\` must be a function`,
  );
  hook!.call(test);
}

/**
 * Assert that a valid test case object is valid.
 * A valid test case must specify a string value for `code`.
 * Optional properties are checked for correct types.
 *
 * @param test - Valid test case object to check
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @throws {AssertionError} If the test case is not valid
 */
function assertValidTestCaseIsWellFormed(test: ValidTestCase, seenTestCases: Set<string>): void {
  assertTestCaseCommonPropertiesAreWellFormed(test);

  // Must not have properties of invalid test cases
  assert(
    !("errors" in test) || test.errors === undefined,
    "Valid test case must not have `errors` property",
  );
  assert(
    !("output" in test) || test.output === undefined,
    "Valid test case must not have `output` property",
  );

  assertNotDuplicateTestCase(test, seenTestCases);
}

/**
 * Assert that an invalid test case object is valid.
 * An invalid test case must specify a string value for `code` and must have an `errors` property.
 * Optional properties are checked for correct types.
 *
 * @param test - Invalid test case object to check
 * @param seenTestCases - Set of serialized test cases to check for duplicates
 * @param ruleName - Name of the rule being tested
 * @throws {AssertionError} If the test case is not valid
 */
function assertInvalidTestCaseIsWellFormed(
  test: InvalidTestCase,
  seenTestCases: Set<string>,
  ruleName: string,
): void {
  assertTestCaseCommonPropertiesAreWellFormed(test);

  // `errors` must be a number greater than 0, or a non-empty array
  const { errors } = test;
  if (typeof errors === "number") {
    assert(errors > 0, "Invalid cases must have `errors` value greater than 0");
  } else {
    assert(
      errors !== undefined,
      `Did not specify errors for an invalid test of rule \`${ruleName}\``,
    );
    assert(
      isArray(errors),
      `Invalid 'errors' property for invalid test of rule \`${ruleName}\`:` +
        `expected a number or an array but got ${errors === null ? "null" : typeof errors}`,
    );
    assert(errors.length !== 0, "Invalid cases must have at least one error");
  }

  // `output` is optional, but if it exists it must be a string or `null`
  if (hasOwn(test, "output")) {
    assert(
      test.output === null || typeof test.output === "string",
      "Test property `output`, if specified, must be a string or null. " +
        "If no autofix is expected, then omit the `output` property or set it to null.",
    );
  }

  assertNotDuplicateTestCase(test, seenTestCases);
}

/**
 * Assert that the common properties of a valid/invalid test case have the correct types.
 * @param {Object} test - Test case object to check
 * @throws {AssertionError} If the test case is not valid
 */
function assertTestCaseCommonPropertiesAreWellFormed(test: TestCase): void {
  assert(typeof test.code === "string", "Test case must specify a string value for `code`");

  // optional properties
  if (test.name) {
    assert(typeof test.name === "string", "Optional test case property `name` must be a string");
  }
  if (hasOwn(test, "only")) {
    assert(typeof test.only === "boolean", "Optional test case property `only` must be a boolean");
  }
  if (hasOwn(test, "filename")) {
    assert(
      typeof test.filename === "string",
      "Optional test case property `filename` must be a string",
    );
  }
  if (hasOwn(test, "options")) {
    assert(Array.isArray(test.options), "Optional test case property `options` must be an array");
  }
}

// Ignored test case properties when checking for test case duplicates
const DUPLICATION_IGNORED_PROPS = new Set(["name", "errors", "output"]);

/**
 * Assert that this test case is not a duplicate of one we have seen before.
 * @param test - Test case object
 * @param seenTestCases - Set of serialized test cases we have seen so far (managed by this function)
 * @throws {AssertionError} If the test case is a duplicate
 */
function assertNotDuplicateTestCase(test: TestCase, seenTestCases: Set<string>): void {
  // If we can't serialize a test case (because it contains a function, RegExp, etc), skip the check.
  // This might happen with properties like: `options`, `plugins`, `settings`, `languageOptions.parser`,
  // `languageOptions.parserOptions`.
  if (!isSerializable(test)) return;

  const serializedTestCase = stringify(test, {
    replacer(key, value) {
      // `this` is the currently stringified object --> only ignore top-level properties
      return test !== this || !DUPLICATION_IGNORED_PROPS.has(key) ? value : undefined;
    },
  });

  assert(!seenTestCases.has(serializedTestCase), "Detected duplicate test case");
  seenTestCases.add(serializedTestCase);
}

/**
 * Check if a value is serializable.
 * Functions or objects like RegExp cannot be serialized by JSON.stringify().
 * Inspired by: https://stackoverflow.com/questions/30579940/reliable-way-to-check-if-objects-is-serializable-in-javascript
 * @param value - Value
 * @param seenObjects - Objects already seen in this path from the root object.
 * @returns {boolean} `true` if the value is serializable
 */
function isSerializable(value: unknown, seenObjects: Set<object> = new Set()): boolean {
  if (!isSerializablePrimitiveOrPlainObject(value)) return false;

  if (value === null || typeof value !== "object") return true;

  // Since this is a depth-first traversal, encountering the same object again means there is a circular reference.
  // Objects with circular references are not serializable.
  if (seenObjects.has(value)) return false;

  for (const property in value) {
    if (!Object.hasOwn(value, property)) continue;

    const prop = (value as { [property]: unknown })[property];
    if (!isSerializablePrimitiveOrPlainObject(prop)) return false;
    if (prop === null || typeof prop !== "object") continue;

    // We're creating a new Set of seen objects because we want to ensure that `val` doesn't appear again in this path,
    // but it can appear in other paths. This allows for reusing objects in the graph, as long as there are no cycles.
    if (!isSerializable(prop, new Set([...seenObjects, value]))) return false;
  }

  return true;
}

/**
 * Check if a value is a primitive or plain object created by the `Object` constructor.
 * @param value - Value to check
 * @returns `true` if `value` is a primitive or plain object
 */
function isSerializablePrimitiveOrPlainObject(value: unknown): boolean {
  return (
    value === null ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    (typeof value === "object" && (value.constructor === Object || isArray(value)))
  );
}

// Add types to `RuleTester` namespace
type _Config = Config;
type _LanguageOptions = LanguageOptions;
type _ParserOptions = ParserOptions;
type _SourceType = SourceType;
type _Language = Language;
type _EcmaFeatures = EcmaFeatures;
type _DescribeFn = DescribeFn;
type _ItFn = ItFn;
type _ValidTestCase = ValidTestCase;
type _InvalidTestCase = InvalidTestCase;
type _TestCases = TestCases;
type _Error = Error;

export namespace RuleTester {
  export type Config = _Config;
  export type LanguageOptions = _LanguageOptions;
  export type ParserOptions = _ParserOptions;
  export type SourceType = _SourceType;
  export type Language = _Language;
  export type EcmaFeatures = _EcmaFeatures;
  export type DescribeFn = _DescribeFn;
  export type ItFn = _ItFn;
  export type ValidTestCase = _ValidTestCase;
  export type InvalidTestCase = _InvalidTestCase;
  export type TestCases = _TestCases;
  export type Error = _Error;
}
