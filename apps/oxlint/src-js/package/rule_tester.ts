/*
 * `RuleTester` class.
 *
 * Heavily based on ESLint's `RuleTester`, but without the complications of configs.
 * Has the same user-facing API as ESLint's version.
 * Code: https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/rule-tester/rule-tester.js
 * License (MIT): https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/LICENSE
 */

import { default as assert, AssertionError } from "node:assert";
import { join as pathJoin, isAbsolute as isAbsolutePath, dirname } from "node:path";
import util from "node:util";
import stableJsonStringify from "json-stable-stringify-without-jsonify";
import { applyFixes } from "../bindings.js";
import { ecmaFeaturesOverride, setEcmaVersion, ECMA_VERSION } from "../plugins/context.ts";
import { registerPlugin, registeredRules } from "../plugins/load.ts";
import { lintFileImpl, resetStateAfterError } from "../plugins/lint.ts";
import { getLineColumnFromOffset, getNodeByRangeIndex } from "../plugins/location.ts";
import { allOptions, setOptions, DEFAULT_OPTIONS_ID } from "../plugins/options.ts";
import { diagnostics, replacePlaceholders, PLACEHOLDER_REGEX } from "../plugins/report.ts";
import { parse } from "./parse.ts";

import type { RequireAtLeastOne } from "type-fest";
import type { FixReport } from "../plugins/fix.ts";
import type { Plugin, Rule } from "../plugins/load.ts";
import type { Options } from "../plugins/options.ts";
import type { DiagnosticData, SuggestionReport } from "../plugins/report.ts";
import type { Settings } from "../plugins/settings.ts";
import type { ParseOptions } from "./parse.ts";

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
  it !== defaultIt && typeof it.only === "function" ? it.only.bind(it) : null;

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
   *
   * Useful if moving test cases over from ESLint's `RuleTester` to Oxlint's.
   * It is recommended to only use this option as a temporary measure and alter the test cases
   * so `eslintCompat` is no longer required.
   *
   * If `true`:
   * - Column offsets in diagnostics are incremented by 1.
   * - Fixes which are adjacent to each other are considered overlapping, and only the first fix is applied.
   * - Defaults `sourceType` to "module" if not provided (otherwise default is "unambiguous").
   * - Disallows `sourceType: "unambiguous"`.
   * - Allows `null` as property value for `globals`.
   *   `globals: { foo: null }` is treated as equivalent to `globals: { foo: "readonly" }`.
   *   ESLint accepts `null`, though this is undocumented. Oxlint does not accept `null`.
   * - Slightly different behavior when `report` is called with `loc` of form `{ line, column }`.
   *
   * All of these match ESLint `RuleTester`'s behavior.
   */
  eslintCompat?: boolean;

  /**
   * Language options.
   */
  languageOptions?: LanguageOptions;

  /**
   * Current working directory for the linter.
   * If not provided, defaults to the directory containing the test file.
   */
  cwd?: string;

  /**
   * Maximum number of additional fix passes to apply.
   * After the first fix pass, re-lints the fixed code and applies fixes again,
   * repeating up to `recursive` additional times (or until no more fixes are produced).
   *
   * - `false` / `null` / `undefined`: no recursion (default)
   * - `true`: 10 extra passes
   * - `number`: N extra passes
   */
  recursive?: boolean | number | null | undefined;
}

/**
 * Language options config.
 */
interface LanguageOptions {
  sourceType?: SourceType;
  globals?: Globals;
  env?: Envs;
  parserOptions?: ParserOptions;
}

/**
 * Language options config, with `parser` and `ecmaVersion` properties, and extended `parserOptions`.
 * These properties should not be present in `languageOptions` config,
 * but could be if test cases are ported from ESLint.
 * For internal use only.
 */
export interface LanguageOptionsInternal extends LanguageOptions {
  ecmaVersion?: number | "latest";
  parser?: {
    parse?: (code: string, options?: Record<string, unknown>) => unknown;
    parseForESLint?: (code: string, options?: Record<string, unknown>) => unknown;
  };
  parserOptions?: ParserOptionsInternal;
}

/**
 * Source type.
 *
 * `'unambiguous'` is not supported in ESLint compatibility mode.
 */
type SourceType = "script" | "module" | "commonjs" | "unambiguous";

/**
 * Value of a property in `globals` object.
 *
 * Note: `null` only supported in ESLint compatibility mode.
 */
type GlobalValue =
  | boolean
  | "true"
  | "writable"
  | "writeable"
  | "false"
  | "readonly"
  | "readable"
  | "off"
  | null;

/**
 * Globals object.
 */
type Globals = Record<string, GlobalValue>;

/**
 * Environments for the file being linted.
 */
export type Envs = Record<string, boolean>;

/**
 * Parser options config.
 */
interface ParserOptions {
  ecmaFeatures?: EcmaFeatures;
  /**
   * Language variant to parse file as.
   *
   * If test case provides a filename, that takes precedence over `lang` option.
   * Language will be inferred from file extension.
   */
  lang?: Language;
  /**
   * `true` to ignore non-fatal parsing errors.
   */
  ignoreNonFatalErrors?: boolean;
}

/**
 * Parser options config, with extended `ecmaFeatures`.
 * These properties should not be present in `languageOptions` config,
 * but could be if test cases are ported from ESLint.
 * For internal use only.
 */
export interface ParserOptionsInternal extends ParserOptions {
  ecmaFeatures?: EcmaFeaturesInternal;
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
 * ECMA features config, with `globalReturn` and `impliedStrict` properties.
 * These properties should not be present in `ecmaFeatures` config,
 * but could be if test cases are ported from ESLint.
 * For internal use only.
 */
interface EcmaFeaturesInternal extends EcmaFeatures {
  /**
   * `true` if file is parsed with top-level `return` statements allowed.
   */
  globalReturn?: boolean;
  /**
   * `true` if file is parsed as strict mode code.
   */
  impliedStrict?: boolean;
}

/**
 * Parser language.
 */
type Language = "js" | "jsx" | "ts" | "tsx" | "dts";

// Number of additional fix passes to apply after the first pass if `recursive: true`
const RECURSIVE_TRUE_PASSES = 10;

// Empty language options
const EMPTY_LANGUAGE_OPTIONS: LanguageOptionsInternal = {};

// `RuleTester` uses this config as its default. Can be overwritten via `RuleTester.setDefaultConfig()`.
let sharedConfig: Config = {};

// ------------------------------------------------------------------------------
// Test cases
// ------------------------------------------------------------------------------

// List of keys that `ValidTestCase` or `InvalidTestCase` can have.
// Must be kept in sync with properties of `ValidTestCase` and `InvalidTestCase` interfaces.
// The type constraints enforce this.
const TEST_CASE_PROP_KEYS_ARRAY = [
  "code",
  "name",
  "only",
  "filename",
  "options",
  "settings",
  "before",
  "after",
  "output",
  "errors",
  // Not a valid key for `TestCase` interface, but present here to prevent prototype pollution in `createConfigForRun`
  "__proto__",
] as const satisfies readonly (TestCaseOwnKeys | "__proto__")[];

type TestCaseOwnKeys = Exclude<keyof ValidTestCase | keyof InvalidTestCase, keyof Config>;
type MissingKeys = Exclude<TestCaseOwnKeys, (typeof TEST_CASE_PROP_KEYS_ARRAY)[number]>;
type KeysSet = MissingKeys extends never ? Set<string> : never;

const TEST_CASE_PROP_KEYS: KeysSet = new Set(TEST_CASE_PROP_KEYS_ARRAY);

/**
 * Test case.
 */
interface TestCase extends Config {
  code: string;
  name?: string;
  only?: boolean;
  filename?: string;
  options?: Options;
  settings?: Settings;
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
  endLine?: number | undefined;
  endColumn?: number | undefined;
  suggestions?: ErrorSuggestion[] | null;
}

/**
 * Expected suggestion in a test case error.
 */
interface ErrorSuggestion {
  desc?: string;
  messageId?: string;
  data?: DiagnosticData;
  output: string;
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
  fixes: FixReport[] | null;
  suggestions: SuggestionReport[] | null;
}

// Default path (without extension) for test cases if not provided
const DEFAULT_FILENAME_BASE = "file";

// Default CWD for test cases if not provided.
// Root of `oxlint` package once bundled into `dist`.
const DEFAULT_CWD = dirname(import.meta.dirname);

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
      itOnly = it.only.bind(it);
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

// In conformance build only, we provide a hook to modify test cases before they're run.
// Hook can be registered by calling `RuleTester.registerModifyTestCaseHook`.
// This is used in conformance tester.
let modifyTestCase: ((test: TestCase) => void) | null = null;

if (CONFORMANCE) {
  (RuleTester as any).registerModifyTestCaseHook = (modify: (test: TestCase) => void) => {
    modifyTestCase = modify;
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
  } else if (CONFORMANCE && (errors as unknown as string) === "__unknown__") {
    // In conformance tests, sometimes test cases don't specify `errors` property
    // (e.g. `eslint-plugin-stylistic`'s test cases). Conformance tester sets `errors` to `"__unknown__"`
    // in those cases. So don't error here.
  } else {
    // `errors` is an array of error objects
    assertErrorCountIsCorrect(diagnostics, errors.length);

    // Sort diagnostics by line and column before comparing to expected errors. ESLint does the same.
    diagnostics.sort((diag1, diag2) => diag1.line - diag2.line || diag1.column - diag2.column);

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

        // Test suggestions
        if (Object.hasOwn(error, "suggestions")) {
          if (error.suggestions == null) {
            // `suggestions: null` means "expect no suggestions"
            assert(diagnostic.suggestions === null, "Rule produced suggestions");
          } else {
            assertSuggestionsAreCorrect(diagnostic, error, messages, test);
          }
        }
      }
    }
  }

  // Test output after fixes
  const { code } = test;
  const eslintCompat = test.eslintCompat === true;

  let fixedCode = runFixes(diagnostics, code, eslintCompat);
  if (fixedCode === null) fixedCode = code;

  // Re-lint and re-fix for additional passes if `recursive` option used
  const { recursive } = test;
  const extraPassCount =
    typeof recursive === "number" ? recursive : recursive === true ? RECURSIVE_TRUE_PASSES : 0;

  if (extraPassCount > 0 && fixedCode !== code) {
    for (let pass = 0; pass < extraPassCount; pass++) {
      const diagnostics = lint({ ...test, code: fixedCode }, plugin);
      const newFixedCode = runFixes(diagnostics, fixedCode, eslintCompat);
      if (newFixedCode === null) break;
      fixedCode = newFixedCode;
    }
  }

  if (Object.hasOwn(test, "output")) {
    const expectedOutput = test.output;
    if (expectedOutput === null) {
      assert.strictEqual(fixedCode, code, "Expected no autofixes to be suggested");
    } else {
      assert.strictEqual(fixedCode, expectedOutput, "Output is incorrect");
      assert.notStrictEqual(
        code,
        expectedOutput,
        "Test property `output` matches `code`. If no autofix is expected, set output to `null`.",
      );
    }
  } else {
    assert.strictEqual(fixedCode, code, "The rule fixed the code. Please add `output` property.");
  }
}

/**
 * Run fixes on code and return fixed code.
 * If no fixes to apply, returns `null`.
 *
 * @param diagnostics - Array of `Diagnostic`s returned by `lint`
 * @param code - Code to run fixes on
 * @returns Fixed code, or `null` if no fixes to apply
 * @throws {Error} If error when applying fixes
 */
function runFixes(diagnostics: Diagnostic[], code: string, eslintCompat: boolean): string | null {
  const fixGroups: FixReport[][] = [];
  for (const diagnostic of diagnostics) {
    if (diagnostic.fixes !== null) fixGroups.push(diagnostic.fixes);
  }
  if (fixGroups.length === 0) return null;

  const fixedCode = applyFixes(code, JSON.stringify(fixGroups), eslintCompat);
  if (fixedCode === null) throw new Error("Failed to apply fixes");

  return fixedCode;
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
  if (Object.hasOwn(error, "message")) {
    assert(
      !Object.hasOwn(error, "messageId"),
      "Error should not specify both `message` and a `messageId`",
    );
    assert(!Object.hasOwn(error, "data"), "Error should not specify both `data` and `message`");
    assertMessageMatches(diagnostic.message, error.message!);
    return;
  }

  assert(
    Object.hasOwn(error, "messageId"),
    "Test error must specify either a `messageId` or `message`",
  );

  // Check `messageId` property
  assertMessageIdIsCorrect(
    diagnostic.messageId,
    diagnostic.message,
    error.messageId!,
    error.data,
    messages,
    "",
  );
}

/**
 * Assert that a `messageId` used by the rule under test is correct, and validate `data` (if provided).
 *
 * @param reportedMessageId - `messageId` from the diagnostic or suggestion
 * @param reportedMessage - Message from the diagnostic or suggestion
 * @param messageId - Expected `messageId` from the test case
 * @param data - Data from the test case (if provided)
 * @param messages - Messages from the rule under test
 * @param prefix - Prefix for assertion error messages (e.g. "" or "Suggestion at index 0: ")
 * @throws {AssertionError} If messageId is not correct
 * @throws {AssertionError} If message tenplate with placeholder data inserted does not match reported message
 */
function assertMessageIdIsCorrect(
  reportedMessageId: string | null,
  reportedMessage: string,
  messageId: string,
  data: DiagnosticData | undefined,
  messages: Record<string, string> | null,
  prefix: string,
): void {
  assert(
    messages !== null,
    `${prefix}Cannot use 'messageId' if rule under test doesn't define 'meta.messages'`,
  );

  if (!Object.hasOwn(messages, messageId)) {
    const legalMessageIds = `[${Object.keys(messages)
      .map((key) => `'${key}'`)
      .join(", ")}]`;
    assert.fail(`${prefix}Invalid messageId '${messageId}'. Expected one of ${legalMessageIds}.`);
  }

  assert.strictEqual(
    reportedMessageId,
    messageId,
    `${prefix}messageId '${reportedMessageId}' does not match expected messageId '${messageId}'`,
  );

  // Check if message contains placeholders for which no data was provided
  const ruleMessage = messages[messageId];
  const unsubstitutedPlaceholders = getUnsubstitutedMessagePlaceholders(
    reportedMessage,
    ruleMessage,
    data,
  );
  if (unsubstitutedPlaceholders.length !== 0) {
    assert.fail(
      `${prefix}The reported message has ` +
        (unsubstitutedPlaceholders.length > 1
          ? `unsubstituted placeholders: ${unsubstitutedPlaceholders.map((name) => `'${name}'`).join(", ")}`
          : `an unsubstituted placeholder '${unsubstitutedPlaceholders[0]}'`) +
        `. Please provide the missing ${unsubstitutedPlaceholders.length > 1 ? "values" : "value"} ` +
        "via the `data` property.",
    );
  }

  // Check `data` is correct by filling in placeholders in the message with provided data and checking that
  // rehydrated message matches the reported message
  if (data !== undefined) {
    const rehydratedMessage = replacePlaceholders(ruleMessage, data);
    assert.strictEqual(
      reportedMessage,
      rehydratedMessage,
      `${prefix}Hydrated message "${rehydratedMessage}" does not match "${reportedMessage}"`,
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

  if (Object.hasOwn(error, "line")) {
    actualLocation.line = diagnostic.line;
    expectedLocation.line = error.line;
  }

  if (Object.hasOwn(error, "column")) {
    actualLocation.column = diagnostic.column + columnOffset;
    expectedLocation.column = error.column;
  }

  // `context.report()` accepts just `loc: { line, column }` for error location.
  // ESLint translates that to `loc: { start: { line, column }, end: null }`.
  // Oxlint instead sets `end` to same offset as `start`.
  //
  // Test cases can specify `endLine: undefined` and `endColumn: undefined` to match this case.
  //
  // In ESLint compat mode, deal with this incompatibility.
  const canVoidEndLocation =
    test.eslintCompat === true &&
    diagnostic.endLine === diagnostic.line &&
    diagnostic.endColumn === diagnostic.column;

  if (Object.hasOwn(error, "endLine")) {
    if (error.endLine === undefined && canVoidEndLocation) {
      actualLocation.endLine = undefined;
    } else {
      actualLocation.endLine = diagnostic.endLine;
    }
    expectedLocation.endLine = error.endLine;
  }

  if (Object.hasOwn(error, "endColumn")) {
    if (error.endColumn === undefined && canVoidEndLocation) {
      actualLocation.endColumn = undefined;
    } else {
      actualLocation.endColumn = diagnostic.endColumn + columnOffset;
    }
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
 * Assert that suggestions reported by the rule under test match expected suggestions.
 * @param diagnostic - Diagnostic emitted by the rule under test
 * @param error - Error object from the test case
 * @param messages - Messages from the rule under test
 * @param test - Test case
 * @throws {AssertionError} If suggestions do not match
 */
function assertSuggestionsAreCorrect(
  diagnostic: Diagnostic,
  error: Error,
  messages: Record<string, string> | null,
  test: TestCase,
): void {
  const actualSuggestions = diagnostic.suggestions ?? [];
  const expectedSuggestions = error.suggestions!;

  assert.strictEqual(
    actualSuggestions.length,
    expectedSuggestions.length,
    `Error should have ${expectedSuggestions.length} suggestion${expectedSuggestions.length > 1 ? "s" : ""}. ` +
      `Instead found ${actualSuggestions.length} suggestion${actualSuggestions.length > 1 ? "s" : ""}.`,
  );

  const eslintCompat = test.eslintCompat === true;

  for (let i = 0; i < expectedSuggestions.length; i++) {
    const actual = actualSuggestions[i]!;
    const expected = expectedSuggestions[i]!;
    const prefix = `Suggestion at index ${i}`;

    // Validate suggestion message (`desc` or `messageId` + `data`)
    assertSuggestionMessageIsCorrect(actual, expected, messages, prefix);

    // Validate output
    assert(Object.hasOwn(expected, "output"), `${prefix}: \`output\` property is required`);

    const suggestedCode = applyFixes(test.code, JSON.stringify([actual.fixes]), eslintCompat);
    assert(suggestedCode !== null, `${prefix}: Failed to apply suggestion fix`);

    assert.strictEqual(
      suggestedCode,
      expected.output,
      `${prefix}: Expected the applied suggestion fix to match the test suggestion output`,
    );

    assert.notStrictEqual(
      expected.output,
      test.code,
      `${prefix}: The output of a suggestion should differ from the original source code`,
    );
  }
}

/**
 * Assert that a suggestion's message matches expectations.
 * @param actual - Actual suggestion from the diagnostic
 * @param expected - Expected suggestion from the test case
 * @param messages - Messages from the rule under test
 * @param prefix - Prefix for assertion error messages
 * @throws {AssertionError} If suggestion message does not match
 */
function assertSuggestionMessageIsCorrect(
  actual: SuggestionReport,
  expected: ErrorSuggestion,
  messages: Record<string, string> | null,
  prefix: string,
): void {
  if (Object.hasOwn(expected, "desc")) {
    assert(
      !Object.hasOwn(expected, "messageId"),
      `${prefix}: Test should not specify both \`desc\` and \`messageId\``,
    );
    assert(
      !Object.hasOwn(expected, "data"),
      `${prefix}: Test should not specify both \`desc\` and \`data\``,
    );
    assert.strictEqual(
      actual.message,
      expected.desc,
      `${prefix}: \`desc\` should be "${expected.desc}" but got "${actual.message}" instead`,
    );
    return;
  }

  if (Object.hasOwn(expected, "messageId")) {
    assertMessageIdIsCorrect(
      actual.messageId,
      actual.message,
      expected.messageId!,
      expected.data,
      messages,
      `${prefix}: `,
    );
    return;
  }

  if (Object.hasOwn(expected, "data")) {
    assert.fail(`${prefix}: Test must specify \`messageId\` if \`data\` is used`);
  }

  assert.fail(`${prefix}: Test must specify either \`messageId\` or \`desc\``);
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
  // `modifyTestCase` is only available in conformance build - it's only for conformance testing.
  if (CONFORMANCE && modifyTestCase !== null) modifyTestCase(merged);

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
    globals: mergeGlobals(localLanguageOptions.globals, baseLanguageOptions.globals),
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
 * Merge globals from test case / config onto globals from base config.
 * @param localGlobals - Globals from test case / config
 * @param baseGlobals - Globals from base config
 * @returns Merged globals
 */
function mergeGlobals(
  localGlobals?: Globals | null,
  baseGlobals?: Globals | null,
): Globals | undefined {
  if (localGlobals == null) return baseGlobals ?? undefined;
  if (baseGlobals == null) return localGlobals;
  return { ...baseGlobals, ...localGlobals };
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

  // Determine path and CWD.
  // If not provided, use default filename based on `parseOptions.lang`,
  // and the directory of this file as CWD.
  // If `filename` is an absolute path, make `cwd` the directory containing `filename`.
  let path: string;
  let { filename, cwd } = test;
  if (filename != null && isAbsolutePath(filename)) {
    if (cwd == null) cwd = dirname(filename);
    path = filename;
  } else {
    if (filename == null) {
      let ext: string | undefined = parseOptions.lang;
      if (ext == null) {
        ext = "js";
      } else if (ext === "dts") {
        ext = "d.ts";
      }
      filename = `${DEFAULT_FILENAME_BASE}.${ext}`;
    }

    if (cwd == null) cwd = DEFAULT_CWD;
    path = pathJoin(cwd, filename);
  }

  try {
    // Register plugin. This adds rule to `registeredRules` array.
    registerPlugin(plugin, null, false, null);

    // Set up options
    const optionsId = setupOptions(test, cwd);

    // Parse file into buffer
    parse(path, test.code, parseOptions);

    // In conformance tests, set `context.languageOptions.ecmaVersion`.
    // This is not supported outside of conformance tests.
    if (CONFORMANCE) setEcmaVersionAndFeatures(test);

    // Get globals and settings
    const globalsJSON = getGlobalsJson(test);
    const settingsJSON = JSON.stringify(test.settings ?? {});

    // Lint file.
    // Buffer is stored already, at index 0. No need to pass it.
    lintFileImpl(path, 0, null, [0], [optionsId], settingsJSON, globalsJSON, null);

    // Return diagnostics
    const ruleId = `${plugin.meta!.name!}/${Object.keys(plugin.rules)[0]}`;

    return diagnostics.map((diagnostic) => {
      let line, column, endLine, endColumn;

      // Convert start/end offsets to line/column.
      // In conformance build, use original `loc` if one was passed to `report`.
      if (!CONFORMANCE || diagnostic.loc == null) {
        ({ line, column } = getLineColumnFromOffset(diagnostic.start));
        ({ line: endLine, column: endColumn } = getLineColumnFromOffset(diagnostic.end));
      } else {
        const { loc } = diagnostic;
        ({ line, column } = loc.start);
        if (loc.end != null) {
          ({ line: endLine, column: endColumn } = loc.end);
        } else {
          endLine = line;
          endColumn = column;
        }
      }

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
        fixes: diagnostic.fixes,
        suggestions: diagnostic.suggestions,
      };
    });
  } finally {
    // Reset state
    registeredRules.length = 0;
    if (allOptions !== null) allOptions.length = 1;

    // Even if there hasn't been an error, do a full reset of state just to be sure.
    // This includes emptying `diagnostics`.
    resetStateAfterError();
  }
}

/**
 * Get parse options for a test case.
 * @param test - Test case
 * @returns Parse options
 */
function getParseOptions(test: TestCase): ParseOptions {
  const parseOptions: ParseOptions = {};

  let languageOptions = test.languageOptions as LanguageOptionsInternal | undefined;
  if (languageOptions == null) languageOptions = EMPTY_LANGUAGE_OPTIONS;

  // Throw error if custom parser is provided
  if (languageOptions.parser != null) throw new Error("Custom parsers are not supported");

  // Handle `languageOptions.sourceType`
  const { sourceType } = languageOptions;
  if (sourceType != null) {
    // `unambiguous` is disallowed in ESLint compatibility mode
    if (test.eslintCompat === true && sourceType === "unambiguous") {
      throw new Error(
        "'unambiguous' source type is not supported in ESLint compatibility mode.\n" +
          "Disable ESLint compatibility mode by setting `eslintCompat` to `false` in the config / test case.",
      );
    }

    parseOptions.sourceType = sourceType;
  } else if (test.eslintCompat === true) {
    // ESLint defaults to `module` if no source type is specified
    parseOptions.sourceType = "module";
  }

  // Handle `languageOptions.parserOptions`
  const { parserOptions } = languageOptions;
  if (parserOptions != null) {
    // Handle `parserOptions.ignoreNonFatalErrors`
    if (parserOptions.ignoreNonFatalErrors === true) parseOptions.ignoreNonFatalErrors = true;

    // Handle `parserOptions.lang`. `filename` takes precedence over `lang` if provided.
    if (test.filename == null) {
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

/**
 * Get globals and envs as JSON for test case.
 *
 * Normalizes globals values to "readonly", "writable", or "off", same as Rust side does.
 * `null` is only supported in ESLint compatibility mode.
 *
 * Removes envs which are false, same as Rust side does.
 *
 * @param test - Test case
 * @returns Globals and envs as JSON string of form `{ "globals": { ... }, "envs": { ... } }`
 */
function getGlobalsJson(test: TestCase): string {
  // Get globals.
  // Normalize values to `readonly`, `writable`, or `off` - same as Rust side does.
  const globals = { ...test.languageOptions?.globals },
    eslintCompat = !!test.eslintCompat;

  for (const key in globals) {
    let value = globals[key];

    switch (value) {
      case "readonly":
      case "writable":
      case "off":
        continue;

      case "writeable":
      case "true":
      case true:
        value = "writable";
        break;

      case "readable":
      case "false":
      case false:
        value = "readonly";
        break;

      // ESLint treats `null` as `readonly` (undocumented).
      // https://github.com/eslint/eslint/blob/ba71baa87265888b582f314163df1d727441e2f1/lib/languages/js/source-code/source-code.js#L119-L149
      // But Oxlint (Rust code) doesn't support it, so we don't support it here either unless in ESLint compatibility mode.
      case null:
        if (eslintCompat) {
          value = "readonly";
          break;
        }

      default:
        throw new Error(
          `'${value}' is not a valid configuration for a global (use 'readonly', 'writable', or 'off')`,
        );
    }

    globals[key] = value;
  }

  // TODO: Tests for `env` in `RuleTester` tests

  // Get envs.
  // Remove properties which are `false` - same as Rust side does.
  const originalEnvs = test.languageOptions?.env;
  const envs: Envs = {};
  if (originalEnvs != null) {
    for (const [key, value] of Object.entries(originalEnvs)) {
      if (value === false) continue;

      // Use `Object.defineProperty` to handle if `key` is "__proto__"
      Object.defineProperty(envs, key, {
        value: true,
        writable: true,
        enumerable: true,
        configurable: true,
      });
    }
  }

  // Serialize globals + envs to JSON
  return JSON.stringify({ globals, envs });
}

/**
 * Set up options for the test case.
 *
 * In linter, all options for all rules are sent over from Rust as a JSON string,
 * and `setOptions` is called to merge them with the default options for each rule.
 * The merged options are stored in a global variable `allOptions`.
 *
 * This function builds a JSON string in same format as Rust does, and calls `setOptions` with it.
 *
 * Returns the options ID to pass to `lintFileImpl` (either 0 for default options, or 1 for user-provided options).
 *
 * @param test - Test case
 * @param cwd - Current working directory for test case
 * @returns Options ID to pass to `lintFileImpl`
 */
function setupOptions(test: TestCase, cwd: string): number {
  // Initial entries for default options
  const allOptions: Options[] = [[]],
    allRuleIds: number[] = [0];

  // If options are provided for test case, add them to `allOptions`
  let optionsId = DEFAULT_OPTIONS_ID;

  const testOptions = test.options;
  if (testOptions != null) {
    allOptions.push(testOptions);
    allRuleIds.push(0);
    optionsId = 1;
  }

  // Serialize to JSON and pass to `setOptions`
  let allOptionsJson: string;
  try {
    allOptionsJson = JSON.stringify({
      options: allOptions,
      ruleIds: allRuleIds,
      cwd,
      workspaceUri: null,
    });
  } catch (err) {
    throw new Error(
      `Failed to serialize options: ${err as (typeof globalThis)["Error"]["prototype"]}`,
    );
  }
  setOptions(allOptionsJson);

  return optionsId;
}

/**
 * Inject:
 * - `languageOptions.ecmaVersion` into `context.languageOptions`.
 * - `languageOptions.parserOptions.ecmaFeatures.globalReturn` into scope analyzer options.
 * - `languageOptions.parserOptions.ecmaFeatures.impliedStrict` into scope analyzer options.
 *
 * This is only supported in conformance tests, where it's necessary to pass some tests.
 * Oxlint doesn't support any ECMA version except latest, or the `globalReturn` or `impliedStrict` ECMA features.
 * @param test - Test case
 */
function setEcmaVersionAndFeatures(test: TestCase) {
  if (!CONFORMANCE) throw new Error("Should be unreachable outside of conformance tests");

  // Set `ecmaVersion`.
  // Same logic as ESLint's `normalizeEcmaVersionForLanguageOptions` function.
  // https://github.com/eslint/eslint/blob/54bf0a3646265060f5f22faef71ec840d630c701/lib/languages/js/index.js#L71-L100
  // Only difference is that we default to `ECMA_VERSION` not `5` if `ecmaVersion` is undefined.
  // In ESLint, the branch for `undefined` is actually dead code, because `undefined` is replaced by default value
  // in an early step of config parsing.
  const languageOptions = test.languageOptions as LanguageOptionsInternal | undefined;
  let ecmaVersion = languageOptions?.ecmaVersion;

  if (typeof ecmaVersion === "number") {
    if (ecmaVersion > 5 && ecmaVersion < 2015) ecmaVersion += 2009;
  } else {
    ecmaVersion = ECMA_VERSION;
  }
  setEcmaVersion(ecmaVersion);

  // Set `globalReturn` and `impliedStrict` in scope analyzer options
  const ecmaFeatures = languageOptions?.parserOptions?.ecmaFeatures;
  ecmaFeaturesOverride.globalReturn = ecmaFeatures?.globalReturn ?? null;
  // Strict mode does not exist in ES3
  ecmaFeaturesOverride.impliedStrict =
    ecmaVersion === 3 ? false : (ecmaFeatures?.impliedStrict ?? null);
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
  if (Object.hasOwn(test, "before")) runHook(test, test.before, "before");
}

/**
 * Runs after hook on the given test case.
 * @param test - Test to run the hook on
 * @throws {Error} - If the hook is not a function
 * @throws {*} - Value thrown by the hook function
 */
function runAfterHook(test: TestCase): void {
  if (Object.hasOwn(test, "after")) runHook(test, test.after, "after");
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
  } else if (CONFORMANCE && (errors as unknown as string) === "__unknown__") {
    // In conformance tests, sometimes test cases don't specify `errors` property
    // (e.g. `eslint-plugin-stylistic`'s test cases). Conformance tester sets `errors` to `"__unknown__"`
    // in those cases. So don't error here.
  } else {
    assert(
      errors !== undefined,
      `Did not specify errors for an invalid test of rule \`${ruleName}\``,
    );
    assert(
      Array.isArray(errors),
      `Invalid 'errors' property for invalid test of rule \`${ruleName}\`:` +
        `expected a number or an array but got ${errors === null ? "null" : typeof errors}`,
    );
    assert(errors.length !== 0, "Invalid cases must have at least one error");
  }

  // `output` is optional, but if it exists it must be a string or `null`
  if (Object.hasOwn(test, "output")) {
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
  if (Object.hasOwn(test, "only")) {
    assert(typeof test.only === "boolean", "Optional test case property `only` must be a boolean");
  }
  if (Object.hasOwn(test, "filename")) {
    assert(
      typeof test.filename === "string",
      "Optional test case property `filename` must be a string",
    );
  }
  if (Object.hasOwn(test, "options")) {
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

  const serializedTestCase = stableJsonStringify(test, {
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
    (typeof value === "object" && (value.constructor === Object || Array.isArray(value)))
  );
}

// Add types to `RuleTester` namespace
type _Config = Config;
type _LanguageOptions = LanguageOptions;
type _Globals = Globals;
type _Envs = Envs;
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
type _ErrorSuggestion = ErrorSuggestion;

export namespace RuleTester {
  export type Config = _Config;
  export type LanguageOptions = _LanguageOptions;
  export type Globals = _Globals;
  export type Envs = _Envs;
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
  export type ErrorSuggestion = _ErrorSuggestion;
}
