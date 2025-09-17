import { execSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const oxlintDirPath = join(import.meta.dirname, '..'),
  distDirPath = join(oxlintDirPath, 'dist'),
  parserDirPath = join(oxlintDirPath, '../../napi/parser');

// Modify `bindings.js` to use correct package names
console.log('Modifying bindings.js...');
const bindingsPath = join(oxlintDirPath, 'src-js/bindings.js');
let bindingsJs = readFileSync(bindingsPath, 'utf8');
bindingsJs = bindingsJs.replace(/require\('@oxlint\/binding-(.+?)'\)/g, (_, name) => {
  name = name.replace(/-msvc(\/|$)/g, '$1');
  return `require('@oxlint/${name}')`;
});
writeFileSync(bindingsPath, bindingsJs);

// Build with tsdown
console.log('Building with tsdown...');
execSync('pnpm tsdown', { stdio: 'inherit', cwd: oxlintDirPath });

// Add `package.json` to `dist` dir.
// `npm/oxlint` package is CommonJS, so we need this file to tell Node.js that `dist` is ESM.
console.log('Adding package.json to dist...');
writeFileSync(
  join(distDirPath, 'package.json'),
  JSON.stringify({ type: 'module' }, null, 2) + '\n',
);
console.log('- Created package.json');

// Copy files from `napi/parser` to `apps/oxlint/dist`
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
