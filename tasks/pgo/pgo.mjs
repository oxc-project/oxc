// oxlint-disable no-console, no-await-in-loop -- Serialize builds and finish training before merging profiles.
import { copyFile, mkdir, readdir, rm, stat, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { verifyCorpus } from "./corpus.mjs";
import { git, largeStackWorker, profileEnvironment, root, run, sha256 } from "./utils.mjs";

const packages = ["parser", "transform", "minify"];
const platforms = {
  "x86_64-unknown-linux-gnu": "linux-x64-gnu",
  "aarch64-unknown-linux-gnu": "linux-arm64-gnu",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-apple-darwin": "darwin-arm64",
  "x86_64-pc-windows-msvc": "win32-x64-msvc",
};
const pgoRustflags = [
  // LLVM 23 can add uninitialized enum payloads to select counters. Until the
  // pinned toolchain includes the fix, collect branch counts without selects.
  // https://github.com/rust-lang/rust/issues/163616
  "-Cllvm-args=-pgo-instr-select=false",
  "-Cllvm-args=-vp-counters-per-site=16",
];

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    corpus: { type: "string" },
    target: { type: "string" },
    help: { type: "boolean", default: false },
  },
});
if (values.help) {
  console.log(`Usage:
  just pgo prepare [--corpus target/pgo/corpus]
  just pgo <package> --corpus target/pgo/corpus [--target <native-triple>]

Preparation clones and installs corpus projects in a temporary directory.
Install the pinned toolchain's profiler with: rustup component add llvm-tools-preview`);
  process.exit(0);
}
const [command] = positionals;
if (positionals.length !== 1 || ![...packages, "all", "prepare"].includes(command)) {
  throw new Error("Choose parser, transform, minify, all, or prepare; see --help");
}
const directory = resolve(root, values.corpus ?? "target/pgo/corpus");
if (command === "prepare") {
  await largeStackWorker(new URL("./prepare.mjs", import.meta.url), {
    workerData: { directory },
  });
}
if (command === "prepare") process.exit(0);
const revision = await git(root, ["rev-parse", "HEAD"]);
const manifest = await verifyCorpus(directory, revision);
const rustc = await run("rustc", ["-vV"], { capture: true });
const host = /^host: (.+)$/m.exec(rustc)?.[1];
const target = values.target ?? host;
const platform = platforms[target];
if (!platform || target !== host) {
  throw new Error(`PGO requires a matching native host/target; host=${host}, target=${target}`);
}
const sysroot = await run("rustc", ["--print", "sysroot"], { capture: true });
const profdata = join(
  sysroot,
  "lib/rustlib",
  host,
  "bin",
  `llvm-profdata${process.platform === "win32" ? ".exe" : ""}`,
);
try {
  await stat(profdata);
} catch {
  throw new Error("Missing llvm-profdata; run rustup component add llvm-tools-preview");
}
// Invoke the CLI with Node rather than a shell shim, including on Windows.
const napi = join(root, "node_modules/@napi-rs/cli/dist/cli.js");
for (const name of command === "all" ? packages : [command]) {
  const work = join(root, "target/pgo", name, target);
  const raw = join(work, "raw");
  const profile = join(work, "merged.profdata");
  const packageDir = join(root, "napi", name);
  const binary = `${name}.${platform}.node`;
  const destination = join(packageDir, name === "parser" ? "src-js" : "", binary);
  await mkdir(work, { recursive: true });
  await rm(raw, { force: true, recursive: true });
  await rm(profile, { force: true });
  await mkdir(raw);
  async function build(phase, flag) {
    const stage = join(work, phase);
    await mkdir(stage, { recursive: true });
    const config = join(work, `${phase}.toml`);
    const flags = [flag, ...pgoRustflags];
    await writeFile(
      config,
      `target.${JSON.stringify(target)}.rustflags = ${JSON.stringify(flags)}\n`,
    );
    const { env, cargoOptions } = profileEnvironment(target, flags, config);
    const args = [
      napi,
      "build",
      "--esm",
      "--platform",
      "--no-js",
      "--dts",
      "binding.d.ts",
      "--features",
      "allocator",
      "--release",
      "--target",
      target,
      "--target-dir",
      join(work, "build"),
      "--output-dir",
      stage,
    ];
    if (target.endsWith("linux-gnu")) args.push("--use-napi-cross");
    args.push("--", "--locked", ...cargoOptions);
    await run(process.execPath, args, { cwd: packageDir, env });
    return join(stage, binary);
  }
  console.log(`Instrumenting ${name} for ${target}`);
  const instrumented = await build("instrumented", `-Cprofile-generate=${raw}`);
  let trained = 0,
    skipped = 0,
    diagnostics = 0,
    downlevelSkipped = 0;
  for (const shard of manifest.shards) {
    const result = JSON.parse(
      await run(
        process.execPath,
        [join(root, "tasks/pgo/train.mjs"), name, instrumented, directory, shard.file],
        {
          env: { ...process.env, LLVM_PROFILE_FILE: join(raw, "%m-%p.profraw") },
          capture: true,
        },
      ),
    );
    trained += result.processed;
    skipped += result.skipped;
    diagnostics += result.diagnostics;
    downlevelSkipped += result.downlevelSkipped;
    console.log(
      `${name}: ${shard.file}: ${result.processed} trained (${result.diagnostics} diagnostics), ${result.skipped} skipped`,
    );
  }
  if (!trained) throw new Error(`No ${name} training inputs`);
  const profiles = (await readdir(raw)).filter((file) => file.endsWith(".profraw"));
  if (!profiles.length) throw new Error(`No profiles generated for ${name}/${target}`);
  // Merge the directory to avoid command-line length limits on Windows.
  await run(profdata, ["merge", "--failure-mode=any", "-o", profile, raw]);
  const summary = await run(profdata, ["show", profile], { capture: true });
  if (!/Maximum function count:\s*[1-9]/.test(summary)) {
    throw new Error("PGO profile has no execution counts");
  }
  console.log(summary);
  // Cargo fingerprints the flag, not the contents of the profile it names.
  // A content-addressed path prevents reusing a build from older training data.
  const profileInput = join(work, `profile-${await sha256(profile)}.profdata`);
  await copyFile(profile, profileInput);
  const optimized = await build("optimized", `-Cprofile-use=${profileInput}`);
  await writeFile(
    join(work, "profile.json"),
    JSON.stringify(
      {
        package: name,
        target,
        revision,
        rustc,
        corpus: await sha256(join(directory, "manifest.json")),
        profile: await sha256(profile),
        binary: await sha256(optimized),
        rawProfiles: profiles.length,
        trained,
        skipped,
        diagnostics,
        downlevelSkipped,
        summary,
      },
      null,
      2,
    ) + "\n",
  );
  await run(process.execPath, [join(root, "tasks/pgo/train.mjs"), name, optimized]);
  await copyFile(optimized, destination);
  console.log(`Validated PGO binding: ${destination}`);
}
