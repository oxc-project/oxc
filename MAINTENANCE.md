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
just oxlint

# cd to test-oxc and run oxlint on all cloned repos
~/path/to/oxc/target/target/release/oxlint
```

* push the version commit, e.g. https://github.com/oxc-project/oxc/commit/31600ac8dea270e169d598e0e3b5b7a16cbb1c71
* clean up the GitHub changelog

# Release crates

Releasing crates is managed by [`cargo-release-oxc`](https://github.com/oxc-project/cargo-release-oxc).

```bash
cargo binstall cargo-release-oxc
```

```bash
cargo release-oxc update --patch
just ready
cargo release-oxc publish
```
