# React Compiler comparison

Recursively compares the published `babel-plugin-react-compiler` pipeline with
the local `oxc-transform-react` NAPI package for every `.jsx` and `.tsx` file in
a directory. Both pipelines run React Compiler first, remove TypeScript syntax,
and lower JSX with the automatic runtime.

Every React Compiler option shared by both implementations is set explicitly to
the same value. The comparison uses the v1 defaults, including its ESLint
suppression rules and disabled exhaustive manual-memo and void-`useMemo`
validations. Per-binding adapters account for the two environment option names
that Babel and Oxc spell differently. Babel-only callback, instrumentation, and
test options are fixed at the neutral values used internally by Oxc. The scanned
directory is passed as `sources` so dependency directories are handled
consistently by both implementations. Babel's TypeScript transform also enables
`allowDeclareFields` so uninitialized class fields use the same emit behavior as
Oxc.

Before comparing, the Babel output is parsed and printed by `oxc-transform`
with no transforms enabled. This ensures both outputs use Oxc code generation
and removes printer-only differences.

Build the native bindings and pass a file or directory to scan:

```sh
pnpm --dir napi/transform build-test
pnpm --dir napi/transform-react build-test
pnpm --filter react_compiler compare ./path/to/source
```

The command prints each differing path relative to the scanned directory, one
per line. Transform diagnostics and the final summary are written to stderr.
It exits with status 1 when outputs differ or either transform fails.

## Options

- `--jobs=<count>` — how many worker threads compare files in parallel.
  Defaults to two fewer than the available parallelism. Workers are recycled
  periodically, and a file that takes longer than two minutes is reported as a
  failure rather than stalling the scan.
- `--report=<file>` — write one JSON object per non-matching file, holding its
  status, category, and diff hunks.
- `--dump=<directory>` — write both pipelines' full output for every
  non-matching file, as `<path>.babel.js` and `<path>.oxc.js`, for diffing by
  hand.

A report file can then be summarized, which prints how the mismatches break
down, which repositories each category comes from, and a representative diff
for each:

```sh
pnpm --filter react_compiler compare ./path/to/source --report=report.jsonl
pnpm --filter react_compiler report report.jsonl
```

## Categories

Babel's JSX transform writes `/*#__PURE__*/` where Oxc's writes
`/* @__PURE__ */`. Neither spelling comes from React Compiler, so files that
differ only in it are reported separately as `pure-annotation` and the spelling
is canonicalized before anything else is compared.

Remaining mismatches are categorized by `categorize.mjs`, which applies known
cosmetic normalizations — dropped comments, elided unused imports, import
layout, whitespace, generated temporary numbering, binding renames, object
shorthand, redundant parentheses — and reports the smallest set of them that
makes the two outputs equal. Reporting the set rather than a single label keeps
a file whose mismatch has several independent causes from being filed under
whichever one happened to be checked first.

A file no combination reconciles is `structural`: the two pipelines emitted
genuinely different code. Two structural cases get their own labels first —
`memoization-scope` when the sides memoized a different number of functions,
and `statement-order` when they emitted the same lines in a different order.

The normalizations are text-level heuristics for triage, not proofs of
equivalence. Every report entry also carries diff hunks, so a category can be
checked against the actual difference.
