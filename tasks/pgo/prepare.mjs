// oxlint-disable no-console, no-await-in-loop -- Install and collect one repository at a time to bound disk usage.
import { copyFile, mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { extname, join } from "node:path";
import { workerData } from "node:worker_threads";
import { CorpusWriter, verifyCorpus } from "./corpus.mjs";
import { git, root, run } from "./utils.mjs";

const { directory } = workerData;
// Keep checkouts outside Oxc's pnpm workspace so package managers use their own config.
const temporary = await mkdtemp(join(tmpdir(), "oxc-pgo-"));
const writer = new CorpusWriter(directory);
const repositories = [];
const extensions = {};
const json = async (path) => JSON.parse(await readFile(path, "utf8"));

function language(filename) {
  if (/\.d\.[cm]?ts$/i.test(filename)) return "dts";
  if (/\.tsx$/i.test(filename)) return "tsx";
  if (/\.[cm]?ts$/i.test(filename)) return "ts";
  return /\.jsx$/i.test(filename) ? "jsx" : "js";
}

async function clone(repository, checkout, ref) {
  await rm(checkout, { force: true, recursive: true });
  const url = `https://github.com/${repository}.git`;
  await run("git", ["clone", "--depth=1", "--no-tags", url, checkout]);
  if (ref) {
    await git(checkout, ["fetch", "--depth=1", "origin", ref]);
    await git(checkout, ["checkout", "--detach", "FETCH_HEAD"]);
  }
}

async function install(checkout) {
  const files = await readdir(checkout);
  if (!files.includes("package.json")) return;
  const { packageManager } = await json(join(checkout, "package.json"));
  const manager =
    packageManager?.split("@")[0]
    ?? (files.includes("pnpm-lock.yaml")
      ? "pnpm"
      : files.includes("yarn.lock")
        ? "yarn"
        : files.some((file) => /^bun.lockb?$/.test(file))
          ? "bun"
          : "npm");
  const args =
    manager === "bun" ? [packageManager ?? "bun", "install"] : ["corepack", manager, "install"];
  const modernYarn =
    manager === "yarn"
    && (files.includes(".yarnrc.yml") || (packageManager && !packageManager.startsWith("yarn@1.")));
  args.push(modernYarn ? "--mode=skip-build" : "--ignore-scripts");
  if (manager === "pnpm" && !files.includes("pnpm-workspace.yaml")) args.push("--ignore-workspace");
  await run("npx", ["--yes", "--prefix", temporary, ...args], {
    cwd: checkout,
    env: {
      ...process.env,
      NODE_ENV: "development",
      COREPACK_ENABLE_DOWNLOAD_PROMPT: "0",
      npm_config_engine_strict: "false",
      YARN_ENABLE_SCRIPTS: "false",
      YARN_ENABLE_IMMUTABLE_INSTALLS: "false",
      YARN_NODE_LINKER: "node-modules",
    },
  });
}

async function collect(checkout, suite, prefix = "") {
  for (const entry of await readdir(join(checkout, prefix), { withFileTypes: true })) {
    const filename = join(prefix, entry.name);
    if (entry.isDirectory() && entry.name !== ".git") {
      await collect(checkout, suite, filename);
    } else if (entry.isFile() && /\.(?:[cm]?[jt]s|[jt]sx)$/i.test(filename)) {
      const extension = extname(filename).toLowerCase();
      const sourceType =
        { ".cjs": "commonjs", ".cts": "commonjs", ".mjs": "module", ".mts": "module" }[extension]
        ?? "unambiguous";
      const bytes = await readFile(join(checkout, filename));
      const encoding =
        { fffe: "utf-16le", feff: "utf-16be" }[bytes.subarray(0, 2).toString("hex")] ?? "utf-8";
      await writer.add(
        {
          suite,
          filename: `${suite}/${filename}`,
          code: new TextDecoder(encoding).decode(bytes),
          options: { lang: language(filename), sourceType },
        },
        transformSync,
      );
      extensions[extension] = (extensions[extension] ?? 0) + 1;
    }
  }
}

async function prepare(repository, path, checkout) {
  console.log(`Preparing ${repository}`);
  let installationError;
  try {
    await install(checkout);
  } catch (error) {
    installationError = error.message;
    console.warn(`Dependency installation failed for ${repository}: ${installationError}`);
  }
  await collect(checkout, path);
  const files = writer.counts[path];
  if (!files) throw new Error(`No JS/TS sources in ${repository}`);
  repositories.push({
    repository,
    path,
    revision: await git(checkout, ["rev-parse", "HEAD"]),
    files,
    installationError,
  });
  console.log(`${repository}: collected ${files} files`);
}

await mkdir(directory, { recursive: true });
// A failed preparation must not leave an old manifest that looks usable.
await rm(join(directory, "manifest.json"), { force: true });
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

for (const name of ["oxc-ecosystem-ci", "monitor-oxc"]) {
  const checkout = join(temporary, name);
  await clone(`oxc-project/${name}`, checkout);
  await prepare(`oxc-project/${name}`, name, checkout);
}
const ecosystem = join(temporary, "oxc-ecosystem-ci");
const monitor = join(temporary, "monitor-oxc");
const matrices = (
  await Promise.all(["oxlint", "oxfmt"].map((tool) => json(join(ecosystem, `${tool}-matrix.json`))))
).flat();
const entries = [...new Map(matrices.map((entry) => [entry.path, entry])).values()];
for (const entry of entries) entry.path = `oxc-ecosystem-ci/repos/${entry.path}`;
for (const entry of await json(join(monitor, "runtime-repos.json"))) {
  const path = `monitor-oxc/repos/${entry.name}`;
  entries.push({ repository: entry.repo, ref: entry.sha, path });
}
for (const { repository, path, ref } of entries) {
  const checkout = join(temporary, "checkout");
  await clone(repository, checkout, ref);
  await prepare(repository, path, checkout);
  await rm(checkout, { force: true, recursive: true });
}
await writer.closeShard();
await writeFile(
  join(directory, "manifest.json"),
  JSON.stringify(
    {
      version: 2,
      oxcRevision: await git(root, ["rev-parse", "HEAD"]),
      repositories,
      counts: writer.counts,
      extensions,
      preparation: writer.preparation,
      shards: writer.shards,
    },
    null,
    2,
  ) + "\n",
);
const manifest = await verifyCorpus(directory);
console.table(extensions);
console.log(
  `Prepared ${Object.values(manifest.counts).reduce((a, b) => a + b, 0)} inputs in ${manifest.shards.length} shards`,
);
console.log(
  `Dependency installation failures: ${repositories.filter((repo) => repo.installationError).length}`,
);
console.log(`Minifier preparation: ${JSON.stringify(manifest.preparation)}`);
await rm(temporary, { force: true, recursive: true });
