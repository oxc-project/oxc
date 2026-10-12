# Development Roadmap

This document distinguishes implemented foundations from possible future work.
Unchecked items are ideas, not committed features or delivery dates. The
[optimization catalog](./OPTIMIZATIONS.md) describes current transforms.

## Phase 1: Foundation ✓

- [x] Basic peephole optimizations
- [x] Constant folding
- [x] Dead code elimination
- [x] Test infrastructure (minsize, coverage, e2e)
- [x] Fixed-point iteration

## Phase 2: Optimization Suite

- [x] Port safe optimizations from Closure Compiler
- [x] Port core optimizations from Terser/UglifyJS
- [x] Advanced constant propagation
- [x] Cross-statement optimizations
- [x] Template literal optimization
- [x] Array/object patterns

## Phase 3: Further Optimization Ideas

- [ ] Broader function inlining (limited IIFE simplification is already implemented)
- [x] Switch statement optimization (known-discriminant case removal and switch folding)
- [ ] Advanced string concatenation (extending `oxc_ecmascript` string operations)
- [ ] Enum unboxing
- [ ] Property collapsing
- [ ] Better RegExp optimization
- [ ] Broader loop optimizations (while/for normalization and for-statement transforms exist)
- [ ] Cross-module optimization
- [ ] More precise side effect analysis (extending `oxc_ecmascript` capabilities)
- [ ] Advanced DCE with escape analysis
- [ ] Additional JavaScript value analysis (the minifier does not consume TypeScript types)
- [ ] Framework-specific optimizations (React, Vue, Angular)

## Further Infrastructure Ideas

- [ ] Broader differential testing (test262 runtime checks and Node.js E2E tests already exist)
- [ ] Fuzzing infrastructure
- [ ] Performance optimization
- [ ] Source map improvements
- [ ] Custom optimization plugins

## Goals

**Size**: Beat Closure Compiler, smallest output for top npm packages
**Correctness**: Preserve JavaScript behavior under documented assumptions, with test262/Babel idempotency and runtime regression coverage
**Performance**: < 2x slower than esbuild, < 10x faster than Terser
**Adoption**: Integration in major build tools

## Contributing

- Port optimizations from Closure Compiler, Terser, esbuild
- Test with real-world code and report issues
- Performance profiling and optimization
- Documentation improvements

See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.
