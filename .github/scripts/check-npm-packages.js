#!/usr/bin/env node
/* eslint-disable no-console */

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

/**
 * Check npm packages before publishing
 * Usage: node check-npm-packages.js <package_dir_pattern> [root_package_path]
 * Example: node check-npm-packages.js "npm/oxlint*"
 * Example: node check-npm-packages.js "release-dir/*" "napi/parser"
 */

function checkPackageExists(packageName) {
  try {
    execSync(`npm view ${packageName} version`, { stdio: "pipe" });
    return true;
  } catch (error) {
    console.error(`Failed to check package ${packageName}:`, error.message);
    return false;
  }
}

function checkPackageFiles(packageDir, packageJson) {
  const files = packageJson.files || [];
  let allFilesExist = true;

  for (const file of files) {
    const filePath = path.join(packageDir, file);
    if (!fs.existsSync(filePath)) {
      console.error(`::error::File does not exist: ${filePath}`);
      allFilesExist = false;
    }
  }

  return allFilesExist;
}

function dryRunPublish(packageDir) {
  try {
    const flags = "--provenance --access public --no-git-checks";
    console.log(`Running dry-run publish for ${packageDir}...`);
    execSync(`pnpm publish ${packageDir}/ ${flags} --dry-run`, {
      stdio: "inherit",
    });
    return true;
  } catch (error) {
    console.error(`::error::Dry-run publish failed for ${packageDir}:`, error.message);
    return false;
  }
}

function checkPackage(packageDir, skipExistenceCheck = false) {
  console.log(packageDir);

  if (!fs.existsSync(packageDir)) {
    console.warn(`::warning::Package directory not found: ${packageDir}`);
    return false;
  }

  const packageJsonPath = path.join(packageDir, "package.json");
  if (!fs.existsSync(packageJsonPath)) {
    console.warn(`::warning::package.json not found in: ${packageDir}`);
    return false;
  }

  // List directory contents
  try {
    execSync(`ls ${packageDir}`, { stdio: "inherit" });
  } catch (error) {
    console.error(`Failed to list directory ${packageDir}:`, error.message);
  }

  // Show package.json
  try {
    execSync(`cat ${packageJsonPath}`, { stdio: "inherit" });
  } catch (error) {
    console.error(`Failed to read ${packageJsonPath}:`, error.message);
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
  const packageName = packageJson.name;

  // Check if package exists on npm (unless skipped)
  if (!skipExistenceCheck) {
    console.log(`Checking if package '${packageName}' exists on npm...`);

    if (!checkPackageExists(packageName)) {
      console.error(`::error::Package '${packageName}' does not exist on npm!`);
      console.log("");
      console.log(
        "======================================================================================================",
      );
      console.log("FIRST PUBLISH REQUIRED");
      console.log(
        "======================================================================================================",
      );
      console.log("");
      console.log(`Package: ${packageName}`);
      console.log("This package needs to be published to npm for the first time manually.");
      console.log("");
      console.log("Instructions:");
      console.log(
        `  1. Create a local project with package name '${packageName}' and version '0.0.1'`,
      );
      console.log("  2. Publish it manually with: npm publish --access public");
      console.log("  3. After successful publish, delete the token from:");
      console.log("     - ~/.npmrc (local file)");
      console.log("     - npm settings (https://www.npmjs.com/settings/YOUR_USERNAME/tokens)");
      console.log(
        "     Note: Manual tokens don't expire automatically and should be deleted for security",
      );
      console.log("");
      console.log(
        "After the initial publish, this workflow will handle all subsequent releases automatically.",
      );
      console.log(
        "======================================================================================================",
      );
      console.log("");
      return false;
    }

    console.log(`✓ Package '${packageName}' exists on npm`);
  }

  // Check files exist
  if (!checkPackageFiles(packageDir, packageJson)) {
    return false;
  }

  // Run dry-run publish
  if (!dryRunPublish(packageDir)) {
    return false;
  }

  console.log("");
  return true;
}

function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.error("Usage: node check-npm-packages.js <package_dir_pattern> [root_package_path]");
    process.exit(1);
  }

  const packagePattern = args[0];
  const rootPackagePath = args[1];
  let exitCode = 0;

  // Expand glob pattern using shell
  let packageDirs = [];
  try {
    const result = execSync(`ls -d ${packagePattern}`, { encoding: "utf-8" });
    packageDirs = result.trim().split("\n").filter(Boolean);
  } catch (error) {
    // Pattern might not match anything, which is ok if we have a root package
    console.error(`Pattern expansion failed for ${packagePattern}:`, error.message);
    if (!rootPackagePath) {
      console.error(`::error::No packages found matching pattern: ${packagePattern}`);
      process.exit(1);
    }
  }

  // Check subpackages
  for (const packageDir of packageDirs) {
    if (!checkPackage(packageDir)) {
      exitCode = 1;
    }
  }

  // Check root package if specified
  if (rootPackagePath) {
    console.log("# Checking root package");
    if (!checkPackage(rootPackagePath, true)) {
      exitCode = 1;
    }
  }

  process.exit(exitCode);
}

main();
