// oxlint-disable no-console, no-await-in-loop -- Prepare repositories sequentially to bound disk usage.
import { spawn } from "node:child_process";
import { copyFile, mkdir, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { join } from "node:path";
import { workerData } from "node:worker_threads";
import { CorpusWriter, verifyCorpus } from "./corpus.mjs";
import { git, root, run } from "./utils.mjs";

const sourceExtension = /\.(?:[cm]?[jt]s|[jt]sx)$/i;

function language(filename) {
  if (/\.d\.[cm]?ts$/i.test(filename)) return "dts";
  if (/\.tsx$/i.test(filename)) return "tsx";
  if (/\.[cm]?ts$/i.test(filename)) return "ts";
  return /\.jsx$/i.test(filename) ? "jsx" : "js";
}

function sourceType(filename) {
  if (/\.[cm][jt]s$/i.test(filename)) {
    return /\.c[jt]s$/i.test(filename) ? "commonjs" : "module";
  }
  return "unambiguous";
}

function repositories(matrices) {
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
async function* sourceFiles(directory, revision) {
  const tree = await git(directory, ["ls-tree", "-r", "-z", revision]);
  const entries = tree
    .split("\0")
    .filter(Boolean)
    .map((line) => {
      const match = /^(100644|100755) blob ([a-f0-9]+)\t(.+)$/s.exec(line);
      return match && sourceExtension.test(match[3]) ? { oid: match[2], filename: match[3] } : null;
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

const { directory, fetch: fetchRepositories } = workerData;
let { ecosystemRoot } = workerData;
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
// Use the shipping release profile when lowering ecosystem TS/JSX for minification.
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
const helper = join(temporary, "transform.node");
await copyFile(join(root, "target/release", library), helper);
const { transformSync } = createRequire(import.meta.url)(helper);
const writer = new CorpusWriter(directory);
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
await writeFile(
  join(directory, "manifest.json"),
  JSON.stringify(
    {
      version: 1,
      oxcRevision: await git(root, ["rev-parse", "HEAD"]),
      ecosystemRevision,
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
