# Release Linter

* Test in large codebases

```bash
mkdir test-oxc && cd test-oxc
git clone --depth=1 git@github.com:microsoft/vscode.git
git clone --depth=1 git@github.com:getsentry/sentry.git
git clone --depth=1 git@github.com:elastic/kibana.git
git clone --depth=1 git@github.com:toeverything/AFFiNE.git
git clone --depth=1 git@github.com:DefinitelyTyped/DefinitelyTyped.git
```

```bash
# cd to oxc
cargo build --release -p oxc_cli --bin oxlint

# cd to test-oxc and run oxlint on all cloned repos
~/path/to/oxc/target/target/release/oxlint
```

* push the version commit, e.g. https://github.com/oxc-project/oxc/commit/31600ac8dea270e169d598e0e3b5b7a16cbb1c71
* clean up the change log

# Release crates

Manually edit all versions specified by `[workspace.dependencies]` in Cargo.toml,
also manually edit each of the crates version.

```bash
sed -i '' 's/0.10.0/0.11.0/' Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_allocator/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_ast/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_codegen/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_sourcemap/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_diagnostics/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_index/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_minifier/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_parser/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_semantic/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_span/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_syntax/Cargo.toml
sed -i '' 's/0.10.0/0.11.0/' crates/oxc_transformer/Cargo.toml

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
cargo publish -p oxc_sourcemap
cargo publish -p oxc_codegen
cargo publish -p oxc_transformer
cargo publish -p oxc_minifier
cargo publish -p oxc
```
