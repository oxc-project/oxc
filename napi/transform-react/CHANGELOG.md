# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.160.0] - 2026-11-23

### 💥 BREAKING CHANGES

- b90b3aa ast: [**BREAKING**] Remove `base` field from `NumericLiteral` and `BigIntLiteral` (#29919) (overlookmotel)

### 🐛 Bug Fixes

- 38cd470 transformer: Share auto-accessor lowering and fix its private storage names (#29449) (Dunqing)
- 4b1efd3 transformer/legacy-decorator: Move class self-references to alias symbol (#30056) (Dunqing)
- bbe2fba transformer/legacy-decorator: Avoid class name shadowing in static setters (#30058) (Dunqing)
- 8921865 transformer/legacy-decorator: Keep static private brand checks on the original class (#30054) (Dunqing)
- abb3593 transformer/legacy-decorator: Share anonymous default class bindings (#29753) (Dunqing)
- 52b5506 transformer/class-properties: Preserve names of anonymous classes (#29931) (Dunqing)
- 08023ad transformer: Keep TypeScript assign semantics for preserved classes (#29930) (Dunqing)
- 831fbca transformer: Don't fold computed key assignments into non-computed method keys (#29912) (Dunqing)
- 6db561f transformer: Keep correct keys on leftover fields with setPublicClassFields (#29911) (Dunqing)
- bdbed9e transformer/async-to-generator: Avoid invalid and shadowing inferred function names (#29815) (YT)
- a7ea78b transformer: Keep classes whose decorators are preserved (#29748) (Dunqing)
- 5ece804 transformer/async-to-generator: Remove leading digits from inferred function names (#29816) (YT)
- b6ff506 react-compiler: Do not treat the receiver of a JSX ref property load as a ref (#29754) (Boshen)
- 2f6a11f semantic: Keep catch parameters in the catch clause scope (#29635) (Dunqing)
- d59aa8e transformer/legacy-decorator: Apply static member decorators last (#27744) (Yunus Ozen)
- 97307e3 transformer/legacy-decorator: Emit `String`/`Number` metadata for enums declared after the class (#29724) (Max Schwenk)
- 3586c66 transformer: Preserve object rest assignment evaluation (#29645) (Boshen)
- 279185a react-compiler: Ignore ESLint suppressions when hooks and memo validations are enabled (#29655) (Boshen)
- f22757c transformer/class-properties: Preserve receivers in TS-wrapped private calls (#29684) (camc314)
- 0e1ef65 transformer/exponentiation-operator: Scope default parameter temporaries (#29685) (camc314)

### ⚡ Performance

- 892f086 various: Avoid copying source text in `OxcDiagnostic::render_with_source_code` callers (#29850) (overlookmotel)

### 📚 Documentation

- 3ff249b skills: Share PR title guidance with triagebot (#29980) (Cameron)
- 1da6ea8 skills: Merge PR guidance into pr-guide skill (#29866) (Dunqing)

Full release notes: [0.160.0](https://github.com/oxc-project/oxc/blob/main/crates/CHANGELOG.md#01600---2026-11-23)

## [0.157.0] - 2026-11-02

### 🚀 Features

- 8dd67a6 transform-react: Expose TypeScript transform options (Tomas Särnhammar)

## [0.156.0] - 2026-10-27

### 🐛 Bug Fixes

- def47b8 react-compiler: Keep manual memoization as written when its dependencies can't be honoured (#28101) (Dunqing)

## [0.152.0] - 2026-09-28

### 🚀 Features

- 3255b89 transform-react: Add `reportDiagnostics` option to expose recoverable React Compiler diagnostics (#26624) (Jack)
- f585cb4 napi: Add threadless WASI builds (#26898) (Boshen)

## [0.148.0] - 2026-08-31

### 🐛 Bug Fixes

- e74de61 transform-react: Match Babel diagnostic reporting (#26128) (Boshen)

## [0.145.0] - 2026-08-18

### 🐛 Bug Fixes

- 345f981 react_compiler: Skip node_modules by default (#25859) (Boshen)
- dec0a86 react-compiler: Standardize diagnostics (#25702) (Boshen)
- c5f188a react-compiler: Use Babel v1 validation defaults (#25676) (Boshen)
- db44651 napi: Disable reuseWorker in browser bindings (#25640) (leaysgur)
- ce35d47 react_compiler: Preserve JSX import source pragmas (#25592) (Boshen)
- 8d7f9cf react_compiler: Honor eslint suppressions (#25394) (Boshen)

### 📚 Documentation

- 627466e transform-react: Document all options (#25800) (Boshen)

## [0.144.0] - 2026-08-10

### 🚀 Features

- 9a37c41 napi: Restructure React transform options (#25326) (Boshen)

### 🐛 Bug Fixes

- 9d19ab3 transform_react: Downgrade nonfatal compiler diagnostics (#25418) (Boshen)
- 937825c react_compiler: Disable exhaustive memo validation by default (#25417) (Boshen)
- 7d8a9b5 react_compiler: Preserve hook spans for React Refresh (#25393) (Boshen)

## [0.143.0] - 2026-08-03

### 🚀 Features

- 85d0f4d napi: Add React Compiler transform binding (#24934) (Boshen)

### 🐛 Bug Fixes

- 5d8ccf5 react_compiler: Preserve nonfatal diagnostics (#25066) (Boshen)
- d349e4a react_compiler: Match eslint suppression gating (#25068) (Boshen)

# Changelog

All notable changes to this package will be documented in this file.
