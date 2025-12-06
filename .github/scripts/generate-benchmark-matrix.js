#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Generate a dynamic matrix for benchmark jobs based on affected components.
 * This script determines which benchmark components need to run based on changed files.
 */

const process = require("process");
const { getChangedFiles } = require("./get-changed-files.js");
const { getCrateDependencies } = require("./utils.js");

// All available benchmark components
const ALL_COMPONENTS = [
  "lexer",
  "parser",
  "transformer",
  "semantic",
  "minifier",
  "codegen",
  "formatter",
  "linter",
  "language_server",
];

// Files that when changed affect all benchmarks
const GLOBAL_FILES = [
  "Cargo.lock",
  "rust-toolchain.toml",
  ".github/workflows/benchmark.yml",
  ".github/scripts/generate-benchmark-matrix.js",
];

/**
 * Check if any global files have changed that affect all benchmarks
 * @param {string[]} changedFiles - Array of changed file paths
 * @returns {boolean} True if global changes detected
 */
function checkGlobalChanges(changedFiles) {
  if (!changedFiles || changedFiles.length === 0) {
    return false;
  }

  for (const file of changedFiles) {
    if (GLOBAL_FILES.some((globalFile) => file === globalFile || file.endsWith(`/${globalFile}`))) {
      console.error(`Global file changed: ${file}`);
      return true;
    }
  }

  return false;
}

/**
 * Map component to its feature
 * @param {string} component - Component name
 * @returns {string} Feature name
 */
function getFeatureForComponent(component) {
  if (component === "linter") {
    return "linter";
  }
  return "compiler";
}

/**
 * Get dependencies for a specific benchmark component
 * @param {string} component - Component name
 * @returns {string[]} Array of dependency names
 */
function getComponentDependencies(component) {
  const feature = getFeatureForComponent(component);
  const deps = getCrateDependencies("oxc_benchmark", {
    features: feature,
    noDefaultFeatures: true,
  });

  if (deps.length === 0) {
    console.error(`Warning: Could not get dependencies for ${component} (feature: ${feature})`);
  }

  return deps;
}

/**
 * Check if a component is affected by the changed files
 * @param {string} component - Component name
 * @param {string[]} changedFiles - Array of changed file paths
 * @returns {boolean} True if component is affected
 */
function isComponentAffected(component, changedFiles) {
  if (!changedFiles || changedFiles.length === 0) {
    return false;
  }

  // Get component dependencies
  const dependencies = getComponentDependencies(component);
  console.error(`Component ${component} dependencies: ${dependencies.join(", ")}`);

  // Check if any dependency files changed
  for (const dep of dependencies) {
    const depPath = `crates/${dep}/`;
    if (changedFiles.some((file) => file.startsWith(depPath))) {
      console.error(`  Component ${component} affected by changes in ${depPath}`);
      return true;
    }
  }

  // Check benchmark and common task files
  if (
    changedFiles.some(
      (file) => file.startsWith("tasks/benchmark/") || file.startsWith("tasks/common/"),
    )
  ) {
    console.error(`  Component ${component} affected by benchmark/common file changes`);
    return true;
  }

  return false;
}

/**
 * Determine which components are affected by changes
 * @returns {Promise<Array<{component: string, feature: string}>>} Array of affected component objects
 */
async function determineAffectedComponents() {
  const changedFiles = await getChangedFiles();

  // Manual trigger - run all benchmarks
  if (changedFiles === null) {
    return ALL_COMPONENTS.map((component) => ({
      component,
      feature: getFeatureForComponent(component),
    }));
  }

  // Check for global changes
  if (checkGlobalChanges(changedFiles)) {
    console.error("Global changes detected - will run all benchmarks");
    return ALL_COMPONENTS.map((component) => ({
      component,
      feature: getFeatureForComponent(component),
    }));
  }

  // Check each component individually
  const affectedComponents = [];

  for (const component of ALL_COMPONENTS) {
    console.error(`\nChecking component: ${component}`);
    if (isComponentAffected(component, changedFiles)) {
      affectedComponents.push({
        component,
        feature: getFeatureForComponent(component),
      });
    }
  }

  if (affectedComponents.length === 0) {
    console.error("\nNo components were affected by the changes");
  } else {
    console.error(
      `\nAffected components: ${affectedComponents.map((obj) => obj.component).join(", ")}`,
    );
  }

  return affectedComponents;
}

/**
 * Main entry point
 */
async function main() {
  try {
    const affectedComponents = await determineAffectedComponents();

    // Output the matrix as JSON array
    // This will be captured by GitHub Actions
    console.log(JSON.stringify(affectedComponents));

    // Set GitHub Actions notice
    if (affectedComponents.length === 0) {
      console.error(
        "::notice title=No benchmarks to run::No components were affected by the changes",
      );
    } else {
      const componentNames = affectedComponents.map((obj) => obj.component).join(", ");
      console.error(`::notice title=Running benchmarks::Affected components: ${componentNames}`);
    }

    process.exit(0);
  } catch (error) {
    console.error("Error generating benchmark matrix:", error);
    // On error, run all benchmarks as a fallback
    const fallbackMatrix = ALL_COMPONENTS.map((component) => ({
      component,
      feature: getFeatureForComponent(component),
    }));
    console.log(JSON.stringify(fallbackMatrix));
    process.exit(0);
  }
}

// Run the script
void main();
