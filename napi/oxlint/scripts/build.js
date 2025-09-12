#!/usr/bin/env node

import { execSync } from 'child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, '..');

console.log('Building with tsdown...');
try {
  execSync('pnpm tsdown', { stdio: 'inherit', cwd: rootDir });
} catch (error) {
  console.error('Build failed:', error);
  process.exit(1);
}

// Create directories after tsdown build.
const distDir = join(rootDir, 'dist');
const distGeneratedDir = join(distDir, 'generated');
const parserRawTransferDir = join(distDir, 'parser', 'raw-transfer');
const parserGeneratedDir = join(distDir, 'parser', 'generated', 'lazy');

// Create all necessary directories
if (!existsSync(distGeneratedDir)) {
  mkdirSync(distGeneratedDir, { recursive: true });
}
if (!existsSync(parserRawTransferDir)) {
  mkdirSync(parserRawTransferDir, { recursive: true });
}
if (!existsSync(parserGeneratedDir)) {
  mkdirSync(parserGeneratedDir, { recursive: true });
}

console.log('Copying generated files...');
// Copy generated constants file
const constantsPath = join(rootDir, 'src-js/generated/constants.mjs');
if (existsSync(constantsPath)) {
  copyFileSync(constantsPath, join(distGeneratedDir, 'constants.mjs'));
  console.log('Copied constants.mjs');
} else {
  console.error(
    'Warning: generated/constants.mjs not found. Make sure to run napi build first.',
  );
}

// Copy parser files
const parserFiles = [
  {
    src: join(rootDir, '../parser/raw-transfer/lazy-common.mjs'),
    dest: join(parserRawTransferDir, 'lazy-common.mjs'),
  },
  {
    src: join(rootDir, '../parser/raw-transfer/node-array.mjs'),
    dest: join(parserRawTransferDir, 'node-array.mjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/walk.mjs'),
    dest: join(parserGeneratedDir, 'walk.mjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/types.mjs'),
    dest: join(parserGeneratedDir, 'types.mjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/constructors.mjs'),
    dest: join(parserGeneratedDir, 'constructors.mjs'),
  },
];

for (const { src, dest } of parserFiles) {
  if (existsSync(src)) {
    copyFileSync(src, dest);
    console.log(`Copied ${src.split('/').pop()}`);
  } else {
    console.error(`Warning: parser file not found: ${src}`);
  }
}

// Copy native .node files that might exist in src-js
const nodeFiles = [
  'oxlint.darwin-arm64.node',
  'oxlint.darwin-x64.node',
  'oxlint.linux-x64-gnu.node',
  'oxlint.linux-arm64-gnu.node',
  'oxlint.linux-x64-musl.node',
  'oxlint.linux-arm64-musl.node',
  'oxlint.win32-x64-msvc.node',
  'oxlint.win32-arm64-msvc.node',
];

for (const nodeFile of nodeFiles) {
  const srcPath = join(rootDir, 'src-js', nodeFile);
  if (existsSync(srcPath)) {
    copyFileSync(srcPath, join(distDir, nodeFile));
    console.log(`Copied ${nodeFile}`);
  }
}

console.log('Build complete!');
