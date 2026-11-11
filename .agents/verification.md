# Verification

Choose checks from the diff, including affected consumers; for stacks, compare
each PR with its parent. Follow owning guides in [AGENTS.md](../AGENTS.md#testing)
for generators, conformance, and bindings.

- **Rust code/tests:** `just lint -p <crate>` and unfiltered `just test -p <crate>`
  where tests exist. Use filtered tests while debugging; unexpected zero tests
  are incomplete. Use conformance for syntax regressions where prescribed.
- **Prose:** formatting and typos only.
- **Rustdoc:** `just doc -p <crate>`; add
  `cargo test -p <crate> --all-features --doc` for changed executable examples.

Reuse passing results until tested code, dependencies, generated inputs,
toolchain, or build configuration changes; commits alone do not invalidate them.
Keep profiles, features, and package selections consistent. Run required
standalone/feature checks separately because grouped builds can mask errors.
Serialize Cargo commands sharing a target directory.

For unclear/workspace-wide impact, broaden to `just test`, `just lint`, and
subsystem checks. Use `just ready` as an optional stable checkpoint instead of
equivalent repeated checks; it needs a clean tracked worktree. Review its output.
For stacks, run the broad checkpoint once at the tip; every mergeable PR still
needs its required CI.
