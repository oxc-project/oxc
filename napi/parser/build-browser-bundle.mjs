import * as esbuild from 'esbuild';
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';

async function main() {
  const args = parseArgs({
    strict: true,
    options: {
      npmDir: {
        type: 'string',
      },
    },
  });

  // bundle wasm.mjs -> browser-bundle.mjs
  await esbuild.build({
    entryPoints: ['./wasm.mjs'],
    outfile: 'browser-bundle.mjs',
    alias: {
      '@oxc-parser/binding-wasm32-wasi': './parser.wasi-browser.js',
    },
    bundle: true,
    platform: 'browser',
    format: 'esm',
    logLevel: 'info',
    plugins: [
      {
        name: 'patch-new-url',
        setup(build) {
          build.onResolve({ filter: /\?url$/ }, async (args) => {
            const path = args.path.replace(/\?url$/, '');
            return {
              namespace: 'new-url-asset',
              path,
            };
          });
          build.onLoad({ namespace: 'new-url-asset', filter: /.*/ }, async (args) => {
            return {
              contents: `export default new URL(${JSON.stringify(args.path)}, import.meta.url).href`,
            };
          });
        },
      },
    ],
  });

  if (args.values.npmDir) {
    const pkgDir = path.resolve(args.values.npmDir, 'wasm32-wasi');

    // add `browser-bundle.mjs` to `package.json:files`
    const pkgFile = path.join(pkgDir, 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgFile, 'utf8'));
    pkg.files.push('browser-bundle.mjs');
    fs.writeFileSync(pkgFile, JSON.stringify(pkg, null, 2));

    // copy `browser-bundle.mjs` to `npm-dir/wasm32-wasi`
    fs.cpSync('browser-bundle.mjs', path.join(pkgDir, 'browser-bundle.mjs'));
  }
}

main();
