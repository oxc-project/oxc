import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const runtimeDir = fileURLToPath(new URL("../", import.meta.url));

function setup(t) {
  const temporary = mkdtempSync(join(tmpdir(), "oxc-runtime-sync-"));
  t.after(() => rmSync(temporary, { recursive: true, force: true }));
  const runtime = join(temporary, "runtime");
  const upstream = join(temporary, "upstream");
  mkdirSync(runtime);
  mkdirSync(upstream);
  for (const entry of ["src", "scripts", "package.json", "upstream.json", "LICENSE"]) {
    cpSync(join(runtimeDir, entry), join(runtime, entry), { recursive: true });
  }
  const manifest = JSON.parse(readFileSync(join(runtime, "upstream.json"), "utf8"));
  for (const [relative, hash] of Object.entries(manifest.files)) {
    const source = Object.hasOwn(manifest.patches, relative)
      ? join(runtimeDir, "test/fixtures/upstream", relative)
      : join(runtime, "src/helpers", relative);
    const contents = readFileSync(source);
    assert.equal(createHash("sha256").update(contents).digest("hex"), hash, relative);
    const target = join(upstream, "helpers", relative);
    mkdirSync(join(target, ".."), { recursive: true });
    writeFileSync(target, contents);
  }
  // The published package has no Oxc compatibility aliases or legacy helpers.
  const localPackage = JSON.parse(readFileSync(join(runtime, "package.json"), "utf8"));
  const exports = {};
  for (const relative of Object.keys(manifest.files)) {
    if (relative.includes("/") || !relative.endsWith(".js")) continue;
    if (manifest.internalHelpers.includes(relative)) continue;
    exports[`./helpers/${relative.slice(0, -3)}`] = {
      node: `./helpers/${relative}`,
      import: `./helpers/esm/${relative}`,
      default: `./helpers/${relative}`,
    };
  }
  exports["./package.json"] = "./package.json";
  writeFileSync(
    join(upstream, "package.json"),
    JSON.stringify({
      name: manifest.name,
      version: manifest.version,
      sideEffects: localPackage.sideEffects,
      exports,
    }),
  );
  cpSync(join(runtime, "LICENSE"), join(upstream, "LICENSE"));
  return { runtime, upstream };
}

function snapshot(directory) {
  return Object.fromEntries(
    readdirSync(directory, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => {
        const file = join(entry.parentPath, entry.name);
        return [file, createHash("sha256").update(readFileSync(file)).digest("hex")];
      }),
  );
}

function sync({ runtime, upstream }) {
  return spawnSync(process.execPath, [join(runtime, "scripts/sync.mjs"), upstream], {
    encoding: "utf8",
  });
}

test("sync preserves every byte on the first and second runs", (t) => {
  const fixture = setup(t);
  const before = snapshot(fixture.runtime);
  for (let run = 0; run < 2; run++) {
    const result = sync(fixture);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(snapshot(fixture.runtime), before);
  }
});

test("sync preserves local import-first precedence across upstream export shapes", (t) => {
  const fixture = setup(t);
  const packagePath = join(fixture.runtime, "package.json");
  const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
  for (const name of ["asyncToGenerator", "wrapAsyncGenerator"]) {
    const key = `./helpers/${name}`;
    const { import: esm, ...conditions } = packageJson.exports[key];
    const reordered = { import: esm, ...conditions };
    packageJson.exports[key] =
      name === "wrapAsyncGenerator" ? [reordered, conditions.default] : reordered;
  }
  writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);
  for (let run = 0; run < 2; run++) {
    const result = sync(fixture);
    assert.equal(result.status, 0, result.stderr);
    const updated = JSON.parse(readFileSync(packagePath, "utf8"));
    for (const name of ["asyncToGenerator", "wrapAsyncGenerator"]) {
      assert.deepEqual(Object.keys(updated.exports[`./helpers/${name}`]), [
        "import",
        "node",
        "default",
      ]);
    }
  }
});

for (const location of ["runtime", "upstream"]) {
  test(`sync refuses ${location} patch drift before writing`, (t) => {
    const fixture = setup(t);
    const relative =
      location === "runtime" ? "src/helpers/setFunctionName.js" : "helpers/setFunctionName.js";
    appendFileSync(join(fixture[location], relative), "\n// drift\n");
    const before = snapshot(fixture.runtime);
    const result = sync(fixture);
    assert.notEqual(result.status, 0);
    assert.match(
      result.stderr,
      location === "runtime" ? /Unrecorded local changes/ : /Review the local patch/,
    );
    assert.deepEqual(snapshot(fixture.runtime), before);
  });
}
