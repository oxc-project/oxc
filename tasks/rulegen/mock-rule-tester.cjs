/**
 * mock-rule-tester.cjs
 *
 * Hooks into Node's module resolution to intercept `require('eslint')`,
 * `require('@typescript-eslint/rule-tester')`, and related imports so that
 * evaluating an upstream test file captures the fully-resolved test arrays.
 *
 * Usage (from compare-tests.mjs):
 *   node --require ./mock-rule-tester.cjs <test-file>
 *
 * After evaluation the captured data is written to stdout as JSON.
 */

"use strict";

const Module = require("module");
const path = require("path");

// ── Global capture store ────────────────────────────────────────────────
// Populated by MockRuleTester.run()
global.__capturedTests = null;

// ── Mock RuleTester ─────────────────────────────────────────────────────

class MockRuleTester {
  constructor(_options) {}

  run(_name, _rule, tests) {
    const valid = (tests.valid || []).map((t) => (typeof t === "string" ? { code: t } : t));
    const invalid = (tests.invalid || []).map((t) => (typeof t === "string" ? { code: t } : t));
    global.__capturedTests = { valid, invalid };
  }
}

// Also act as a named export when destructured
MockRuleTester.RuleTester = MockRuleTester;

// ── Noop helpers commonly imported by test files ────────────────────────

const noopFn = () => {};

/**
 * Tagged template literal that just concatenates the parts — used by
 * typescript-eslint's `noFormat` and similar helpers.
 */
function taggedIdentity(strings, ...values) {
  let result = "";
  for (let i = 0; i < strings.length; i++) {
    result += strings[i];
    if (i < values.length) result += String(values[i]);
  }
  return result;
}

/**
 * Factory that returns a new MockRuleTester — used by typescript-eslint's
 * `createRuleTester()` and similar helpers.
 */
function createMockRuleTester() {
  return new MockRuleTester();
}

const noopObj = new Proxy(
  {},
  {
    get(_target, prop) {
      if (prop === "__esModule") return true;
      if (prop === "default") return noopObj;
      if (prop === "RuleTester") return MockRuleTester;
      // Tagged template literal helpers (noFormat, dedent, etc.)
      if (prop === "noFormat" || prop === "dedent") return taggedIdentity;
      // Factory functions that create rule testers
      if (prop === "createRuleTester" || prop === "createRuleTesterWithTypes")
        return createMockRuleTester;
      // getFixturesRootDir returns a valid directory path
      if (prop === "getFixturesRootDir")
        return () => path.join(require("os").tmpdir(), "oxc-fixtures-mock");
      // Return a function-like proxy for anything else
      return typeof prop === "string" ? noopFn : undefined;
    },
  },
);

// ── Module resolution hook ──────────────────────────────────────────────

const originalResolveFilename = Module._resolveFilename;

const INTERCEPTED = new Set([
  "eslint",
  "eslint/use-at-your-own-risk",
  "@typescript-eslint/rule-tester",
  "@typescript-eslint/utils",
  "@typescript-eslint/utils/ts-eslint",
  "@typescript-eslint/utils/eslint-utils",
  "@typescript-eslint/parser",
  "@typescript-eslint/typescript-estree",
]);

Module._resolveFilename = function (request, parent, isMain, options) {
  // Intercept known packages
  if (INTERCEPTED.has(request)) {
    // Return a path that we control – we register a fake module below
    return `__mock__/${request}`;
  }

  // Intercept relative paths that look like ESLint fixture testers
  // e.g. "../../fixtures/testers/rule-tester" in eslint's own test files
  if (
    request.includes("fixtures/testers/rule-tester") ||
    request.includes("fixtures/testers/RuleTester")
  ) {
    return "__mock__/eslint";
  }

  // Intercept direct rule-tester imports (ESLint internal)
  // e.g. "../../../lib/rule-tester/rule-tester" — exports the class directly
  if (request.includes("lib/rule-tester")) {
    return "__mock__/rule-tester-direct";
  }

  // Intercept relative imports to rule source files
  // e.g. "../../../lib/rules/no-empty-function" — the test imports the rule itself
  if (
    request.includes("/lib/rules/") ||
    request.includes("/src/rules/") ||
    request.includes("/rules/")
  ) {
    return "__mock__/rule-source";
  }

  // Intercept vitest/mocha/jest globals that some files import
  if (
    request === "vitest" ||
    request === "mocha" ||
    request === "jest" ||
    request === "node:test"
  ) {
    return `__mock__/${request}`;
  }

  // Catch-all for any unresolvable module: return a noop mock
  // This prevents crashes from test helpers, fixtures, etc.
  try {
    return originalResolveFilename.call(this, request, parent, isMain, options);
  } catch (err) {
    if (err.code === "MODULE_NOT_FOUND") {
      return "__mock__/fallback";
    }
    throw err;
  }
};

// ── Register fake modules in the cache ──────────────────────────────────

function registerMock(id, exports) {
  const m = new Module(id);
  m.loaded = true;
  m.exports = exports;
  Module._cache[id] = m;
}

// eslint — needs { RuleTester }
const eslintMock = {
  RuleTester: MockRuleTester,
  Linter: class {},
  Rule: {},
};
registerMock("__mock__/eslint", eslintMock);
registerMock("__mock__/eslint/use-at-your-own-risk", noopObj);

// Direct rule-tester import (module.exports = class RuleTester)
registerMock("__mock__/rule-tester-direct", MockRuleTester);

// Rule source — returns a noop rule object (the test file imports the rule
// but we only care about the test cases passed to RuleTester.run())
const noopRule = {
  meta: { type: "problem", schema: [], messages: {} },
  create: () => ({}),
};
registerMock("__mock__/rule-source", noopRule);

// Fallback for any unresolvable module
registerMock("__mock__/fallback", noopObj);

// @typescript-eslint packages
const tsRuleTesterMock = {
  RuleTester: MockRuleTester,
  noFormat: taggedIdentity,
  dedent: taggedIdentity,
  noopRuleTesterFn: noopFn,
};
registerMock("__mock__/@typescript-eslint/rule-tester", tsRuleTesterMock);
registerMock("__mock__/@typescript-eslint/utils", noopObj);
registerMock("__mock__/@typescript-eslint/utils/ts-eslint", {
  ...noopObj,
  RuleTester: MockRuleTester,
  ESLintUtils: noopObj,
});
registerMock("__mock__/@typescript-eslint/utils/eslint-utils", noopObj);
registerMock("__mock__/@typescript-eslint/parser", noopObj);
registerMock("__mock__/@typescript-eslint/typescript-estree", noopObj);

// vitest / mocha / jest stubs
const testFrameworkMock = {
  describe: (name, fn) => fn(),
  it: (name, fn) => {},
  test: (name, fn) => {},
  expect: () =>
    new Proxy(
      {},
      {
        get() {
          return noopFn;
        },
      },
    ),
  beforeAll: noopFn,
  afterAll: noopFn,
  beforeEach: noopFn,
  afterEach: noopFn,
  vi: { fn: noopFn, mock: noopFn },
};
registerMock("__mock__/vitest", testFrameworkMock);
registerMock("__mock__/mocha", testFrameworkMock);
registerMock("__mock__/jest", testFrameworkMock);
registerMock("__mock__/node:test", testFrameworkMock);
