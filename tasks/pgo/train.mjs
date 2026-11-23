// oxlint-disable no-console, no-await-in-loop -- Complete native calls before reusing their input and profile buffers.
import { createRequire } from "node:module";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { isMainThread } from "node:worker_threads";
import { readShard } from "./corpus.mjs";
import { largeStackWorker, root } from "./utils.mjs";

async function trainRecord(name, binding, record, index) {
  const { filename, code, options } = record;
  // Async NAPI tasks run on libuv's fixed-size stacks, rather than our large
  // worker stack. Keep pathological generated inputs on the synchronous path.
  const asynchronous = index % 8 === 0 && code.length < 16 * 1024;
  if (name === "parser") {
    const parse = asynchronous ? binding.parse : binding.parseSync;
    const result = await parse(filename, code, {
      ...options,
      experimentalRawTransfer: true,
      range: index % 2 === 0,
      showSemanticErrors: index % 16 === 0,
    });
    return result.errors.length ? "diagnostics" : "trained";
  }
  if (name === "transform") {
    const transform = asynchronous ? binding.transform : binding.transformSync;
    let diagnostics = false;
    // Private auto-accessors are not supported by class-field lowering yet.
    // Conservatively keep accessor-containing inputs on esnext; every source
    // still trains TS/JSX lowering, parsing, diagnostics, and code generation.
    const targets = /\baccessor\b/.test(code) ? ["esnext"] : ["esnext", "es2015"];
    for (const target of targets) {
      const result = await transform(filename, code, {
        ...options,
        target,
        sourcemap: index % 2 === 0,
      });
      if (typeof result.code !== "string" || !Array.isArray(result.errors)) {
        throw new Error("Invalid transform result");
      }
      diagnostics ||= result.errors.length > 0;
    }
    return diagnostics ? "diagnostics" : "trained";
  }
  if (record.minify === null) return "skipped";
  // The minifier accepts plain JS. Preserve script/CommonJS versus module parsing
  // after TypeScript and JSX have been lowered during preparation.
  const suffix =
    options.sourceType === "module"
      ? ".mjs"
      : ["script", "commonjs"].includes(options.sourceType)
        ? ".cjs"
        : ".js";
  const minify = asynchronous ? binding.minify : binding.minifySync;
  const result = await minify(filename + suffix, record.minify ?? code, {
    sourcemap: index % 2 === 0,
  });
  if (typeof result.code !== "string" || !Array.isArray(result.errors)) {
    throw new Error(`Invalid minifier result: ${filename}`);
  }
  // Some ecosystem fixtures contain semantic errors,
  // e.g. repeated default exports, even after TS/JSX syntax has been removed.
  // Their diagnostics are useful training data, rather than harness failures.
  return result.errors.length ? "diagnostics" : "trained";
}

async function runBinding() {
  const [name, library, directory, shardFile] = process.argv.slice(2);
  // Loading the precise .node first prevents a loader fallback from disguising a
  // broken instrumented library as a successful WASM training run.
  createRequire(import.meta.url)(library);
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = library;
  process.env.NAPI_RS_FORCE_WASI = "false";
  const entry = name === "parser" ? "napi/parser/src-js/index.js" : `napi/${name}/index.js`;
  const binding = await import(pathToFileURL(join(root, entry)));
  // Without corpus arguments, smoke-test the optimized binding through its public API.
  if (directory === undefined) {
    const result = await trainRecord(
      name,
      binding,
      {
        filename: "smoke.ts",
        code: "export const answer: number = 42;",
        options: { lang: "ts", sourceType: "module" },
        minify: "export const answer = 42;",
      },
      0,
    );
    if (result !== "trained") throw new Error(`${name}: optimized binding smoke test failed`);
    return;
  }
  const manifest = JSON.parse(await readFile(join(directory, "manifest.json"), "utf8"));
  const shard = manifest.shards.find((item) => item.file === shardFile);
  if (!shard) throw new Error(`Unknown shard: ${shardFile}`);
  let processed = 0,
    skipped = 0,
    diagnostics = 0,
    downlevelSkipped = 0;
  for await (const record of readShard(directory, shard)) {
    try {
      const result = await trainRecord(name, binding, record, processed + skipped);
      if (name === "transform" && /\baccessor\b/.test(record.code)) downlevelSkipped++;
      if (result === "skipped") skipped++;
      else {
        processed++;
        if (result === "diagnostics") diagnostics++;
      }
    } catch (error) {
      throw new Error(`${name}: ${record.filename}`, { cause: error });
    }
  }
  console.log(JSON.stringify({ processed, skipped, diagnostics, downlevelSkipped }));
}

if (isMainThread) {
  await largeStackWorker(new URL(import.meta.url), { argv: process.argv.slice(2) });
} else {
  await runBinding();
}
