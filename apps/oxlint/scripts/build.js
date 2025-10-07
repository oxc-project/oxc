import { execSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const oxlintDirPath = join(import.meta.dirname, '..'),
  distDirPath = join(oxlintDirPath, 'dist');

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

// Lazy implementation
/*
// Copy files from `napi/parser` to `apps/oxlint/dist`
console.log('Copying files from parser...');

const parserDirPath = join(oxlintDirPath, '../../napi/parser');

const parserFilePaths = [
  'src-js/raw-transfer/lazy-common.js',
  'src-js/raw-transfer/node-array.js',
  'generated/lazy/constructors.js',
  'generated/lazy/type_ids.js',
  'generated/lazy/walk.js',
  'generated/deserialize/ts_range_loc_parent_no_parens.js',
];

for (const parserFilePath of parserFilePaths) {
  copyFile(join(parserDirPath, parserFilePath), join(distDirPath, parserFilePath));
}
*/

// Copy files from `src-js/generated` to `dist/generated`
console.log('Copying generated files...');

const generatedFilePaths = ['deserialize.js'];
for (const filePath of generatedFilePaths) {
  copyFile(join(oxlintDirPath, 'src-js/generated', filePath), join(distDirPath, 'generated', filePath));
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
