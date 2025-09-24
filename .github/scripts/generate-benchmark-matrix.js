#!/usr/bin/env node

/**
 * Generate a dynamic matrix for benchmark jobs based on affected components.
 * This script determines which benchmark components need to run based on changed files.
 */

const { execSync } = require('child_process');
const process = require('process');
const https = require('https');

// All available benchmark components
const ALL_COMPONENTS = ['lexer', 'parser', 'transformer', 'semantic', 'minifier', 'codegen', 'formatter', 'linter'];

// Files that when changed affect all benchmarks
const GLOBAL_FILES = [
  'Cargo.lock',
  'rust-toolchain.toml',
  '.github/workflows/benchmark.yml',
  '.github/scripts/generate-benchmark-matrix.js',
];

/**
 * Execute a shell command and return the output
 * @param {string} command - Command to execute
 * @returns {string} Command output
 */
function exec(command) {
  try {
    return execSync(command, { encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch (error) {
    console.error(error);
    return '';
  }
}

/**
 * Make a GitHub API request
 * @param {string} path - API path
 * @returns {Promise<any>} API response
 */
function githubApi(path) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path,
      headers: {
        'User-Agent': 'oxc-benchmark-matrix',
        'Accept': 'application/vnd.github.v3+json',
      },
    };

    // Add authorization if token is available
    const token = process.env.GITHUB_TOKEN;
    if (token) {
      options.headers['Authorization'] = `token ${token}`;
    }

    https.get(options, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        if (res.statusCode === 200) {
          resolve(JSON.parse(data));
        } else {
          reject(new Error(`GitHub API error: ${res.statusCode} ${data}`));
        }
      });
    }).on('error', reject);
  });
}

/**
 * Get changed files based on the GitHub event type
 * @returns {Promise<string[]>} Array of changed file paths
 */
async function getChangedFiles() {
  const eventName = process.env.GITHUB_EVENT_NAME;
  const repository = process.env.GITHUB_REPOSITORY;
  const sha = process.env.GITHUB_SHA;
  const prNumber = process.env.GITHUB_PR_NUMBER;
  const ref = process.env.GITHUB_REF;

  console.error(`Event: ${eventName}`);
  console.error(`Repository: ${repository}`);
  console.error(`SHA: ${sha}`);
  console.error(`Ref: ${ref}`);

  if (eventName === 'workflow_dispatch') {
    console.error('Manual trigger - will run all benchmarks');
    return null; // Signal to run all benchmarks
  }

  let files = [];

  try {
    if (eventName === 'pull_request' && prNumber) {
      // For PR, use GitHub API to get changed files
      console.error(`Getting changed files for PR #${prNumber}`);
      const prFiles = await githubApi(`/repos/${repository}/pulls/${prNumber}/files?per_page=100`);
      files = prFiles.map(f => f.filename);
    } else if (sha && repository) {
      // For push to main, get the commit and compare with parent
      console.error(`Getting changed files for commit ${sha}`);
      const commit = await githubApi(`/repos/${repository}/commits/${sha}`);
      files = commit.files ? commit.files.map(f => f.filename) : [];
    } else {
      // No valid parameters for API calls
      console.error('Error: Missing required environment variables for GitHub API');
      console.error('Will run all benchmarks as fallback');
      return null; // Signal to run all benchmarks
    }
  } catch (error) {
    console.error(`Error getting changed files via API: ${error.message}`);
    console.error('Will run all benchmarks as fallback');
    return null; // Signal to run all benchmarks
  }

  console.error(`Changed files (${files.length}):`);
  files.forEach(f => console.error(`  - ${f}`));

  return files;
}

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
    if (GLOBAL_FILES.some(globalFile => file === globalFile || file.endsWith(`/${globalFile}`))) {
      console.error(`Global file changed: ${file}`);
      return true;
    }
  }

  return false;
}

/**
 * Get dependencies for a specific benchmark component
 * @param {string} component - Component name
 * @returns {string[]} Array of dependency names
 */
function getComponentDependencies(component) {
  const command =
    `cargo tree -p oxc_benchmark --features ${component} --no-default-features -f "{lib}" -e normal --no-dedupe --prefix none 2>/dev/null | grep oxc | sort -u`;
  const output = exec(command);

  if (!output) {
    console.error(`Warning: Could not get dependencies for ${component}`);
    return [];
  }

  return output.split('\n').filter(dep => dep && dep !== 'oxc_benchmark');
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
  console.error(`Component ${component} dependencies: ${dependencies.join(', ')}`);

  // Check if any dependency files changed
  for (const dep of dependencies) {
    const depPath = `crates/${dep}/`;
    if (changedFiles.some(file => file.startsWith(depPath))) {
      console.error(`  Component ${component} affected by changes in ${depPath}`);
      return true;
    }
  }

  // Check benchmark and common task files
  if (changedFiles.some(file => file.startsWith('tasks/benchmark/') || file.startsWith('tasks/common/'))) {
    console.error(`  Component ${component} affected by benchmark/common file changes`);
    return true;
  }

  return false;
}

/**
 * Determine which components are affected by changes
 * @returns {Promise<string[]>} Array of affected component names
 */
async function determineAffectedComponents() {
  const changedFiles = await getChangedFiles();

  // Manual trigger - run all benchmarks
  if (changedFiles === null) {
    return ALL_COMPONENTS;
  }

  // Check for global changes
  if (checkGlobalChanges(changedFiles)) {
    console.error('Global changes detected - will run all benchmarks');
    return ALL_COMPONENTS;
  }

  // Check each component individually
  const affectedComponents = [];

  for (const component of ALL_COMPONENTS) {
    console.error(`\nChecking component: ${component}`);
    if (isComponentAffected(component, changedFiles)) {
      affectedComponents.push(component);
    }
  }

  if (affectedComponents.length === 0) {
    console.error('\nNo components were affected by the changes');
  } else {
    console.error(`\nAffected components: ${affectedComponents.join(', ')}`);
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
      console.error('::notice title=No benchmarks to run::No components were affected by the changes');
    } else {
      console.error(`::notice title=Running benchmarks::Affected components: ${affectedComponents.join(', ')}`);
    }

    process.exit(0);
  } catch (error) {
    console.error('Error generating benchmark matrix:', error);
    // On error, run all benchmarks as a fallback
    console.log(JSON.stringify(ALL_COMPONENTS));
    process.exit(0);
  }
}

// Run the script
void main();
