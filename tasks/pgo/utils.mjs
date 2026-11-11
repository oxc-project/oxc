import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";

export const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

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

export function profileEnvironment(target, flags, configPath, base = process.env) {
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
  // napi-rs promotes CARGO_BUILD_RUSTFLAGS to RUSTFLAGS before invoking Cargo.
  const flagKey = ["CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", targetKey].find(
    (key) => env[key] !== undefined,
  );
  if (!flagKey) return { env, cargoOptions: ["--config", configPath] };
  const separator = flagKey === "CARGO_ENCODED_RUSTFLAGS" ? "\x1f" : " ";
  if (separator === " " && flags.some((flag) => /\s/.test(flag))) {
    throw new Error(`PGO paths cannot contain whitespace with ${flagKey} set`);
  }
  env[flagKey] += separator + flags.join(separator);
  return { env, cargoOptions: [] };
}
