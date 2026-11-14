# Reproduce

Reproduce only; do not diagnose or edit tracked source files. Always write
`<triageDir>/report.md`, including on an early exit or infrastructure failure.

## Decide whether to continue

Read the issue body and all comments, with later maintainer or reporter
clarifications taking precedence. Finish this screening before any build or
installation. Write the report and return immediately for a supported skip.

Return a skipped result when:

- `not-actionable`: it is a feature request, question, general performance
  idea, explicitly unsupported behavior, or otherwise not a bug report.
- `missing-details`: there is no runnable input or steps, or no expected
  behavior/output to compare.
- `unsupported-version`: it is demonstrated only on an old Oxc version and
  cannot be shown on the checked-out revision.
- `host-specific` or `unsupported-runtime`: reproduction fundamentally
  requires a host or runtime absent from the Ubuntu runner.
- `maintainer-override`: a commenter whose `authorAssociation` is `MEMBER`,
  `COLLABORATOR`, or `OWNER` explicitly says not to auto-triage the issue.

Do not treat a report as stale merely because its stated version is old: first
try its minimal reproduction against the checkout. When a report includes an
Oxc playground URL, decode it with `.agents/skills/oxc-playground-url/SKILL.md`
to preserve its source, options, stages, and lint rules.

## Build and run the affected tool

Use the smallest existing example, CLI, or test harness that exercises the
reported path. For example, `cargo run -p oxc_parser --example parser --
<file>` exercises parsing and `cargo run -p oxlint -- <args>` exercises the
Rust CLI. Node.js bindings, JavaScript plugins, and embedded formatter support
may require a package build; follow the affected package's instructions
rather than assuming a Rust-only reproduction covers them.

Use development builds unless the issue depends on release behavior or
performance. Keep source files, configuration, third-party dependencies, and
captured output under `<triageDir>`; Cargo's normal `target/` output is ignored.
Inspect linked source and package scripts before executing them; prefer
copying the relevant files over running an entire third-party project.

Compare identical input, filename/extension, configuration, parser mode, and
relevant flags. Choose the reference from the affected subsystem's guidance:

- Parsing and semantics: ECMAScript/TypeScript grammar and the repository's
  Test262, Babel, and TypeScript conformance expectations.
- Lint rules: the corresponding upstream rule and version, with the same
  enabled rules and options; account for documented Oxlint differences.
- Formatting: the Prettier version and options prescribed by the affected
  formatter's `AGENTS.md`, including documented divergences.
- Transformations, code generation, and declarations: the affected crate's
  fixtures and reference implementation or language contract.
- Minification: preservation of observable behavior; different output text
  alone is not a bug.

Do not assume the latest npm release is the repository's intended reference.
Capture commands, versions, stdout, stderr, exit status, panics, and timeouts.
Confirm a nearby control case when useful. A crash on supported input can be
reproduced without a reference binary, but a claimed compatibility difference
needs the appropriate reference or specification evidence.

## Report and return

Write `report.md` with the issue claim, affected subsystem, exact commands,
revisions, minimal input and configuration, complete observed outputs, exit
statuses, expected behavior and its source, and one of:

- `REPRODUCED`: include the exact defect and reference or specification evidence.
- `NOT REPRODUCED`: include what was tried and what matched.
- `SKIPPED: <reason>`: include one supported `skippedReason` and why.

Return `reproducible: true` only for `REPRODUCED`. Return `skipped: true` only
for a documented early exit and set `skippedReason` to that reason; otherwise
return `skipped: false` and `skippedReason: null`. An infrastructure failure
is not proof that the reported behavior is correct.
