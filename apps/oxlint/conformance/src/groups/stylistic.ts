import { unindent } from "eslint-vitest-rule-tester";
import { RuleTester } from "../rule_tester.ts";

import type { TestGroup } from "../index.ts";
import type {
  ValidTestCase,
  InvalidTestCase,
  TestCase,
  LanguageOptions,
  ParserOptions,
} from "../rule_tester.ts";
import type { RuleTester as RuleTesterType } from "#oxlint/rule-tester";
import type { Rule } from "#oxlint/plugin";

type Config = RuleTesterType.Config;

const group: TestGroup = {
  name: "stylistic",

  submoduleName: "stylistic",
  testFilesDirPath: "packages/eslint-plugin/rules",

  transformTestFilename(filename: string) {
    // Each rule has its own directory in `packages/eslint-plugin/rules`.
    // Test files are in those subdirectories.
    // e.g. `packages/eslint-plugin/rules/indent/indent.test.ts`
    if (!filename.endsWith(".test.ts")) return null;
    const parts = filename.split("/");
    if (parts.length !== 2) return null;
    return parts[0];
  },

  prepare(require: NodeJS.Require, mock: (path: string, value: unknown) => void) {
    // Load the copy of `@typescript-eslint/parser` which is used by the test cases
    const tsEslintParser = require("@typescript-eslint/parser");

    // Mock `eslint-plugin-stylistic`'s rule tester, to use conformance `RuleTester`
    mock("../../../shared/test-utils/runner.ts", createStylisticRuleRunnerMock(tsEslintParser));
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Skip test cases which start with `/* eslint */` comments.
    // `RuleTester` does not support enabling other rules beyond the rule under test.
    if (code.match(/^\s*\/\*\s*eslint\s/)) return true;

    // Invalid code, cannot parse
    if (
      err.message === "Parsing failed" &&
      ruleName === "comma-spacing" &&
      ["let foo,", "let foo ,"].includes(code)
    ) {
      return true;
    }

    // Test cases should be parsed as Flow. AST is different when parsed as TS.
    if (
      ruleName === "object-curly-newline" &&
      test._parser?.specifier === "@babel/eslint-parser" &&
      (test.languageOptions?.parserOptions as any)?.babelOptions?.parserOpts?.plugins?.includes(
        "flow",
      ) &&
      [
        "function foo({\n a,\n b\n} : { a : string, b : string }) {}",
        "function foo({ a, b } : { a : string, b : string }) {}",
      ].includes(code)
    ) {
      return true;
    }

    // Invalid code, Oxc's AST does not match TS-ESLint
    if (
      ruleName === "space-infix-ops" &&
      compact(code) === "class Test {\n accessor optional?= false;\n }"
    ) {
      return true;
    }

    // Code contains `do` expressions which Oxc parser does not support
    if (
      ruleName === "jsx-indent" &&
      err.message === "Parsing failed" &&
      code.match(/^\s*<span>\s*\{\(?do \{/)
    ) {
      return true;
    }

    // Use custom language plugin
    if (
      (ruleName === "eol-last" || ruleName === "linebreak-style") &&
      (test as any).configs?.plugins !== undefined
    ) {
      return true;
    }

    // Faulty test cases - no message or message ID provided for the errors
    if (
      ruleName === "one-var-declaration-per-line" &&
      err.message === "Test error must specify either a `messageId` or `message`"
    ) {
      return true;
    }

    // `eslint-vitest-rule-tester` does not detect duplicate test cases, and tests do contain duplicates
    if (err.message === "Detected duplicate test case") {
      return true;
    }

    // TS parser incorrectly parses closing JSX tag with whitespace before `/`.
    // We don't skip these tests because we should be able to make them pass.
    /*
    if (
      ruleName === "jsx-tag-spacing" &&
      (code.startsWith('<App prop="foo">< /App>') ||
        code.startsWith('<div className="bar">< /div>;') ||
        compact(code).startsWith('<div className="bar"><\n /div>;'))
    ) {
      return true;
    }
    */

    return false;
  },

  ruleTesters: [],
  parsers: [
    { specifier: "@typescript-eslint/parser", lang: "ts" },
    { specifier: "@babel/eslint-parser", lang: "ts" },
  ],
};

export default group;

/**
 * Options passed to `run` function in `eslint-plugin-stylistic`'s rule tester module.
 */
interface StylisticRunOptions {
  name: string;
  rule: Rule;
  valid: (ValidTestCase | string)[];
  invalid: InvalidTestCase[];
  lang?: "js" | "ts" | "json" | "css";
  parserOptions?: StylisticParserOptions;
  recursive?: number | false;
  linterOptions?: unknown;
  configs?: unknown;
}

/**
 * `eslint-vitest-rule-tester` takes a single `parserOptions` object,
 * which comprises of options which end up in `languageOptions` and `languageOptions.parserOptions`.
 */
interface StylisticParserOptions extends ParserOptions {
  ecmaVersion?: number | "latest";
  sourceType?: "script" | "module" | "commonjs";
}

/**
 * `eslint-vitest-rule-tester` takes `parserOptions` as a property of test case.
 */
type StylisticTestCase = TestCase & { parserOptions?: StylisticParserOptions };

// List of keys that `StylisticRunOptions` can have.
// Must be kept in sync with properties of `StylisticRunOptions`.
// The type constraints enforce this.
const OPTIONS_KEYS_ARRAY = [
  "name",
  "rule",
  "valid",
  "invalid",
  "lang",
  "parserOptions",
  "recursive",
  "linterOptions",
  "configs",
] as const satisfies readonly (keyof StylisticRunOptions)[];

type MissingKeys = Exclude<keyof StylisticRunOptions, (typeof OPTIONS_KEYS_ARRAY)[number]>;
type KeysSet = MissingKeys extends never ? Set<string> : never;

const OPTIONS_KEYS: KeysSet = new Set(OPTIONS_KEYS_ARRAY);

/**
 * Create a module to replace `eslint-plugin-stylistic`'s rule runner module,
 * which presents the same API, but used conformance `RuleTester`.
 *
 * @param tsEslintParser - TSESLint parser module
 * @returns Module to replace `eslint-plugin-stylistic`'s rule runner module with
 */
function createStylisticRuleRunnerMock(tsEslintParser: any) {
  return {
    run(options: StylisticRunOptions) {
      // Validate options
      const extraKeys = Object.keys(options).filter((key) => !OPTIONS_KEYS.has(key));
      if (extraKeys.length > 0) {
        throw new Error(`Unexpected keys in options passed to \`run\`: ${extraKeys.join(", ")}`);
      }

      // Get parser from `lang` option.
      // If no `lang` option provided, default is TS.
      let parser: LanguageOptions["parser"] | null = null;
      const { lang } = options;
      if (lang === undefined || lang === "ts") {
        parser = tsEslintParser;
      } else if (lang !== "js") {
        // 'json' | 'css'.
        // Cause "custom parsers are not supported" error later on by setting an unknown parser.
        parser = { _unsupportedParser: true, lang } as LanguageOptions["parser"];
      }

      const config: Config = {};
      if (parser !== null || options.parserOptions != null) {
        const languageOptions =
          options.parserOptions == null ? {} : getLanguageOptions(options.parserOptions, undefined);
        if (parser !== null) languageOptions.parser = parser;
        config.languageOptions = languageOptions;
      }

      // Add other options to config.
      // These don't have any effect, but we add them so they appear in snapshot.
      if (options.linterOptions != null) (config as any).linterOptions = options.linterOptions;
      if (options.configs != null) (config as any).configs = options.configs;

      // Convert test cases
      const valid = options.valid ?? [];
      for (const test of valid) {
        if (typeof test === "object") modifyValidTestCase(test);
      }

      const invalid = options.invalid ?? [];
      for (const test of invalid) {
        modifyInvalidTestCase(test);
      }

      // Run tests
      const tester = new RuleTester(config);
      tester.run(options.name, options.rule, { valid, invalid });
    },

    unindent,
    $: unindent,
  };
}

/**
 * Modify a valid test case from `eslint-vitest-rule-tester`'s object shape to what `RuleTester` expects.
 * @param test - Test case
 */
function modifyValidTestCase(test: ValidTestCase) {
  modifyTestCase(test);
}

/**
 * Modify an invalid test case from `eslint-vitest-rule-tester`'s object shape to what `RuleTester` expects.
 * @param test - Test case
 */
function modifyInvalidTestCase(test: InvalidTestCase) {
  modifyTestCase(test);

  // Handle difference in `errors` property
  const { errors } = test;
  if (errors == null) {
    // `eslint-vitest-rule-tester` allows `errors` prop to be missing.
    // Set it to `__unknown__`. `RuleTester` will skip the check that errors match expected.
    (test.errors as unknown as string) = "__unknown__";
  } else if (Array.isArray(errors)) {
    // `eslint-vitest-rule-tester` treats strings as message IDs
    for (let i = 0; i < errors.length; i++) {
      const error = errors[i];
      if (typeof error === "string") errors[i] = { messageId: error };
    }
  }
}

/**
 * `eslint-vitest-rule-tester` takes `parserOptions` and `parser` as properties of test case.
 * Move them to be properties of `languageOptions`.
 * @param test - Test case
 */
function modifyTestCase(test: StylisticTestCase) {
  const { parserOptions } = test;
  if (parserOptions != null) {
    delete test.parserOptions;
    test.languageOptions = getLanguageOptions(parserOptions, test.languageOptions);
  }

  const parser = test.parser as LanguageOptions["parser"];
  if (parser != null) {
    if (test.languageOptions == null) test.languageOptions = {};
    test.languageOptions.parser = parser;
    delete test.parser;
  }
}

/**
 * Get language options from `StylisticParserOptions` and `LanguageOptions`.
 * @param parserOptions - Parser options from options passed to `run` function, or included as properties of test case
 * @param languageOptions - Language options from test case object
 * @returns `LanguageOptions` to add to config / test case, combining `parserOptions` and `languageOptions`
 */
function getLanguageOptions(
  parserOptions: StylisticParserOptions,
  languageOptions?: LanguageOptions,
): LanguageOptions {
  languageOptions = { ...languageOptions };
  parserOptions = { ...parserOptions };

  if (parserOptions.ecmaVersion != null) {
    languageOptions.ecmaVersion ??= parserOptions.ecmaVersion;
    delete parserOptions.ecmaVersion;
  }

  if (parserOptions.sourceType != null) {
    languageOptions.sourceType ??= parserOptions.sourceType;
    delete parserOptions.sourceType;
  }

  if (Object.keys(parserOptions).length !== 0) languageOptions.parserOptions = parserOptions;

  return languageOptions;
}

/**
 * Compact whitespace in code.
 * @param code - Code
 * @returns Code with whitespace compacted
 */
function compact(code: string): string {
  return code.trim().replace(/\n\s+/g, "\n ");
}
