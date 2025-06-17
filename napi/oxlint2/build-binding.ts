import { globSync } from 'glob';
import { spawnSync } from 'node:child_process';
import { rmSync } from 'node:fs';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const args = process.argv.slice(2);
console.log(`args: `, args);

const cmd = spawnSync(
  'pnpm',
  [
    'napi',
    'build',
    '-o=./src',
    '--manifest-path',
    './Cargo.toml',
    '--platform',
    '-p',
    'oxc_linter_napi',
    '--esm',
    '--js',
    'binding.js',
    '--dts',
    'binding.d.ts',
    '--no-const-enum',
    '--no-dts-cache',
    ...args,
  ],
  {
    stdio: 'inherit', // Directly inherit stdio (preserves colors)
    env: { ...process.env, RUSTC_COLOR: 'always' }, // Force color output
    shell: true,
    cwd: __dirname,
  },
);

if (cmd.status !== 0) {
  globSync('src/oxlint-binding.*.node', {
    absolute: true,
    cwd: __dirname,
  }).forEach((file) => {
    rmSync(file, { force: true, recursive: true });
  });

  globSync('./src/oxlint-binding.*.wasm', {
    absolute: true,
    cwd: __dirname,
  }).forEach((file) => {
    rmSync(file, { recursive: true, force: true });
  });

  console.error('Command failed!');
  process.exit(cmd.status);
}
