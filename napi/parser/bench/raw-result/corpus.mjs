import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

export function loadCorpus(directory, limit = Infinity) {
  assert.ok(
    limit === Infinity || (Number.isInteger(limit) && limit > 0),
    "limit must be a positive integer",
  );
  const names = fs
    .readdirSync(path.join(directory, "src"), { recursive: true })
    .filter(
      (name) => /\.(?:[cm]?[jt]sx?)$/.test(name) && !name.split(path.sep).includes("node_modules"),
    )
    .sort();
  const selected = Number.isFinite(limit)
    ? names
        .filter((_, i) => i % Math.max(1, Math.floor(names.length / limit)) === 0)
        .slice(0, limit)
    : names;
  const hash = createHash("sha256");
  const files = selected.map((name) => {
    const filename = `src/${name}`;
    const source = fs.readFileSync(path.join(directory, filename), "utf8");
    hash.update(filename).update("\0").update(source).update("\0");
    return { filename, source, ts: /\.[cm]?tsx?$/.test(name), jsx: /\.[jt]sx$/.test(name) };
  });
  return {
    files,
    metadata: {
      files: files.length,
      bytes: files.reduce((sum, file) => sum + Buffer.byteLength(file.source), 0),
      sha256: hash.digest("hex"),
    },
  };
}
