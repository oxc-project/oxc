import { globSync } from 'glob';
import fs from 'node:fs';
import { defineConfig } from 'rolldown';

import nodePath from 'node:path';

const isBrowserPkg = false;

const pkgRoot = __dirname;
const outputDir = nodePath.resolve(pkgRoot, 'dist');
const isReleasingCI = false;

function copy() {
  // wasm build rely on `.node` binaries. But we don't want to copy `.node` files
  // to the dist folder, so we need to distinguish between `.wasm` and `.node` files.
  const wasmFiles = globSync('./src/oxlint-binding.*.wasm', {
    absolute: true,
  });
  const nodeFiles = globSync('./src/oxlint-binding.*.node', {
    absolute: true,
  });

  //   // Binary build is on the separate step on CI
  //   if (!isCI) {
  //     if (isBrowserPkg && !wasmFiles.length) {
  //       throw new Error("No WASM files found.");
  //     }
  //     if (!isBrowserPkg && !nodeFiles.length) {
  //       throw new Error("No Node files found.");
  //     }
  //   }

  const copyTo = nodePath.resolve(outputDir);
  fs.mkdirSync(copyTo, { recursive: true });

  if (!isReleasingCI) {
    // Released `rolldown` package import binary via `@rolldown/binding-<platform>` packages.
    // There's no need to copy binary files to dist folder.

    if (isBrowserPkg) {
      // Move the wasm file to dist
      wasmFiles.forEach((file) => {
        const fileName = nodePath.basename(file);
        if (isBrowserPkg && fileName.includes('debug')) {
          // NAPI-RS now generates a debug wasm binary no matter how and we don't want to ship it to npm.
          console.log('[build:done]', 'Skipping', file);
        } else {
          console.log('[build:done]', 'Copying', file, `to ${copyTo}`);
          fs.cpSync(file, nodePath.join(copyTo, fileName));
        }
      });

      const browserShims = globSync('./src/*wasi*js', { absolute: true });
      browserShims.forEach((file) => {
        const fileName = nodePath.basename(file);
        console.log('[build:done]', 'Copying', file, `to ${copyTo}`);
        fs.cpSync(file, nodePath.join(copyTo, fileName));
      });
    } else {
      // Move the binary file to dist
      nodeFiles.forEach((file) => {
        const fileName = nodePath.basename(file);
        console.log('[build:done]', 'Copying', file, `to ${copyTo}`);
        fs.cpSync(file, nodePath.join(copyTo, fileName));
      });
    }
  }
}

copy();

export default defineConfig({
  input: {
    cli: './src/index.ts',
  },
  platform: 'node',
  external: [/oxlint-binding\..*\.node/],
});
