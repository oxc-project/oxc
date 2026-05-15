#!/usr/bin/env node
// Upstream parity runner: compiles a list of fixtures using
// babel-plugin-react-compiler@19.2.6 (from the React submodule) and emits
// per-fixture diagnostic categories as JSON.
//
// Inputs: a list of absolute fixture paths supplied as CLI args or, if no args
// are provided, read line-by-line from stdin.
// Output: a single JSON document on stdout:
//   {
//     "fixtures": [
//       { "path": "...", "diagnostics": [{ "category": "..." }], "kind": "ok|skip|error", "error": "..." }
//     ]
//   }
//
// The runner uses the upstream test harness's pragma parser
// (`parseConfigPragmaForTests`) so each fixture's `@`-pragmas seed
// `CompilerOptions` exactly the way upstream snap tests do. A logger is
// attached that captures every `CompileError` event; `panicThreshold` is left
// at the default ('none') so that the logger sees ALL errors emitted by
// `processFn`, not just the first one that escapes a single function.

'use strict';

const path = require('path');
const fs = require('fs');

const COMPILER_ROOT = path.resolve(
  __dirname,
  '..',
  '..',
  '..',
  '..',
  'tasks',
  'react_compiler',
  'react',
  'compiler',
);
const PLUGIN_DIST = path.join(
  COMPILER_ROOT,
  'packages/babel-plugin-react-compiler/dist/index.js',
);
const BABEL_CORE = path.join(COMPILER_ROOT, 'node_modules/@babel/core');

function loadDeps() {
  if (!fs.existsSync(PLUGIN_DIST)) {
    throw new Error(
      `Upstream plugin dist not found: ${PLUGIN_DIST}. Run \`yarn install && yarn workspace babel-plugin-react-compiler run build\` in tasks/react_compiler/react/compiler.`,
    );
  }
  if (!fs.existsSync(BABEL_CORE)) {
    throw new Error(`@babel/core not found at ${BABEL_CORE}. Run yarn install.`);
  }
  // eslint-disable-next-line global-require
  const plugin = require(PLUGIN_DIST);
  // eslint-disable-next-line global-require
  const babel = require(BABEL_CORE);
  return { plugin, babel };
}

function readFixtureList() {
  const argv = process.argv.slice(2);
  if (argv.length > 0) {
    return argv;
  }
  // Read newline-separated paths from stdin
  const raw = fs.readFileSync(0, 'utf8');
  return raw
    .split('\n')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

function isFlow(src) {
  // Match upstream snap's heuristic: `@flow` anywhere triggers flow parsing
  // (which we don't support here — skip).
  return src.indexOf('@flow') !== -1;
}

function processFixture(absPath, plugin, babel) {
  let src;
  try {
    src = fs.readFileSync(absPath, 'utf8');
  } catch (e) {
    return { path: absPath, kind: 'error', error: 'read_failed: ' + e.message, diagnostics: [] };
  }
  if (isFlow(src)) {
    return { path: absPath, kind: 'skip', reason: 'flow', diagnostics: [] };
  }
  const firstLine = src.substring(0, src.indexOf('\n') === -1 ? src.length : src.indexOf('\n'));
  let baseOpts;
  try {
    baseOpts = plugin.parseConfigPragmaForTests(firstLine, { compilationMode: 'all' });
  } catch (e) {
    return {
      path: absPath,
      kind: 'error',
      error: 'pragma_error: ' + (e && e.message ? e.message : String(e)),
      diagnostics: [],
    };
  }

  const events = [];
  const opts = {
    ...baseOpts,
    environment: { ...baseOpts.environment },
    logger: {
      logEvent: (_filename, event) => {
        events.push(event);
      },
    },
    // Leave panicThreshold at the parsed default ('none' per upstream
    // defaultOptions). Critical errors still throw, but the logger captures
    // all CompileError events regardless.
  };

  const ext = path.extname(absPath).toLowerCase();
  const parserPlugins = ['jsx'];
  if (ext === '.ts' || ext === '.tsx') {
    parserPlugins.push('typescript');
  }

  let thrown = null;
  try {
    babel.transformSync(src, {
      plugins: [[plugin.default, opts]],
      parserOpts: { plugins: parserPlugins, sourceType: 'module' },
      filename: path.basename(absPath),
      babelrc: false,
      configFile: false,
      // Discard output; we only care about diagnostics
      ast: false,
      code: false,
    });
  } catch (e) {
    thrown = e;
  }

  // Prefer logger events when available -- they capture every diagnostic
  // emitted across all functions in the file, not just the one that escapes
  // the panic handler.
  const diagnostics = [];
  const seen = new Set();
  for (const ev of events) {
    if (ev && ev.kind === 'CompileError' && ev.detail) {
      const d = ev.detail;
      const cat = (d.options && d.options.category) || d.category;
      if (typeof cat === 'string') {
        // dedupe identical (category, fnLoc) pairs to avoid double-counting
        // when both logger + throw fire for the same detail.
        const key = `${cat}:${(ev.fnLoc && JSON.stringify(ev.fnLoc.start || null)) || ''}:${diagnostics.length}`;
        if (!seen.has(key)) {
          seen.add(key);
          diagnostics.push({ category: String(cat) });
        }
      }
    }
  }

  // If nothing was logged but something threw, fall back to thrown details
  // (e.g. config errors that throw before any logging happens).
  if (diagnostics.length === 0 && thrown) {
    if (thrown && thrown.name === 'ReactCompilerError' && Array.isArray(thrown.details)) {
      for (const d of thrown.details) {
        const cat = (d.options && d.options.category) || d.category;
        if (typeof cat === 'string') {
          diagnostics.push({ category: String(cat) });
        }
      }
    } else if (thrown) {
      diagnostics.push({ category: 'PipelineError' });
    }
  }

  return { path: absPath, kind: 'ok', diagnostics };
}

function main() {
  const fixtures = readFixtureList();
  const { plugin, babel } = loadDeps();
  const results = [];
  for (const f of fixtures) {
    results.push(processFixture(f, plugin, babel));
  }
  process.stdout.write(JSON.stringify({ fixtures: results }, null, 0));
}

main();
