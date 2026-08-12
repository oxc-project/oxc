#!/usr/bin/env node
// oxlint-disable no-console

// Clone submodules in parallel for faster setup
// Usage: node clone-parallel.mjs [test262] [babel] [typescript] [estree-conformance] [node-compat-table]
// Arguments: "true" or "false" for each submodule

import { spawn } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";

// Submodule commit SHAs - updated automatically by .github/workflows/update_submodules.yml
const TEST262_SHA = "be13516fb6441b950ba8a3df97eb34062c186972";
const BABEL_SHA = "1eac4481473df52fbbcb452c4dca8d79039dbb63";
const TYPESCRIPT_SHA = "b465fdbfe175304d9b977da137b2c178ae1091d3";
const ESTREE_CONFORMANCE_SHA = "8d4e1faac9f350f914af9b8a6ec02c2d31ca799c";
const NODE_COMPAT_TABLE_SHA = "499beb6f1daa36f10c26b85a7f3ec3b3448ded23";

const repoRoot = join(import.meta.dirname, "..", "..");

// Parse command line arguments (default all to true)
const args = process.argv.slice(2);
const TEST262 = args[0] !== "false";
const BABEL = args[1] !== "false";
const TYPESCRIPT = args[2] !== "false";
const ESTREE_CONFORMANCE = args[3] !== "false";
const NODE_COMPAT_TABLE = args[4] !== "false";

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
