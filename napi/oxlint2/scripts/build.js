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
const constantsPath = join(rootDir, 'src-js/generated/constants.cjs');
if (existsSync(constantsPath)) {
  copyFileSync(constantsPath, join(distGeneratedDir, 'constants.cjs'));
  console.log('Copied constants.cjs');
} else {
  console.error(
    'Warning: generated/constants.cjs not found. Make sure to run napi build first.',
  );
}

// Copy parser files
const parserFiles = [
  {
    src: join(rootDir, '../parser/raw-transfer/lazy-common.js'),
    dest: join(parserRawTransferDir, 'lazy-common.cjs'),
  },
  {
    src: join(rootDir, '../parser/raw-transfer/node-array.js'),
    dest: join(parserRawTransferDir, 'node-array.cjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/walk.js'),
    dest: join(parserGeneratedDir, 'walk.cjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/types.js'),
    dest: join(parserGeneratedDir, 'types.cjs'),
  },
  {
    src: join(rootDir, '../parser/generated/lazy/constructors.js'),
    dest: join(parserGeneratedDir, 'constructors.cjs'),
  },
];

for (const { src, dest } of parserFiles) {
  if (existsSync(src)) {
    // replace a any `require(`*.js`)` with `require(`*`.cjs`)`
    const content = String(readFileSync(src));
    const updatedContent = content.replace(
      /require\((['"])(.*?)(\.js)(['"])\)/g,
      (match, p1, p2, p3, p4) => {
        return `require(${p1}${p2}.cjs${p4})`;
      },
    );
    writeFileSync(dest, updatedContent);

    //
    // copyFileSync(src, dest);
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
