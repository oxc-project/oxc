// oxlint-disable no-await-in-loop -- Verify shards sequentially to bound disk reads.
import { once } from "node:events";
import { createReadStream, createWriteStream } from "node:fs";
import { readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { pipeline } from "node:stream/promises";
import { createGzip, createGunzip } from "node:zlib";
import { sha256 } from "./utils.mjs";

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

export async function verifyCorpus(directory, revision) {
  const manifest = JSON.parse(await readFile(join(directory, "manifest.json"), "utf8"));
  if (manifest.version !== 1 || !manifest.shards?.length || !manifest.repositories?.length) {
    throw new Error("Invalid or empty PGO corpus manifest");
  }
  if (revision && manifest.oxcRevision !== revision) {
    throw new Error("PGO corpus belongs to a different Oxc revision");
  }
  if (
    Object.keys(manifest.counts).length !== manifest.repositories.length
    || manifest.repositories.some(
      (repo) => !(repo.files > 0) || manifest.counts[`ecosystem/${repo.path}`] !== repo.files,
    )
  ) {
    throw new Error("PGO corpus does not match its ecosystem repositories");
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
