# oxc_react_compiler_upstream

Benchmark-only adapter between Oxc's AST and the Babel-shaped Rust AST accepted by React's
upstream compiler crates. The React dependency is pinned in the workspace manifest to commit
`eb8feb71096eec5c885b2a4c7d8d030d3622f265` so comparisons are reproducible.

This crate intentionally includes both AST conversions in the measured path. It exists to compare
that upstream architecture with Oxc's native `oxc_react_compiler` port and is not published.
