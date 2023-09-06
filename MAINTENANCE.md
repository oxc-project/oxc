# Release crates

Manually edit all versions specified by `[workspace.dependencies]` in Cargo.toml,
also manually edit each of the crates version.

```bash
sed -i '' 's/0.1.2/0.1.3/' crates/oxc/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_allocator/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_ast/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_ast_lower/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_diagnostics/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_formatter/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_hir/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_index/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_minifier/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_parser/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_semantic/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_span/Cargo.toml
sed -i '' 's/0.1.2/0.1.3/' crates/oxc_syntax/Cargo.toml

cargo build
git add .
git commit
just ready
```

Run the following commands, the order is important.

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
