/**
 * Common utilities for GitHub Actions scripts
 */

const { execSync } = require('child_process');

/**
 * Execute a shell command and return the output
 * @param {string} command - Command to execute
 * @returns {string} Command output
 */
function exec(command) {
  try {
    return execSync(command, { encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch (error) {
    console.error(`Error executing command: ${command}`);
    console.error(error.message);
    return '';
  }
}

/**
 * Get dependencies for one or more crates using cargo tree
 * @param {string | string[]} packages - Package name(s) to query
 * @param {object} options - Additional options
 * @param {string} [options.features] - Features to enable
 * @param {boolean} [options.noDefaultFeatures] - Disable default features
 * @returns {string[]} Array of dependency crate names (excluding the queried package(s))
 */
function getCrateDependencies(packages, options = {}) {
  const pkgs = Array.isArray(packages) ? packages : [packages];
  const packageArgs = pkgs.map(pkg => `-p ${pkg}`).join(' ');

  let command = `cargo tree ${packageArgs} -f "{lib}" -e normal --no-dedupe --prefix none`;

  if (options.features) {
    command += ` --features ${options.features}`;
  }

  if (options.noDefaultFeatures) {
    command += ' --no-default-features';
  }

  command += ' 2>/dev/null | grep oxc | sort -u';

  const output = exec(command);

  if (!output) {
    console.error(`Warning: Could not get dependencies for ${pkgs.join(', ')}`);
    return [];
  }

  // Filter out the queried packages themselves
  return output.split('\n').filter(dep => dep && !pkgs.includes(dep));
}

/**
 * Check if any changed files affect specified crates or paths
 * @param {string[] | null} changedFiles - Array of changed file paths, or null
 * @param {string[]} crates - Array of crate names to check
 * @param {string[]} [additionalPaths] - Additional paths to check (e.g., 'tasks/benchmark/')
 * @returns {boolean} True if any file affects the specified crates or paths
 */
function checkFilesAffectCrates(changedFiles, crates, additionalPaths = []) {
  if (!changedFiles || changedFiles.length === 0) {
    return false;
  }

  // Check if any changed file affects a crate
  for (const crate of crates) {
    const cratePath = `crates/${crate}/`;
    if (changedFiles.some(file => file.startsWith(cratePath))) {
      console.error(`File affects crate ${crate}`);
      return true;
    }
  }

  // Check additional paths
  for (const path of additionalPaths) {
    if (changedFiles.some(file => file.startsWith(path))) {
      console.error(`File affects path ${path}`);
      return true;
    }
  }

  return false;
}

module.exports = {
  exec,
  getCrateDependencies,
  checkFilesAffectCrates,
};
