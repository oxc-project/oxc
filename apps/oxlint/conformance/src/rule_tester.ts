/*
 * Shim of `RuleTester` class.
 */

// @ts-expect-error - internal module of ESLint with no types
import eslintGlobals from "../submodules/eslint/conf/globals.js";
import { createRequire } from "node:module";
import { RuleTester } from "#oxlint";
import { describe, it } from "./capture.ts";
import { ESLINT_RULES_TESTS_DIR_PATH } from "./run.ts";
import { FILTER_ONLY_CODE } from "./filter.ts";

import type { Rule } from "#oxlint";

type DescribeFn = RuleTester.DescribeFn;
type ItFn = RuleTester.ItFn;
type TestCases = RuleTester.TestCases;
type ValidTestCase = RuleTester.ValidTestCase;
type InvalidTestCase = RuleTester.InvalidTestCase;
type LanguageOptions = RuleTester.LanguageOptions;
type Globals = RuleTester.Globals;
export type TestCase = ValidTestCase | InvalidTestCase;

const { isArray } = Array;

/**
 * Language options config, with `parser` and `ecmaVersion` properties.
 * This is a copy of `RuleTester`'s internal type of the same name.
 */
interface LanguageOptionsInternal extends LanguageOptions {
  ecmaVersion?: number | "latest";
  parser?: {
    parse?: (code: string, options?: Record<string, unknown>) => unknown;
    parseForESLint?: (code: string, options?: Record<string, unknown>) => unknown;
  };
}

// Get `@typescript-eslint/parser` module.
// Load the instance which would be loaded by files in ESLint's `tests/lib/rules` directory.
const require = createRequire(ESLINT_RULES_TESTS_DIR_PATH);
const tsEslintParser = require("@typescript-eslint/parser");

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

  // Apply filter to test cases.
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    if (FILTER_ONLY_CODE !== null) {
      const codeMatchesFilter = isArray(FILTER_ONLY_CODE)
        ? (code: string) => FILTER_ONLY_CODE!.includes(code)
        : (code: string) => code === FILTER_ONLY_CODE;

      tests = {
        valid: tests.valid.filter((test) => {
          const code = typeof test === "string" ? test : test.code;
          return codeMatchesFilter(code);
        }),
        invalid: tests.invalid.filter((test) => codeMatchesFilter(test.code)),
      };
    }

    super.run(ruleName, rule, tests);
  }
}

// Register hook to modify test cases before they are run.
// `registerModifyTestCaseHook` is only present in debug builds, so it's not part of the `RuleTester` type def.
(RuleTester as any).registerModifyTestCaseHook(modifyTestCase);

function modifyTestCase(test: TestCase): void {
  // Enable ESLint compat mode.
  // This makes `RuleTester` adjust column indexes in diagnostics to match ESLint's behavior.
  test.eslintCompat = true;

  // Ignore parsing errors. ESLint's test cases include invalid code.
  const languageOptions = { ...test.languageOptions } as LanguageOptionsInternal;
  test.languageOptions = languageOptions;

  const parserOptions = { ...languageOptions.parserOptions };
  languageOptions.parserOptions = parserOptions;

  parserOptions.ignoreNonFatalErrors = true;

  // Build the globals object
  languageOptions.globals = getGlobals(languageOptions);

  // Disable env - we're providing all globals explicitly via `languageOptions.globals`.
  // Setting `env` to empty object prevents the default "builtin" env from being applied.
  languageOptions.env = {};

  // If test case uses `@typescript-eslint/parser` as parser, set `parserOptions.lang = "ts"`
  if (languageOptions.parser === tsEslintParser) {
    delete languageOptions.parser;
    parserOptions.lang = parserOptions.ecmaFeatures?.jsx === true ? "tsx" : "ts";
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
