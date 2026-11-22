import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { availableParallelism } from "node:os";
import { extname, relative, resolve } from "node:path";
import { parseArgs } from "node:util";
import { Worker } from "node:worker_threads";

import { exclusion } from "./selection.ts";
import { compareResults } from "./results.ts";
import type { EvaluatorResult } from "./evaluator.ts";
import type { Fixture } from "./worker.ts";

const root = resolve(import.meta.dirname, "../../..");
const fixturesRoot = resolve(root, "crates/oxc_react_compiler/fixtures");
const extensions = new Set([".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts"]);
const { values } = parseArgs({ options: { filter: { type: "string" } } });
const inputs: { path: string; source: string }[] = [];
const excluded = new Map<string, number>();

for (const entry of readdirSync(fixturesRoot, { recursive: true, withFileTypes: true })) {
  if (!entry.isFile() || !extensions.has(extname(entry.name))) continue;
  const filename = resolve(entry.parentPath, entry.name);
  const path = relative(fixturesRoot, filename).replaceAll("\\", "/");
  if (values.filter && !path.includes(values.filter)) continue;
  const source = readFileSync(filename, "utf8").replaceAll(/\r\n?/g, "\n");
  const reason = exclusion(path, source);
  if (reason !== null) {
    excluded.set(reason, (excluded.get(reason) ?? 0) + 1);
    if (values.filter) process.stdout.write(`EXCLUDED ${path}: ${reason}\n`);
  } else {
    inputs.push({ path, source });
  }
}
inputs.sort((a, b) => a.path.localeCompare(b.path));
if (inputs.length === 0) throw new Error("No runnable fixtures selected");

const compilation = spawnSync(
  "cargo",
  ["run", "--quiet", "-p", "oxc_sprout", "--profile", "coverage"],
  {
    cwd: root,
    input: inputs.map((input) => JSON.stringify(input)).join("\n") + "\n",
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  },
);
if (compilation.stderr) process.stderr.write(compilation.stderr);
if (compilation.error) throw compilation.error;
if (compilation.status !== 0) throw new Error("Sprout compilation bridge failed");
const fixtures: Fixture[] = compilation.stdout
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
if (fixtures.length !== inputs.length) throw new Error("Compilation bridge omitted fixtures");

let cursor = 0;
let passed = 0;
let failed = 0;
async function runWorker(): Promise<void> {
  // Each lane has its own React module cache, including warning deduplication and
  // dispatcher globals. Both lanes see fixtures in the same order.
  const original = new Worker(new URL("./worker.ts", import.meta.url));
  const compiled = new Worker(new URL("./worker.ts", import.meta.url));
  function evaluate(
    worker: Worker,
    fixture: Fixture,
    isCompiled: boolean,
  ): Promise<EvaluatorResult> {
    return new Promise((done, reject) => {
      const finish = (result: EvaluatorResult) => {
        clearTimeout(timer);
        worker.removeAllListeners("message");
        worker.removeAllListeners("error");
        done(result);
      };
      const timer = setTimeout(
        () => reject(new Error(`Evaluation timed out: ${fixture.path}`)),
        10_000,
      );
      worker.once("message", finish);
      worker.once("error", (error) => {
        clearTimeout(timer);
        reject(error);
      });
      worker.postMessage({ fixture, compiled: isCompiled });
    });
  }
  try {
    while (cursor < fixtures.length) {
      const fixture = fixtures[cursor++]!;
      // oxlint-disable-next-line no-await-in-loop -- Each worker must finish its fixture before accepting another.
      const [originalResult, compiledResult] = await Promise.all([
        fixture.source.includes("@disableNonForgetInSprout")
          ? null
          : evaluate(original, fixture, false),
        evaluate(compiled, fixture, true),
      ]);
      const result = compareResults(originalResult, compiledResult);
      if (result.kind === "success") {
        passed++;
        if (values.filter) process.stdout.write(`PASS ${fixture.path}\n${result.value}\n`);
      } else {
        failed++;
        process.stderr.write(`FAIL ${fixture.path}\n${result.value}\n\n`);
      }
    }
  } finally {
    await Promise.all([original.terminate(), compiled.terminate()]);
  }
}
await Promise.all(
  Array.from({ length: Math.min(2, availableParallelism(), fixtures.length) }, runWorker),
);
process.stdout.write(
  `Sprout: ${passed} passed, ${failed} failed, ${[...excluded.values()].reduce((a, b) => a + b, 0)} excluded\n`,
);
for (const [reason, count] of [...excluded].sort(([a], [b]) => a.localeCompare(b))) {
  process.stdout.write(`  ${count}: ${reason}\n`);
}
if (failed > 0) process.exitCode = 1;
