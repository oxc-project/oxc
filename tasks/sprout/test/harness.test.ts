import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";

import { runSprout } from "../src/index.ts";
import { exclusion } from "../src/selection.ts";
import { evaluatorCode } from "../src/transform.ts";
import type { EvaluatorResult } from "../src/evaluator.ts";

const root = new URL("./fixtures/", import.meta.url);
for (const name of readdirSync(root).sort()) {
  const fixture = JSON.parse(readFileSync(new URL(name, root), "utf8"));
  void test(name, async () => {
    const result = await runSprout(
      evaluatorCode(fixture.original, name + ".tsx"),
      evaluatorCode(fixture.compiled, name + ".tsx"),
    );
    assert.equal(result.kind, fixture.expected, result.value);
  });
}

void test("unhandled fixture rejections fail before the worker accepts another fixture", async () => {
  const worker = new Worker(new URL("../src/worker.ts", import.meta.url));
  try {
    const evaluate = (source: string) =>
      new Promise<EvaluatorResult>((resolve, reject) => {
        worker.once("message", resolve);
        worker.once("error", reject);
        worker.postMessage({
          fixture: { path: "async.js", source, code: source, diagnostics: [] },
          compiled: false,
        });
      });
    const result = await evaluate(`
      function Component() { Promise.reject(new Error("fixture rejection")); return null; }
      export const FIXTURE_ENTRYPOINT = {fn: Component, params: []};
    `);
    assert.equal(result.kind, "UnexpectedError");
    assert.match(result.value, /Unhandled fixture rejection: Error: fixture rejection/);
    const next = await evaluate(`
      function Component() { return "next fixture"; }
      export const FIXTURE_ENTRYPOINT = {fn: Component, params: []};
    `);
    assert.equal(next.kind, "ok");
    assert.deepEqual(next.logs, []);
  } finally {
    await worker.terminate();
  }
});

void test("selection requires real entrypoints and preserves explicit exclusions", () => {
  assert.equal(
    exclusion("test.js", "export const FIXTURE_ENTRYPOINT={fn:Component,params:[]}"),
    null,
  );
  assert.equal(exclusion("test.js", "export const FIXTURE_ENTRYPOINT={fn:Component}"), null);
  assert.throws(
    () =>
      exclusion(
        "test.js",
        "export const FIXTURE_ENTRYPOINT={fn:Component,params:['TodoAdd'],isComponent:'TodoAdd'}",
      ),
    /replace the TodoAdd/,
  );
  assert.throws(
    () => exclusion("test.js", "// FIXTURE_ENTRYPOINT\nfunction Component(){}"),
    /export a usable FIXTURE_ENTRYPOINT/,
  );
  assert.throws(() => exclusion("new.js", "function Component(){}"), /new.js: export a usable/);
  assert.equal(
    exclusion(
      "script.js",
      "// @script\nexports = {FIXTURE_ENTRYPOINT: {fn: Component, params:[]}}",
    ),
    null,
  );
  assert.match(exclusion("loop-unused-let.js", "")!, /loop has no exit/);
  assert.equal(exclusion("nested/error.invalid-test.js", ""), "Expected compiler diagnostics");
  assert.equal(exclusion("test.js", '// @outputMode:"lint"'), "Lint-only fixture");
});

void test("an empty filter fails without compiling", () => {
  const result = spawnSync(
    process.execPath,
    [
      fileURLToPath(new URL("../src/runner.ts", import.meta.url)),
      "--filter",
      "__sprout_no_such_fixture__",
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1);
  assert.match(result.stderr, /No runnable fixtures selected/);
});
