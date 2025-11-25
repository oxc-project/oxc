import { writeFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { bench, describe } from "vitest";
import { parseRawSync } from "./src-js/bindings.js";
import { parseAsync, parseSync } from "./src-js/index.js";

// Internals
import { DATA_POINTER_POS_32, PROGRAM_OFFSET } from "./generated/constants.js";
import { deserialize as deserializeJS } from "./generated/deserialize/js.js";
import { deserialize as deserializeTS } from "./generated/deserialize/ts.js";
import { walkProgram } from "./generated/lazy/walk.js";
import { isJsAst, prepareRaw, returnBufferToCache } from "./src-js/raw-transfer/common.js";
import { TOKEN } from "./src-js/raw-transfer/lazy-common.js";
import { getVisitorsArr, Visitor } from "./src-js/raw-transfer/visitor.js";

// Same fixtures as used in Rust parser benchmarks
let fixtureUrls = [
  "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/checker.ts",
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/cal.com.tsx",
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/RadixUIAdoptionSection.jsx",
  "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.0.269/build/pdf.mjs",
  "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
];

// For sharding in CI - specify single fixture to run benchmarks on
let benchStandard = bench,
  benchRaw = bench;
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
const cacheDirPath = pathJoin(import.meta.dirname, "../../target");

// Load fixtures
const fixtures = await Promise.all(
  fixtureUrls.map(async (url) => {
    const filename = url.split("/").at(-1),
      path = pathJoin(cacheDirPath, filename);

    let code;
    try {
      code = await readFile(path, "utf8");
    } catch {
      const res = await fetch(url);
      code = await res.text();
      await writeFile(path, code);
    }

    return { filename, code };
  }),
);

// Run benchmarks
for (const { filename, code } of fixtures) {
  // oxlint-disable-next-line jest/valid-title
  describe(filename, () => {
    benchStandard("parser_napi", () => {
      const ret = parseSync(filename, code);
      // Read returned object's properties to execute getters which deserialize
      // oxlint-disable-next-line no-unused-vars
      const { program, comments, module, errors } = ret;
    });

    benchRaw("parser_napi_raw", () => {
      const ret = parseSync(filename, code, { experimentalRawTransfer: true });
      // Read returned object's properties to execute getters
      // oxlint-disable-next-line no-unused-vars
      const { program, comments, module, errors } = ret;
    });

    benchStandard("parser_napi_async", async () => {
      const ret = await parseAsync(filename, code);
      // Read returned object's properties to execute getters which deserialize
      // oxlint-disable-next-line no-unused-vars
      const { program, comments, module, errors } = ret;
    });

    benchRaw("parser_napi_async_raw", async () => {
      const ret = await parseAsync(filename, code, { experimentalRawTransfer: true });
      // Read returned object's properties to execute getters
      // oxlint-disable-next-line no-unused-vars
      const { program, comments, module, errors } = ret;
    });

    benchRaw("parser_napi_raw_no_deser", () => {
      const { buffer, sourceByteLen } = prepareRaw(code);
      parseRawSync(filename, buffer, sourceByteLen, {});
      returnBufferToCache(buffer);
    });

    // Prepare buffer but don't deserialize
    const { buffer, sourceByteLen } = prepareRaw(code);
    parseRawSync(filename, buffer, sourceByteLen, {});
    const deserialize = isJsAst(buffer) ? deserializeJS : deserializeTS;

    benchRaw("parser_napi_raw_deser_only", () => {
      deserialize(buffer, code, sourceByteLen, true);
    });

    // oxlint-disable-next-line no-unused-vars
    let debuggerCount = 0;
    const debuggerVisitor = new Visitor({
      DebuggerStatement(_debuggerStmt) {
        debuggerCount++;
      },
    });

    // oxlint-disable-next-line no-unused-vars
    let identCount = 0;
    const identVisitor = new Visitor({
      BindingIdentifier(_ident) {
        identCount++;
      },
      IdentifierReference(_ident) {
        identCount++;
      },
      IdentifierName(_ident) {
        identCount++;
      },
    });

    // These 4 currently not working, due to 2 instances of `Visitor` getting loaded via CJS and ESM.
    // TODO: Fix it.
    /*
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
        DebuggerStatement(_debuggerStmt) {
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
        BindingIdentifier(_ident) {
          identCount++;
        },
        IdentifierReference(_ident) {
          identCount++;
        },
        IdentifierName(_ident) {
          identCount++;
        },
      });
      visit(identVisitor);
      dispose();
    });
    */

    const debuggerVisitorsArr = getVisitorsArr(debuggerVisitor);
    const identVisitorsArr = getVisitorsArr(identVisitor);

    const ast = {
      buffer,
      sourceText: code,
      sourceByteLen,
      sourceIsAscii: code.length === sourceByteLen,
      nodes: null, // Initialized in bench functions
      token: TOKEN,
    };

    const programPos = buffer.uint32[DATA_POINTER_POS_32] + PROGRAM_OFFSET;

    benchRaw("parser_napi_raw_lazy_visit_only(debugger)", () => {
      ast.nodes = new Map();
      debuggerCount = 0;
      walkProgram(programPos, ast, debuggerVisitorsArr);
    });

    benchRaw("parser_napi_raw_lazy_visit_only(ident)", () => {
      ast.nodes = new Map();
      identCount = 0;
      walkProgram(programPos, ast, identVisitorsArr);
    });
  });
}
