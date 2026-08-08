import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import { getConfig } from "../../.github/scripts/napi-emscripten/config.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../..");
const packageDirectory = path.resolve(process.argv[2] ?? "");
const packageManifestPath = path.join(packageDirectory, "package.json");
if (!process.argv[2] || !fs.existsSync(packageManifestPath)) {
  throw new Error("Usage: pnpm test <path-to-napi-package>");
}

const packageManifest = JSON.parse(fs.readFileSync(packageManifestPath, "utf8"));
const config = getConfig(packageManifest.name);
const packageRequire = createRequire(packageManifestPath);
const testRequire = createRequire(import.meta.url);
const wranglerManifestPath = testRequire.resolve("wrangler/package.json");
const wranglerManifest = JSON.parse(fs.readFileSync(wranglerManifestPath, "utf8"));
const wranglerBin = path.resolve(
  path.dirname(wranglerManifestPath),
  typeof wranglerManifest.bin === "string" ? wranglerManifest.bin : wranglerManifest.bin.wrangler,
);
const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "oxc-emscripten-"));
const copiedPackage = path.join(temporaryDirectory, "root-package");
const releaseDirectory = path.join(temporaryDirectory, "release-dir");
const nodeModules = path.join(temporaryDirectory, "node_modules");
const port = 19_000 + Math.floor(Math.random() * 1_000);

function symlinkPackage(packageName, source) {
  const destination = path.join(nodeModules, ...packageName.split("/"));
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.symlinkSync(source, destination, "dir");
}

function workerSource(packageName) {
  switch (packageName) {
    case "oxc-parser":
      return `import { parse, parseSync } from "oxc-parser";
export default {
  async fetch() {
    const syncResult = parseSync("test.ts", "const value: number = 1", {});
    const asyncResult = await parse("test.ts", "const value: number = 1", {});
    return new Response(
      syncResult.program.type === "Program" && asyncResult.program.type === "Program"
        ? "ok"
        : "invalid",
    );
  },
};
`;
    case "oxc-minify":
      return `import { minify, minifySync } from "oxc-minify";
export default {
  async fetch() {
    const syncResult = minifySync("test.js", "const value = 1 + 2", {});
    const asyncResult = await minify("test.js", "const value = 1 + 2", {});
    return new Response(syncResult.code === asyncResult.code ? syncResult.code : "invalid");
  },
};
`;
    case "oxc-transform":
      return `import { transform, transformSync } from "oxc-transform";
export default {
  async fetch() {
    const syncResult = transformSync("test.ts", "const value: number = 1", {});
    const asyncResult = await transform("test.ts", "const value: number = 1", {});
    return new Response(syncResult.code === asyncResult.code ? syncResult.code : "invalid");
  },
};
`;
    case "oxc-transform-react":
      return `import { transform, transformSync } from "oxc-transform-react";
export default {
  async fetch() {
    const syncResult = transformSync(
      "test.jsx",
      "export function Component() { return <div /> }",
      { lang: "jsx", reactCompiler: false },
    );
    const asyncResult = await transform(
      "test.jsx",
      "export function Component() { return <div /> }",
      { lang: "jsx", reactCompiler: false },
    );
    return new Response(syncResult.code === asyncResult.code ? syncResult.code : "invalid");
  },
};
`;
    default:
      throw new Error(`Missing worker fixture for ${packageName}`);
  }
}

function validateResponse(packageName, response) {
  switch (packageName) {
    case "oxc-parser":
      return response === "ok";
    case "oxc-minify":
      return response === "const value=3;";
    case "oxc-transform":
      return response.includes("const value = 1");
    case "oxc-transform-react":
      return response.includes("jsx");
    default:
      return false;
  }
}

function waitForExit(child, timeout) {
  if (child.exitCode !== null) return Promise.resolve();
  return Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    new Promise((resolve) => setTimeout(resolve, timeout)),
  ]);
}

let wrangler;
try {
  fs.cpSync(packageDirectory, copiedPackage, {
    recursive: true,
    filter(source) {
      return !["node_modules", "npm-dir"].includes(path.basename(source));
    },
  });
  fs.symlinkSync(
    path.join(packageDirectory, "node_modules"),
    path.join(copiedPackage, "node_modules"),
  );
  fs.mkdirSync(releaseDirectory, { recursive: true });

  const prepareResult = spawnSync(
    process.execPath,
    [
      path.join(repositoryRoot, ".github/scripts/napi-emscripten/prepare-package.mjs"),
      "--package-dir",
      copiedPackage,
      "--artifacts-dir",
      path.join(repositoryRoot, "target/emscripten"),
      "--npm-dir",
      releaseDirectory,
    ],
    { cwd: repositoryRoot, encoding: "utf8" },
  );
  if (prepareResult.status !== 0) {
    throw new Error(prepareResult.stderr || prepareResult.stdout || "Package preparation failed");
  }

  const companionDirectory = path.join(releaseDirectory, "wasm32-emscripten");
  const preparedRootManifest = JSON.parse(
    fs.readFileSync(path.join(copiedPackage, "package.json"), "utf8"),
  );
  const companionManifest = JSON.parse(
    fs.readFileSync(path.join(companionDirectory, "package.json"), "utf8"),
  );
  if (
    companionManifest.name !== config.bindingPackage ||
    companionManifest.version !== packageManifest.version ||
    preparedRootManifest.optionalDependencies[config.bindingPackage] !== packageManifest.version
  ) {
    throw new Error("Invalid Emscripten release package metadata");
  }

  fs.rmSync(path.join(copiedPackage, "node_modules"));
  fs.mkdirSync(nodeModules, { recursive: true });
  symlinkPackage(packageManifest.name, copiedPackage);
  symlinkPackage(config.bindingPackage, companionDirectory);
  for (const dependency of ["@emnapi/core", "@emnapi/runtime"]) {
    symlinkPackage(dependency, path.dirname(packageRequire.resolve(`${dependency}/package.json`)));
  }

  fs.writeFileSync(path.join(temporaryDirectory, "worker.mjs"), workerSource(packageManifest.name));
  fs.writeFileSync(
    path.join(temporaryDirectory, "wrangler.json"),
    `${JSON.stringify(
      {
        name: `oxc-${config.directory}-emscripten-test`,
        main: "worker.mjs",
        // NAPI object finalizers require WeakRef and FinalizationRegistry in workerd.
        compatibility_date: "2025-05-05",
      },
      null,
      2,
    )}\n`,
  );

  wrangler = spawn(
    process.execPath,
    [wranglerBin, "dev", "--config", "wrangler.json", "--port", String(port), "--ip", "127.0.0.1"],
    {
      cwd: temporaryDirectory,
      env: { ...process.env, NO_COLOR: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  let logs = "";
  wrangler.stdout.on("data", (chunk) => {
    logs += chunk;
  });
  wrangler.stderr.on("data", (chunk) => {
    logs += chunk;
  });

  async function validateWorker(attempt = 0) {
    if (wrangler.exitCode !== null) {
      throw new Error(`Wrangler exited with code ${wrangler.exitCode}:\n${logs}`);
    }
    if (attempt === 100) {
      throw new Error(`Wrangler did not become ready:\n${logs}`);
    }

    let response;
    try {
      response = await fetch(`http://127.0.0.1:${port}`);
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 100));
      return validateWorker(attempt + 1);
    }

    const body = await response.text();
    if (!response.ok || !validateResponse(packageManifest.name, body)) {
      throw new Error(`Unexpected response ${response.status}: ${body}\n${logs}`);
    }
    process.stdout.write(`Validated ${packageManifest.name} in workerd\n`);
  }

  await validateWorker();
} finally {
  if (wrangler?.exitCode === null) {
    wrangler.kill("SIGTERM");
    await waitForExit(wrangler, 5_000);
    if (wrangler.exitCode === null) wrangler.kill("SIGKILL");
  }
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
