/*
 * Test and bench against `es-module-lexer`
 */
import assert from 'assert';
import fs from 'fs';

import * as esModuleLexer from 'es-module-lexer';
import oxc from './index.js';

const root = '/Users/boshen/github/es-module-lexer/test/samples';

main();

async function main() {
  await esModuleLexer.init;

  const files = fs
    .readdirSync(root)
    .filter((x) => x.endsWith('.js'))
    .map((file) => {
      const source = fs.readFileSync(`${root}/${file}`);
      return {
        file,
        code: source.toString(),
        size: source.byteLength,
      };
    });

  function test(source) {
    const [imports, exports, facade, hasModuleSyntax] = esModuleLexer.parse(source);
    const moduleLexer = oxc.moduleLexerSync(source);
    assert(moduleLexer.imports.length == imports.length);
    assert(moduleLexer.exports.length == exports.length);
    assert(moduleLexer.facade == facade);
    assert(moduleLexer.hasModuleSyntax == hasModuleSyntax);
  }

  function testAll(files) {
    for (const file of files) {
      test(file.code);
    }
  }

  testAll(files);

  const n = 25;

  bench('es-module-lexer', esModuleLexer.parse);
  bench('oxc', oxc.moduleLexerSync);

  function bench(name, fn) {
    console.log('-----------');
    console.log(name);

    doRun();

    function timeRun(code) {
      const start = process.hrtime.bigint();
      fn(code);
      const end = process.hrtime.bigint();
      return Math.round(Number(end - start) / 1e6);
    }

    function doRun() {
      console.log('Cold Run, All Samples');
      let totalSize = 0;
      {
        let total = 0;
        files.forEach(({ code, size }) => {
          totalSize += size;
          total += timeRun(code);
        });
        console.log(`test/samples/*.js (${Math.round(totalSize / 1e3)} KiB)`);
        console.log(`> ${total + 'ms'}`);
      }

      console.log(`\nWarm Runs (average of ${n} runs)`);
      files.forEach(({ file, code, size }) => {
        console.log(`${file} (${Math.round(size / 1e3)} KiB)`);

        let total = 0;
        for (let i = 0; i < n; i++) {
          total += timeRun(code);
        }

        console.log(`> ${total / n + 'ms'}`);
      });

      console.log(`\nWarm Runs, All Samples (average of ${n} runs)`);
      {
        let total = 0;
        for (let i = 0; i < n; i++) {
          files.forEach(({ code }) => {
            total += timeRun(code);
          });
        }
        console.log(`test/samples/*.js (${Math.round(totalSize / 1e3)} KiB)`);
        console.log(`> ${total / n + 'ms'}`);
      }
    }
  }
}
