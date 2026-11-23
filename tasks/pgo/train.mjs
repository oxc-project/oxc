// oxlint-disable no-console, no-await-in-loop -- Complete native calls before reusing their input and profile buffers.
import { createRequire } from "node:module";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { isMainThread } from "node:worker_threads";
import { readShard } from "./corpus.mjs";
import { largeStackWorker, root } from "./utils.mjs";

const parserJson = { name: "json", options: {} };
const transformDefault = { name: "default", options: {} };
const transformSourcemap = { name: "sourcemap", options: { sourcemap: true } };
const jsxLanguages = ["jsx", "tsx"];
const typescriptLanguages = ["ts", "tsx"];
const minifyTarget = { compress: { target: "es2019" }, mangle: { keepNames: true } };
const schedules = {
  parser: [
    parserJson,
    { name: "raw", options: { experimentalRawTransfer: true } },
    parserJson,
    { name: "json-ranges", options: { range: true, preserveParens: false } },
    parserJson,
    parserJson,
    { name: "json-ts-ast", languages: ["js", "jsx"], options: { astType: "ts" } },
    { name: "raw-semantic", options: { experimentalRawTransfer: true, showSemanticErrors: true } },
  ],
  transform: [
    transformSourcemap,
    transformDefault,
    { name: "browser", options: { target: "es2020", sourcemap: true } },
    transformSourcemap,
    { name: "legacy", options: { target: "es2015" } },
    { name: "declarations", languages: typescriptLanguages, options: { sourcemap: true } },
    transformSourcemap,
    {
      name: "jsx-development",
      languages: jsxLanguages,
      options: { jsx: { development: true }, sourcemap: true },
    },
    { name: "jsx-preserve", languages: jsxLanguages, options: { jsx: "preserve" } },
    {
      name: "declarations-with-js",
      languages: typescriptLanguages,
      options: { typescript: { declaration: { stripInternal: true } }, sourcemap: true },
    },
    {
      name: "explicit-type-imports",
      languages: typescriptLanguages,
      options: { typescript: { onlyRemoveTypeImports: true } },
    },
  ],
  minify: [
    { name: "default", options: {} },
    { name: "sourcemap", options: { sourcemap: true } },
    { name: "compress-target", options: minifyTarget },
  ],
};
const bundleSchedule = [
  {
    name: "bundle-minify",
    options: { ...minifyTarget, codegen: { legalComments: "eof" }, sourcemap: true },
  },
  {
    name: "bundle-readable",
    options: {
      compress: false,
      mangle: false,
      codegen: { removeWhitespace: false, legalComments: "inline" },
      sourcemap: true,
    },
  },
  {
    name: "bundle-ascii",
    options: {
      compress: { target: "es2015" },
      codegen: { asciiOnly: true, legalComments: "inline" },
      sourcemap: true,
    },
  },
];

async function trainRecord(name, binding, record, workload, asynchronous, smoke = false) {
  const { filename, code, options } = record;
  if (workload.languages && !workload.languages.includes(options.lang)) {
    workload = name === "parser" ? parserJson : transformSourcemap;
  }
  let result;
  let downlevelSkipped = false;
  if (name === "parser") {
    const parse = asynchronous ? binding.parse : binding.parseSync;
    result = await parse(filename, code, { ...options, ...workload.options });
    // The default wrapper already takes the serialized AST from Rust. Only
    // materialize the JSON AST in the smoke test, since V8 isn't instrumented.
    if (smoke && !result.errors.length && result.program.type !== "Program") {
      throw new Error("Invalid parser AST");
    }
  } else if (name === "transform") {
    if (workload.name === "declarations") {
      const emit = asynchronous ? binding.isolatedDeclaration : binding.isolatedDeclarationSync;
      result = await emit(filename, code, workload.options);
    } else {
      // Keep private auto-accessors away from targets that can lower class fields.
      if (workload.options.target && /\baccessor\b/.test(code)) {
        downlevelSkipped = true;
        workload = workload.options.sourcemap ? transformSourcemap : transformDefault;
      }
      const transform = asynchronous ? binding.transform : binding.transformSync;
      result = await transform(filename, code, { ...options, ...workload.options });
    }
  } else {
    // Preserve script/CommonJS versus module parsing after TS and JSX lowering.
    const module = options.sourceType === "module";
    const suffix = module
      ? ".mjs"
      : ["script", "commonjs"].includes(options.sourceType)
        ? ".cjs"
        : ".js";
    const minify = asynchronous ? binding.minify : binding.minifySync;
    const minifyOptions = module ? { ...workload.options, module: true } : workload.options;
    result = await minify(filename + suffix, record.minify ?? code, minifyOptions);
  }
  if (!Array.isArray(result.errors) || (name !== "parser" && typeof result.code !== "string")) {
    throw new Error(`Invalid ${name} result for ${workload.name}`);
  }
  if (smoke && !result.errors.length) {
    if (workload.options.sourcemap && !result.map) {
      throw new Error(`Missing ${name} source map for ${workload.name}`);
    }
    if (
      workload.name === "declarations-with-js"
      && (!result.code || !result.declaration || !result.declarationMap)
    ) {
      throw new Error("Missing combined declaration output or source map");
    }
    if (workload.name === "jsx-preserve" && !result.code.includes("<span>")) {
      throw new Error("Missing preserved JSX");
    }
    if (workload.name === "bundle-readable" && !result.code.includes("@license")) {
      throw new Error("Missing inline license comment");
    }
    if (workload.name === "bundle-ascii" && result.code.includes("é")) {
      throw new Error("Expected ASCII code");
    }
  }
  return {
    workload: `${workload.name}-${asynchronous ? "async" : "sync"}`,
    diagnostics: result.errors.length > 0,
    downlevelSkipped,
  };
}

async function runBinding() {
  const [name, library, directory, shardFile, offset = "0"] = process.argv.slice(2);
  const schedule = schedules[name];
  // Loading the precise .node first prevents a loader fallback from disguising a
  // broken instrumented library as a successful WASM training run.
  createRequire(import.meta.url)(library);
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = library;
  process.env.NAPI_RS_FORCE_WASI = "false";
  const entry = name === "parser" ? "napi/parser/src-js/index.js" : `napi/${name}/index.js`;
  const binding = await import(pathToFileURL(join(root, entry)));
  // Without corpus arguments, smoke-test each workload through the public API.
  if (directory === undefined) {
    const record =
      name === "parser"
        ? {
            filename: "smoke.jsx",
            code: "export function Answer() { return <span>{42}</span>; }",
            options: { lang: "jsx", sourceType: "module" },
          }
        : {
            filename: "smoke.tsx",
            code: "export function Answer(): JSX.Element { return <span>{42}</span>; }",
            options: { lang: "tsx", sourceType: "module" },
            minify: "/*! @license smoke */\nexport const answer = ({ value: 'é' })?.value;",
          };
    const workloads = name === "minify" ? [...schedule, ...bundleSchedule] : schedule;
    for (const workload of new Set(workloads)) {
      for (const asynchronous of [false, true]) {
        const result = await trainRecord(name, binding, record, workload, asynchronous, true);
        if (result.diagnostics) {
          throw new Error(`${name}: smoke test failed for ${result.workload}`);
        }
      }
    }
    return;
  }
  const manifest = JSON.parse(await readFile(join(directory, "manifest.json"), "utf8"));
  const shard = manifest.shards.find((item) => item.file === shardFile);
  if (!shard) throw new Error(`Unknown shard: ${shardFile}`);
  let processed = 0,
    skipped = 0,
    diagnostics = 0,
    downlevelSkipped = 0;
  const workloads = {};
  const start = Number(offset);
  for await (const record of readShard(directory, shard)) {
    const index = start + processed + skipped;
    if (name === "minify" && record.minify === null) {
      skipped++;
      continue;
    }
    try {
      const code = name === "minify" ? (record.minify ?? record.code) : record.code;
      // Sample independently of the workload schedule. Keep larger inputs on
      // the training worker's stack instead of libuv's fixed-size stacks.
      const asynchronous = index % 7 === 0 && code.length < 16 * 1024;
      const workload =
        name === "minify" && record.bundleIndex !== undefined
          ? bundleSchedule[record.bundleIndex % bundleSchedule.length]
          : schedule[index % schedule.length];
      const result = await trainRecord(name, binding, record, workload, asynchronous);
      processed++;
      if (result.diagnostics) diagnostics++;
      if (result.downlevelSkipped) downlevelSkipped++;
      workloads[result.workload] = (workloads[result.workload] ?? 0) + 1;
    } catch (error) {
      throw new Error(`${name}: ${record.filename}`, { cause: error });
    }
  }
  console.log(JSON.stringify({ processed, skipped, diagnostics, downlevelSkipped, workloads }));
}

if (isMainThread) {
  await largeStackWorker(new URL("./train.mjs", import.meta.url), { argv: process.argv.slice(2) });
} else {
  await runBinding();
}
