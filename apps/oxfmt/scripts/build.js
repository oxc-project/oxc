// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const oxfmtDirPath = join(import.meta.dirname, ".."),
  distDirPath = join(oxfmtDirPath, "dist");

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: oxfmtDirPath });

// NOTE: `prettier-plugin-tailwindcss` keeps deps as module strings (e.g. "prettier/plugins/html").
// Since we bundle plugins as separate chunks, it won't be able to resolve them at runtime.
// Rewrite them to bundled local chunks so runtime doesn't require external `prettier` package.
// See also: https://github.com/oxc-project/oxc/issues/19293
patchTailwindPluginModuleSpecifiers(distDirPath);

// Copy native `.node` files from `src-js`
console.log("Copying `.node` files...");

for (const filename of readdirSync(join(oxfmtDirPath, "src-js"))) {
  if (!filename.endsWith(".node")) continue;
  copyFile(join(oxfmtDirPath, "src-js", filename), join(distDirPath, filename));
}

console.log("Build complete!");

/**
 * Copy a file, creating parent directories if needed.
 * @param {string} srcPath - Source file path, absolute
 * @param {string} destPath - Destination file path, absolute
 * @returns {void}
 */
function copyFile(srcPath, destPath) {
  mkdirSync(join(destPath, ".."), { recursive: true });
  copyFileSync(srcPath, destPath);
  console.log(`- Copied ${srcPath.split("/").pop()}`);
}

// ---

/**
 * Rewrite bundled Tailwind plugin module strings to local chunk paths.
 *
 * Example:
 * - "prettier/plugins/html" -> `new URL("./html-xxxx.js", import.meta.url).href`
 *
 * @param {string} distDirPath - dist directory path, absolute
 * @returns {void}
 */
function patchTailwindPluginModuleSpecifiers(distDirPath) {
  const pluginSpecifierPrefix = '"prettier/plugins/';
  const pluginNameRe = /"prettier\/plugins\/([a-z-]+)"/g;

  const distFiles = readdirSync(distDirPath);
  const tailwindChunk = findTailwindPluginIndexChunk(distDirPath, distFiles, pluginSpecifierPrefix);

  let { filename, source } = tailwindChunk;

  const pluginNames = [...new Set([...source.matchAll(pluginNameRe)].map(([, name]) => name))];
  for (const pluginName of pluginNames) {
    const pluginChunk = findOneFileByPattern(
      distFiles,
      (filename) => filename.startsWith(`${pluginName}-`) && filename.endsWith(".js"),
      `${pluginName}-*.js`,
    );
    source = source.replaceAll(
      `"prettier/plugins/${pluginName}"`,
      `new URL("./${pluginChunk}", import.meta.url).href`,
    );
  }

  writeFileSync(join(distDirPath, filename), source);
  console.log("🔧", `Patched prettier/plugins/{${pluginNames.join(",")}} in ${filename}`);
}

/**
 * Find Tailwind plugin's bundled index chunk by its runtime module specifiers.
 *
 * @param {string} distDirPath - dist directory path, absolute
 * @param {string[]} files - Filename list
 * @param {string} pluginSpecifierPrefix - Partial module specifier marker
 * @returns {{ filename: string, source: string }}
 */
function findTailwindPluginIndexChunk(distDirPath, files, pluginSpecifierPrefix) {
  const jsFiles = files.filter((filename) => filename.endsWith(".js"));
  const candidates = [];

  for (const filename of jsFiles) {
    const source = readFileSync(join(distDirPath, filename), "utf8");
    if (source.includes(pluginSpecifierPrefix)) candidates.push({ filename, source });
  }

  if (candidates.length !== 1) {
    throw new Error(
      [
        'Expected exactly 1 Tailwind plugin index chunk containing "prettier/plugins/*" specifiers.',
        `Found ${candidates.length}: ${candidates.map((item) => item.filename).join(", ")}`,
      ].join(" "),
    );
  }

  return candidates[0];
}

/**
 * Find exactly one file by predicate.
 *
 * @param {string[]} files - Filename list
 * @param {(filename: string) => boolean} isMatch - Match predicate
 * @param {string} patternLabel - Pattern label for error message
 * @returns {string} Matched filename
 */
function findOneFileByPattern(files, isMatch, patternLabel) {
  const matched = files.filter((filename) => isMatch(filename));
  if (matched.length !== 1) {
    throw new Error(
      `Expected exactly 1 file for pattern ${patternLabel}, got ${matched.length}: ${matched.join(", ")}`,
    );
  }
  return matched[0];
}
