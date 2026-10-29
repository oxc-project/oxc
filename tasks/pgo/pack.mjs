// oxlint-disable no-console, no-await-in-loop -- Stage packages and validate each archive before making it downloadable.
import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, resolve, win32 } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { parseTriple } from "@napi-rs/cli";
import { git, pnpm, packages, root, run, sha256, targets } from "./utils.mjs";

export function ciVersion(base, runId, attempt, revision) {
  if (
    ![runId, attempt].every((value) => /^(0|[1-9]\d*)$/.test(value))
    || !/^[a-f0-9]{40,64}$/i.test(revision)
  ) {
    throw new Error("Invalid CI version identity");
  }
  return `${base.split("+")[0]}-pgo.${runId}.${attempt}.g${revision.slice(0, 8)}`;
}

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

async function writeJson(path, value) {
  await writeFile(path, JSON.stringify(value, null, 2) + "\n");
}

async function copyPublicFiles(source, destination, manifest, licenseRoot) {
  await mkdir(destination, { recursive: true });
  for (const file of manifest.files) {
    if (
      isAbsolute(file)
      || win32.isAbsolute(file)
      || file.split(/[\\/]/).includes("..")
      || /[*?!]/.test(file)
    ) {
      throw new Error(`Unsupported package.files entry: ${file}`);
    }
    await mkdir(dirname(join(destination, file)), { recursive: true });
    await cp(join(source, file), join(destination, file), { recursive: true });
  }
  for (const file of ["README.md", "LICENSE"]) {
    let path = join(source, file);
    try {
      await stat(path);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      path = join(licenseRoot, file);
    }
    await cp(path, join(destination, file));
  }
}

export async function stagePackage({
  source,
  typesSource,
  library,
  target,
  version,
  directory,
  licenseRoot = root,
}) {
  const manifest = await readJson(join(source, "package.json"));
  await copyPublicFiles(source, directory, manifest, licenseRoot);
  const { platform, arch, abi, platformArchABI } = parseTriple(target);
  const binary = `${manifest.napi.binaryName}.${platformArchABI}.node`;
  const bindingPath = join(dirname(manifest.main), binary).replaceAll("\\", "/");
  await cp(library, join(directory, bindingPath));
  manifest.version = version;
  manifest.os = [platform];
  manifest.cpu = [arch];
  delete manifest.libc;
  if (platform === "linux" && abi === "gnu") manifest.libc = ["glibc"];
  manifest.files = [...manifest.files, bindingPath];
  delete manifest.browser;
  delete manifest.scripts;
  delete manifest.devDependencies;
  delete manifest.optionalDependencies;
  delete manifest.bundleDependencies;
  delete manifest.bundledDependencies;
  if (manifest.dependencies?.["@oxc-project/types"]) {
    const types = await readJson(join(typesSource, "package.json"));
    assert.equal(types.name, "@oxc-project/types");
    const bundled = join(directory, "node_modules/@oxc-project/types");
    await copyPublicFiles(typesSource, bundled, types, licenseRoot);
    delete types.scripts;
    delete types.devDependencies;
    await writeJson(join(bundled, "package.json"), types);
    manifest.dependencies[types.name] = types.version;
    manifest.bundledDependencies = [types.name];
  }
  for (const dependencies of [manifest.dependencies, manifest.peerDependencies]) {
    if (
      Object.values(dependencies ?? {}).some((version) => /^(workspace|catalog):/.test(version))
    ) {
      throw new Error("Unresolved workspace dependency in staged package");
    }
  }
  await writeJson(join(directory, "package.json"), manifest);
  return { manifest, bindingPath };
}

export async function packArchive(directory, output, temporary) {
  await mkdir(output, { recursive: true });
  const blankConfig = join(temporary, "npmrc");
  await writeFile(blankConfig, "");
  const packed = JSON.parse(
    await pnpm(
      [
        "pack",
        "--ignore-workspace",
        "--json",
        "--userconfig",
        blankConfig,
        "--pack-destination",
        output,
      ],
      { cwd: directory, capture: true },
    ),
  );
  return { tarball: packed.filename, files: packed.files.map((file) => file.path) };
}

export async function installArchive(tarball, temporary) {
  const consumer = join(temporary, "consumer");
  await mkdir(consumer, { recursive: true });
  await writeJson(join(consumer, "package.json"), { private: true, type: "module" });
  const blankConfig = join(temporary, "npmrc");
  await writeFile(blankConfig, "");
  await pnpm(
    [
      "add",
      tarball,
      "--ignore-workspace",
      "--offline",
      "--ignore-scripts",
      "--store-dir",
      join(temporary, "store"),
      "--state-dir",
      join(temporary, "state"),
      "--userconfig",
      blankConfig,
    ],
    { cwd: consumer, capture: true },
  );
  return consumer;
}

async function verifyInstalled(consumer, name, manifest, bindingPath, profile) {
  const installed = join(consumer, "node_modules", manifest.name);
  assert.equal(
    await sha256(join(installed, bindingPath)),
    profile.binary,
    "Installed binding differs from validated PGO binary",
  );
  const installedManifest = await readJson(join(installed, "package.json"));
  assert.equal(installedManifest.version, manifest.version);
  if (manifest.bundledDependencies?.includes("@oxc-project/types")) {
    const types = await readJson(join(installed, "node_modules/@oxc-project/types/package.json"));
    assert.equal(types.version, manifest.dependencies[types.name]);
    await stat(join(installed, "node_modules/@oxc-project/types", types.types));
  }
  const env = { ...process.env, NAPI_RS_FORCE_WASI: "false" };
  delete env.NAPI_RS_NATIVE_LIBRARY_PATH;
  delete env.NODE_PATH;
  // Test the packed addon itself, then the public wrapper from an isolated
  // consumer. No repository workspace or registry binding can satisfy imports.
  await run(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `
    import assert from 'node:assert/strict';
    import { createRequire } from 'node:module';
    const require = createRequire(import.meta.url);
    const native = require(${JSON.stringify(join(installed, bindingPath))});
    const api = await import(${JSON.stringify(manifest.name)});
    const name = ${JSON.stringify(name)};
    if (name === 'parser') {
      assert.equal(typeof native.parseSync, 'function');
      const result = api.parseSync('smoke.ts', 'export const answer: number = 42;');
      assert.equal(result.errors.length, 0);
      assert.equal(result.program.body.length, 1);
    } else if (name === 'transform') {
      assert.equal(typeof native.transformSync, 'function');
      const result = api.transformSync('smoke.ts', 'export const answer: number = 42;');
      assert.equal(result.errors.length, 0);
      assert.match(result.code, /answer\\s*=\\s*42/);
    } else {
      assert.equal(typeof native.minifySync, 'function');
      const result = api.minifySync('smoke.js', 'export function square(n) { return n * n; }');
      assert.equal(result.errors.length, 0);
      assert.ok(result.code.length > 0);
    }
  `,
    ],
    { cwd: consumer, env },
  );
}

export async function packPackage(
  name,
  target,
  {
    output,
    runId = process.env.GITHUB_RUN_ID ?? "0",
    attempt = process.env.GITHUB_RUN_ATTEMPT ?? "1",
  } = {},
) {
  if (!packages.includes(name) || !targets.includes(target)) {
    throw new Error("Packing requires one supported package and target");
  }
  const work = join(root, "target/pgo", name, target);
  const profile = await readJson(join(work, "profile.json"));
  assert.equal(profile.package, name);
  assert.equal(profile.target, target);
  assert.equal(
    profile.revision,
    await git(root, ["rev-parse", "HEAD"]),
    "PGO build belongs to another commit",
  );
  const source = join(root, "napi", name);
  const original = await readJson(join(source, "package.json"));
  const binary = `${original.napi.binaryName}.${parseTriple(target).platformArchABI}.node`;
  const library = join(work, "optimized", binary);
  assert.equal(await sha256(library), profile.binary, "PGO binary checksum mismatch");
  const version = ciVersion(original.version, runId, attempt, profile.revision);
  const temporary = await mkdtemp(join(tmpdir(), "oxc-pgo-package-"));
  try {
    const directory = join(temporary, "package");
    const { manifest, bindingPath } = await stagePackage({
      source,
      typesSource: join(root, "npm/oxc-types"),
      library,
      target,
      version,
      directory,
    });
    const { tarball, files } = await packArchive(directory, join(temporary, "packed"), temporary);
    assert.ok(files.includes(bindingPath), "Native binding missing from tarball");
    if (name === "parser") {
      assert.ok(
        files.includes("node_modules/@oxc-project/types/types.d.ts"),
        "Matching parser types missing from tarball",
      );
    }
    const consumer = await installArchive(tarball, temporary);
    await verifyInstalled(consumer, name, manifest, bindingPath, profile);
    output = resolve(output ?? join(work, "npm"));
    await mkdir(output, { recursive: true });
    const filename = `${manifest.name}-${version}-${target}.tgz`;
    await cp(tarball, join(output, filename));
    await cp(join(work, "profile.json"), join(output, "profile.json"));
    console.log(`Validated npm package: ${join(output, filename)}`);
    return join(output, filename);
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      package: { type: "string" },
      target: { type: "string" },
      output: { type: "string" },
    },
  });
  await packPackage(values.package, values.target, { output: values.output });
}
