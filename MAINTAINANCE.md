# Release crates

Manually edit all versions specified by `[workspace.dependencies]` in Cargo.toml,
also manually edit each of the crates version.

Install `cargo-smart-release`, run

```bash
cargo smart-release --no-changelog --no-tag --no-push --dry-run-cargo-publish oxc
```

Run again with `--dry-run-cargo-publish` when everything compiles.
