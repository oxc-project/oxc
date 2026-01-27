import { join as pathJoin } from "node:path";
import { isAbsolute as isUnixAbsolute } from "node:path/posix";
import { isAbsolute as isWinAbsolute, resolve as resolveWin32 } from "node:path/win32";
import globals from "globals";
// @ts-expect-error - No declaration file
import merge from "lodash.merge";
import { RuleTester } from "../rule_tester.ts";

import type { TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase, TestCases, ValidTestCase } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugin";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "sonarjs",

  submoduleName: "sonarjs",
  testFilesDirPath: "packages/jsts/src/rules",

  transformTestFilename(filename: string) {
    // Each rule has its own directory in `packages/jsts/src/rules`, each dir named `Sxxx` e.g. `S100`.
    // Test files are in those subdirectories.
    // e.g. `packages/jsts/src/rules/S100/unit.test.ts`
    if (!filename.endsWith(".test.ts")) return null;
    const parts = filename.split("/");
    if (parts.length !== 2) return null;

    const name = parts[0];
    if (!name.match(/^S\d+$/)) return null;
    return name;
  },

  prepare(require: NodeJS.Require, mock: (path: string, value: unknown) => void) {
    // Load the copy of `@typescript-eslint/parser` which is used by the test cases
    const tsEslintParser = require("@typescript-eslint/parser") as TSEslintParser;

    // Mock SonarJS's rule tester, to use conformance `RuleTester`
    mock(
      "../../tests/tools/testers/rule-tester.ts",
      createSonarJsRuleTesterModuleMock(tsEslintParser),
    );
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Skip test cases which include `// eslint-disable` comments.
    // These are not handled by `RuleTester`.
    if (code.match(/\/[/*]\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

    // Faulty test cases - valid test cases with `errors` property
    if (ruleName === "S2755" && err.message === "Valid test case must not have `errors` property") {
      return true;
    }

    // Only valid code in ES3, and Oxc parser does not support ES3
    if (
      ruleName === "S1527" &&
      code.includes("var implements;") &&
      (test.languageOptions?.parserOptions as any)?.ecmaVersion === 3 &&
      err.message === "Parsing failed"
    ) {
      return true;
    }

    // TODO: Delete this exclusion
    if (ruleName === "S125") {
      // "Sections of code should not be commented out"
      // Rule uses `context.languageOptions.parser.parse` to parse code. Unsupported.
      return true;
    }

    return false;
  },

  ruleTesters: [],

  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;

function mergeWithoutCloningParser(
  base: LanguageOptions,
  options?: LanguageOptions,
): LanguageOptions {
  const merged = merge({}, base, options);

  if (options && Object.hasOwn(options, "parser")) {
    merged.parser = options.parser;
  } else if (Object.hasOwn(base, "parser")) {
    merged.parser = base.parser;
  }

  return merged;
}

function createSonarJsRuleTesterModuleMock(tsEslintParser: TSEslintParser) {
  class NoTypeCheckingRuleTester extends DefaultParserRuleTester {
    constructor(options?: LanguageOptions) {
      super(mergeWithoutCloningParser({ parser: tsEslintParser }, options));
    }
  }

  return { DefaultParserRuleTester, NoTypeCheckingRuleTester };
}

const baseLanguageOptions = {
  ecmaVersion: 2022,
  sourceType: "module",
  globals: {
    ...globals.es2025,
  },
  parserOptions: {
    // The single run makes that typescript-eslint uses normal programs instead of use watch programs
    // We need watch programs for replace contents of the placeholder file in the program
    // https://github.com/typescript-eslint/typescript-eslint/blob/d24a82854d06089cbd2a8801f2982fd4781f3701/packages/typescript-estree/src/parseSettings/inferSingleRun.ts#L44
    disallowAutomaticSingleRunInference: true,
    ecmaFeatures: {
      jsx: true,
    },
  },
} as LanguageOptions;

const isWindows = process.platform === "win32";

const placeHolderFilePath = pathJoin(
  toUnixPath(import.meta.dirname),
  "fixtures",
  "placeholder.tsx",
);

class DefaultParserRuleTester extends RuleTester {
  constructor(options?: LanguageOptions) {
    super({
      // files: ["**/*.js", "**/*.jsx", "**/*.ts", "**/*.tsx"],
      languageOptions: mergeWithoutCloningParser(baseLanguageOptions, options),
    });
  }

  run(name: string, rule: Rule, tests: TestCases): void {
    for (const testCase of tests.valid as ValidTestCase[]) {
      testCase.filename ??= placeHolderFilePath;
    }
    for (const testCase of tests.invalid) {
      testCase.filename ??= placeHolderFilePath;
    }
    super.run(name, rule, tests);
  }
}

function toUnixPath(filePath: string) {
  if (isWindows && isAbsolutePath(filePath)) {
    // On Windows, resolve to add drive letter if missing
    filePath = resolveWin32(filePath);
  }
  return filePath.replaceAll(/[\\/]+/g, "/");
}

function isAbsolutePath(path: string) {
  // Check for Windows drive letter (e.g., 'c:', 'C:', 'D:')
  // Node's isAbsolute considers 'c:' as relative (drive-relative), but we treat it as absolute
  if (/^[a-zA-Z]:/.test(path)) {
    return true;
  }
  return isUnixAbsolute(path) || isWinAbsolute(path.replaceAll(/[\\/]+/g, "\\"));
}
