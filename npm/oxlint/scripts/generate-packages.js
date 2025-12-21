// Post-build script for oxlint npm package.
// Handles tasks that NAPI tooling doesn't cover:
// 1. Patching peerDependencies for oxlint-tsgolint
// 2. Copying dist files from apps/oxlint/dist
// 3. Copying language server binaries into NAPI platform packages

// oxlint-disable no-console

import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** `<REPO ROOT>/npm/oxlint` */
const OXLINT_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
/** `<REPO ROOT>` */
const REPO_ROOT = resolve(OXLINT_ROOT, "../..");
/** `<REPO ROOT>/npm/oxlint/package.json` */
const MANIFEST_PATH = resolve(OXLINT_ROOT, "package.json");
/** `<REPO ROOT>/apps/oxlint/dist` */
const OXLINT_DIST_SRC = resolve(REPO_ROOT, "apps/oxlint/dist");
/** `<REPO ROOT>/npm/oxlint/dist` */
const OXLINT_DIST_DEST = resolve(OXLINT_ROOT, "dist");

const OXLS_BIN_NAME = "oxc_language_server";

// Mapping from Rust target triple to NAPI platform directory name
// Based on @napi-rs/cli's platform naming convention
const RUST_TRIPLE_TO_NAPI_DIR = {
  // Windows
  'x86_64-pc-windows-msvc': 'win32-x64-msvc',
  'aarch64-pc-windows-msvc': 'win32-arm64-msvc',
  'i686-pc-windows-msvc': 'win32-ia32-msvc',
  
  // macOS
  'x86_64-apple-darwin': 'darwin-x64',
  'aarch64-apple-darwin': 'darwin-arm64',
  
  // Linux GNU
  'x86_64-unknown-linux-gnu': 'linux-x64-gnu',
  'aarch64-unknown-linux-gnu': 'linux-arm64-gnu',
  'armv7-unknown-linux-gnueabihf': 'linux-arm-gnueabihf',
  'powerpc64le-unknown-linux-gnu': 'linux-ppc64-gnu',
  's390x-unknown-linux-gnu': 'linux-s390x-gnu',
  'riscv64gc-unknown-linux-gnu': 'linux-riscv64-gnu',
  
  // Linux musl
  'x86_64-unknown-linux-musl': 'linux-x64-musl',
  'aarch64-unknown-linux-musl': 'linux-arm64-musl',
  'armv7-unknown-linux-musleabihf': 'linux-arm-musleabihf',
  'riscv64gc-unknown-linux-musl': 'linux-riscv64-musl',
  
  // Android
  'aarch64-linux-android': 'android-arm64',
  'armv7-linux-androideabi': 'android-arm-eabi',
  
  // OpenHarmony
  'aarch64-unknown-linux-ohos': 'openharmony-arm64',
  
  // FreeBSD
  'x86_64-unknown-freebsd': 'freebsd-x64',
};

// Platforms that skip Rust binary build (no language server available)
const SKIP_LANGUAGE_SERVER_PLATFORMS = new Set([
  'riscv64gc-unknown-linux-musl',
  'aarch64-linux-android',
  'aarch64-unknown-linux-ohos',
]);

/**
 * Patch peerDependencies for oxlint-tsgolint.
 * optionalDependencies are handled by NAPI's pre-publish command.
 */
function patchManifest() {
  const manifestData = JSON.parse(fs.readFileSync(MANIFEST_PATH).toString("utf-8"));

  // Do not automatically install 'oxlint-tsgolint'.
  // https://docs.npmjs.com/cli/v11/configuring-npm/package-json#peerdependenciesmeta
  manifestData.peerDependencies = {
    "oxlint-tsgolint": ">=0.10.0",
  };
  manifestData.peerDependenciesMeta = {
    "oxlint-tsgolint": {
      optional: true,
    },
  };

  console.log(`Patching manifest ${MANIFEST_PATH}`);
  fs.writeFileSync(MANIFEST_PATH, JSON.stringify(manifestData, null, 2));
}

/**
 * Copy `dist` directory from `apps/oxlint/dist` to `npm/oxlint/dist`.
 * `apps/oxlint/scripts/build.js` must be run before this script.
 */
function copyDistFiles() {
  console.log(`Copying dist files from ${OXLINT_DIST_SRC} to ${OXLINT_DIST_DEST}`);
  fs.cpSync(OXLINT_DIST_SRC, OXLINT_DIST_DEST, { recursive: true });
}

/**
 * Update platform package.json to include language server binary in files array.
 * @param {string} packageDir - Path to the platform package directory
 * @param {boolean} isWindows - Whether this is a Windows platform
 */
function updatePlatformPackageJson(packageDir, isWindows) {
  const pkgPath = resolve(packageDir, 'package.json');
  
  if (!fs.existsSync(pkgPath)) {
    console.warn(`  Warning: package.json not found at ${pkgPath}`);
    return;
  }

  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
  const binaryExt = isWindows ? '.exe' : '';
  const binaryName = `${OXLS_BIN_NAME}${binaryExt}`;

  // Add to files array
  pkg.files = pkg.files || [];
  if (!pkg.files.includes(binaryName)) {
    pkg.files.push(binaryName);
  }

  // Add to publishConfig.executableFiles
  pkg.publishConfig = pkg.publishConfig || {};
  pkg.publishConfig.executableFiles = pkg.publishConfig.executableFiles || [];
  if (!pkg.publishConfig.executableFiles.includes(binaryName)) {
    pkg.publishConfig.executableFiles.push(binaryName);
  }

  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2));
  console.log(`  Updated package.json`);
}

/**
 * Copy language server binaries from artifacts into NAPI platform packages.
 * Expects binaries to be extracted from archives and available in the artifacts directory.
 * 
 * @param {string} artifactsDir - Directory containing extracted language server binaries
 * @param {string} npmDir - Directory containing NAPI platform packages (e.g., npm-oxlint/)
 */
function copyLanguageServerBinaries(artifactsDir, npmDir) {
  console.log(`\nCopying language server binaries...`);
  console.log(`  Artifacts dir: ${artifactsDir}`);
  console.log(`  NPM dir: ${npmDir}`);

  if (!fs.existsSync(artifactsDir)) {
    console.warn(`  Warning: Artifacts directory not found: ${artifactsDir}`);
    console.warn(`  Skipping language server binary copy.`);
    return;
  }

  if (!fs.existsSync(npmDir)) {
    console.warn(`  Warning: NPM directory not found: ${npmDir}`);
    console.warn(`  Skipping language server binary copy.`);
    return;
  }

  let copiedCount = 0;
  let skippedCount = 0;

  for (const [rustTriple, napiDir] of Object.entries(RUST_TRIPLE_TO_NAPI_DIR)) {
    // Skip platforms that don't have language server binaries
    if (SKIP_LANGUAGE_SERVER_PLATFORMS.has(rustTriple)) {
      console.log(`  Skipping ${rustTriple} (no Rust binary built)`);
      skippedCount++;
      continue;
    }

    const isWindows = rustTriple.includes('windows');
    const binaryExt = isWindows ? '.exe' : '';
    const binaryName = `${OXLS_BIN_NAME}-${rustTriple}${binaryExt}`;
    const binarySource = resolve(artifactsDir, binaryName);
    
    const packageDir = resolve(npmDir, napiDir);
    const binaryTarget = resolve(packageDir, `${OXLS_BIN_NAME}${binaryExt}`);

    // Check if source binary exists
    if (!fs.existsSync(binarySource)) {
      console.warn(`  Warning: Binary not found: ${binaryName}`);
      skippedCount++;
      continue;
    }

    // Check if package directory exists
    if (!fs.existsSync(packageDir)) {
      console.warn(`  Warning: Package directory not found: ${napiDir}`);
      skippedCount++;
      continue;
    }

    // Copy the binary to package root
    console.log(`  Copying ${rustTriple} -> ${napiDir}/`);
    fs.copyFileSync(binarySource, binaryTarget);
    fs.chmodSync(binaryTarget, 0o755);

    // Update package.json to include binary in files array
    updatePlatformPackageJson(packageDir, isWindows);

    copiedCount++;
  }

  console.log(`\nLanguage server binaries: ${copiedCount} copied, ${skippedCount} skipped`);
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    artifactsDir: null,
    npmDir: null,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--artifacts-dir' && i + 1 < args.length) {
      options.artifactsDir = resolve(args[i + 1]);
      i++;
    } else if (args[i] === '--npm-dir' && i + 1 < args.length) {
      options.npmDir = resolve(args[i + 1]);
      i++;
    }
  }

  return options;
}

// Main execution
const options = parseArgs();

patchManifest();
copyDistFiles();

// Copy language server binaries if directories are provided
if (options.artifactsDir && options.npmDir) {
  copyLanguageServerBinaries(options.artifactsDir, options.npmDir);
} else {
  console.log(`\nSkipping language server binary copy (no --artifacts-dir or --npm-dir provided)`);
}
