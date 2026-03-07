#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Check if a NAPI package should be built based on changed files.
 * Takes crate name(s) and additional watch paths as CLI args.
 *
 * Usage:
 *   node check-napi-changes.js <crate_name> [additional_paths...]
 *
 * Examples:
 *   node check-napi-changes.js oxlint apps/oxlint/
 *   node check-napi-changes.js oxfmt apps/oxfmt/
 */

const { getChangedFiles } = require("./get-changed-files.js");
const { getCrateDependencies, checkFilesAffectCrates } = require("./utils.js");

/**
 * Paths that should always trigger a rebuild
 */
const ALWAYS_RUN_PATHS = [
  ".github/workflows/ci.yml",
  "Cargo.toml",
  "Cargo.lock",
  "pnpm-lock.yaml",
];

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.error("Usage: check-napi-changes.js <crate_name> [additional_paths...]");
    process.exit(1);
  }

  const crateName = args[0];
  const additionalPaths = args.slice(1);

  try {
    const changedFiles = await getChangedFiles();

    // null means manual trigger or error - always run
    if (changedFiles === null) {
      console.error("No changed files list (manual trigger or error) - will run");
      console.log("true");
      process.exit(0);
    }

    if (changedFiles.length === 0) {
      console.error("No files changed - will skip");
      console.log("false");
      process.exit(0);
    }

    // Check always-run paths
    for (const file of changedFiles) {
      for (const path of ALWAYS_RUN_PATHS) {
        if (file === path || file.startsWith(path)) {
          console.error(`File ${file} matches always-run path ${path} - will run`);
          console.log("true");
          process.exit(0);
        }
      }
    }

    // Get crate dependencies
    const deps = getCrateDependencies(crateName);
    console.error(`Dependencies for ${crateName} (${deps.length}):`);
    console.error(`  ${deps.join(", ")}`);

    // Check if any changed file affects a dependency crate or additional paths
    const shouldRun = checkFilesAffectCrates(changedFiles, deps, additionalPaths);

    console.log(shouldRun ? "true" : "false");

    if (shouldRun) {
      console.error(`::notice title=NAPI ${crateName}::Will build and test ${crateName}`);
    } else {
      console.error(`::notice title=NAPI ${crateName}::Will skip ${crateName}`);
    }

    process.exit(0);
  } catch (error) {
    console.error(`Error checking changes for ${crateName}:`, error);
    // On error, run as fallback
    console.log("true");
    console.error(
      `::warning title=NAPI ${crateName} check error::Error occurred, running as fallback`,
    );
    process.exit(0);
  }
}

void main();
