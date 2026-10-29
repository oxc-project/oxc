import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Readable } from "node:stream";
import { test } from "node:test";
import {
  CorpusWriter,
  jsonRecords,
  readShard,
  repositories,
  sourceFiles,
  verifyCorpus,
} from "./corpus.mjs";
import { trainRecord } from "./train.mjs";
import { git, language, largeStackWorker, profileEnvironment, run, sourceType } from "./utils.mjs";

async function temporary(t) {
  const path = await mkdtemp(join(tmpdir(), "oxc-pgo-"));
  t.after(() => rm(path, { recursive: true, force: true }));
  return path;
}

await test("training workers propagate failures to their parent", async () => {
  await largeStackWorker(new URL("data:text/javascript,void 0"));
  await assert.rejects(
    largeStackWorker(new URL("data:text/javascript,throw new Error('training failed')")),
    /training failed/,
  );
});

await test("JSONL preserves Unicode line separators and UTF-8 across stream chunks", async () => {
  const record = { code: "counter\u2028\u2029 café" };
  const bytes = Buffer.from(JSON.stringify(record) + "\n");
  const input = Readable.from(Array.from(bytes, (byte) => Buffer.from([byte])));
  assert.deepEqual(await Array.fromAsync(jsonRecords(input)), [record]);
});

await test("ecosystem matrices deduplicate repositories and reject conflicting or unsafe paths", () => {
  const entry = { repository: "org/repo", path: "org/repo", ref: "main" };
  assert.deepEqual(repositories([[entry], [entry]]), [entry]);
  assert.throws(() => repositories([[entry, { ...entry, ref: "other" }]]), /Conflicting/);
  assert.throws(() => repositories([[{ ...entry, path: "../outside" }]]), /Invalid/);
  assert.throws(() => repositories([[]]), /Empty/);
});

await test("source enumeration reads committed blobs without touching modified or untracked inputs", async (t) => {
  const directory = await temporary(t);
  await run("git", ["init", "--quiet", directory]);
  await writeFile(join(directory, "space name.ts"), "export const n: number = 1;\n");
  await writeFile(join(directory, "ignore.json"), "{}");
  await git(directory, ["add", "."]);
  await git(directory, [
    "-c",
    "user.name=PGO",
    "-c",
    "user.email=pgo@example.com",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "--quiet",
    "-m",
    "fixture",
  ]);
  await writeFile(join(directory, "space name.ts"), "modified");
  await writeFile(join(directory, "untracked.js"), "untracked");
  const files = await Array.fromAsync(sourceFiles(directory, "HEAD"));
  assert.deepEqual(files, [{ filename: "space name.ts", code: "export const n: number = 1;\n" }]);
  assert.equal(await readFile(join(directory, "space name.ts"), "utf8"), "modified");
});

await test("corpus round trip preserves metadata, prepares TS/JSX, reports errors, and detects corruption", async (t) => {
  const directory = await temporary(t);
  const writer = new CorpusWriter(directory, 1);
  const transform = (filename) =>
    filename === "bad.ts"
      ? { code: "", errors: ["syntax error"] }
      : { code: "export const n = 1;", errors: [] };
  const record = (filename) => ({
    suite: "fixture",
    filename,
    code: "const n: number = 1;",
    options: { lang: language(filename), sourceType: "module" },
    expectedFailure: false,
  });
  await writer.add(record("good.ts"), transform);
  await writer.add(record("bad.ts"), transform);
  await writer.closeShard();
  assert.equal(writer.shards.length, 2);
  const manifest = {
    version: 1,
    oxcRevision: "revision",
    repositories: [{ revision: "repo" }],
    counts: writer.counts,
    shards: writer.shards,
  };
  await writeFile(join(directory, "manifest.json"), JSON.stringify(manifest));
  await verifyCorpus(directory, { revision: "revision", complete: false });
  const first = await Array.fromAsync(readShard(directory, writer.shards[0]));
  const second = await Array.fromAsync(readShard(directory, writer.shards[1]));
  assert.equal(first[0].minify, "export const n = 1;");
  assert.equal(first[0].options.sourceType, "module");
  assert.equal(second[0].minify, null);
  assert.equal(writer.preparation.diagnostics, 1);
  await assert.rejects(
    verifyCorpus(directory, { revision: "other", complete: false }),
    /different Oxc revision/,
  );
  await assert.rejects(verifyCorpus(directory), /missing a conformance suite/);
  await writeFile(join(directory, writer.shards[0].file), "corrupt");
  await assert.rejects(verifyCorpus(directory, { complete: false }), /checksum mismatch/);
});

await test("PGO flags preserve Cargo environment precedence or append through target configuration", () => {
  const target = "x86_64-pc-windows-msvc";
  const flag = "-Cprofile-generate=/tmp/raw";
  assert.deepEqual(profileEnvironment(target, flag, "/tmp/overlay", {}).cargoOptions, [
    "--config",
    "/tmp/overlay",
  ]);
  const original = {
    CARGO_ENCODED_RUSTFLAGS: "-C\x1ftarget-feature=+crt-static",
    RUSTFLAGS: "ignored",
  };
  assert.equal(
    profileEnvironment(target, flag, "", { CARGO_BUILD_RUSTFLAGS: "--cfg example" }).env
      .CARGO_BUILD_RUSTFLAGS,
    "--cfg example " + flag,
  );
  assert.equal(
    profileEnvironment(target, flag, "", original).env.CARGO_ENCODED_RUSTFLAGS,
    original.CARGO_ENCODED_RUSTFLAGS + "\x1f" + flag,
  );
  assert.equal(
    profileEnvironment(target, flag, "", { RUSTFLAGS: "-C target-feature=+crt-static" }).env
      .RUSTFLAGS,
    "-C target-feature=+crt-static " + flag,
  );
  assert.equal(
    profileEnvironment(target, flag, "", {
      CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS: "-C target-feature=+crt-static",
    }).env.CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS,
    "-C target-feature=+crt-static " + flag,
  );
});

await test("native C dependencies retain caller flags and do not inherit Rust PGO instrumentation", () => {
  const { env } = profileEnvironment(
    "aarch64-apple-darwin",
    "-Cprofile-generate=/tmp/raw",
    "/tmp/config",
    { CFLAGS: "-O2" },
  );
  assert.equal(env.CFLAGS_aarch64_apple_darwin, "-O2 -fno-profile-generate -fno-profile-use");
  assert.equal(env.CXXFLAGS_aarch64_apple_darwin, " -fno-profile-generate -fno-profile-use");
});

await test("training materializes parser getters and feeds minifier prepared JS with the correct source mode", async () => {
  let getters = 0;
  const parsed = {
    get program() {
      getters++;
      return { body: [] };
    },
    module: {},
    comments: [],
    errors: [],
  };
  let calls = 0;
  await trainRecord(
    "parser",
    {
      parse: () => {
        calls++;
        return parsed;
      },
      rawTransferSupported: () => true,
    },
    { filename: "input.ts", code: "let n: number", options: { lang: "ts", sourceType: "module" } },
    0,
  );
  assert.equal(getters, 2);
  assert.equal(calls, 2);
  const record = {
    filename: "input.ts",
    code: "let n: number",
    minify: "let n;",
    options: { lang: "ts", sourceType: "script" },
  };
  await trainRecord(
    "minify",
    {
      minifySync(filename, code) {
        assert.equal(filename, "input.ts.cjs");
        assert.equal(code, "let n;");
        return { code: "let n;", errors: [] };
      },
    },
    record,
    1,
  );
  assert.equal(await trainRecord("minify", {}, { ...record, minify: null }, 1), "skipped");
  assert.equal(sourceType("test.cts"), "commonjs");
  assert.equal(sourceType("test.mjs"), "module");
  assert.equal(language("test.d.mts"), "dts");
});

await test("negative parser inputs collect native diagnostics without deserializing a recovered AST", async () => {
  const binding = {
    parseSync: () => ({
      get program() {
        throw new Error("Invalid recovered JSON");
      },
      errors: [{}],
      comments: [],
      module: {},
    }),
    rawTransferSupported: () => false,
  };
  assert.equal(
    await trainRecord(
      "parser",
      binding,
      {
        filename: "negative.tsx",
        code: "<\\u0061/>",
        options: { lang: "tsx", sourceType: "module" },
      },
      1,
    ),
    "diagnostics",
  );
});

await test("minifier semantic diagnostics in conformance inputs are recorded as training outcomes", async () => {
  const result = await trainRecord(
    "minify",
    { minifySync: () => ({ code: "", errors: [{}] }) },
    {
      filename: "duplicate-exports.js",
      code: "export default 1; export default 2;",
      options: { lang: "js", sourceType: "module" },
    },
    1,
  );
  assert.equal(result, "diagnostics");
});
