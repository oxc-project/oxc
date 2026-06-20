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

> [!IMPORTANT]
> `oxc_react_compiler` is a Rust port of the **oxc-project fork** of react-compiler
> (`~/github/oxc-project/oxc-react-compiler/react-compiler`), **not** of any published
> npm release. The fork has diverged from npm by ~a year (e.g. it alphabetically
> sorts reactive-scope dependencies via `compareScopeDependency`; older npm builds
> don't). So comparing against **npm** mostly measures *version drift*, while comparing
> against the **fork** measures true *port fidelity*. Use `BPRC` to pick the reference.

### Comparing against the fork (recommended — true port fidelity)

```bash
# build the fork's babel plugin once
cd ~/github/oxc-project/oxc-react-compiler/react-compiler
yarn install && yarn workspace babel-plugin-react-compiler build && cd -

# point the harness at the fork's built plugin
BPRC=~/github/oxc-project/oxc-react-compiler/react-compiler/packages/babel-plugin-react-compiler/dist/index.js \
  REPOS=/path/to/oxc-ecosystem-ci/repos node compare.mjs
```

Without `BPRC`, the pinned npm `babel-plugin-react-compiler` in `package.json` is used.

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

## Findings

**0 oxc crashes** across thousands of real-world files (robustness).

| reference | identical-output on memoized files |
| --- | --- |
| npm `babel-plugin-react-compiler@334f00b` (a year stale) | ~31% — dominated by **version drift** |
| the **fork** oxc actually ports (`yarn build`, via `BPRC`) | **~86%** — true port fidelity |

Against the fork, the remaining ~14% are genuine port gaps:

- **mutation / immutability inference false-positives** (most serious): oxc rejects
  some legal mutations the fork accepts — e.g. `ref.current = x` on a `useRef` value
  raises `[ReactCompiler] Immutability: This value cannot be modified`, which errors
  the whole component and emits an **empty body**. Fixing the inference to treat ref
  `.current` (and other allowed mutations) as mutable closes these.
- **memoization decisions**: oxc declines to compile a few components the fork does.
- **cache-slot counts**: `_c(33)` vs `_c(31)` — different memoization granularity.
- **outlining**: minor differences in which inline callbacks get outlined.

The `~31%` vs npm is almost entirely the fork's alphabetical dependency sort
(`compareScopeDependency`) that the stale npm build lacks — i.e. oxc is *correct*
relative to what it ports; the npm number is not a fair fidelity measure.
