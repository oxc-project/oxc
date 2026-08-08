import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import { getConfig, TARGET } from "./config.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");
const packageDirectory = process.cwd();
const packageJson = JSON.parse(
  fs.readFileSync(path.join(packageDirectory, "package.json"), "utf8"),
);
const config = getConfig(packageJson.name);
const outputDirectory = path.join(repositoryRoot, "target/emscripten", config.directory);
const targetReleaseDirectory = path.join(repositoryRoot, "target", TARGET, "release");

const packageRequire = createRequire(path.join(packageDirectory, "package.json"));
const emnapiDirectory = path.dirname(packageRequire.resolve("emnapi/package.json"));
const emnapiArchive = path.join(emnapiDirectory, "lib/wasm32-wasip1/libemnapi-basic-napi-rs.a");

function run(command, args, options = {}) {
  return execFileSync(command, args, {
    cwd: repositoryRoot,
    encoding: options.encoding,
    stdio: options.encoding ? ["ignore", "pipe", "pipe"] : "inherit",
  });
}

for (const command of ["emcc", "emnm"]) {
  try {
    run(command, ["--version"], { encoding: "utf8" });
  } catch {
    throw new Error(`${command} is required. Activate the pinned Emscripten SDK first.`);
  }
}

if (!fs.existsSync(emnapiArchive)) {
  throw new Error(`Emnapi basic archive not found: ${emnapiArchive}`);
}

fs.rmSync(outputDirectory, { recursive: true, force: true });
fs.mkdirSync(outputDirectory, { recursive: true });

run("cargo", [
  "rustc",
  "-p",
  config.crate,
  "--target",
  TARGET,
  "--release",
  "--lib",
  "--",
  "--crate-type",
  "staticlib",
]);

const archive = [
  path.join(targetReleaseDirectory, `lib${config.crate}.a`),
  path.join(targetReleaseDirectory, "deps", `lib${config.crate}.a`),
].find((candidate) => fs.existsSync(candidate));
if (!archive) {
  throw new Error(`Cargo did not produce a static archive for ${config.crate}`);
}

const symbols = run("emnm", ["--defined-only", "--extern-only", archive], {
  encoding: "utf8",
});
const registrationFunctions = [
  ...new Set(
    symbols
      .split("\n")
      .map((line) => line.trim().split(/\s+/).at(-1))
      .filter((name) => name?.startsWith("__napi_register__")),
  ),
].sort((left, right) => left.localeCompare(right));

if (registrationFunctions.length === 0) {
  throw new Error(`No N-API registration functions found in ${archive}`);
}

const exportedFunctions = [
  ...registrationFunctions.map((name) => `_${name}`),
  "_malloc",
  "_free",
  "_napi_register_wasm_v1",
  "_node_api_module_get_api_version_v1",
  "_emnapi_create_env",
  "_emnapi_delete_env",
];
const outputGlue = path.join(outputDirectory, "binding.mjs");

run("emcc", [
  "-O2",
  "--no-entry",
  archive,
  path.join(scriptDirectory, "module-api.c"),
  emnapiArchive,
  "-Wl,--import-undefined",
  "-Wno-js-compiler",
  "-sWASM_BIGINT=1",
  "-sALLOW_MEMORY_GROWTH=1",
  "-sALLOW_TABLE_GROWTH=1",
  "-sFILESYSTEM=0",
  "-sENVIRONMENT=worker",
  "-sMODULARIZE=1",
  "-sEXPORT_ES6=1",
  "-sSTACK_SIZE=8MB",
  `-sEXPORTED_FUNCTIONS=${JSON.stringify(exportedFunctions)}`,
  "-sWARN_ON_UNDEFINED_SYMBOLS=0",
  "-sERROR_ON_UNDEFINED_SYMBOLS=0",
  "-o",
  outputGlue,
]);

const outputWasm = path.join(outputDirectory, "binding.wasm");
const glue = fs.readFileSync(outputGlue, "utf8");
if (!glue.includes("ENVIRONMENT_IS_WORKER=true")) {
  throw new Error("Emscripten output is not specialized for a worker environment");
}
if (glue.includes("WorkerGlobalScope")) {
  throw new Error("Emscripten output contains browser-only WorkerGlobalScope detection");
}

const wasmModule = new WebAssembly.Module(fs.readFileSync(outputWasm));
const wasmImports = WebAssembly.Module.imports(wasmModule);
if (wasmImports.some(({ name }) => name === "memory")) {
  throw new Error("Emscripten artifact unexpectedly imports WebAssembly memory");
}

fs.writeFileSync(
  path.join(outputDirectory, "artifact.json"),
  `${JSON.stringify(
    {
      target: TARGET,
      rootPackage: config.packageName,
      bindingPackage: config.bindingPackage,
      version: packageJson.version,
      exports: config.exports,
      registrationFunctions,
    },
    null,
    2,
  )}\n`,
);

process.stdout.write(
  `Built ${config.bindingPackage} in ${path.relative(repositoryRoot, outputDirectory)}\n`,
);
