import assert from "node:assert/strict";
import { createWriteStream } from "node:fs";
import { mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import { availableParallelism } from "node:os";
import { dirname, extname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isMainThread, parentPort, Worker, workerData } from "node:worker_threads";

import { transformSync as babelTransformSync } from "@babel/core";
import transformReactJsx from "@babel/plugin-transform-react-jsx";
import transformTypescript from "@babel/plugin-transform-typescript";
import reactCompiler from "babel-plugin-react-compiler";
import { transformSync as oxcCodegenSync } from "oxc-transform";
import { transformSync as oxcReactCompilerSync } from "oxc-transform-react";

import { categorizeDifference } from "./categorize.mjs";

const SOURCE_EXTENSIONS = new Set([".jsx", ".tsx"]);
const DEFAULT_ESLINT_SUPPRESSION_RULES_V1 = [
  "react-hooks/exhaustive-deps",
  "react-hooks/rules-of-hooks",
];
const REACT_COMPILER_ENVIRONMENT_OPTIONS_V1 = {
  customMacros: null,
  enableResetCacheOnSourceFileChanges: null,
  enablePreserveExistingMemoizationGuarantees: true,
  validatePreserveExistingMemoizationGuarantees: true,
  validateExhaustiveMemoizationDependencies: false,
  validateExhaustiveEffectDependencies: "off",
  enableOptionalDependencies: true,
  enableNameAnonymousFunctions: false,
  validateHooksUsage: true,
  validateRefAccessDuringRender: true,
  validateNoSetStateInRender: true,
  enableUseKeyedState: false,
  validateNoSetStateInEffects: false,
  validateNoDerivedComputationsInEffects: false,
  validateNoDerivedComputationsInEffectsExperimental: false,
  validateNoJsxInTryStatements: false,
  validateStaticComponents: false,
  validateNoCapitalizedCalls: null,
  validateBlocklistedImports: null,
  validateSourceLocations: false,
  validateNoImpureFunctionsInRender: false,
  validateNoFreezingKnownMutableFunctions: false,
  enableAssumeHooksFollowRulesOfReact: true,
  enableTransitivelyFreezeFunctionExpressions: true,
  enableFunctionOutlining: true,
  enableJsxOutlining: false,
  assertValidMutableRanges: false,
  enableCustomTypeDefinitionForReanimated: false,
  enableTreatRefLikeIdentifiersAsRefs: true,
  enableTreatSetIdentifiersAsStateSetters: false,
  validateNoVoidUseMemo: false,
  enableAllowSetStateFromRefsInEffects: true,
  enableVerboseNoSetStateInEffect: false,
  enableForest: false,
};
const REACT_COMPILER_OPTIONS_V1 = {
  compilationMode: "infer",
  panicThreshold: "none",
  target: "19",
  gating: null,
  dynamicGating: null,
  noEmit: false,
  outputMode: null,
  eslintSuppressionRules: DEFAULT_ESLINT_SUPPRESSION_RULES_V1,
  flowSuppressions: true,
  ignoreUseNoForget: false,
  customOptOutDirectives: null,
};
const BABEL_TYPESCRIPT_OPTIONS = {
  allExtensions: true,
  isTSX: true,
  // Match Oxc's default TypeScript emit for uninitialized class fields.
  allowDeclareFields: true,
};

// A worker is replaced once it reaches this many files, so Babel's per-run
// allocations cannot accumulate across a large scan.
const FILES_PER_WORKER_LIFETIME = 500;
// A file that outruns this budget is reported as a failure and its worker is
// replaced, so one pathological input cannot stall the whole scan.
const FILE_TIMEOUT_MS = 120_000;
// Babel's JSX transform writes `/*#__PURE__*/` where Oxc's writes
// `/* @__PURE__ */`. Both are equivalent, and neither comes from React
// Compiler, so files differing only in this spelling get their own status and
// the rest are diffed with the spelling canonicalized.
const PURE_ANNOTATION = /\/\*\s*[#@]__PURE__\s*\*\//g;
// Diff hunks kept per file, and the longest hunk body kept per side.
const DIFF_MAX_HUNKS = 10;
const DIFF_MAX_HUNK_LINES = 20;
// Files needing more edits than this are reported as a single whole-file hunk
// instead, which keeps the diff of a wildly diverging output cheap.
const DIFF_MAX_EDITS = 400;

if (isMainThread) {
  await main();
} else {
  runWorker(workerData);
}

async function main() {
  const { directory, filenames, jobs, reportPath, dumpDirectory } = await parseArguments(
    process.argv.slice(2),
  );
  const report = reportPath === null ? null : createWriteStream(reportPath, { flags: "w" });
  const counts = { same: 0, "pure-annotation": 0, different: 0, failed: 0 };

  await runPool({ directory, filenames, jobs, dumpDirectory }, (result) => {
    counts[result.status]++;
    if (result.status === "same") {
      return;
    }
    console.log(result.path);
    if (result.status === "failed") {
      console.error(`${result.path}: ${result.message}`);
    }
    report?.write(`${JSON.stringify(result)}\n`);
  });

  // A trailing summary record keeps the report self-describing, so a reader
  // knows what share of the scan the listed mismatches represent.
  report?.write(`${JSON.stringify({ status: "summary", compared: filenames.length, counts })}\n`);
  if (report !== null) {
    await new Promise((done) => report.end(done));
  }

  const differenceCount = counts["pure-annotation"] + counts.different + counts.failed;
  console.error(
    `Compared ${filenames.length} files: ${differenceCount} different ` +
      `(${counts["pure-annotation"]} only in __PURE__ annotation spelling), ` +
      `${counts.failed} failed to transform.`,
  );

  if (differenceCount > 0) {
    process.exitCode = 1;
  }
}

async function parseArguments(args) {
  let targetArgument = null;
  let jobs = Math.max(1, availableParallelism() - 2);
  let reportPath = null;
  let dumpDirectory = null;

  for (const arg of args) {
    const [name, value] = splitOption(arg);
    switch (name) {
      case "--jobs":
        jobs = Number.parseInt(value, 10);
        assert(Number.isInteger(jobs) && jobs > 0, `Invalid --jobs value: ${value}`);
        break;
      case "--report":
        assert(value !== "", "--report requires a file path");
        reportPath = resolve(invocationDirectory(), value);
        break;
      case "--dump":
        assert(value !== "", "--dump requires a directory path");
        dumpDirectory = resolve(invocationDirectory(), value);
        break;
      default:
        assert(!name.startsWith("--"), `Unknown option: ${name}`);
        assert(targetArgument === null, "Pass exactly one file or directory");
        targetArgument = arg;
    }
  }

  assert(
    targetArgument !== null,
    "Usage: pnpm --filter react_compiler compare <file|directory> " +
      "[--jobs=<count>] [--report=<file>] [--dump=<directory>]",
  );
  const target = resolve(invocationDirectory(), targetArgument);
  const isDirectory = (await stat(target)).isDirectory();
  return {
    // Reported paths and the React Compiler `sources` option are both anchored
    // at the scanned directory, which for a single file is the one holding it.
    directory: isDirectory ? target : dirname(target),
    filenames: isDirectory ? await findSourceFiles(target) : [target],
    jobs,
    reportPath,
    dumpDirectory,
  };
}

function splitOption(arg) {
  const separator = arg.indexOf("=");
  return separator === -1 ? [arg, ""] : [arg.slice(0, separator), arg.slice(separator + 1)];
}

function invocationDirectory() {
  return process.env.INIT_CWD ?? process.cwd();
}

async function findSourceFiles(root) {
  const entries = await readdir(root, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await findSourceFiles(path)));
    } else if (entry.isFile() && SOURCE_EXTENSIONS.has(extname(entry.name).toLowerCase())) {
      files.push(path);
    }
  }

  return files.sort();
}

/**
 * Compares every file across a pool of workers. Each worker is replaced when it
 * reaches its file budget, times out, or dies. `onResult` sees the files in
 * input order, so results can be streamed out without buffering the whole scan.
 */
async function runPool({ directory, filenames, jobs, dumpDirectory }, onResult) {
  const finished = new Map();
  let nextIndex = 0;
  let flushIndex = 0;

  function complete(index, result) {
    finished.set(index, result);
    while (finished.has(flushIndex)) {
      onResult(finished.get(flushIndex));
      finished.delete(flushIndex);
      flushIndex++;
      reportProgress(flushIndex, filenames.length);
    }
  }

  await Promise.all(
    Array.from({ length: Math.min(jobs, filenames.length) }, async () => {
      let worker = null;
      let filesOnWorker = 0;

      try {
        while (nextIndex < filenames.length) {
          const index = nextIndex++;
          if (worker === null || filesOnWorker >= FILES_PER_WORKER_LIFETIME) {
            await worker?.terminate();
            worker = startWorker({ directory, dumpDirectory });
            filesOnWorker = 0;
          }

          const { result, workerIsUsable } = await compareOnWorker(worker, filenames[index]);
          filesOnWorker++;
          if (!workerIsUsable) {
            await worker.terminate();
            worker = null;
          }
          complete(index, { path: relative(directory, filenames[index]), ...result });
        }
      } finally {
        await worker?.terminate();
      }
    }),
  );
}

function startWorker(options) {
  const worker = new Worker(fileURLToPath(import.meta.url), { workerData: options });
  // Failures are surfaced per file by `compareOnWorker`. Without a permanent
  // listener an error raised between files would take down the whole scan.
  worker.on("error", () => {});
  return worker;
}

/**
 * Sends one file to a worker. A timed-out or crashed worker is reported as a
 * failure for that file and flagged for replacement.
 */
function compareOnWorker(worker, filename) {
  return new Promise((done) => {
    const timeout = setTimeout(
      () => finish({ status: "failed", message: `Timed out after ${FILE_TIMEOUT_MS}ms` }, false),
      FILE_TIMEOUT_MS,
    );

    function finish(result, workerIsUsable) {
      clearTimeout(timeout);
      worker.off("message", onMessage);
      worker.off("error", onError);
      worker.off("exit", onExit);
      done({ result, workerIsUsable });
    }

    function onMessage(result) {
      finish(result, true);
    }
    function onError(error) {
      finish({ status: "failed", message: `Worker error: ${error?.message ?? error}` }, false);
    }
    function onExit(code) {
      finish({ status: "failed", message: `Worker exited with code ${code}` }, false);
    }

    worker.on("message", onMessage);
    worker.on("error", onError);
    worker.on("exit", onExit);
    worker.postMessage(filename);
  });
}

function reportProgress(completedCount, total) {
  if (completedCount % 1000 === 0 || completedCount === total) {
    console.error(`  ${completedCount}/${total} compared`);
  }
}

function runWorker({ directory, dumpDirectory }) {
  const { babelOptions, oxcOptions } = buildOptions(directory);

  parentPort.on("message", async (filename) => {
    let result;
    try {
      const sourceText = await readFile(filename, "utf8");
      const babelOutput = transformWithBabel(filename, sourceText, babelOptions);
      const oxcOutput = transformWithOxc(filename, sourceText, oxcOptions);
      result = compareOutputs(babelOutput, oxcOutput);
      if (dumpDirectory !== null && result.status !== "same") {
        await dumpOutputs(dumpDirectory, relative(directory, filename), babelOutput, oxcOutput);
      }
    } catch (error) {
      result = {
        status: "failed",
        message: error instanceof Error ? error.message : String(error),
      };
    }
    parentPort.postMessage(result);
  });
}

/** Writes both pipelines' output for one file, for diffing by hand. */
async function dumpOutputs(dumpDirectory, displayPath, babelOutput, oxcOutput) {
  const target = resolve(dumpDirectory, displayPath);
  await mkdir(dirname(target), { recursive: true });
  await Promise.all([
    writeFile(`${target}.babel.js`, babelOutput),
    writeFile(`${target}.oxc.js`, oxcOutput),
  ]);
}

function buildOptions(directory) {
  const {
    validateNoDerivedComputationsInEffectsExperimental,
    validateNoJsxInTryStatements,
    ...sharedEnvironmentOptions
  } = REACT_COMPILER_ENVIRONMENT_OPTIONS_V1;

  return {
    babelOptions: {
      ...REACT_COMPILER_OPTIONS_V1,
      logger: null,
      enableReanimatedCheck: true,
      sources: [directory],
      environment: {
        ...sharedEnvironmentOptions,
        // These two Babel spellings differ from the Oxc NAPI property names.
        validateNoDerivedComputationsInEffects_exp:
          validateNoDerivedComputationsInEffectsExperimental,
        validateNoJSXInTryStatements: validateNoJsxInTryStatements,
        // Oxc has the same neutral defaults internally, but does not expose these
        // callback/test-only options across the NAPI boundary.
        customHooks: new Map(),
        moduleTypeProvider: null,
        flowTypeProvider: null,
        enableEmitHookGuards: null,
        enableEmitInstrumentForget: null,
        throwUnknownException__testonly: false,
      },
    },
    oxcOptions: {
      // napi-rs represents optional object/list fields by omission and does not
      // accept the explicit `null` values used by Babel's resolved configuration.
      ...omitNullOptionsForNapi(REACT_COMPILER_OPTIONS_V1),
      sources: [directory],
      environment: {
        ...omitNullOptionsForNapi(sharedEnvironmentOptions),
        validateNoDerivedComputationsInEffectsExp:
          validateNoDerivedComputationsInEffectsExperimental,
        validateNoJsxInTryStatements,
      },
    },
  };
}

function transformWithBabel(filename, sourceText, babelOptions) {
  const plugins = [[reactCompiler, babelOptions]];
  if (extname(filename).toLowerCase() === ".tsx") {
    plugins.push([transformTypescript, BABEL_TYPESCRIPT_OPTIONS]);
  }
  plugins.push([transformReactJsx, { runtime: "automatic" }]);

  const result = babelTransformSync(sourceText, {
    babelrc: false,
    comments: true,
    configFile: false,
    filename,
    plugins,
    sourceMaps: false,
    sourceType: "unambiguous",
  });

  assert(result?.code, "Babel did not produce code");

  const normalizedResult = oxcCodegenSync(filename, result.code, {
    lang: "js",
    sourceType: "unambiguous",
  });
  assertNoErrors(normalizedResult, "Oxc failed to codegen Babel output");
  assert(normalizedResult.code, "Oxc did not codegen Babel output");
  return normalizedResult.code;
}

function transformWithOxc(filename, sourceText, oxcOptions) {
  const result = oxcReactCompilerSync(filename, sourceText, { reactCompiler: oxcOptions });
  assertNoErrors(result, "oxc-transform-react failed");
  assert(result.code, "oxc-transform-react did not produce code");
  return result.code;
}

function compareOutputs(babelOutput, oxcOutput) {
  if (babelOutput === oxcOutput) {
    return { status: "same" };
  }

  const babelNormalized = babelOutput.replace(PURE_ANNOTATION, "/*@__PURE__*/");
  const oxcNormalized = oxcOutput.replace(PURE_ANNOTATION, "/*@__PURE__*/");
  if (babelNormalized === oxcNormalized) {
    return { status: "pure-annotation" };
  }
  return {
    status: "different",
    ...categorizeDifference(babelNormalized, oxcNormalized),
    ...computeDiff(babelNormalized, oxcNormalized),
  };
}

/**
 * Reports every place the two outputs diverge, rather than one span covering
 * all of them. A file typically mixes unrelated differences, so a single span
 * would let the first one hide all the rest.
 */
function computeDiff(babelOutput, oxcOutput) {
  const babelLines = babelOutput.split("\n");
  const oxcLines = oxcOutput.split("\n");

  let prefix = 0;
  while (
    prefix < babelLines.length &&
    prefix < oxcLines.length &&
    babelLines[prefix] === oxcLines[prefix]
  ) {
    prefix++;
  }
  let babelEnd = babelLines.length;
  let oxcEnd = oxcLines.length;
  while (
    babelEnd > prefix &&
    oxcEnd > prefix &&
    babelLines[babelEnd - 1] === oxcLines[oxcEnd - 1]
  ) {
    babelEnd--;
    oxcEnd--;
  }

  const babelBody = babelLines.slice(prefix, babelEnd);
  const oxcBody = oxcLines.slice(prefix, oxcEnd);
  const hunks = diffHunks(babelBody, oxcBody, prefix) ?? [
    { line: prefix + 1, babel: babelBody, oxc: oxcBody },
  ];

  return {
    hunkCount: hunks.length,
    hunks: hunks.slice(0, DIFF_MAX_HUNKS).map((hunk) => ({
      line: hunk.line,
      babel: truncateHunkSide(hunk.babel),
      oxc: truncateHunkSide(hunk.oxc),
    })),
  };
}

/**
 * Myers diff, capped at `DIFF_MAX_EDITS` edits. Returns `null` when the two
 * sides are too far apart to be worth aligning line by line.
 */
function diffHunks(babelLines, oxcLines, lineOffset) {
  const maxEdits = Math.min(DIFF_MAX_EDITS, babelLines.length + oxcLines.length);
  const origin = maxEdits + 1;
  const furthest = new Int32Array(2 * maxEdits + 3);
  const trace = [];

  for (let edits = 0; edits <= maxEdits; edits++) {
    trace.push(furthest.slice());
    for (let diagonal = -edits; diagonal <= edits; diagonal += 2) {
      let babelIndex = takesDeletion(furthest, origin, diagonal, edits)
        ? furthest[origin + diagonal - 1] + 1
        : furthest[origin + diagonal + 1];
      let oxcIndex = babelIndex - diagonal;
      while (
        babelIndex < babelLines.length &&
        oxcIndex < oxcLines.length &&
        babelLines[babelIndex] === oxcLines[oxcIndex]
      ) {
        babelIndex++;
        oxcIndex++;
      }
      furthest[origin + diagonal] = babelIndex;
      if (babelIndex >= babelLines.length && oxcIndex >= oxcLines.length) {
        return collectHunks(trace, babelLines, oxcLines, edits, origin, lineOffset);
      }
    }
  }

  return null;
}

function takesDeletion(furthest, origin, diagonal, edits) {
  if (diagonal === -edits) {
    return false;
  }
  return diagonal === edits || furthest[origin + diagonal - 1] >= furthest[origin + diagonal + 1];
}

/** Walks the Myers trace backwards, grouping adjacent edits into hunks. */
function collectHunks(trace, babelLines, oxcLines, edits, origin, lineOffset) {
  const hunks = [];
  let babelIndex = babelLines.length;
  let oxcIndex = oxcLines.length;
  let hunk = null;

  function openHunk() {
    if (hunk === null) {
      hunk = { babel: [], oxc: [] };
      hunks.push(hunk);
    }
    return hunk;
  }

  function closeHunk(babelStart) {
    if (hunk !== null) {
      hunk.line = lineOffset + babelStart + 1;
      hunk.babel.reverse();
      hunk.oxc.reverse();
      hunk = null;
    }
  }

  for (let edit = edits; edit > 0; edit--) {
    const furthest = trace[edit];
    const diagonal = babelIndex - oxcIndex;
    const previousDiagonal = takesDeletion(furthest, origin, diagonal, edit)
      ? diagonal - 1
      : diagonal + 1;
    const previousBabelIndex = furthest[origin + previousDiagonal];
    const previousOxcIndex = previousBabelIndex - previousDiagonal;

    if (babelIndex > previousBabelIndex && oxcIndex > previousOxcIndex) {
      // Matching lines separate this edit from the ones already collected.
      closeHunk(babelIndex);
      do {
        babelIndex--;
        oxcIndex--;
      } while (babelIndex > previousBabelIndex && oxcIndex > previousOxcIndex);
    }
    if (babelIndex > previousBabelIndex) {
      openHunk().babel.push(babelLines[--babelIndex]);
    } else if (oxcIndex > previousOxcIndex) {
      openHunk().oxc.push(oxcLines[--oxcIndex]);
    }
  }
  closeHunk(babelIndex);

  return hunks.reverse();
}

function truncateHunkSide(lines) {
  return lines.length <= DIFF_MAX_HUNK_LINES
    ? lines
    : [
        ...lines.slice(0, DIFF_MAX_HUNK_LINES),
        `… ${lines.length - DIFF_MAX_HUNK_LINES} more lines`,
      ];
}

function assertNoErrors(result, message) {
  const errors = result.errors.filter(({ severity }) => severity === "Error");
  assert.equal(errors.length, 0, `${message}\n${errors.map(({ message }) => message).join("\n")}`);
}

function omitNullOptionsForNapi(options) {
  return Object.fromEntries(Object.entries(options).filter(([, value]) => value !== null));
}
