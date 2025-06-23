import { writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join as pathJoin } from 'node:path';
import { bench, describe } from 'vitest';
import bindings from './bindings.js';
import { experimentalGetLazyVisitor, parseAsync, parseSync } from './index.js';

// Use `require` not `import` to load these internal modules, to avoid evaluating the modules
// twice as ESM and CJS
const require = createRequire(import.meta.filename);
const deserializeJS = require('./generated/deserialize/js.js');
const deserializeTS = require('./generated/deserialize/ts.js');
const { isJsAst, prepareRaw, returnBufferToCache } = require('./raw-transfer/common.js');

// Same fixtures as used in Rust parser benchmarks
let fixtureUrls = [
  'https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts',
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx',
  'https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx',
  'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs',
  'https://cdn.jsdelivr.net/npm/antd@5.12.5/dist/antd.js',
];

// For sharding in CI - specify single fixture to run benchmarks on
let benchStandard = bench, benchRaw = bench;
let shard = process.env.SHARD;
if (shard) {
  shard *= 1;
  if (shard % 2 === 0) {
    benchRaw = bench.skip;
  } else {
    benchStandard = bench.skip;
    shard--;
  }
  fixtureUrls = [fixtureUrls[shard / 2]];
}

// Same directory as Rust benchmarks use for downloaded files
// to avoid re-downloading if Rust benchmarks already downloaded
const cacheDirPath = pathJoin(import.meta.dirname, '../../target');

// Load fixtures
const fixtures = await Promise.all(fixtureUrls.map(async (url) => {
  const filename = url.split('/').at(-1),
    path = pathJoin(cacheDirPath, filename);

  let code;
  try {
    code = await readFile(path, 'utf8');
  } catch {
    const res = await fetch(url);
    code = await res.text();
    await writeFile(path, code);
  }

  return { filename, code };
}));

// Run benchmarks
for (const { filename, code } of fixtures) {
  describe(filename, () => {
    benchStandard('parser_napi', () => {
      const ret = parseSync(filename, code);
      // Read returned object's properties to execute getters which deserialize
      const { program, comments, module, errors } = ret;
    });

    benchRaw('parser_napi_raw', () => {
      const ret = parseSync(filename, code, { experimentalRawTransfer: true });
      // Read returned object's properties to execute getters
      const { program, comments, module, errors } = ret;
    });

    benchStandard('parser_napi_async', async () => {
      const ret = await parseAsync(filename, code);
      // Read returned object's properties to execute getters which deserialize
      const { program, comments, module, errors } = ret;
    });

    benchRaw('parser_napi_async_raw', async () => {
      const ret = await parseAsync(filename, code, { experimentalRawTransfer: true });
      // Read returned object's properties to execute getters
      const { program, comments, module, errors } = ret;
    });

    benchRaw('parser_napi_raw_no_deser', () => {
      const { buffer, sourceByteLen } = prepareRaw(code);
      bindings.parseSyncRaw(filename, buffer, sourceByteLen, {});
      returnBufferToCache(buffer);
    });

    // Prepare buffer but don't deserialize
    const { buffer, sourceByteLen } = prepareRaw(code);
    bindings.parseSyncRaw(filename, buffer, sourceByteLen, {});
    const deserialize = isJsAst(buffer) ? deserializeJS : deserializeTS;

    benchRaw('parser_napi_raw_deser_only', () => {
      deserialize(buffer, code, sourceByteLen);
    });

    // Create visitors
    const Visitor = experimentalGetLazyVisitor();

    let debuggerCount = 0;
    const debuggerVisitor = new Visitor({
      DebuggerStatement(debuggerStmt) {
        debuggerCount++;
      },
    });

    let identCount = 0;
    const identVisitor = new Visitor({
      BindingIdentifier(ident) {
        identCount++;
      },
      IdentifierReference(ident) {
        identCount++;
      },
      IdentifierName(ident) {
        identCount++;
      },
    });

    benchRaw('parser_napi_raw_lazy_visit(debugger)', () => {
      const { visit, dispose } = parseSync(filename, code, { experimentalLazy: true });
      debuggerCount = 0;
      visit(debuggerVisitor);
      dispose();
    });

    benchRaw('parser_napi_raw_lazy_visit(ident)', () => {
      const { visit, dispose } = parseSync(filename, code, { experimentalLazy: true });
      identCount = 0;
      visit(identVisitor);
      dispose();
    });

    benchRaw('parser_napi_raw_lazy_visitor(debugger)', () => {
      const { visit, dispose } = parseSync(filename, code, { experimentalLazy: true });
      debuggerCount = 0;
      const debuggerVisitor = new Visitor({
        DebuggerStatement(debuggerStmt) {
          debuggerCount++;
        },
      });
      visit(debuggerVisitor);
      dispose();
    });

    benchRaw('parser_napi_raw_lazy_visitor(ident)', () => {
      const { visit, dispose } = parseSync(filename, code, { experimentalLazy: true });
      identCount = 0;
      const identVisitor = new Visitor({
        BindingIdentifier(ident) {
          identCount++;
        },
        IdentifierReference(ident) {
          identCount++;
        },
        IdentifierName(ident) {
          identCount++;
        },
      });
      visit(identVisitor);
      dispose();
    });
  });
}
