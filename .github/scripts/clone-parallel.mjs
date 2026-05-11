#!/usr/bin/env node
// oxlint-disable no-console

// Clone submodules in parallel for faster setup
// Usage: node clone-parallel.mjs [test262] [babel] [typescript] [prettier] [estree-conformance] [node-compat-table]
// Arguments: "true" or "false" for each submodule

import { spawn } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";

// Submodule commit SHAs - updated automatically by .github/workflows/update_submodules.yml
// NOTE: Prettier version is now pinned to `v3.8.2` (not updated by workflow above), Update manually as needed
const TEST262_SHA = "d0c1b4555b03dd404873fd6422a4b5da00136500";
const BABEL_SHA = "6402dbbfb608f27a411f73fb61b25df053f47530";
const TYPESCRIPT_SHA = "f350b52331494b68c90ab02e2b6d0828d2a22a74";
const PRETTIER_SHA = "d7108a79ec745c04292aabf22c4c1adbd690b191";
const ESTREE_CONFORMANCE_SHA = "d968f2ca324ee6c024e6efb04978bb45649943ab";
const NODE_COMPAT_TABLE_SHA = "499beb6f1daa36f10c26b85a7f3ec3b3448ded23";

const repoRoot = join(import.meta.dirname, "..", "..");

// Parse command line arguments (default all to true)
const args = process.argv.slice(2);
const TEST262 = args[0] !== "false";
const BABEL = args[1] !== "false";
const TYPESCRIPT = args[2] !== "false";
const PRETTIER = args[3] !== "false";
const ESTREE_CONFORMANCE = args[4] !== "false";
const NODE_COMPAT_TABLE = args[5] !== "false";

/**
 * Run a git command and return a promise
 * @param {string[]} gitArgs - Arguments to pass to git
 * @param {string} cwd - Working directory
 * @returns {Promise<void>}
 */
function runGit(gitArgs, cwd) {
  return new Promise((resolve, reject) => {
    const proc = spawn("git", gitArgs, {
      cwd,
      stdio: ["ignore", "ignore", "pipe"],
    });

    let stderr = "";
    proc.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    proc.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`git ${gitArgs.join(" ")} failed: ${stderr}`));
      }
    });

    proc.on("error", reject);
  });
}

/**
 * Clone or update a repository
 * @param {boolean} shouldClone - Whether to clone this repo
 * @param {string} repo - GitHub repo path (e.g., "tc39/test262")
 * @param {string} path - Local path relative to repo root
 * @param {string} ref - Git ref (SHA) to checkout
 * @param {string} name - Display name for logging
 * @returns {Promise<{success: boolean, name: string, error?: string}>}
 */
async function cloneRepo(shouldClone, repo, path, ref, name) {
  if (!shouldClone) {
    console.log(`Skipping ${name}`);
    return { success: true, name, skipped: true };
  }

  console.log(`Starting clone of ${name}...`);

  const fullPath = join(repoRoot, path);
  const gitDir = join(fullPath, ".git");
  const repoUrl = `https://github.com/${repo}.git`;

  try {
    // Create parent directory if needed
    const parentDir = dirname(fullPath);
    if (!existsSync(parentDir)) {
      mkdirSync(parentDir, { recursive: true });
    }

    if (existsSync(gitDir)) {
      // Directory exists with git repo - ensure origin URL is correct
      try {
        await runGit(["remote", "set-url", "origin", repoUrl], fullPath);
      } catch {
        await runGit(["remote", "add", "origin", repoUrl], fullPath);
      }
    } else {
      // Directory doesn't exist or has no git repo - initialize it
      if (!existsSync(fullPath)) {
        mkdirSync(fullPath, { recursive: true });
      }
      await runGit(["init", "--quiet"], fullPath);
      await runGit(["remote", "add", "origin", repoUrl], fullPath);
    }

    // Fetch and checkout the specific commit
    await runGit(["fetch", "--quiet", "--depth", "1", "origin", ref], fullPath);
    await runGit(["reset", "--hard", ref], fullPath);
    await runGit(["clean", "-f", "-q"], fullPath);

    console.log(`[OK] Completed clone of ${name}`);
    return { success: true, name };
  } catch (error) {
    console.error(`[FAILED] Clone operation failed for ${name}: ${error.message}`);
    return { success: false, name, error: error.message };
  }
}

async function main() {
  console.log("Cloning submodules in parallel...");

  // Start all clone operations in parallel
  const results = await Promise.all([
    cloneRepo(TEST262, "tc39/test262", "tasks/coverage/test262", TEST262_SHA, "test262"),
    cloneRepo(BABEL, "babel/babel", "tasks/coverage/babel", BABEL_SHA, "babel"),
    cloneRepo(
      TYPESCRIPT,
      "microsoft/TypeScript",
      "tasks/coverage/typescript",
      TYPESCRIPT_SHA,
      "typescript",
    ),
    cloneRepo(
      PRETTIER,
      "prettier/prettier",
      "tasks/prettier_conformance/prettier",
      PRETTIER_SHA,
      "prettier",
    ),
    cloneRepo(
      ESTREE_CONFORMANCE,
      "oxc-project/estree-conformance",
      "tasks/coverage/estree-conformance",
      ESTREE_CONFORMANCE_SHA,
      "estree-conformance",
    ),
    cloneRepo(
      NODE_COMPAT_TABLE,
      "compat-table/node-compat-table",
      "tasks/coverage/node-compat-table",
      NODE_COMPAT_TABLE_SHA,
      "node-compat-table",
    ),
  ]);

  // Count failures (excluding skipped)
  const failedCount = results.filter((r) => !r.success).length;

  if (failedCount === 0) {
    console.log("[OK] All submodule clones completed successfully!");
    process.exit(0);
  } else {
    console.error(`[FAILED] ${failedCount} clone operation(s) failed`);
    process.exit(1);
  }
}

await main();
