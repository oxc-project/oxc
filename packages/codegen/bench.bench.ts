// Benchmarks. Run with `pnpm run bench`.
//
// Source maps are not measured. `printSync` is given no `sourceMap`, so it selects the builds
// compiled without source map support, which is what most callers use.
//
// Imports from `dist`, which must hold a RELEASE build. `pnpm run build-dev` replaces `dist` with a debug build,
// which keeps `debugAssert` calls live and does not represent shipped performance.
// `pnpm run bench` builds a release build first, so benchmarking that way is always correct.

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import { parseSync } from "oxc-parser";
import { bench, describe } from "vitest";

import { printSync as oxcPrintSync } from "./dist/index.js";

// `TestFiles::minimal()` from `tasks/common/src/test_file.rs`, which the Rust `codegen` benchmark
// uses, plus 2 larger pure-JS files from `TestFiles::minifier()` for throughput on real bundles.
const FIXTURE_URLS = [
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@cd3bc3d431452b640f5dfcabbc22a8d8a388f393/RadixUIAdoptionSection.jsx",
  "https://cdn.jsdelivr.net/npm/react@17.0.2/cjs/react.development.js",
  "https://cdn.jsdelivr.net/gh/excalidraw/excalidraw@f6d85bc80fe328e8f472636eb0d541f7bb891aa0/packages/excalidraw/components/App.tsx",
  "https://cdn.jsdelivr.net/gh/microsoft/TypeScript@v5.3.3/src/compiler/binder.ts",
  "https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@cd3bc3d431452b640f5dfcabbc22a8d8a388f393/kitchen-sink.tsx",
  "https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.js",
  "https://cdn.jsdelivr.net/npm/antd@4.16.1/dist/antd.js",
];

// Same directory the Rust benchmarks download to, so nothing is downloaded twice
const CACHE_DIR_PATH = pathJoin(import.meta.dirname, "../../target");

// `experimentalRawTransfer` is not in the published `ParserOptions` type - it is experimental
// and untyped. Declaring the object here rather than inline at the call site keeps TypeScript's
// excess property check (which only applies to object literals) from rejecting it, without a cast.
//
// `preserveParens: false` because this printer deliberately does not support the redundant
// `ParenthesizedExpression` / `TSParenthesizedType` nodes.
//
// `experimentalRawTransfer` builds the AST with the JS deserializer rather than via JSON,
// which is the pairing this package is meant to be used in - node `type` strings come out
// as source literals, so the printer's type switches compare pointers rather than characters.
const PARSE_OPTIONS = {
  preserveParens: false,
  experimentalRawTransfer: true,
};

// Timings, raised well above Vitest's defaults of 500ms timed and 100ms warmup.
//
// The defaults leave enough run-to-run variance to swamp a 1-2% change,
// which is the size of win a printer optimization typically produces.
// The long warmup matters just as much as the long run:
// V8 has to tier the printer up to optimized code before any of the timed iterations start,
// or the first results measure the interpreter instead.
const BENCH_OPTIONS = { time: 3000, warmupTime: 1000, iterations: 50, warmupIterations: 20 };

/**
 * Load a fixture, downloading it and caching it on disk if it isn't there already.
 *
 * @param url - URL to download the fixture from
 * @returns Fixture's filename and source text
 */
async function loadFixture(url: string): Promise<{ filename: string; code: string }> {
  const filename = url.split("/").at(-1)!,
    path = pathJoin(CACHE_DIR_PATH, filename);

  let code: string;
  try {
    code = await readFile(path, "utf8");
  } catch {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`Failed to download ${url}: ${res.status}`);
    code = await res.text();
    await mkdir(CACHE_DIR_PATH, { recursive: true });
    await writeFile(path, code);
  }

  return { filename, code };
}

// `tiny.js` is one line long, so its result is a comparison of the two printers' fixed overhead
// rather than of their throughput. Fixtures are ordered by size.
const fixtures = [
  { filename: "tiny.js", code: "export default printSync;\n" },
  ...(await Promise.all(FIXTURE_URLS.map(loadFixture))),
];
fixtures.sort((fixture1, fixture2) => fixture1.code.length - fixture2.code.length);

for (const { filename, code } of fixtures) {
  const { program, errors } = parseSync(filename, code, PARSE_OPTIONS);
  if (errors.length > 0) throw new Error(`Failed to parse ${filename}: ${errors[0].message}`);

  // `.tsx`/`.jsx` fixtures need JSX-safe printing of type parameter lists (`<T,>`)
  const options = { jsx: filename.endsWith("x"), ts: /\.tsx?$/.test(filename) };

  // Print once before timing, so a benchmark can never measure the printer bailing out.
  oxcPrintSync(program, options);

  describe(`${filename} (${code.length} bytes)`, () => {
    bench(
      "oxc-codegen",
      () => {
        oxcPrintSync(program, options);
      },
      BENCH_OPTIONS,
    );
  });
}
