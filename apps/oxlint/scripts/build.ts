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

// Copy native `.node` files from `src-js`
console.log('Copying `.node` files...');

for (const filename of readdirSync(join(oxlintDirPath, 'src-js'))) {
  if (!filename.endsWith('.node')) continue;
  copyFile(join(oxlintDirPath, 'src-js', filename), join(distDirPath, filename));
}

console.log('Build complete!');

/**
 * Copy a file, creating parent directories if needed.
 * @param srcPath - Source file path, absolute
 * @param destPath - Destination file path, absolute
 */
function copyFile(srcPath: string, destPath: string): void {
  mkdirSync(join(destPath, '..'), { recursive: true });
  copyFileSync(srcPath, destPath);
  console.log(`- Copied ${srcPath.split('/').pop()}`);
}
