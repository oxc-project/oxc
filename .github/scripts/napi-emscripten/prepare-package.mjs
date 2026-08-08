import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

function parseArguments() {
  const values = {};
  for (let index = 2; index < process.argv.length; index += 2) {
    const key = process.argv[index];
    const value = process.argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      throw new Error("Expected --package-dir, --artifacts-dir and --npm-dir arguments");
    }
    values[key.slice(2)] = path.resolve(value);
  }
  for (const key of ["package-dir", "artifacts-dir", "npm-dir"]) {
    if (!values[key]) throw new Error(`Missing --${key}`);
  }
  return values;
}

function findArtifact(directory, packageName) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const result = findArtifact(entryPath, packageName);
      if (result) return result;
    } else if (entry.name === "artifact.json") {
      const metadata = JSON.parse(fs.readFileSync(entryPath, "utf8"));
      if (metadata.rootPackage === packageName) {
        return { directory: directory, metadata };
      }
    }
  }
  return undefined;
}

function packageEntry(exports, expectedRegistrationCount) {
  const namedExports = exports.map((name) => `export const ${name} = binding.${name};`).join("\n");
  return `import { createNapiModule } from "@emnapi/core";
import { asyncWork, tsfn } from "@emnapi/core/plugins";
import { getDefaultContext } from "@emnapi/runtime";
import createEmscriptenModule from "./binding.mjs";
import wasmModule from "./binding.wasm";

const napiModule = createNapiModule({
  context: getDefaultContext(),
  asyncWorkPoolSize: 0,
  plugins: [asyncWork, tsfn],
});

let wasmInstance;
let initializationRandomState = 0x6d2b79f5;
let initializing = true;
const emscriptenModule = await createEmscriptenModule({
  instantiateWasm(imports, done) {
    imports.env = {
      ...imports.env,
      ...napiModule.imports.env,
      ...napiModule.imports.napi,
      ...napiModule.imports.emnapi,
    };
    const systemRandomGet = imports.wasi_snapshot_preview1.random_get;
    imports.wasi_snapshot_preview1.random_get = (buffer, size) => {
      if (!initializing) return systemRandomGet(buffer, size);

      // workerd disallows random generation at module scope. NAPI-RS only requests
      // these bytes while registering its fixed export metadata; later requests use
      // Emscripten's system random implementation.
      const bytes = new Uint8Array(wasmInstance.exports.memory.buffer, buffer, size);
      for (let index = 0; index < bytes.length; index++) {
        initializationRandomState ^= initializationRandomState << 13;
        initializationRandomState ^= initializationRandomState >>> 17;
        initializationRandomState ^= initializationRandomState << 5;
        bytes[index] = initializationRandomState;
      }
      return 0;
    };
    wasmInstance = new WebAssembly.Instance(wasmModule, imports);
    done(wasmInstance, wasmModule);
    return wasmInstance.exports;
  },
});

const registrationFunctions = Object.keys(emscriptenModule).filter((name) =>
  name.startsWith("___napi_register__"),
);
if (registrationFunctions.length !== ${expectedRegistrationCount}) {
  throw new Error(
    \`Expected ${expectedRegistrationCount} N-API registration functions, found \${registrationFunctions.length}\`,
  );
}
for (const name of registrationFunctions) emscriptenModule[name]();

const binding = napiModule.init({ instance: wasmInstance, module: wasmModule });
initializing = false;

${namedExports}
export default binding;
`;
}

const args = parseArguments();
const rootManifestPath = path.join(args["package-dir"], "package.json");
const rootManifest = JSON.parse(fs.readFileSync(rootManifestPath, "utf8"));
const artifact = findArtifact(args["artifacts-dir"], rootManifest.name);
if (!artifact) {
  throw new Error(`No Emscripten artifact found for ${rootManifest.name}`);
}
if (artifact.metadata.version !== rootManifest.version) {
  throw new Error(
    `Emscripten artifact version ${artifact.metadata.version} does not match ${rootManifest.version}`,
  );
}

const packageRequire = createRequire(rootManifestPath);
const coreVersion = packageRequire("@emnapi/core/package.json").version;
const runtimeVersion = packageRequire("@emnapi/runtime/package.json").version;
const destination = path.join(args["npm-dir"], "wasm32-emscripten");
fs.rmSync(destination, { recursive: true, force: true });
fs.mkdirSync(destination, { recursive: true });
fs.copyFileSync(
  path.join(artifact.directory, "binding.mjs"),
  path.join(destination, "binding.mjs"),
);
fs.copyFileSync(
  path.join(artifact.directory, "binding.wasm"),
  path.join(destination, "binding.wasm"),
);
fs.writeFileSync(
  path.join(destination, "index.js"),
  packageEntry(artifact.metadata.exports, artifact.metadata.registrationFunctions.length),
);

const companionManifest = {
  name: artifact.metadata.bindingPackage,
  version: rootManifest.version,
  description: `${rootManifest.description} (Emscripten binding)`,
  license: rootManifest.license,
  repository: rootManifest.repository,
  type: "module",
  main: "./index.js",
  exports: {
    ".": "./index.js",
    "./binding.wasm": "./binding.wasm",
    "./package.json": "./package.json",
  },
  files: ["index.js", "binding.mjs", "binding.wasm"],
  sideEffects: true,
  dependencies: {
    "@emnapi/core": coreVersion,
    "@emnapi/runtime": runtimeVersion,
  },
  publishConfig: rootManifest.publishConfig,
};
fs.writeFileSync(
  path.join(destination, "package.json"),
  `${JSON.stringify(companionManifest, null, 2)}\n`,
);

rootManifest.optionalDependencies ??= {};
rootManifest.optionalDependencies[artifact.metadata.bindingPackage] = rootManifest.version;
fs.writeFileSync(rootManifestPath, `${JSON.stringify(rootManifest, null, 2)}\n`);

process.stdout.write(`Prepared ${artifact.metadata.bindingPackage}@${rootManifest.version}\n`);
