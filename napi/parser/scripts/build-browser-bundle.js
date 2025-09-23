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

  // bundle wasm.js -> browser-bundle.js
  await esbuild.build({
    entryPoints: ['./src-js/wasm.js'],
    outfile: 'browser-bundle.js',
    alias: {
      '@oxc-parser/binding-wasm32-wasi': './src-js/parser.wasi-browser.js',
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

    // add `browser-bundle.js` to `package.json:files`
    const pkgFile = path.join(pkgDir, 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgFile, 'utf8'));
    pkg.files.push('browser-bundle.js');
    fs.writeFileSync(pkgFile, JSON.stringify(pkg, null, 2));

    // copy `browser-bundle.js` to `<npmDir>/wasm32-wasi`
    fs.cpSync('browser-bundle.js', path.join(pkgDir, 'browser-bundle.js'));
  }
}

await main();
