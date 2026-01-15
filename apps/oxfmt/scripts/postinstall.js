#!/usr/bin/env node
/**
 * esbuild-style postinstall optimization for faster CLI startup.
 *
 * Replaces the Node.js bin wrapper (`bin/oxfmt`) with a hardlink to the native
 * Rust binary, reducing startup time from ~50ms to ~3ms (warm cache).
 *
 * This approach is inspired by esbuild's `maybeOptimizePackage()` function:
 * @see https://github.com/evanw/esbuild/blob/main/lib/npm/node-install.ts
 *
 * How it works:
 * 1. Finds the platform-specific native binary from `@oxfmt/{platform}` package
 * 2. Creates a hardlink to avoid duplicating disk space
 * 3. Atomically replaces `bin/oxfmt` JS wrapper with the native binary
 *
 * Limitations:
 * - Only works on Unix (Windows needs .exe extension for binaries)
 * - Skipped with Yarn (to avoid idempotency issues across reinstalls)
 * - The pure Rust binary only supports JS/TS/TOML formatting
 * - For non-JS files (JSON, YAML, CSS, etc.), the NAPI+Prettier approach is needed
 *
 * This optimization is ideal for:
 * - LSP usage (oxc-zed) where JS/TS formatting is the primary use case
 * - CI pipelines where startup time matters
 *
 * To disable this optimization, set OXFMT_NO_POSTINSTALL=1 environment variable.
 *
 * Related issues:
 * - https://github.com/oxc-project/oxc/issues/14294 (oxlint startup time)
 * - https://github.com/oxc-project/oxc/issues/16606 (oxfmt performance)
 */
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

// ESM compatibility: create require function for resolving node_modules packages
const require = createRequire(import.meta.url);

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const binPath = path.join(__dirname, "..", "bin", "oxfmt");

function isYarn() {
  const { npm_config_user_agent } = process.env;
  return npm_config_user_agent && /\byarn\//.test(npm_config_user_agent);
}

function getPlatformPackage() {
  const platform = os.platform();
  const arch = os.arch();

  // Map Node.js platform/arch to oxfmt package names
  const platformMap = {
    "darwin-arm64": "@oxfmt/darwin-arm64",
    "darwin-x64": "@oxfmt/darwin-x64",
    "linux-x64": "@oxfmt/linux-x64-gnu",
    "linux-arm64": "@oxfmt/linux-arm64-gnu",
    "win32-x64": "@oxfmt/win32-x64",
    "win32-arm64": "@oxfmt/win32-arm64",
  };

  return platformMap[`${platform}-${arch}`];
}

function findNativeBinary() {
  const pkg = getPlatformPackage();
  if (!pkg) return null;

  try {
    // Find the platform-specific package
    const pkgPath = path.dirname(require.resolve(`${pkg}/package.json`));
    const binName = os.platform() === "win32" ? "oxfmt.exe" : "oxfmt";
    const binFile = path.join(pkgPath, binName);
    return fs.existsSync(binFile) ? binFile : null;
  } catch {
    return null;
  }
}

function maybeOptimize() {
  // Skip if explicitly disabled
  if (process.env.OXFMT_NO_POSTINSTALL === "1") {
    console.log("[oxfmt] Postinstall optimization disabled via OXFMT_NO_POSTINSTALL=1");
    return;
  }

  // Skip on Windows (binary needs .exe extension, bin scripts work differently)
  if (os.platform() === "win32") return;

  // Skip with Yarn (idempotency issues with reinstalls)
  if (isYarn()) return;

  const nativeBinary = findNativeBinary();
  if (!nativeBinary) {
    // Native binary not found - likely not shipped yet or --no-optional was used
    return;
  }

  const tempPath = binPath + ".tmp";
  try {
    // Hardlink preserves inode, saves disk space
    fs.linkSync(nativeBinary, tempPath);
    // Atomic replace using rename
    fs.renameSync(tempPath, binPath);
    console.log("[oxfmt] Optimized: using native binary for faster startup (~15ms vs ~400ms)");
  } catch (e) {
    // Optimization is optional, don't fail install on errors
    // This can happen if filesystem doesn't support hardlinks (e.g., some network mounts)
    try {
      fs.unlinkSync(tempPath);
    } catch {}
  }
}

maybeOptimize();
