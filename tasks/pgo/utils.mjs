import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";

export const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export const packages = ["parser", "transform", "minify"];
export const pgoRustflags = [
  // LLVM 23 can add uninitialized enum payloads to select counters. Until the
  // pinned toolchain includes the fix, collect branch counts without selects.
  // https://github.com/rust-lang/rust/issues/163616
  "-Cllvm-args=-pgo-instr-select=false",
  "-Cllvm-args=-vp-counters-per-site=16",
];
export const targets = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
];

export function pnpm(args, options) {
  // `pnpm node` supplies the actual executable or JS entry point, including on
  // Windows where spawning the pnpm.cmd shim without a shell is unsupported.
  const executable = process.env.npm_execpath;
  if (!executable || !basename(executable).startsWith("pnpm")) {
    throw new Error("Run PGO tooling with pnpm node (or just pgo)");
  }
  return /\.[cm]?js$/.test(executable)
    ? run(process.execPath, [executable, ...args], options)
    : run(executable, args, options);
}

export function largeStackWorker(url, options = {}) {
  // The full corpus contains generated inputs with hundreds of thousands of
  // nested statements. Reserve stack address space; pages are committed on demand.
  const worker = new Worker(url, {
    ...options,
    execArgv: [],
    resourceLimits: { stackSizeMb: 512 },
  });
  return new Promise((resolvePromise, reject) => {
    worker.once("error", reject);
    worker.once("exit", (code) =>
      code === 0 ? resolvePromise() : reject(new Error(`PGO worker failed (${code})`)),
    );
  });
}

// Use argument arrays throughout: filenames and corpus paths must never be shell code.
export async function run(
  command,
  args,
  { cwd = root, env = process.env, capture = false, input } = {},
) {
  const child = spawn(command, args, {
    cwd,
    env,
    stdio: [input === undefined ? "ignore" : "pipe", capture ? "pipe" : "inherit", "inherit"],
  });
  const chunks = [];
  if (capture) child.stdout.on("data", (chunk) => chunks.push(chunk));
  if (input !== undefined) child.stdin.end(input);
  await new Promise((resolvePromise, reject) => {
    child.on("error", reject);
    child.on("close", (code, signal) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`${command} ${args.join(" ")} failed (${signal ?? code})`));
    });
  });
  return capture ? Buffer.concat(chunks).toString("utf8").trimEnd() : undefined;
}

export function git(directory, args, options = {}) {
  return run("git", ["-C", directory, ...args], { ...options, capture: true });
}

export async function sha256(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

export function language(filename) {
  if (/\.d\.[cm]?ts$/i.test(filename)) return "dts";
  if (/\.tsx$/i.test(filename)) return "tsx";
  if (/\.[cm]?ts$/i.test(filename)) return "ts";
  return /\.jsx$/i.test(filename) ? "jsx" : "js";
}

export function sourceType(filename) {
  if (/\.[cm][jt]s$/i.test(filename)) {
    return /\.c[jt]s$/i.test(filename) ? "commonjs" : "module";
  }
  return "unambiguous";
}

export function profileEnvironment(target, flags, configPath, base = process.env) {
  if (typeof flags === "string") flags = [flags];
  const env = { ...base };
  if (target.endsWith("apple-darwin") || target.endsWith("linux-gnu")) {
    // cc-rs forwards Rust's PGO flags to C/C++ dependencies. Their compiler may
    // use a different profile layout than Rust's LLVM runtime (notably Apple
    // clang and mimalloc), so keep native dependencies out of instrumentation.
    // https://github.com/rust-lang/rust/issues/163640
    for (const prefix of ["CFLAGS", "CXXFLAGS"]) {
      const key = `${prefix}_${target.replaceAll("-", "_")}`;
      const existing =
        env[key] ?? env[`${prefix}_${target}`] ?? env[`TARGET_${prefix}`] ?? env[prefix] ?? "";
      env[key] = `${existing} -fno-profile-generate -fno-profile-use`;
    }
  }
  // Respect Cargo's precedence. If no environment override exists, use a Cargo
  // config overlay so its array is appended to the repository's target flags.
  const targetKey = `CARGO_TARGET_${target.toUpperCase().replaceAll("-", "_")}_RUSTFLAGS`;
  if (env.CARGO_ENCODED_RUSTFLAGS !== undefined) {
    env.CARGO_ENCODED_RUSTFLAGS += `\x1f${flags.join("\x1f")}`;
  } else if (env.RUSTFLAGS !== undefined) {
    // Encoded flags avoid splitting absolute paths containing spaces.
    if (flags.some((flag) => /\s/.test(flag))) {
      throw new Error("PGO paths cannot contain whitespace with RUSTFLAGS set");
    }
    env.RUSTFLAGS += ` ${flags.join(" ")}`;
  } else if (env.CARGO_BUILD_RUSTFLAGS !== undefined) {
    // napi-rs promotes this variable to RUSTFLAGS before invoking Cargo.
    if (flags.some((flag) => /\s/.test(flag))) {
      throw new Error("PGO paths cannot contain whitespace with CARGO_BUILD_RUSTFLAGS set");
    }
    env.CARGO_BUILD_RUSTFLAGS += ` ${flags.join(" ")}`;
  } else if (env[targetKey] !== undefined) {
    if (flags.some((flag) => /\s/.test(flag))) {
      throw new Error("PGO paths cannot contain whitespace with target RUSTFLAGS set");
    }
    env[targetKey] += ` ${flags.join(" ")}`;
  } else {
    return { env, cargoOptions: ["--config", configPath] };
  }
  return { env, cargoOptions: [] };
}
