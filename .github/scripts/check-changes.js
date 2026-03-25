#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Generic change detection script for CI jobs.
 *
 * Include mode: checks if changes affect specified crates or their transitive dependencies.
 * node check-changes.js --packages oxc_minifier --paths tasks/minsize/
 *
 * Exclude mode: skips only if ALL changed files belong to excluded crate directories.
 * node check-changes.js --exclude oxc_linter,oxc_language_server --paths napi/,npm/
 *
 * Paths-only mode: run if any changed file matches the given paths (no cargo tree).
 * node check-changes.js --paths apps/oxlint/,npm/oxlint/,napi/oxlint/
 */

const { getChangedFiles } = require("./get-changed-files.js");
const { getCrateDependencies, checkFilesAffectCrates } = require("./utils.js");

/**
 * Parse CLI arguments into structured options.
 *
 * @returns {{ packages: string[]; exclude: string[]; paths: string[] }}
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const result = { packages: [], exclude: [], paths: [] };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--packages" && args[i + 1]) {
      result.packages = args[++i].split(",").filter(Boolean);
    } else if (arg === "--exclude" && args[i + 1]) {
      result.exclude = args[++i].split(",").filter(Boolean);
    } else if (arg === "--paths" && args[i + 1]) {
      result.paths = args[++i].split(",").filter(Boolean);
    }
  }

  return result;
}

/**
 * Include mode: run if changes affect any of the specified crates (+ transitive deps) or paths.
 *
 * @param {string[] | null} changedFiles
 * @param {string[]} packages - Root crate names
 * @param {string[]} paths - Additional trigger paths
 * @returns {boolean}
 */
function shouldRunInclude(changedFiles, packages, paths) {
  if (changedFiles === null) {
    console.error("No changed files list (manual trigger or error) - will run");
    return true;
  }

  if (changedFiles.length === 0) {
    console.error("No files changed - will skip");
    return false;
  }

  // Check additional trigger paths first (no cargo tree needed)
  for (const file of changedFiles) {
    for (const p of paths) {
      if (file.startsWith(p)) {
        console.error(`File ${file} matches trigger path ${p} - will run`);
        return true;
      }
    }
  }

  // If no changed files are in crates/, cargo tree check cannot match — skip it
  if (!changedFiles.some((file) => file.startsWith("crates/"))) {
    console.error("No files changed in crates/ - will skip");
    return false;
  }

  // Resolve transitive dependencies via cargo tree
  let allCrates;
  try {
    const deps = getCrateDependencies(packages);
    allCrates = [...packages, ...deps];
    console.error(`Checking crates (${allCrates.length}): ${allCrates.join(", ")}`);
  } catch (error) {
    console.error(`cargo tree failed: ${error.message} - will run as fallback`);
    return true;
  }

  if (allCrates.length === 0) {
    console.error("Warning: No crates resolved - will run as fallback");
    return true;
  }

  return checkFilesAffectCrates(changedFiles, allCrates);
}

/**
 * Exclude mode: skip only if ALL changed files belong to excluded crate directories. Files matching
 * `paths` always trigger the job. No cargo tree — exclusion is intentionally shallow.
 *
 * @param {string[] | null} changedFiles
 * @param {string[]} excludeCrates - Crate names to exclude
 * @param {string[]} paths - Additional paths that always trigger the job
 * @returns {boolean}
 */
function shouldRunExclude(changedFiles, excludeCrates, paths) {
  if (changedFiles === null) {
    console.error("No changed files list (manual trigger or error) - will run");
    return true;
  }

  if (changedFiles.length === 0) {
    console.error("No files changed - will skip");
    return false;
  }

  // Check always-trigger paths first
  for (const file of changedFiles) {
    for (const p of paths) {
      if (file.startsWith(p)) {
        console.error(`File ${file} matches always-trigger path ${p} - will run`);
        return true;
      }
    }
  }

  const excludePaths = excludeCrates.map((c) => `crates/${c}/`);

  // Skip only if EVERY changed file is inside an excluded crate directory
  const allExcluded = changedFiles.every((file) => excludePaths.some((ep) => file.startsWith(ep)));

  if (allExcluded) {
    console.error(
      `All ${changedFiles.length} changed files are in excluded crates (${excludeCrates.join(", ")}) - will skip`,
    );
    return false;
  }

  console.error("Changes found outside excluded crates - will run");
  return true;
}

/**
 * Paths-only mode: run if any changed file matches any of the given paths.
 * Used by jobs that don't need cargo tree dependency checking.
 */
function shouldRunPathsOnly(changedFiles, paths) {
  if (changedFiles === null) {
    console.error("No changed files list (manual trigger or error) - will run");
    return true;
  }

  if (changedFiles.length === 0) {
    console.error("No files changed - will skip");
    return false;
  }

  for (const file of changedFiles) {
    for (const p of paths) {
      if (file.startsWith(p)) {
        console.error(`File ${file} matches trigger path ${p} - will run`);
        return true;
      }
    }
  }

  console.error("No files match trigger paths - will skip");
  return false;
}

async function main() {
  try {
    const opts = parseArgs();

    if (opts.packages.length === 0 && opts.exclude.length === 0 && opts.paths.length === 0) {
      console.error("Error: must specify --packages, --exclude, or --paths");
      console.log("true");
      process.exit(0);
    }

    const changedFiles = await getChangedFiles();

    let shouldRun;
    if (opts.exclude.length > 0) {
      shouldRun = shouldRunExclude(changedFiles, opts.exclude, opts.paths);
    } else if (opts.packages.length > 0) {
      shouldRun = shouldRunInclude(changedFiles, opts.packages, opts.paths);
    } else {
      shouldRun = shouldRunPathsOnly(changedFiles, opts.paths);
    }

    console.log(shouldRun ? "true" : "false");

    if (shouldRun) {
      console.error("::notice title=Change detection::Will run this job");
    } else {
      console.error("::notice title=Change detection::Will skip this job");
    }

    process.exit(0);
  } catch (error) {
    console.error("Error in change detection:", error);
    // On error, run as a fallback
    console.log("true");
    console.error("::warning title=Change detection error::Error occurred, running as fallback");
    process.exit(0);
  }
}

void main();
