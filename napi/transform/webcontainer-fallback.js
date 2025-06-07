const fs = require('node:fs');
const childProcess = require('node:child_process');

const pkg = JSON.parse(
  fs.readFileSync(require.resolve('oxc-transform/package.json'), 'utf-8'),
);
const version = pkg.version;
const baseDir = `/tmp/oxc-transform-${version}`;
const bindingEntry = `${baseDir}/node_modules/@oxc-transform/binding-wasm32-wasi/transform.wasi.cjs`;

if (!fs.existsSync(bindingEntry)) {
  fs.rmSync(baseDir, { recursive: true, force: true });
  fs.mkdirSync(baseDir, { recursive: true });
  const bindingPkg = `@oxc-transform/binding-wasm32-wasi@${version}`;
  // eslint-disable-next-line: no-console
  console.log(`[oxc-transform] Downloading ${bindingPkg} on WebContainer...`);
  childProcess.execFileSync('pnpm', ['i', bindingPkg], {
    cwd: baseDir,
    stdio: 'inherit',
  });
}

module.exports = require(bindingEntry);
