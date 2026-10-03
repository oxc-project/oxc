import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { describe, it, setCurrentTest } from "../capture.ts";
import { SHOULD_SKIP_CODE } from "../filter.ts";
import repos from "../../repos.json" with { type: "json" };

import type { TestGroup } from "../index.ts";
import type { TestCase } from "../rule_tester.ts";

interface JestAssertion {
  fullName: string;
  ancestorTitles: string[];
  status: string;
  failureMessages: string[];
}

interface JestResult {
  testResults: { assertionResults: JestAssertion[]; message: string }[];
}

const group: TestGroup = {
  name: "nx",
  ...repos.nx,
  submoduleName: "nx",
  testFilesDirPath: "packages/eslint-plugin/src/rules",

  transformTestFilename(filename) {
    // dependency-checks needs a JSON parser. nx-plugin-checks tests helpers,
    // not the rule. Neither suite can establish JS plugin conformance.
    return filename === "enforce-module-boundaries.spec.ts" ? "enforce-module-boundaries" : null;
  },

  runTestFile(path) {
    // Keep Jest's hoisted mocks, fixture hooks, assertions and inline snapshot.
    // Only Linter.verify is replaced, in setup.cjs, with Oxlint's test linter.
    const root = join(dirname(path), "../../../..");
    const require = createRequire(path);
    const temporary = fs.mkdtempSync(join(tmpdir(), "oxlint-nx-"));
    const resultsPath = join(temporary, "results.json");
    const casesPath = join(temporary, "cases.jsonl");
    try {
      const result = spawnSync(
        process.execPath,
        [
          require.resolve("jest/bin/jest"),
          "--config",
          join(import.meta.dirname, "nx/config.cjs"),
          "--runInBand",
          "--json",
          "--outputFile",
          resultsPath,
          "--runTestsByPath",
          path,
        ],
        {
          cwd: root,
          encoding: "utf8",
          maxBuffer: 16 * 1024 * 1024,
          env: {
            ...process.env,
            NX_DAEMON: "false",
            NX_CONFORMANCE_ROOT: root,
            NX_CONFORMANCE_OXLINT: join(import.meta.dirname, "../../../dist/plugins-dev.js"),
            NX_CONFORMANCE_CASES: casesPath,
          },
        },
      );
      if (result.error) throw result.error;
      assert(result.signal === null, `Jest terminated with ${result.signal}`);
      assert(fs.existsSync(resultsPath), result.stderr || result.stdout);
      const report = JSON.parse(fs.readFileSync(resultsPath, "utf8")) as JestResult;
      assert.equal(report.testResults.length, 1);
      const suite = report.testResults[0];
      assert(suite.assertionResults.length > 0, suite.message);
      const cases = new Map<string, TestCase[]>();
      if (fs.existsSync(casesPath)) {
        for (const line of fs.readFileSync(casesPath, "utf8").trim().split("\n")) {
          const entry = JSON.parse(line) as { name: string; calls: TestCase[] };
          assert(!cases.has(entry.name), `Duplicate test name: ${entry.name}`);
          cases.set(entry.name, entry.calls);
        }
      }
      for (const assertion of suite.assertionResults) {
        const calls = cases.get(assertion.fullName);
        if (calls?.length === 1 && SHOULD_SKIP_CODE(calls[0].code)) continue;
        describe(assertion.ancestorTitles.join(" > "), () => {
          const test = calls?.[0] ?? { code: assertion.fullName };
          it(test.code, () => {
            setCurrentTest(test);
            assert.equal(
              calls?.length,
              1,
              `Expected exactly one Oxlint invocation per test\n${assertion.failureMessages.join("\n")}`,
            );
            assert.equal(assertion.status, "passed", assertion.failureMessages.join("\n"));
          });
        });
      }
      assert(
        result.status === 0 || suite.assertionResults.some((test) => test.status === "failed"),
        result.stderr || `Jest exited with ${result.status}`,
      );
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  },

  ruleTesters: [],
  parsers: [],
};

export default group;
