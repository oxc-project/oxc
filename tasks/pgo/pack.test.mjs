import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parseTriple } from "@napi-rs/cli";
import { ciVersion, installArchive, packArchive, selectMatrix, stagePackage } from "./pack.mjs";
import { packages, sha256, targets } from "./utils.mjs";

await test("ad hoc selectors cover all combinations and reject unsupported packages and targets", () => {
  for (const name of ["all", ...packages]) {
    for (const target of ["all", ...targets]) {
      const matrix = selectMatrix(name, target);
      assert.equal(matrix.length, (name === "all" ? 3 : 1) * (target === "all" ? 5 : 1));
      assert.equal(
        new Set(matrix.map((row) => `${row.package}/${row.target}`)).size,
        matrix.length,
      );
      for (const row of matrix) {
        assert.ok(packages.includes(row.package));
        assert.ok(targets.includes(row.target));
        assert.ok(row.os);
        if (name !== "all") assert.equal(row.package, name);
        if (target !== "all") assert.equal(row.target, target);
      }
    }
  }
  assert.equal(selectMatrix("parser", "aarch64-unknown-linux-gnu")[0].os, "ubuntu-24.04-arm");
  assert.equal(selectMatrix("minify", "x86_64-apple-darwin")[0].os, "macos-15-intel");
  assert.throws(() => selectMatrix("playground"), /Unsupported package/);
  assert.throws(() => selectMatrix("parser", "wasm32-wasip1"), /Unsupported target/);
});

await test("CI versions identify the run and commit with valid prerelease identifiers", () => {
  const revision = "01234567".repeat(5);
  assert.equal(ciVersion("1.2.3", "123", "2", revision), "1.2.3-pgo.123.2.g01234567");
  assert.equal(ciVersion("1.2.3+build", "0", "1", revision), "1.2.3-pgo.0.1.g01234567");
  assert.throws(() => ciVersion("1.2.3", "0123", "2", revision), /Invalid CI version/);
  assert.throws(() => ciVersion("1.2.3", "1", "1", "wrong"), /Invalid CI version/);
});

async function fixture(t) {
  const directory = await mkdtemp(join(tmpdir(), "oxc-pgo-pack-test-"));
  t.after(() => rm(directory, { force: true, recursive: true }));
  const source = join(directory, "source");
  const typesSource = join(directory, "types");
  await mkdir(join(source, "src-js"), { recursive: true });
  await mkdir(typesSource);
  await writeFile(join(directory, "LICENSE"), "MIT fixture license\n");
  await writeFile(join(directory, "README.md"), "Fixture types package\n");
  await writeFile(join(source, "README.md"), "Fixture parser package\n");
  await writeFile(join(source, "src-js/index.js"), "export const installed = true;\n");
  await writeFile(
    join(source, "src-js/index.d.ts"),
    'export type { Program } from "@oxc-project/types";\n',
  );
  const manifest = {
    name: "oxc-pgo-pack-fixture",
    version: "1.2.3",
    type: "module",
    main: "src-js/index.js",
    files: ["src-js/index.js", "src-js/index.d.ts"],
    browser: "browser.js",
    napi: { binaryName: "parser" },
    dependencies: { "@oxc-project/types": "workspace:^" },
    devDependencies: { buildTool: "catalog:" },
    optionalDependencies: { "@oxc-parser/binding-linux-x64-gnu": "1.2.3" },
    scripts: { install: "exit 1" },
  };
  await writeFile(join(source, "package.json"), JSON.stringify(manifest));
  await writeFile(
    join(typesSource, "package.json"),
    JSON.stringify({
      name: "@oxc-project/types",
      version: "1.2.3",
      types: "types.d.ts",
      files: ["types.d.ts"],
    }),
  );
  await writeFile(join(typesSource, "types.d.ts"), "export interface Program {}\n");
  const library = join(directory, "fixture.node");
  await writeFile(library, "native fixture bytes");
  return {
    source,
    typesSource,
    library,
    directory,
    licenseRoot: directory,
    version: "1.2.3-pgo.1.1.g01234567",
  };
}

await test("staging preserves public files, bundles matching types, and adds native platform metadata", async (t) => {
  const options = await fixture(t);
  const before = await readFile(join(options.source, "package.json"), "utf8");
  await Promise.all(
    targets.map(async (target) => {
      const directory = join(options.directory, target);
      const { manifest, bindingPath } = await stagePackage({ ...options, target, directory });
      const { platform, arch, abi } = parseTriple(target);
      assert.deepEqual(manifest.os, [platform]);
      assert.deepEqual(manifest.cpu, [arch]);
      assert.deepEqual(manifest.libc, abi === "gnu" ? ["glibc"] : undefined);
      assert.ok(manifest.files.includes(bindingPath));
      assert.equal(manifest.dependencies["@oxc-project/types"], "1.2.3");
      assert.deepEqual(manifest.bundledDependencies, ["@oxc-project/types"]);
      assert.equal(manifest.scripts, undefined);
      assert.equal(manifest.devDependencies, undefined);
      assert.equal(manifest.optionalDependencies, undefined);
      assert.equal(manifest.browser, undefined);
      assert.equal(await sha256(join(directory, bindingPath)), await sha256(options.library));
      assert.equal(
        await readFile(join(directory, "node_modules/@oxc-project/types/types.d.ts"), "utf8"),
        "export interface Program {}\n",
      );
    }),
  );
  assert.equal(await readFile(join(options.source, "package.json"), "utf8"), before);
});

await test("packed archive installs offline with native bytes and bundled parser types intact", async (t) => {
  const options = await fixture(t);
  const target = targets.find((value) => {
    const parsed = parseTriple(value);
    return parsed.platform === process.platform && parsed.arch === process.arch;
  });
  assert.ok(target, "Tooling checks require one of the native PGO hosts");
  const directory = join(options.directory, "package");
  const { manifest, bindingPath } = await stagePackage({ ...options, target, directory });
  const { tarball, files } = await packArchive(
    directory,
    join(options.directory, "packed"),
    options.directory,
  );
  assert.ok(files.includes(bindingPath));
  assert.ok(files.includes("src-js/index.d.ts"));
  assert.ok(files.includes("LICENSE"));
  assert.ok(files.includes("node_modules/@oxc-project/types/types.d.ts"));
  const consumer = await installArchive(tarball, options.directory);
  const installed = join(consumer, "node_modules", manifest.name);
  assert.equal(await sha256(join(installed, bindingPath)), await sha256(options.library));
  assert.equal(
    JSON.parse(await readFile(join(installed, "node_modules/@oxc-project/types/package.json")))
      .version,
    "1.2.3",
  );
});
