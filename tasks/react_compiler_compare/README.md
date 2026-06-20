# react_compiler_compare

Compares [`babel-plugin-react-compiler`](https://www.npmjs.com/package/babel-plugin-react-compiler)
against oxc's native React Compiler (exposed via `oxc-transform`'s `reactCompilerSync`)
over real-world source files.

## Setup

```bash
# 1. Build the napi binding that exposes the compiler to JS
cd napi/transform && pnpm build && cd -

# 2. Install the babel comparison deps (isolated; not part of the pnpm workspace)
cd tasks/react_compiler_compare && npm install
```

The pinned `babel-plugin-react-compiler` version
(`0.0.0-experimental-334f00b-20240725`) is the **exact commit oxc's port was based
on**, so differences reflect port fidelity rather than version drift. Bump it in
`package.json` to compare against a newer release.

## Run

```bash
# default: stride-sample 5000 .jsx/.tsx files across all repos
REPOS=/path/to/oxc-ecosystem-ci/repos node compare.mjs

# options
node compare.mjs --cap=2000              # sample size (default 5000)
node compare.mjs --all                   # every candidate file (slow)
node compare.mjs --repos=kibana,next     # only these repos
node compare.mjs --report=out.md         # report path
```

Env: `REPOS` (scan dir, default `../../../oxc-ecosystem-ci/repos`),
`OXC_TRANSFORM` (path to the built `oxc-transform` entry, default
`../../napi/transform/index.js`).

## Methodology

Both compilers run on each file; **both outputs are then normalized** through
`@babel/parser` + `@babel/generator` (formatting metadata stripped) and
string-compared, so only *semantic* differences in the memoization count — not
quote style / spacing / codegen formatting. A file "counts" toward the rate only
when react-compiler actually memoized something (otherwise it's a trivial no-op
match). Mismatches are bucketed by dominant cause.

## Findings (sample: 5000 files, `0.0.0-experimental-334f00b-20240725`)

- **0** oxc crashes across ~5000 real-world files (robustness).
- On files where the compiler memoized something: **~31% byte-identical** to babel
  after normalization.
- Mismatch causes, in order:
  - **dependency / cache-slot ordering** (largest): same dependencies and
    semantics, but ordered differently in the `$[i] !== dep` change-checks, which
    cascades into different slot numbering — e.g.
    `$[4] !== isOn || $[5] !== isDisabled` (babel) vs
    `$[4] !== isDisabled || $[5] !== isOn` (oxc).
  - **memoization decisions**: oxc compiles some functions babel skips (and vice
    versa), notably around `forwardRef`/nested components.
  - **cache-slot counts**: `_c(33)` vs `_c(31)` — different memoization
    granularity.
  - **outlining**: oxc outlines some inline callbacks (`setOpen(_temp)`) babel
    leaves inline.

The dependency-ordering difference is the single biggest lever: it is the most
common cause and is semantically equivalent, so aligning oxc's reactive-scope
dependency ordering with babel's would move the match rate substantially.
