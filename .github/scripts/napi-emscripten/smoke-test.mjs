import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

import { getConfig } from "./config.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../..");
const packageJsonPath = path.join(process.cwd(), "package.json");
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const config = getConfig(packageJson.name);
const packageRequire = createRequire(packageJsonPath);
const { createNapiModule } = await import(pathToFileURL(packageRequire.resolve("@emnapi/core")));
const { asyncWork, tsfn } = await import(
  pathToFileURL(packageRequire.resolve("@emnapi/core/plugins"))
);
const { getDefaultContext } = await import(
  pathToFileURL(packageRequire.resolve("@emnapi/runtime"))
);
const artifactDirectory = path.join(repositoryRoot, "target/emscripten", config.directory);
const metadata = JSON.parse(fs.readFileSync(path.join(artifactDirectory, "artifact.json"), "utf8"));
const wasmModule = new WebAssembly.Module(
  fs.readFileSync(path.join(artifactDirectory, "binding.wasm")),
);
const { default: createEmscriptenModule } = await import(
  pathToFileURL(path.join(artifactDirectory, "binding.mjs"))
);
const napiModule = createNapiModule({
  context: getDefaultContext(),
  asyncWorkPoolSize: 0,
  plugins: [asyncWork, tsfn],
});

let wasmInstance;
const emscriptenModule = await createEmscriptenModule({
  instantiateWasm(imports, done) {
    imports.env = {
      ...imports.env,
      ...napiModule.imports.env,
      ...napiModule.imports.napi,
      ...napiModule.imports.emnapi,
    };
    wasmInstance = new WebAssembly.Instance(wasmModule, imports);
    done(wasmInstance, wasmModule);
    return wasmInstance.exports;
  },
});

const registrations = Object.keys(emscriptenModule).filter((name) =>
  name.startsWith("___napi_register__"),
);
if (registrations.length !== metadata.registrationFunctions.length) {
  throw new Error(
    `Expected ${metadata.registrationFunctions.length} registrations, found ${registrations.length}`,
  );
}
for (const name of registrations) emscriptenModule[name]();
const binding = napiModule.init({ instance: wasmInstance, module: wasmModule });

for (const name of config.exports) {
  if (!(name in binding)) throw new Error(`Missing public export: ${name}`);
}

switch (config.packageName) {
  case "oxc-parser": {
    const syncResult = binding.parseSync("test.ts", "const value: number = 1", {});
    const asyncResult = await binding.parse("test.ts", "const value: number = 1", {});
    if (!syncResult.program.includes('"type":"Program"')) throw new Error("Invalid parser AST");
    if (!asyncResult.program.includes('"type":"Program"')) {
      throw new Error("Invalid async parser AST");
    }
    break;
  }
  case "oxc-minify": {
    const syncResult = binding.minifySync("test.js", "const value = 1 + 2", {});
    const asyncResult = await binding.minify("test.js", "const value = 1 + 2", {});
    if (syncResult.code !== "const value=3;" || asyncResult.code !== syncResult.code) {
      throw new Error("Invalid minifier output");
    }
    break;
  }
  case "oxc-transform": {
    const syncResult = binding.transformSync("test.ts", "const value: number = 1", {});
    const asyncResult = await binding.transform("test.ts", "const value: number = 1", {});
    if (!syncResult.code.includes("const value = 1") || asyncResult.code !== syncResult.code) {
      throw new Error("Invalid transformer output");
    }
    break;
  }
  case "oxc-transform-react": {
    const source = "export function Component() { return <div /> }";
    const options = { lang: "jsx", reactCompiler: false };
    const syncResult = binding.transformSync("test.jsx", source, options);
    const asyncResult = await binding.transform("test.jsx", source, options);
    if (!syncResult.code.includes("jsx") || asyncResult.code !== syncResult.code) {
      throw new Error("Invalid React transformer output");
    }
    break;
  }
}

process.stdout.write(`Validated ${config.bindingPackage}\n`);
