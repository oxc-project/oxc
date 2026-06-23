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
> don't). So comparing against **npm** mostly measures _version drift_, while comparing
> against the **fork** measures true _port fidelity_. Use `BPRC` to pick the reference.

### Comparing against built source (recommended — true port fidelity)

Build either the oxc fork **or** upstream `react/react` main and point `BPRC` at the
built plugin. As of 2026-06-18 the oxc fork is a same-day mirror of upstream main, so
both give the same result (~86%):

```bash
# upstream react/react main (sparse-clone just the compiler)
git clone --depth 1 --filter=blob:none --sparse https://github.com/react/react.git /tmp/react-upstream
cd /tmp/react-upstream && git sparse-checkout set compiler
cd compiler && yarn install && yarn workspace babel-plugin-react-compiler build && cd -
BPRC=/tmp/react-upstream/compiler/packages/babel-plugin-react-compiler/dist/index.js \
  REPOS=/path/to/oxc-ecosystem-ci/repos node compare.mjs

# ...or the oxc fork it directly ports
#   cd ~/github/oxc-project/oxc-react-compiler/react-compiler
#   yarn install && yarn workspace babel-plugin-react-compiler build
#   BPRC=.../packages/babel-plugin-react-compiler/dist/index.js node compare.mjs
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
string-compared, so only _semantic_ differences in the memoization count — not
quote style / spacing / codegen formatting. A file "counts" toward the rate only
when react-compiler actually memoized something (otherwise it's a trivial no-op
match). Mismatches are bucketed by dominant cause.

## Findings

**0 oxc crashes** across thousands of real-world files (robustness).

| reference                                                | identical-output on memoized files    |
| -------------------------------------------------------- | ------------------------------------- |
| npm `babel-plugin-react-compiler@334f00b` (a year stale) | ~31% — dominated by **version drift** |
| upstream `react/react` main (built, via `BPRC`)          | **85.6%** (1747/2040 over 3000 files) |
| the oxc fork it ports (same-day mirror of upstream)      | 85.6% — identical to upstream main    |

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
(`compareScopeDependency`) that the stale npm build lacks — i.e. oxc is _correct_
relative to what it ports; the npm number is not a fair fidelity measure.
