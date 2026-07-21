#!/usr/bin/env node
// oxlint-disable no-console -- This is a command-line reporting tool.

import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { availableParallelism, tmpdir } from "node:os";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, extname, join, relative, resolve } from "node:path";
import { isMainThread, parentPort, Worker } from "node:worker_threads";

import babel from "@babel/core";
import reactCompilerPlugin from "babel-plugin-react-compiler";

import { transformSync as oxcTransformSync } from "../index.js";

const { transformSync: babelTransformSync } = babel;
const require = createRequire(import.meta.url);
const babelVersion = require("@babel/core/package.json").version;
const reactCompilerVersion = require("babel-plugin-react-compiler/package.json").version;

const DEFAULT_ROOT = resolve(import.meta.dirname, "../../../../oxc-ecosystem-ci/repos");
const DEFAULT_REPORT = join(tmpdir(), "oxc-react-compiler-comparison.json");
const SOURCE_EXTENSIONS = new Set([".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts"]);
const NON_ISSUE_STATUSES = new Set(["match", "unchanged", "skipped-too-large"]);

function usage() {
  console.log(`Usage: pnpm run compare-react-compiler -- [root-or-file] [options]

Compare the React Compiler in the local oxc-transform binding with
babel-plugin-react-compiler using both compilers' default plugin options.

Options:
  --repo <name>          Only compare a named repository below the root
  --jobs <number>        Worker count (default: available CPUs, capped at 8)
  --limit <number>       Compare at most this many files
  --max-bytes <number>   Skip files larger than this (default: 1048576)
  --max-details <number> Include details for this many issues (default: 100)
  --report <path>        JSON report path (default: ${DEFAULT_REPORT})
  --help                 Show this help

The default root is ${DEFAULT_ROOT}. Declaration files are excluded because they
are not transform inputs. Repository roots are enumerated with git ls-files.`);
}

function parsePositiveInteger(value, flag) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new Error(`${flag} requires a positive integer, received ${value}`);
  }
  return parsed;
}

function parseArgs(argv) {
  const options = {
    root: DEFAULT_ROOT,
    repo: undefined,
    jobs: Math.min(8, Math.max(1, availableParallelism())),
    limit: undefined,
    maxBytes: 1024 * 1024,
    maxDetails: 100,
    report: DEFAULT_REPORT,
  };
  let rootSet = false;

  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    if (argument === "--") continue;
    if (argument === "--help" || argument === "-h") {
      usage();
      process.exit(0);
    }
    if (!argument.startsWith("-")) {
      if (rootSet) throw new Error(`Unexpected positional argument: ${argument}`);
      options.root = resolve(argument);
      rootSet = true;
      continue;
    }

    const value = argv[++index];
    if (value === undefined) throw new Error(`${argument} requires a value`);
    switch (argument) {
      case "--repo":
        options.repo = value;
        break;
      case "--jobs":
        options.jobs = parsePositiveInteger(value, argument);
        break;
      case "--limit":
        options.limit = parsePositiveInteger(value, argument);
        break;
      case "--max-bytes":
        options.maxBytes = parsePositiveInteger(value, argument);
        break;
      case "--max-details":
        options.maxDetails = parsePositiveInteger(value, argument);
        break;
      case "--report":
        options.report = resolve(value);
        break;
      default:
        throw new Error(`Unknown option: ${argument}`);
    }
  }
  return options;
}

function isDeclarationFile(path) {
  return /\.d\.[cm]?ts$/iu.test(path);
}

function isSourceFile(path) {
  return SOURCE_EXTENSIONS.has(extname(path).toLowerCase()) && !isDeclarationFile(path);
}

function isGitRepository(path) {
  return existsSync(join(path, ".git"));
}

function trackedFiles(repository) {
  const output = execFileSync("git", ["-C", repository, "ls-files", "-z"], {
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  });
  return output
    .split("\0")
    .filter(Boolean)
    .filter(isSourceFile)
    .map((path) => resolve(repository, path));
}

function findFiles(root, repositoryName) {
  if (!existsSync(root)) throw new Error(`Input does not exist: ${root}`);
  if (statSync(root).isFile()) {
    if (repositoryName !== undefined) throw new Error("--repo cannot be used with a file input");
    if (!isSourceFile(root)) throw new Error(`Unsupported source extension: ${root}`);
    return [root];
  }

  if (isGitRepository(root)) {
    if (repositoryName !== undefined && repositoryName !== basename(root)) return [];
    return trackedFiles(root).sort();
  }

  const repositories = readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join(root, entry.name))
    .filter(isGitRepository)
    .filter((path) => repositoryName === undefined || basename(path) === repositoryName)
    .sort();
  if (repositories.length === 0) {
    throw new Error(`No git repositories found below ${root}`);
  }
  return repositories.flatMap(trackedFiles).sort();
}

function sourceKind(filename) {
  const extension = extname(filename).toLowerCase();
  const typescript =
    extension === ".ts" || extension === ".tsx" || extension === ".mts" || extension === ".cts";
  const jsx = extension === ".tsx" || !typescript;
  const sourceType =
    extension === ".mjs" || extension === ".mts"
      ? "module"
      : extension === ".cjs" || extension === ".cts"
        ? "commonjs"
        : "unambiguous";
  const lang = typescript ? (jsx ? "tsx" : "ts") : "jsx";
  return { extension, jsx, lang, sourceType, typescript };
}

function babelOptions(filename, reactCompiler, comments) {
  const kind = sourceKind(filename);
  const parserPlugins = ["decorators"];
  if (kind.typescript) {
    parserPlugins.push([
      "typescript",
      {
        disallowAmbiguousJSXLike: kind.extension === ".mts" || kind.extension === ".cts",
        dts: false,
        isTSX: kind.jsx,
      },
    ]);
  }
  if (kind.jsx) parserPlugins.push("jsx");

  return {
    babelrc: false,
    cloneInputAst: false,
    code: true,
    comments,
    configFile: false,
    filename,
    generatorOpts: { comments, compact: false },
    parserOpts: {
      allowAwaitOutsideFunction: true,
      allowReturnOutsideFunction: kind.sourceType === "commonjs",
      plugins: parserPlugins,
      sourceType: kind.sourceType === "commonjs" ? "script" : kind.sourceType,
    },
    plugins: reactCompiler ? [[reactCompilerPlugin, {}]] : [],
    sourceMaps: false,
    sourceType: kind.sourceType === "commonjs" ? "script" : kind.sourceType,
  };
}

function comparisonCleanupPlugin() {
  return {
    visitor: {
      EmptyStatement(path) {
        path.remove();
      },
      Program: {
        exit(path, state) {
          if (!state.opts.renameBindings) return;

          const bindings = [];
          const seen = new Set();
          function collectBindings(scope) {
            for (const binding of Object.values(scope.bindings)) {
              if (!seen.has(binding)) {
                seen.add(binding);
                bindings.push(binding);
              }
            }
          }

          collectBindings(path.scope);
          path.traverse({
            Scopable(scopePath) {
              collectBindings(scopePath.scope);
            },
          });
          bindings.sort(
            (left, right) =>
              left.identifier.start - right.identifier.start ||
              left.identifier.end - right.identifier.end,
          );
          for (const [index, binding] of bindings.entries()) {
            binding.scope.rename(binding.identifier.name, `__comparison_binding_${index}__`);
          }
        },
      },
    },
  };
}

function canonicalizeForComparison(filename, source, renameBindings) {
  const options = babelOptions(filename, false, false);
  options.generatorOpts = { comments: false, compact: true, minified: true };
  options.plugins = [[comparisonCleanupPlugin, { renameBindings }]];
  return babelTransformSync(source, options)?.code;
}

function cosmeticDifferenceKind(filename, left, right) {
  try {
    if (
      canonicalizeForComparison(filename, left, false) ===
      canonicalizeForComparison(filename, right, false)
    ) {
      return "comments-or-empty-statements";
    }
    if (
      canonicalizeForComparison(filename, left, true) ===
      canonicalizeForComparison(filename, right, true)
    ) {
      return "alpha-renaming-only";
    }
  } catch {
    // Keep the original structural classification if canonicalization cannot parse
    // an output. The main comparison already records parser failures separately.
  }
  return undefined;
}

function compilerOutputInfo(code) {
  const runtimeImport = code.match(
    /import\s*\{(?<specifiers>[^}]*)\}\s*from\s*["']react\/compiler-runtime["']/u,
  );
  const cacheSpecifier = runtimeImport?.groups.specifiers.match(
    /(?:^|,)\s*c(?:\s+as\s+(?<local>[$\w]+))?\s*(?:,|$)/u,
  );
  const cacheName = cacheSpecifier?.groups.local ?? (cacheSpecifier ? "c" : undefined);
  const memoCacheSlots = [];
  if (cacheName !== undefined) {
    const escapedName = cacheName.replaceAll(/[$()*+.?[\\\]^{|}]/gu, "\\$&");
    const callPattern = new RegExp(`(?<![$\\w])${escapedName}\\((\\d+)\\)`, "gu");
    for (const match of code.matchAll(callPattern)) memoCacheSlots.push(Number(match[1]));
  }
  return {
    emitsMemoCache: runtimeImport !== null,
    memoCacheSlots,
  };
}

function analyzeOutputMismatch(filename, babelOutput, oxcOutput) {
  const cosmeticKind = cosmeticDifferenceKind(filename, babelOutput, oxcOutput);
  const babel = compilerOutputInfo(babelOutput);
  const oxc = compilerOutputInfo(oxcOutput);
  let differenceKind = cosmeticKind;
  if (differenceKind === undefined) {
    if (
      babel.emitsMemoCache !== oxc.emitsMemoCache ||
      babel.memoCacheSlots.length !== oxc.memoCacheSlots.length
    ) {
      differenceKind = "compile-selection";
    } else if (babel.memoCacheSlots.some((slots, index) => slots !== oxc.memoCacheSlots[index])) {
      differenceKind = "cache-layout";
    } else {
      differenceKind = "structural";
    }
  }
  return {
    babelEmitsMemoCache: babel.emitsMemoCache,
    babelMemoCacheSlots: babel.memoCacheSlots,
    differenceKind,
    oxcEmitsMemoCache: oxc.emitsMemoCache,
    oxcMemoCacheSlots: oxc.memoCacheSlots,
  };
}

function runBabel(filename, source, reactCompiler, comments) {
  const result = babelTransformSync(source, babelOptions(filename, reactCompiler, comments));
  if (result?.code === undefined || result.code === null) {
    throw new Error("Babel returned no code");
  }
  return result.code;
}

function runOxc(filename, source, reactCompiler) {
  const kind = sourceKind(filename);
  const result = oxcTransformSync(filename, source, {
    lang: kind.lang,
    plugins: reactCompiler ? { reactCompiler: true } : undefined,
    sourceType: kind.sourceType,
  });
  const errors = result.errors.filter((error) => error.severity === "Error");
  const errorText = errors.map((error) => error.message).join("\n");
  // With panicThreshold="none", React Compiler reports recoverable per-function
  // bailouts at Error severity while still producing a complete transform. A fatal
  // compile is distinguishable by the absence of generated code.
  if (errors.length > 0 && result.code.length === 0) {
    throw new Error(errorText);
  }
  return {
    code: result.code,
    errorText,
    errors: errors.length,
    warnings: result.errors.filter((error) => error.severity === "Warning").length,
  };
}

function oxcDiagnosticInfo(result) {
  return result.errors === 0
    ? {}
    : { oxcDiagnostic: truncate(result.errorText, 4_000), oxcErrors: result.errors };
}

function errorMessage(error) {
  const message = error instanceof Error ? error.message : String(error);
  return truncate(message, 4_000);
}

function truncate(value, maxLength = 20_000) {
  if (value === undefined || value.length <= maxLength) return value;
  return `${value.slice(0, maxLength)}\n/* truncated: ${value.length - maxLength} characters omitted */`;
}

function compareFile(filename, maxBytes, details = false) {
  let source;
  try {
    const { size } = statSync(filename);
    if (size > maxBytes) return { status: "skipped-too-large", size };
    source = readFileSync(filename, "utf8");
  } catch (error) {
    return { error: errorMessage(error), status: "read-error" };
  }

  let normalizedSource;
  try {
    // Both compilers receive exactly the same Babel-parsed source. Preserve comments
    // because the default React Compiler options honor ESLint and Flow suppression
    // comments. The output classifier ignores comment attachment differences.
    normalizedSource = runBabel(filename, source, false, true);
  } catch (error) {
    return { error: errorMessage(error), status: "babel-parse-error" };
  }

  let oxcBaseline;
  try {
    oxcBaseline = runOxc(filename, normalizedSource, false);
  } catch (error) {
    return { error: errorMessage(error), status: "oxc-parse-error" };
  }

  let oxcCompiled;
  let oxcError;
  try {
    oxcCompiled = runOxc(filename, normalizedSource, true);
  } catch (error) {
    oxcError = errorMessage(error);
  }

  let babelCompiled;
  let babelError;
  try {
    babelCompiled = runBabel(filename, normalizedSource, true, true);
  } catch (error) {
    babelError = errorMessage(error);
  }

  if (oxcError !== undefined || babelError !== undefined) {
    const status =
      oxcError !== undefined && babelError !== undefined
        ? "both-error"
        : oxcError !== undefined
          ? "oxc-error"
          : "babel-error";
    return { babelError, oxcError, status };
  }

  let babelOutput;
  try {
    // Babel leaves TypeScript and JSX syntax in place. Pass its compiler output
    // through Oxc without React Compiler so both sides use the same downstream
    // TypeScript, JSX, and code-generation pipeline. Do this before detecting
    // changes as well, otherwise Babel's partial removal of TypeScript syntax can
    // be misclassified as a compiler-only change.
    babelOutput = runOxc(filename, babelCompiled, false);
  } catch (error) {
    return { error: errorMessage(error), status: "babel-output-error" };
  }

  const oxcChanged = oxcCompiled.code !== oxcBaseline.code;
  const babelChanged = babelOutput.code !== oxcBaseline.code;
  if (!oxcChanged && !babelChanged) {
    return { ...oxcDiagnosticInfo(oxcCompiled), status: "unchanged" };
  }
  if (oxcChanged && !babelChanged) {
    const info = compilerOutputInfo(oxcCompiled.code);
    return {
      ...(details
        ? { normalizedSource: truncate(oxcBaseline.code), oxcOutput: truncate(oxcCompiled.code) }
        : {}),
      differenceKind:
        cosmeticDifferenceKind(filename, oxcBaseline.code, oxcCompiled.code) ??
        (info.emitsMemoCache ? "compile-selection" : "structural"),
      oxcEmitsMemoCache: info.emitsMemoCache,
      ...oxcDiagnosticInfo(oxcCompiled),
      oxcMemoCacheSlots: info.memoCacheSlots,
      status: "oxc-only-change",
    };
  }
  if (!oxcChanged && babelChanged) {
    const info = compilerOutputInfo(babelOutput.code);
    return {
      ...(details
        ? {
            babelOutput: truncate(babelOutput.code),
            normalizedSource: truncate(oxcBaseline.code),
          }
        : {}),
      babelEmitsMemoCache: info.emitsMemoCache,
      babelMemoCacheSlots: info.memoCacheSlots,
      differenceKind:
        cosmeticDifferenceKind(filename, oxcBaseline.code, babelOutput.code) ??
        (info.emitsMemoCache ? "compile-selection" : "structural"),
      ...oxcDiagnosticInfo(oxcCompiled),
      status: "babel-only-change",
    };
  }

  if (babelOutput.code === oxcCompiled.code) {
    return {
      babelWarnings: babelOutput.warnings,
      ...oxcDiagnosticInfo(oxcCompiled),
      oxcWarnings: oxcCompiled.warnings,
      status: "match",
    };
  }
  return {
    ...(details
      ? {
          babelOutput: truncate(babelOutput.code),
          normalizedSource: truncate(normalizedSource),
          oxcOutput: truncate(oxcCompiled.code),
        }
      : {}),
    ...analyzeOutputMismatch(filename, babelOutput.code, oxcCompiled.code),
    babelWarnings: babelOutput.warnings,
    ...oxcDiagnosticInfo(oxcCompiled),
    oxcWarnings: oxcCompiled.warnings,
    status: "output-mismatch",
  };
}

function runWorkers(files, jobs, maxBytes, onResult) {
  return new Promise((resolvePromise, reject) => {
    let completed = 0;
    let nextIndex = 0;
    let stopped = false;
    const workers = [];

    function stop(error) {
      if (stopped) return;
      stopped = true;
      for (const worker of workers) void worker.terminate();
      reject(error);
    }

    function dispatch(worker) {
      if (nextIndex >= files.length) {
        if (completed === files.length && !stopped) {
          stopped = true;
          for (const activeWorker of workers) void activeWorker.terminate();
          resolvePromise();
        }
        return;
      }
      const index = nextIndex++;
      worker.postMessage({ filename: files[index], index, maxBytes });
    }

    const workerCount = Math.min(jobs, files.length);
    for (let index = 0; index < workerCount; index++) {
      const worker = new Worker(import.meta.filename);
      workers.push(worker);
      worker.on("error", stop);
      worker.on("exit", (code) => {
        if (!stopped && code !== 0) stop(new Error(`Comparison worker exited with code ${code}`));
      });
      worker.on("message", (message) => {
        completed++;
        onResult(files[message.index], message.result, completed);
        dispatch(worker);
      });
      dispatch(worker);
    }
  });
}

function increment(record, key) {
  record[key] = (record[key] ?? 0) + 1;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  let files = findFiles(options.root, options.repo);
  if (options.limit !== undefined) files = files.slice(0, options.limit);
  if (files.length === 0) throw new Error("No matching source files found");

  console.log(
    `Comparing ${files.length.toLocaleString()} files with ${options.jobs} workers ` +
      `(Babel ${babelVersion}, React Compiler ${reactCompilerVersion})`,
  );

  const counts = {};
  const repositoryCounts = {};
  const issueSummaries = [];
  const issueFiles = [];
  let oxcDiagnosticFileCount = 0;
  const startedAt = Date.now();
  let lastProgress = startedAt;

  await runWorkers(files, options.jobs, options.maxBytes, (filename, result, completed) => {
    increment(counts, result.status);
    const relativePath = relative(options.root, filename) || basename(filename);
    const repository = /[\\/]/u.test(relativePath)
      ? relativePath.split(/[\\/]/u, 1)[0]
      : basename(options.root);
    repositoryCounts[repository] ??= {};
    increment(repositoryCounts[repository], result.status);
    if (result.oxcErrors !== undefined) oxcDiagnosticFileCount++;
    if (!NON_ISSUE_STATUSES.has(result.status) || result.oxcErrors !== undefined) {
      issueSummaries.push({ file: relativePath, ...result });
      if (issueFiles.length < options.maxDetails) issueFiles.push(filename);
    }

    const now = Date.now();
    if (now - lastProgress >= 10_000 || completed === files.length) {
      const rate = completed / ((now - startedAt) / 1000);
      console.log(
        `${completed.toLocaleString()}/${files.length.toLocaleString()} ` +
          `(${rate.toFixed(1)} files/s, ${Object.entries(counts)
            .map(([status, count]) => `${status}=${count}`)
            .join(", ")})`,
      );
      lastProgress = now;
    }
  });

  const issues = issueSummaries.map((issue, index) => {
    if (index < issueFiles.length) {
      Object.assign(issue, compareFile(issueFiles[index], options.maxBytes, true));
    }
    return issue;
  });
  const issueCount = issues.length;
  const report = {
    comparedAt: new Date().toISOString(),
    durationSeconds: Number(((Date.now() - startedAt) / 1000).toFixed(3)),
    root: options.root,
    versions: {
      babel: babelVersion,
      babelPluginReactCompiler: reactCompilerVersion,
      oxcTransform: require("../package.json").version,
    },
    compilerOptions: {},
    counts,
    detailedIssueCount: issueFiles.length,
    issueCount,
    issueDetailsTruncated: issueCount > issueFiles.length,
    issues,
    oxcDiagnosticFileCount,
    repositoryCounts,
    totalFiles: files.length,
  };
  writeFileSync(options.report, `${JSON.stringify(report, null, 2)}\n`);

  console.log(`Report: ${options.report}`);
  console.log(`Issues: ${issueCount.toLocaleString()}`);
}

if (isMainThread) {
  main().catch((error) => {
    console.error(errorMessage(error));
    process.exitCode = 1;
  });
} else {
  parentPort.on("message", ({ filename, index, maxBytes }) => {
    parentPort.postMessage({ index, result: compareFile(filename, maxBytes) });
  });
}
