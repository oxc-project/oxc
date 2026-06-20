// Compare `babel-plugin-react-compiler` (npm) against oxc's native React
// Compiler exposed via `oxc-transform`'s `reactCompilerSync`, over real-world
// files (e.g. the clones in oxc-ecosystem-ci/repos).
//
// Methodology: run both compilers on each file, then normalize BOTH outputs
// through @babel/parser + @babel/generator (stripping formatting metadata) and
// string-compare, so only *semantic* differences in the memoization count.
//
// Setup:
//   cd tasks/react_compiler_compare && npm install
//   (build the napi binding first: cd napi/transform && pnpm build)
//
// Usage:
//   REPOS=/path/to/oxc-ecosystem-ci/repos node compare.mjs [--cap=N] [--all] \
//     [--repos=kibana,next] [--report=path]
//
// Env:
//   REPOS          dir to scan for *.jsx/*.tsx (default: ../../../oxc-ecosystem-ci/repos)
//   OXC_TRANSFORM  path to the built oxc-transform entry (default: ../../napi/transform/index.js)

import { readFileSync, writeFileSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import * as babel from '@babel/core';
import { parse } from '@babel/parser';
import _generate from '@babel/generator';

const generate = _generate.default || _generate;
const ReactCompilerMod = await import('babel-plugin-react-compiler');
const ReactCompiler = ReactCompilerMod.default || ReactCompilerMod;
const bprcVersion = (() => {
  try { return JSON.parse(readFileSync(new URL('./node_modules/babel-plugin-react-compiler/package.json', import.meta.url))).version; }
  catch { return 'unknown'; }
})();

const here = fileURLToPath(new URL('.', import.meta.url));
const oxcTransformPath = process.env.OXC_TRANSFORM || new URL('../../napi/transform/index.js', import.meta.url).href;
const { reactCompilerSync } = await import(oxcTransformPath);

const REPOS_DIR = process.env.REPOS || fileURLToPath(new URL('../../../oxc-ecosystem-ci/repos', import.meta.url));

const args = Object.fromEntries(
  process.argv.slice(2).map((a) => {
    const m = a.match(/^--([^=]+)(?:=(.*))?$/);
    return m ? [m[1], m[2] ?? true] : [a, true];
  }),
);
const CAP = args.all ? Infinity : Number(args.cap ?? 5000);
const REPORT = args.report ?? `${here}report.md`;
const reposFilter = args.repos ? new Set(String(args.repos).split(',')) : null;

function parserPlugins(file) {
  const isTs = file.endsWith('.tsx') || file.endsWith('.ts');
  const isJsx = file.endsWith('.tsx') || file.endsWith('.jsx') || file.endsWith('.js');
  const isFlow = file.endsWith('.flow.js') || file.endsWith('.flow');
  const p = [];
  if (isTs) p.push('typescript');
  if (isFlow) p.push('flow');
  if (isJsx) p.push('jsx');
  return p;
}

function babelCompile(src, filename, plugins) {
  const res = babel.transformSync(src, {
    filename,
    babelrc: false,
    configFile: false,
    browserslistConfigFile: false,
    sourceType: 'module',
    parserOpts: { plugins, sourceType: 'module', allowReturnOutsideFunction: true },
    plugins: [[ReactCompiler, { target: '19' }]],
    cloneInputAst: false,
  });
  return res.code;
}

// Strip per-node formatting metadata so @babel/generator emits a canonical form
// (default quotes, no original raw tokens, no comments/locations). Without this,
// generator preserves each parse's `extra.raw`, so 'x' vs "x" would falsely diff.
function stripFormatting(node) {
  if (!node || typeof node !== 'object') return;
  if (Array.isArray(node)) { for (const c of node) stripFormatting(c); return; }
  delete node.extra;
  delete node.start; delete node.end; delete node.loc; delete node.range;
  delete node.leadingComments; delete node.trailingComments; delete node.innerComments;
  for (const k in node) { const v = node[k]; if (v && typeof v === 'object') stripFormatting(v); }
}

function canon(code, plugins) {
  const ast = parse(code, { sourceType: 'module', plugins, errorRecovery: false });
  stripFormatting(ast);
  return generate(ast, { compact: false, concise: false, retainLines: false, comments: false }).code;
}

const memoized = (code) => code.includes('react/compiler-runtime') || /\b_c\(\d/.test(code);
const cacheCount = (code) => { const m = code.match(/_c\((\d+)\)/); return m ? +m[1] : null; };

// Bucket a mismatch by its dominant cause (heuristic, on canonicalized outputs).
function classify(cb, co, bMemo, oMemo) {
  if (bMemo !== oMemo) return 'memo-decision';
  const bc = cacheCount(cb), oc = cacheCount(co);
  if (bc !== null && oc !== null && bc !== oc) return 'cache-count';
  const bt = (cb.match(/_temp\d*/g) || []).length, ot = (co.match(/_temp\d*/g) || []).length;
  if (bt !== ot) return 'outlining';
  const norm = (s) => s.split('\n').map((l) => l.trim()).filter(Boolean).sort().join('\n');
  if (norm(cb) === norm(co)) return 'line-order';
  return 'other';
}

function shortDiff(cb, co) {
  const ba = cb.split('\n'), oa = co.split('\n');
  const out = [];
  const max = Math.max(ba.length, oa.length);
  for (let i = 0; i < max && out.length < 16; i++) {
    if (ba[i] !== oa[i]) {
      if (ba[i] !== undefined) out.push(`- ${ba[i].trim().slice(0, 120)}`);
      if (oa[i] !== undefined) out.push(`+ ${oa[i].trim().slice(0, 120)}`);
    }
  }
  return out.join('\n');
}

// ---- collect candidate files ----
const findCmd =
  `find ${REPOS_DIR} -type f \\( -name '*.jsx' -o -name '*.tsx' \\) ` +
  `-not -path '*/node_modules/*' -not -path '*/dist/*' -not -path '*/build/*' ` +
  `-not -path '*/.next/*' -not -name '*.min.js'`;
let files = execSync(findCmd, { maxBuffer: 1 << 30 }).toString().trim().split('\n').filter(Boolean);
if (reposFilter) files = files.filter((f) => reposFilter.has(f.slice(REPOS_DIR.length + 1).split('/')[0]));
files.sort();
const total = files.length;
if (files.length > CAP) {
  const stride = files.length / CAP;
  const sampled = [];
  for (let i = 0; i < CAP; i++) sampled.push(files[Math.floor(i * stride)]);
  files = sampled;
}
console.log(`candidates=${total} sampled=${files.length} cap=${CAP === Infinity ? 'all' : CAP} bprc=${bprcVersion}`);

const stats = { processed: 0, babelError: 0, oxcError: 0, bothNoop: 0, memoMatch: 0, memoMismatch: 0, noopMismatch: 0, canonError: 0 };
const buckets = {};
const bucketSamples = {};
const byRepo = {};

let n = 0;
for (const file of files) {
  n++;
  if (n % 500 === 0) console.log(`  ${n}/${files.length}  memoMatch=${stats.memoMatch} memoMismatch=${stats.memoMismatch} babelErr=${stats.babelError}`);
  const repo = file.slice(REPOS_DIR.length + 1).split('/')[0];
  let src;
  try { src = readFileSync(file, 'utf8'); } catch { continue; }
  if (src.length > 400_000) continue;
  const plugins = parserPlugins(file);

  let babelCode;
  try { babelCode = babelCompile(src, file, plugins); } catch { stats.babelError++; continue; }

  let oxc;
  try { oxc = reactCompilerSync(file, src); } catch { stats.oxcError++; continue; }

  stats.processed++;
  const bMemo = memoized(babelCode);
  const oMemo = oxc.changed || memoized(oxc.code);

  let cb, co;
  try { cb = canon(babelCode, plugins); co = canon(oxc.code, plugins); } catch { stats.canonError++; continue; }

  const equal = cb === co;
  byRepo[repo] ??= { match: 0, mismatch: 0 };
  if (equal) {
    byRepo[repo].match++;
    if (bMemo) stats.memoMatch++; else stats.bothNoop++;
  } else {
    byRepo[repo].mismatch++;
    if (bMemo || oMemo) {
      stats.memoMismatch++;
      const cause = classify(cb, co, bMemo, oMemo);
      buckets[cause] = (buckets[cause] ?? 0) + 1;
      (bucketSamples[cause] ??= []);
      if (bucketSamples[cause].length < 4) bucketSamples[cause].push({ file: file.slice(REPOS_DIR.length + 1), diff: shortDiff(cb, co) });
    } else {
      stats.noopMismatch++;
    }
  }
}

const interesting = stats.memoMatch + stats.memoMismatch;
const pct = interesting ? ((stats.memoMatch / interesting) * 100).toFixed(2) : 'n/a';
const lines = [];
lines.push(`# react-compiler: oxc \`reactCompilerSync\` vs babel-plugin-react-compiler@${bprcVersion}`);
lines.push('');
lines.push(`candidates(.jsx/.tsx)=${total}  sampled=${files.length}`);
lines.push('');
lines.push('## Results');
lines.push(`- processed (both compiled): **${stats.processed}**`);
lines.push(`- babel parse/transform errors (out of scope): ${stats.babelError}`);
lines.push(`- oxc threw: ${stats.oxcError}`);
lines.push(`- canonicalization errors: ${stats.canonError}`);
lines.push('');
lines.push('### On files where react-compiler memoized something (the real comparison)');
lines.push(`- **match: ${stats.memoMatch}**`);
lines.push(`- **mismatch: ${stats.memoMismatch}**`);
lines.push(`- => identical-output rate on memoized files: **${pct}%**`);
lines.push('');
lines.push('### Non-memoized files');
lines.push(`- both no-op, canon-equal: ${stats.bothNoop}`);
lines.push(`- canon differs without memo: ${stats.noopMismatch}`);
lines.push('');
lines.push('## Mismatch causes (on memoized files; - babel / + oxc)');
for (const [cause, count] of Object.entries(buckets).sort((a, b) => b[1] - a[1])) lines.push(`- **${cause}**: ${count}`);
lines.push('');
lines.push('## Per-repo match/mismatch');
for (const [repo, r] of Object.entries(byRepo).sort((a, b) => b[1].mismatch - a[1].mismatch)) {
  if (r.mismatch) lines.push(`- ${repo}: ${r.match} match / ${r.mismatch} mismatch`);
}
lines.push('');
lines.push('## Samples by cause (- babel / + oxc, canonicalized)');
for (const [cause, samples] of Object.entries(bucketSamples)) {
  lines.push(`\n### cause: ${cause}`);
  for (const s of samples) {
    lines.push(`\n**${s.file}**`);
    lines.push('```diff');
    lines.push(s.diff || '(diff beyond first 16 changed lines)');
    lines.push('```');
  }
}
const report = lines.join('\n');
writeFileSync(REPORT, report);
console.log('\n' + report.split('## Samples by cause')[0]);
console.log(`\nfull report -> ${REPORT}`);
