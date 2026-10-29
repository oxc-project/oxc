// oxlint-disable no-console, no-await-in-loop -- Serialize builds and finish training before merging profiles.
import { copyFile, mkdir, readdir, rm, stat, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { verifyCorpus } from "./corpus.mjs";
import { packPackage } from "./pack.mjs";
import {
  git,
  largeStackWorker,
  packages,
  pgoRustflags,
  profileEnvironment,
  root,
  run,
  sha256,
  targets,
} from "./utils.mjs";

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    "ecosystem-root": { type: "string" },
    corpus: { type: "string" },
    target: { type: "string" },
    fetch: { type: "boolean", default: false },
    help: { type: "boolean", default: false },
  },
});
if (values.help) {
  console.log(`Usage:
  just pgo <parser|transform|minify|all> --ecosystem-root /path/to/oxc-ecosystem-ci
  just pgo prepare --fetch [--corpus target/pgo/corpus]
  just pgo <package> --corpus target/pgo/corpus [--target <native-triple>]

The local ecosystem root is read-only. --fetch clones inputs under target/pgo.
Install the pinned toolchain's profiler with: rustup component add llvm-tools-preview`);
  process.exit(0);
}
const [command] = positionals;
if (positionals.length !== 1 || ![...packages, "all", "prepare"].includes(command)) {
  throw new Error("Choose parser, transform, minify, all, or prepare; see --help");
}
if (values.fetch && values["ecosystem-root"]) throw new Error("Choose --fetch or --ecosystem-root");
const directory = resolve(root, values.corpus ?? "target/pgo/corpus");
if (command === "prepare" || values["ecosystem-root"] || values.fetch) {
  if (!values.fetch && !values["ecosystem-root"]) {
    throw new Error("Preparation requires --ecosystem-root or --fetch");
  }
  await largeStackWorker(new URL("./prepare.mjs", import.meta.url), {
    workerData: {
      directory,
      ecosystemRoot: values["ecosystem-root"] && resolve(values["ecosystem-root"]),
      fetch: values.fetch,
    },
  });
}
if (command === "prepare") process.exit(0);
const revision = await git(root, ["rev-parse", "HEAD"]);
const manifest = await verifyCorpus(directory, { revision });
const rustc = await run("rustc", ["-vV"], { capture: true });
const host = /^host: (.+)$/m.exec(rustc)?.[1];
const target = values.target ?? host;
if (!targets.includes(target) || target !== host) {
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
const platform = {
  "x86_64-unknown-linux-gnu": "linux-x64-gnu",
  "aarch64-unknown-linux-gnu": "linux-arm64-gnu",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-apple-darwin": "darwin-arm64",
  "x86_64-pc-windows-msvc": "win32-x64-msvc",
}[target];
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
        [join(root, "tasks/pgo/train.mjs"), name, directory, instrumented, shard.file],
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
  await packPackage(name, target);
  await copyFile(optimized, destination);
  console.log(`Validated PGO package and binding: ${destination}`);
}
