// Global constants defined at build time by TSDown.

// `true` if build emits source maps
declare const SOURCEMAPS: boolean;

// `true` if build supports TypeScript syntax.
// In builds where it's `false`, `tsdown_plugins/strip_ts.ts` also removes the switch arms
// fenced with `/* IF TS */` / `/* END_IF */` comments.
declare const TS: boolean;

// `true` if is debug build.
// Built with `pnpm run build-dev`, which sets `DEBUG=true`. Debug builds keep `debugAssert` calls
// (see `src/asserts.ts`). Release builds strip them in `tsdown_plugins/remove_asserts.ts`.
declare const DEBUG: boolean;

// `true` if is benchmark build.
// Built with `pnpm run bench`, which sets `BENCHMARKS=true`. Benchmark builds honor the
// `skipSourcemapGeneration` option, so a benchmark can measure the print pass without the final
// source map generation. In normal builds the option does nothing and the check folds away.
declare const BENCHMARKS: boolean;
