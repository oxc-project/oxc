// oxlint-disable no-console
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";
import { Session } from "node:inspector/promises";
import { parseSync, Visitor } from "../../src-js/index.js";
import { printSync } from "../../../../packages/codegen/dist/index.js";
import { loadCorpus } from "./corpus.mjs";

const [
  directory,
  variant = "baseline",
  mode = "raw",
  destination,
  limitText = "all",
  profile = "none",
] = process.argv.slice(2);
assert.ok(
  directory && destination,
  "worker.mjs CORPUS VARIANT MODE OUTPUT [LIMIT] [none|cpu|heap]",
);
assert.ok(["raw", "maps", "retain", "verify"].includes(mode));
const warmupRetention = process.env.OXC_BENCH_WARMUP ?? "matching";
assert.ok(["matching", "discard"].includes(warmupRetention));
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const { parseSyncRaw } = await import(
  variant === "poc"
    ? "../../src-js/raw-transfer/eager.js"
    : pathToFileURL(path.join(root, "target/raw-result", `${variant}.mjs`))
);
const { files, metadata } = loadCorpus(
  directory,
  limitText === "all" ? Infinity : Number(limitText),
);
const rawOptions = { experimentalRawTransfer: true };
const retained = [];
let checksum = 0;
let errors = 0;
let imports = 0;
let nodes = 0;
let outputBytes = 0;
const digest = createHash("sha256");
let currentImports;
const jsonImports = new Visitor({
  ImportDeclaration(node) {
    currentImports.push(node.source.value);
  },
});
const visit = new Visitor({
  ImportDeclaration(node) {
    currentImports.push(node.source.value);
    imports++;
    checksum += node.source.value.length;
  },
});
function countNodes(node) {
  if (node === null || typeof node !== "object") return;
  if (typeof node.type === "string") nodes++;
  for (const value of Object.values(node)) {
    if (Array.isArray(value)) {
      for (const child of value) countNodes(child);
    } else if (value && typeof value === "object") countNodes(value);
  }
}
function execute(file, measured = false) {
  const { filename, source, ts, jsx } = file;
  const result = parseSyncRaw(filename, source, rawOptions);
  checksum += result.program.body.length;
  errors += result.errors.length;
  if (mode === "retain" && measured) retained.push(result);
  if (mode === "maps") {
    const printed = printSync(result.program, {
      ts,
      jsx,
      sourcemap: true,
      sourceText: source,
      sourceFilename: filename,
    });
    checksum += printed.code.length + printed.map.mappings.length;
  }
  if (mode === "verify") {
    const json = parseSync(filename, source);
    for (const key of ["program", "module", "comments", "errors"]) {
      assert.deepEqual(result[key], json[key], `${filename}: ${key}`);
    }
    assert.equal(result.errors.length, 0, filename);
    const options = { ts, jsx, sourceText: source, sourceFilename: filename, sourcemap: true };
    const expected = printSync(json.program, options);
    const actual = printSync(result.program, options);
    assert.deepEqual(actual, expected, filename);
    const reparsed = parseSyncRaw(filename, actual.code, rawOptions);
    assert.equal(reparsed.errors.length, 0, `${filename}: reparse`);
    assert.equal(
      printSync(reparsed.program, { ts, jsx }).code,
      actual.code,
      `${filename}: fixed point`,
    );
    const expectedImports = [];
    const actualImports = [];
    currentImports = expectedImports;
    jsonImports.visit(json.program);
    currentImports = actualImports;
    visit.visit(result.program);
    assert.deepEqual(actualImports, expectedImports, `${filename}: ordered imports`);
    countNodes(result.program);
    digest.update(filename).update("\0").update(actual.code).update("\0");
    outputBytes += Buffer.byteLength(actual.code);
  }
}
if (mode !== "verify") {
  for (let pass = 0; pass < 2; pass++) {
    for (let i = 0; i < files.length; i += Math.max(1, Math.floor(files.length / 256))) {
      execute(files[i], mode === "retain" && warmupRetention === "matching");
    }
  }
}
retained.length = 0;
checksum = errors = 0;
globalThis.gc();
let session;
if (profile !== "none") {
  session = new Session();
  session.connect();
  if (profile === "heap") {
    await session.post("HeapProfiler.enable");
    await session.post("HeapProfiler.startSampling", {
      samplingInterval: 32768,
      includeObjectsCollectedByMajorGC: true,
      includeObjectsCollectedByMinorGC: true,
    });
  } else {
    await session.post("Profiler.enable");
    await session.post("Profiler.start");
  }
}
console.log("OXC_MEASUREMENT_START");
const cpu = process.cpuUsage();
const start = performance.now();
for (const file of files) execute(file, true);
const elapsedMilliseconds = performance.now() - start;
const cpuUsage = process.cpuUsage(cpu);
console.log("OXC_MEASUREMENT_END");
if (session) {
  const { profile: data } = await session.post(
    profile === "heap" ? "HeapProfiler.stopSampling" : "Profiler.stop",
  );
  fs.writeFileSync(
    destination + (profile === "heap" ? ".heapprofile" : ".cpuprofile"),
    JSON.stringify(data),
  );
  session.disconnect();
}
const heapBeforeGc = process.memoryUsage();
if (mode === "retain") {
  assert.equal(retained.length, files.length);
  for (const result of retained) {
    checksum += result.program.body.length + result.comments.length + result.errors.length;
  }
}
globalThis.gc();
const heapAfterGc = process.memoryUsage();
const report = {
  variant,
  mode,
  node: process.version,
  v8: process.versions.v8,
  flags: process.execArgv,
  profile,
  warmup: { passes: mode === "verify" ? 0 : 2, retention: warmupRetention },
  corpus: metadata,
  elapsedMilliseconds,
  cpuMilliseconds: (cpuUsage.user + cpuUsage.system) / 1000,
  maxRssMiB: process.resourceUsage().maxRSS / 1024,
  heapBeforeGc,
  heapAfterGc,
  retained: retained.length,
  checksum,
  errors,
  imports,
  nodes,
  outputBytes,
  outputSha256: digest.digest("hex"),
};
fs.writeFileSync(destination, JSON.stringify(report, null, 2) + "\n");
assert.equal(errors, 0);
process.stdout.write(JSON.stringify(report) + "\n", () => process.exit(0));
