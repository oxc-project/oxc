#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Check if conformance tests should run based on changed files.
 * Uses cargo tree to determine dependencies of oxc_coverage crate.
 */

const { getChangedFiles } = require("./get-changed-files.js");
const { getCrateDependencies } = require("./utils.js");

/**
 * Get all crates that conformance tests depend on
 * @returns {string[]} Array of crate names
 */
function getCoverageDependencies() {
  const packages = ["oxc_coverage", "oxc_transform_conformance", "oxc_prettier_conformance"];
  const deps = getCrateDependencies(packages);

  console.error(`Conformance dependencies (${deps.length}):`);
  console.error(`  ${deps.join(", ")}`);

  return deps;
}

/**
 * Directories that should always trigger conformance tests
 */
const ALWAYS_RUN_PATHS = [
  "tasks/coverage/",
  "tasks/common/",
  "tasks/oxc_transform_conformance/",
  "tasks/oxc_prettier_conformance/",
  "pnpm-lock.yaml",
];

/**
 * Check if conformance tests should run based on changed files
 * @param {string[] | null} changedFiles - Array of changed file paths, or null for "run all"
 * @returns {boolean} True if conformance should run
 */
function shouldRunConformance(changedFiles) {
  // null means manual trigger or error - always run
  if (changedFiles === null) {
    console.error("No changed files list (manual trigger or error) - will run conformance");
    return true;
  }

  // No files changed - skip conformance
  if (changedFiles.length === 0) {
    console.error("No files changed - will skip conformance");
    return false;
  }

  // Check for paths that should always trigger conformance
  for (const file of changedFiles) {
    for (const path of ALWAYS_RUN_PATHS) {
      if (file.startsWith(path)) {
        console.error(`File ${file} matches always-run path ${path} - will run conformance`);
        return true;
      }
    }
  }

  // Get dependencies
  const dependencies = getCoverageDependencies();

  if (dependencies.length === 0) {
    console.error("Warning: No dependencies found - will run conformance as fallback");
    return true;
  }

  // Check if any changed file affects a dependency crate
  for (const dep of dependencies) {
    const cratePath = `crates/${dep}/`;
    for (const file of changedFiles) {
      if (file.startsWith(cratePath)) {
        console.error(`File ${file} affects dependency ${dep} - will run conformance`);
        return true;
      }
    }
  }

  console.error("No files affect oxc_coverage dependencies - will skip conformance");
  return false;
}

/**
 * Main entry point
 */
async function main() {
  try {
    const changedFiles = await getChangedFiles();
    const shouldRun = shouldRunConformance(changedFiles);

    // Output for GitHub Actions
    console.log(shouldRun ? "true" : "false");

    // Set GitHub Actions notice
    if (shouldRun) {
      console.error("::notice title=Conformance tests::Will run conformance tests");
    } else {
      console.error("::notice title=Conformance tests::Will skip conformance tests");
    }

    process.exit(0);
  } catch (error) {
    console.error("Error checking conformance changes:", error);
    // On error, run conformance as a fallback
    console.log("true");
    console.error(
      "::warning title=Conformance check error::Error occurred, running conformance as fallback",
    );
    process.exit(0);
  }
}

// Run the script
void main();
