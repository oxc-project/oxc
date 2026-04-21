# Development Roadmap

## Phase 1: Foundation ✓

- [x] Basic peephole optimizations
- [x] Constant folding
- [x] Dead code elimination
- [x] Test infrastructure (minsize, coverage, e2e)
- [x] Fixed-point iteration

## Phase 2: Complete Optimization Suite ✓

- [x] Port safe optimizations from Closure Compiler
- [x] Port core optimizations from Terser/UglifyJS
- [x] Advanced constant propagation
- [x] Cross-statement optimizations
- [x] Template literal optimization
- [x] Array/object patterns

## Phase 3: Advanced Optimizations (Current)

- [ ] Function inlining (when provably safe)
- [ ] Switch statement optimization
- [ ] Advanced string concatenation (extending `oxc_ecmascript` string operations)
- [ ] Enum unboxing
- [ ] Property collapsing
- [ ] Better RegExp optimization
- [ ] Loop optimizations
- [ ] Cross-module optimization
- [ ] Perfect side effect analysis (extending `oxc_ecmascript` capabilities)
- [ ] Advanced DCE with escape analysis
- [ ] Type-aware optimizations (from TS types)
- [ ] Framework-specific optimizations (React, Vue, Angular)

## Phase 4: Production Ready

- [ ] Differential testing framework
- [ ] Fuzzing infrastructure
- [ ] Performance optimization
- [ ] Source map improvements
- [ ] Custom optimization plugins

## Goals

**Size**: Beat Closure Compiler, smallest output for top npm packages
**Correctness**: 100% test262/Babel/TypeScript conformance
**Performance**: < 2x slower than esbuild, < 10x faster than Terser
**Adoption**: Integration in major build tools

## Contributing

- Port optimizations from Closure Compiler, Terser, esbuild
- Test with real-world code and report issues
- Performance profiling and optimization
- Documentation improvements

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
