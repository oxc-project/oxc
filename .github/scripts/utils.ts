/**
 * Common utilities for GitHub Actions scripts
 */

import { execSync } from 'node:child_process';

interface GetCrateDependenciesOptions {
  features?: string;
  noDefaultFeatures?: boolean;
}

/**
 * Execute a shell command and return the output
 * @param command - Command to execute
 * @returns Command output
 */
function exec(command: string): string {
  try {
    return execSync(command, { encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch (error) {
    console.error(`Error executing command: ${command}`);
    console.error(error instanceof Error ? error.message : String(error));
    return '';
  }
}

/**
 * Get dependencies for one or more crates using cargo tree
 * @param packages - Package name(s) to query
 * @param options - Additional options
 * @returns Array of dependency crate names (excluding the queried package(s))
 */
function getCrateDependencies(
  packages: string | string[],
  options: GetCrateDependenciesOptions = {}
): string[] {
  const pkgs = Array.isArray(packages) ? packages : [packages];
  const packageArgs = pkgs.map((pkg) => `-p ${pkg}`).join(' ');

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
  return output.split('\n').filter((dep) => dep && !pkgs.includes(dep));
}

/**
 * Check if any changed files affect specified crates or paths
 * @param changedFiles - Array of changed file paths, or null
 * @param crates - Array of crate names to check
 * @param additionalPaths - Additional paths to check (e.g., 'tasks/benchmark/')
 * @returns True if any file affects the specified crates or paths
 */
function checkFilesAffectCrates(
  changedFiles: string[] | null,
  crates: string[],
  additionalPaths: string[] = []
): boolean {
  if (!changedFiles || changedFiles.length === 0) {
    return false;
  }

  // Check if any changed file affects a crate
  for (const crate of crates) {
    const cratePath = `crates/${crate}/`;
    if (changedFiles.some((file) => file.startsWith(cratePath))) {
      console.error(`File affects crate ${crate}`);
      return true;
    }
  }

  // Check additional paths
  for (const path of additionalPaths) {
    if (changedFiles.some((file) => file.startsWith(path))) {
      console.error(`File affects path ${path}`);
      return true;
    }
  }

  return false;
}

export { exec, getCrateDependencies, checkFilesAffectCrates };
