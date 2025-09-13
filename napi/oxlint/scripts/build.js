import { execSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const oxlintDirPath = join(import.meta.dirname, '..'),
  distDirPath = join(oxlintDirPath, 'dist'),
  parserDirPath = join(oxlintDirPath, '../parser');

// Build with tsdown
console.log('Building with tsdown...');
execSync('pnpm tsdown', { stdio: 'inherit', cwd: oxlintDirPath });

// Copy files from `napi/parser` to `napi/oxlint/dist`
console.log('Copying files from parser...');

const parserFilePaths = [
  'raw-transfer/lazy-common.mjs',
  'raw-transfer/node-array.mjs',
  'generated/lazy/constructors.mjs',
  'generated/lazy/types.mjs',
  'generated/lazy/walk.mjs',
];

for (const parserFilePath of parserFilePaths) {
  copyFile(join(parserDirPath, parserFilePath), join(distDirPath, parserFilePath));
}

// Copy native `.node` files from `src-js`
console.log('Copying `.node` files...');

for (const filename of readdirSync(join(oxlintDirPath, 'src-js'))) {
  if (!filename.endsWith('.node')) continue;
  copyFile(join(oxlintDirPath, 'src-js', filename), join(distDirPath, filename));
}

console.log('Build complete!');

/**
 * Copy a file, creating parent directories if needed.
 * @param {string} srcPath - Source file path, absolute
 * @param {string} destPath - Destination file path, absolute
 * @returns {void}
 */
function copyFile(srcPath, destPath) {
  mkdirSync(join(destPath, '..'), { recursive: true });
  copyFileSync(srcPath, destPath);
  console.log(`- Copied ${srcPath.split('/').pop()}`);
}
