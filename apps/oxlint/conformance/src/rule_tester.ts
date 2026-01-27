/*
 * Shim of `RuleTester` class.
 */

// @ts-expect-error - internal module of ESLint with no types
import eslintGlobals from "../submodules/eslint/conf/globals.js";
import { RuleTester } from "#oxlint/rule-tester";
import { describe, it, setCurrentTest } from "./capture.ts";
import { SHOULD_SKIP_CODE } from "./filter.ts";

import type { Rule } from "#oxlint/plugin";
import type { ParserDetails } from "./index.ts";
import type {
  LanguageOptionsInternal,
  ParserOptionsInternal,
} from "../../src-js/package/rule_tester.ts";

type DescribeFn = RuleTester.DescribeFn;
type ItFn = RuleTester.ItFn;
type TestCases = RuleTester.TestCases;
type Globals = RuleTester.Globals;
export type Language = RuleTester.Language;
export type LanguageOptions = LanguageOptionsInternal;
export type ParserOptions = ParserOptionsInternal;

interface TestCaseExtension {
  languageOptions?: LanguageOptionsInternal;
  // Parser was specified as `test.parser` (path string) in old ESLint versions
  parser?: string;
  _parser?: ParserDetails;
}

export type ValidTestCase = RuleTester.ValidTestCase & TestCaseExtension;
export type InvalidTestCase = RuleTester.InvalidTestCase & TestCaseExtension;
export type TestCase = ValidTestCase | InvalidTestCase;

// Maps of parser modules and parser paths to parser details (language + specifier)
export const parserModules: Map<unknown, ParserDetails> = new Map();
export const parserModulePaths: Map<string, ParserDetails> = new Map();

// Set up `RuleTester` to use our hooks
RuleTester.describe = describe;
RuleTester.it = it;

/**
 * Shim of `RuleTester` class.
 * Prevents overriding `describe` and `it` properties.
 */
class RuleTesterShim extends RuleTester {
  // Prevent changing `describe` or `it` properties

  static get describe(): DescribeFn {
    return describe;
  }

  static set describe(_value: DescribeFn) {
    throw new Error("Cannot override `describe` property");
  }

  static get it(): ItFn {
    return it;
  }

  static set it(_value: ItFn) {
    throw new Error("Cannot override `it` property");
  }

  static get itOnly(): ItFn {
    return it.only;
  }

  static set itOnly(_value: ItFn) {
    throw new Error("Cannot override `itOnly` property");
  }

  // Apply filter to test cases and add `before` hook to store test case in `currentTest`
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    tests = {
      valid: tests.valid
        .map((test) => {
          if (typeof test === "string") test = { code: test };
          return addBeforeHook(test);
        })
        .filter(shouldRunTestCase),
      invalid: tests.invalid.map(addBeforeHook).filter(shouldRunTestCase),
    };

    super.run(ruleName, rule, tests);
  }
}

/**
 * Add `before` hook to test case to store test case in `currentTest`.
 * @param test - Test case
 * @returns Test case
 */
function addBeforeHook<T extends TestCase>(test: T): T {
  // Clone test in case `after` hook modifies it (edge case)
  const clonedTest = { ...test };

  if (Object.hasOwn(test, "before")) {
    const originalBefore = test.before;
    test.before = function (this) {
      setCurrentTest(clonedTest);
      originalBefore!.call(this);
    };
  } else {
    // Non-enumerable property so that test case remains serializable
    // (for `isSerializable` check in `assertNotDuplicateTestCase`)
    Object.defineProperty(test, "before", {
      value: () => setCurrentTest(clonedTest),
      writable: true,
      configurable: true,
      enumerable: false,
    });
  }

  return test;
}

/**
 * Check if test case should run.
 * @param test - Test case
 * @returns `true` if test case should run, `false` if is filtered out
 */
function shouldRunTestCase(test: TestCase): boolean {
  return !SHOULD_SKIP_CODE(test.code);
}

// Register hook to modify test cases before they are run.
// `registerModifyTestCaseHook` is only present in debug builds, so it's not part of the `RuleTester` type def.
(RuleTester as any).registerModifyTestCaseHook(modifyTestCase);

/**
 * Modify test case before running it.
 * Store test case in `currentTest` so it can be accessed in `it` function.
 * @param test - Test case
 */
function modifyTestCase(test: TestCase): void {
  let { languageOptions } = test;

  // Record current test case.
  // Clone it to avoid including the changes to the original test case made below.
  // Replace `languageOptions.parser` with `{}` to avoid verbose output in snapshots.
  const storedTest = { ...test };
  if (languageOptions?.parser != null) {
    storedTest.languageOptions = { ...languageOptions, parser: {} };
  }
  setCurrentTest(storedTest);

  // Enable ESLint compat mode.
  // This makes `RuleTester` adjust column indexes in diagnostics to match ESLint's behavior.
  test.eslintCompat = true;

  // Ignore parsing errors. ESLint's test cases include invalid code.
  languageOptions = { ...test.languageOptions };
  test.languageOptions = languageOptions;

  const parserOptions = { ...languageOptions.parserOptions };
  languageOptions.parserOptions = parserOptions;

  parserOptions.ignoreNonFatalErrors = true;

  // Build the globals object
  languageOptions.globals = getGlobals(languageOptions);

  // Disable env - we're providing all globals explicitly via `languageOptions.globals`.
  // Setting `env` to empty object prevents the default "builtin" env from being applied.
  languageOptions.env = {};

  // If test case uses a known parser, set `parserOptions.lang` to match.
  // Parser can be specified as:
  // - Current ESLint: `test.languageOptions.parser` (parser object)
  // - Old ESLint versions: `test.parser` (absolute path to parser)
  let parserDetails: ParserDetails | null = null;

  if (languageOptions.parser != null) {
    parserDetails = parserModules.get(languageOptions.parser) ?? null;
    if (parserDetails !== null) delete languageOptions.parser;
  }

  if (test.parser != null) {
    if (parserDetails !== null) {
      throw new Error("Both `test.parser` and `test.languageOptions.parser` specified");
    }
    parserDetails = parserModulePaths.get(test.parser) ?? null;
    if (parserDetails === null) {
      // Set `languageOptions.parser` so an error is thrown.
      // Store in stored test case so appears in snapshot.
      (languageOptions as any).parser = { specifier: "__unknownParser", path: test.parser };
      (storedTest as any)._parser = { specifier: "__unknownParser", path: test.parser };
    }
    delete test.parser;
  }

  if (parserDetails !== null) {
    let { lang } = parserDetails;
    if (parserOptions.ecmaFeatures?.jsx === true) {
      if (lang === "ts") {
        lang = "tsx";
      } else if (lang === "js") {
        lang = "jsx";
      }
    }
    parserOptions.lang = lang;

    // Store parser details in test case so tests using different parsers don't get detected as duplicates.
    // Store in stored test case so they appear in snapshot.
    (test as any)._parser = parserDetails;
    storedTest._parser = parserDetails;
  }
}

/**
 * Combine globals from test case and ESLint's preset based on `languageOptions.ecmaVersion`
 * and `languageOptions.sourceType`.
 * @param languageOptions - Language options from test case
 * @returns Globals object
 */
function getGlobals(languageOptions: LanguageOptionsInternal): Globals {
  const globals: Globals = {};

  // Set up globals to match ESLint's behavior.
  // ESLint sets globals based on `languageOptions.ecmaVersion` and `languageOptions.sourceType`.
  // See `applyLanguageOptions` in ESLint `lib/languages/js/source-code/source-code.js`.
  //
  // By default (when no `ecmaVersion` is specified), ESLint uses "latest" which maps to the
  // most recent ES version globals. This is equivalent to TS-ESLint's `lib: ["esnext"]`.
  const ecmaVersionPreset = getGlobalsForEcmaVersion(languageOptions.ecmaVersion);

  addGlobalsFromPreset(ecmaVersionPreset, globals);

  // Add CommonJS globals if `sourceType` is `"commonjs"`
  if (languageOptions.sourceType === "commonjs") {
    addGlobalsFromPreset(eslintGlobals.commonjs, globals);
  }

  // Add any existing globals from the test case (test case globals take priority)
  Object.assign(globals, languageOptions.globals);

  return globals;
}

/**
 * Get globals for a given ECMAScript version.
 * This matches ESLint's `getGlobalsForEcmaVersion` function in `source-code.js`.
 *
 * @param ecmaVersion - ECMAScript version (e.g., 3, 5, 6, 2015, 2020, "latest")
 * @returns Globals object for that version
 */
function getGlobalsForEcmaVersion(
  ecmaVersion: number | "latest" | undefined,
): Record<string, boolean> {
  switch (ecmaVersion) {
    case 3:
      return eslintGlobals.es3;
    case 5:
      return eslintGlobals.es5;
    case undefined:
    case "latest":
      // "latest" or unspecified = use the most recent ES version (currently es2026)
      return eslintGlobals.es2026;
    default:
      if (ecmaVersion < 2015) {
        // Versions 6-14 map to es2015-es2023 (version + 2009)
        return eslintGlobals[`es${ecmaVersion + 2009}`];
      }
      // es2015 and later use the year directly
      return eslintGlobals[`es${ecmaVersion}`];
  }
}

/**
 * Add vars to `globals` from a globals preset.
 * @param preset - Globals preset object
 * @param globals - Globals object to add to
 */
function addGlobalsFromPreset(preset: Record<string, boolean>, globals: Globals) {
  for (const name in preset) {
    globals[name] = preset[name] ? "writable" : "readonly";
  }
}

export { RuleTesterShim as RuleTester };
