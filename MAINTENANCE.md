# Release crates

Manually edit all versions specified by `[workspace.dependencies]` in Cargo.toml,
also manually edit each of the crates version.

Run

```bash
cargo publish -p oxc_allocator
cargo publish -p oxc_index
cargo publish -p oxc_span
cargo publish -p oxc_syntax
cargo publish -p oxc_ast
cargo publish -p oxc_diagnostics
cargo publish -p oxc_parser
cargo publish -p oxc_semantic
cargo publish -p oxc_formatter
cargo publish -p oxc_hir
cargo publish -p oxc_ast_lower
cargo publish -p oxc_minifier
cargo publish -p oxc
```
