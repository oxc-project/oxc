// oxlint-disable no-console, no-await-in-loop -- Stream inputs sequentially to bound memory and disk usage.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createReadStream, createWriteStream } from "node:fs";
import { mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pipeline } from "node:stream/promises";
import { pathToFileURL } from "node:url";
import { createGzip, createGunzip } from "node:zlib";
import { git, language, root, run, sha256, sourceType } from "./utils.mjs";

const sourceExtension = /\.(?:[cm]?[jt]s|[jt]sx)$/i;
const requiredSuites = [
  "test262",
  "babel-parser",
  "typescript",
  "misc",
  "acorn-jsx",
  "babel-transform",
  "oxc-transform",
];

// Split on LF bytes so JSON strings, Unicode line separators, and UTF-8 sequences
// remain intact across stream chunks.
export async function* jsonRecords(input) {
  let pieces = [];
  for await (const chunk of input) {
    let start = 0,
      end;
    while ((end = chunk.indexOf(10, start)) !== -1) {
      pieces.push(chunk.subarray(start, end));
      yield JSON.parse(Buffer.concat(pieces).toString("utf8"));
      pieces = [];
      start = end + 1;
    }
    if (start < chunk.length) pieces.push(chunk.subarray(start));
  }
  if (pieces.length) yield JSON.parse(Buffer.concat(pieces).toString("utf8"));
}

export function repositories(matrices) {
  const result = new Map();
  for (const item of matrices.flat()) {
    if (
      !/^[\w.-]+\/[\w.-]+$/.test(item.repository)
      || !item.ref
      || !item.path
      || item.path.split(/[\\/]/).some((part) => part === ".." || part === "")
    ) {
      throw new Error(`Invalid ecosystem entry: ${JSON.stringify(item)}`);
    }
    const previous = result.get(item.path);
    if (previous && (previous.repository !== item.repository || previous.ref !== item.ref)) {
      throw new Error(`Conflicting ecosystem entries for ${item.path}`);
    }
    result.set(item.path, item);
  }
  if (!result.size) throw new Error("Empty ecosystem matrices");
  return [...result.values()].sort((a, b) => a.path.localeCompare(b.path, "en"));
}

// Read committed blobs in one git process per repository, including files outside
// a sparse checkout. Installed dependencies and uncommitted changes are excluded.
export async function* sourceFiles(directory, revision, extension = sourceExtension) {
  const tree = await git(directory, ["ls-tree", "-r", "-z", revision]);
  const entries = tree
    .split("\0")
    .filter(Boolean)
    .map((line) => {
      const match = /^(100644|100755) blob ([a-f0-9]+)\t(.+)$/s.exec(line);
      return match && extension.test(match[3]) ? { oid: match[2], filename: match[3] } : null;
    })
    .filter(Boolean);
  const child = spawn("git", ["-C", directory, "cat-file", "--batch"], {
    stdio: ["pipe", "pipe", "inherit"],
  });
  const completed = new Promise((resolvePromise, reject) => {
    child.on("error", reject);
    child.on("close", (code) =>
      code === 0 ? resolvePromise() : reject(new Error(`git cat-file failed (${code})`)),
    );
  });
  // Attach a rejection handler before consuming the stream.
  completed.catch(() => {});
  const iterator = child.stdout[Symbol.asyncIterator]();
  let buffer = Buffer.alloc(0);
  async function read(size) {
    while (buffer.length < size) {
      const next = await iterator.next();
      if (next.done) throw new Error("Truncated git blob stream");
      buffer = Buffer.concat([buffer, next.value]);
    }
    const result = buffer.subarray(0, size);
    buffer = buffer.subarray(size);
    return result;
  }
  try {
    for (const { oid, filename } of entries) {
      child.stdin.write(`${oid}\n`);
      let header = "";
      while (!header.endsWith("\n")) header += (await read(1)).toString();
      const match = /^[a-f0-9]+ blob (\d+)\n$/.exec(header);
      if (!match) throw new Error(`Invalid git blob header: ${header}`);
      const bytes = await read(Number(match[1]));
      const encoding =
        bytes[0] === 0xff && bytes[1] === 0xfe
          ? "utf-16le"
          : bytes[0] === 0xfe && bytes[1] === 0xff
            ? "utf-16be"
            : "utf-8";
      yield { filename, code: new TextDecoder(encoding).decode(bytes) };
      if ((await read(1))[0] !== 10) throw new Error("Invalid git blob delimiter");
    }
    child.stdin.end();
    await completed;
  } finally {
    child.stdin.destroy();
    if (child.exitCode === null) child.kill();
  }
}

export class CorpusWriter {
  constructor(directory, shardBytes = 64 * 1024 * 1024) {
    this.directory = directory;
    this.shardBytes = shardBytes;
    this.shards = [];
    this.counts = {};
    this.preparation = { javascript: 0, transformed: 0, diagnostics: 0, declarations: 0 };
    this.bytes = 0;
    this.records = 0;
  }

  async closeShard() {
    if (!this.stream) return;
    this.stream.end();
    await this.finished;
    const path = join(this.directory, this.filename);
    this.shards.push({
      file: this.filename,
      records: this.records,
      bytes: (await stat(path)).size,
      sha256: await sha256(path),
    });
    this.stream = undefined;
  }

  async add(record, transformSync) {
    const { lang } = record.options;
    const result = transformSync(record.filename, record.code, {
      ...record.options,
      target: "esnext",
    });
    if (result.errors.length) {
      record.minify = null;
      this.preparation.diagnostics++;
    } else if (lang === "dts" || !result.code.trim()) {
      record.minify = null;
      this.preparation.declarations++;
    } else if (lang !== "js") {
      record.minify = result.code;
      this.preparation.transformed++;
    } else {
      this.preparation.javascript++;
    }
    const line = JSON.stringify(record) + "\n";
    const bytes = Buffer.byteLength(line);
    if (this.stream && this.bytes + bytes > this.shardBytes) await this.closeShard();
    if (!this.stream) {
      this.filename = `shard-${String(this.shards.length).padStart(4, "0")}.jsonl.gz`;
      this.stream = createGzip();
      this.finished = pipeline(this.stream, createWriteStream(join(this.directory, this.filename)));
      this.finished.catch(() => {});
      this.bytes = 0;
      this.records = 0;
    }
    if (!this.stream.write(line)) await once(this.stream, "drain");
    this.bytes += bytes;
    this.records++;
    this.counts[record.suite] = (this.counts[record.suite] ?? 0) + 1;
  }
}

export async function verifyCorpus(directory, { revision, complete = true } = {}) {
  const manifest = JSON.parse(await readFile(join(directory, "manifest.json"), "utf8"));
  if (manifest.version !== 1 || !manifest.shards?.length || !manifest.repositories?.length) {
    throw new Error("Invalid or empty PGO corpus manifest");
  }
  if (revision && manifest.oxcRevision !== revision) {
    throw new Error("PGO corpus belongs to a different Oxc revision");
  }
  if (complete && requiredSuites.some((suite) => !manifest.counts[suite])) {
    throw new Error("PGO corpus is missing a conformance suite");
  }
  const seen = new Set();
  for (const shard of manifest.shards) {
    if (
      !/^shard-\d+\.jsonl\.gz$/.test(shard.file)
      || seen.has(shard.file)
      || !(shard.records > 0)
    ) {
      throw new Error("Invalid PGO corpus shard");
    }
    seen.add(shard.file);
    const path = join(directory, shard.file);
    if ((await stat(path)).size !== shard.bytes || (await sha256(path)) !== shard.sha256) {
      throw new Error(`PGO corpus checksum mismatch: ${shard.file}`);
    }
  }
  const total = manifest.shards.reduce((sum, shard) => sum + shard.records, 0);
  if (total !== Object.values(manifest.counts).reduce((sum, count) => sum + count, 0)) {
    throw new Error("PGO corpus record counts do not match");
  }
  return manifest;
}

export async function* readShard(directory, shard) {
  const input = createReadStream(join(directory, shard.file));
  const unzip = createGunzip();
  const completed = pipeline(input, unzip);
  completed.catch(() => {});
  let count = 0;
  try {
    for await (const record of jsonRecords(unzip)) {
      if (
        typeof record.code !== "string"
        || typeof record.filename !== "string"
        || !["js", "jsx", "ts", "tsx", "dts"].includes(record.options?.lang)
        || !["module", "script", "commonjs", "unambiguous"].includes(record.options?.sourceType)
      ) {
        throw new Error(`Invalid corpus record in ${shard.file}`);
      }
      count++;
      yield record;
    }
    await completed;
    if (count !== shard.records) throw new Error(`PGO shard record count mismatch: ${shard.file}`);
  } finally {
    input.destroy();
    unzip.destroy();
  }
}

export async function prepareCorpus(directory, ecosystemRoot, fetchRepositories) {
  await mkdir(directory, { recursive: true });
  // A failed preparation must not leave an old manifest that looks usable.
  await rm(join(directory, "manifest.json"), { force: true });
  const temporary = join(root, "target/pgo/prepare");
  await mkdir(temporary, { recursive: true });
  if (fetchRepositories) {
    ecosystemRoot = join(temporary, "ecosystem");
    await rm(ecosystemRoot, { force: true, recursive: true });
    await run("git", [
      "clone",
      "--depth=1",
      "https://github.com/oxc-project/oxc-ecosystem-ci.git",
      ecosystemRoot,
    ]);
  }
  const ecosystemRevision = await git(ecosystemRoot, ["rev-parse", "HEAD"]);
  const matrices = await Promise.all(
    ["oxlint", "oxfmt"].map(async (tool) =>
      JSON.parse(await git(ecosystemRoot, ["show", `HEAD:${tool}-matrix.json`])),
    ),
  );
  const entries = repositories(matrices);
  const coverage = join(temporary, "coverage.jsonl");
  const transforms = join(temporary, "transforms.jsonl");
  await run("cargo", [
    "run",
    "--locked",
    "--profile",
    "coverage",
    "-p",
    "oxc_coverage",
    "--",
    "pgo-corpus",
    "--output",
    coverage,
  ]);
  await run("cargo", [
    "run",
    "--locked",
    "--profile",
    "coverage",
    "-p",
    "oxc_transform_conformance",
    "--",
    "--export-pgo",
    transforms,
  ]);
  // Preparation must accept the same inputs as shipping bindings, including
  // negative conformance fixtures that violate debug-only transformer checks.
  await run("cargo", [
    "build",
    "--locked",
    "--release",
    "--features",
    "allocator",
    "-p",
    "oxc_transform_napi",
  ]);
  const library =
    process.platform === "win32"
      ? "oxc_transform_napi.dll"
      : process.platform === "darwin"
        ? "liboxc_transform_napi.dylib"
        : "liboxc_transform_napi.so";
  const { copyFile } = await import("node:fs/promises");
  const helper = join(temporary, "transform.node");
  await copyFile(join(root, "target/release", library), helper);
  const previous = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = helper;
  const { transformSync } = await import(pathToFileURL(join(root, "napi/transform/index.js")));
  if (previous === undefined) delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
  else process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previous;
  const writer = new CorpusWriter(directory);
  for (const path of [coverage, transforms]) {
    for await (const record of jsonRecords(createReadStream(path))) {
      await writer.add(record, transformSync);
    }
  }
  const revisions = [];
  for (const entry of entries) {
    const checkout = fetchRepositories
      ? join(temporary, "checkout")
      : join(ecosystemRoot, "repos", entry.path);
    if (fetchRepositories) {
      await rm(checkout, { force: true, recursive: true });
      await run("git", [
        "clone",
        "--depth=1",
        "--filter=blob:none",
        "--no-checkout",
        "--branch",
        entry.ref,
        `https://github.com/${entry.repository}.git`,
        checkout,
      ]);
      await git(checkout, ["sparse-checkout", "set", "--no-cone", "--stdin"], {
        input:
          ["*.js", "*.jsx", "*.mjs", "*.cjs", "*.ts", "*.tsx", "*.mts", "*.cts"].join("\n") + "\n",
      });
      await git(checkout, ["checkout", "--detach"]);
    }
    const revision = await git(checkout, ["rev-parse", "HEAD"]);
    let files = 0;
    console.log(`Preparing ${entry.repository}@${revision}`);
    for await (const { filename, code } of sourceFiles(checkout, revision)) {
      await writer.add(
        {
          suite: `ecosystem/${entry.path}`,
          filename: `ecosystem/${entry.path}/${filename}`,
          code,
          options: { lang: language(filename), sourceType: sourceType(filename) },
          expectedFailure: false,
        },
        transformSync,
      );
      files++;
    }
    if (!files) throw new Error(`No JS/TS sources in ${entry.repository}`);
    revisions.push({ repository: entry.repository, path: entry.path, revision, files });
    if (fetchRepositories) await rm(checkout, { force: true, recursive: true });
  }
  await writer.closeShard();
  const suites = {};
  for (const name of ["test262", "babel", "typescript", "estree-conformance"]) {
    suites[name] = await git(join(root, "tasks/coverage", name), ["rev-parse", "HEAD"]);
  }
  await writeFile(
    join(directory, "manifest.json"),
    JSON.stringify(
      {
        version: 1,
        oxcRevision: await git(root, ["rev-parse", "HEAD"]),
        ecosystemRevision,
        suites,
        repositories: revisions,
        counts: writer.counts,
        preparation: writer.preparation,
        shards: writer.shards,
      },
      null,
      2,
    ) + "\n",
  );
  const manifest = await verifyCorpus(directory);
  console.log(
    `Prepared ${Object.values(manifest.counts).reduce((a, b) => a + b, 0)} inputs in ${manifest.shards.length} shards`,
  );
  console.log(`Minifier preparation: ${JSON.stringify(manifest.preparation)}`);
  return resolve(directory);
}
