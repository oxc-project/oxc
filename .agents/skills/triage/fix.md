# Fix

Before attempting a fix, read `issue.assignees` from the JSON event payload at
`GITHUB_EVENT_PATH`. If the issue has any assignees, leave the fix to them:
do not edit source code or add a fix regression. Append to
`<triageDir>/report.md` that investigation is complete and implementation was
left to the assignees, then return `fixed: false` with `commitMessage: null`.
If assignment information cannot be read, report that and return the same
result without attempting a fix.

Implement a fix only when verification found `bug` and diagnosis has specific,
medium- or high-confidence evidence. Otherwise append why no safe fix was
attempted and return `fixed: false` with `commitMessage: null`.

Read `<triageDir>/report.md`, `AGENTS.md`, `.agents/verification.md`, and any
nearer `AGENTS.md` before editing.

## Implement and add a regression

Make the smallest change that fixes the reported behavior. Preserve Oxc's
allocation and performance conventions and existing user changes. Avoid
unrelated cleanup, new features, dependency changes, and new benchmarks.

Add a regression that fails before the fix and passes after it in the owning
subsystem's established location and format. Parser regressions belong in
`tasks/coverage/misc/pass/` or `fail/`; linter rule cases belong in their inline
`Tester`; formatter changes follow the affected crate's fixtures and guide.
Use the other subsystems' locations listed in the root `AGENTS.md`.

Do not hand-edit generated directories. When the source of generated output
changes, run its owning generator as required by the repository guide. Review
snapshot changes using `.agents/skills/insta-snapshots/SKILL.md`; accept only
changes explained by the fix, including conformance improvements.

## Verify

Rerun the exact reproduction with rebuilt code and the same reference or
specification used before. Follow `.agents/verification.md`: for Rust changes,
run `just lint -p <crate>` and unfiltered `just test -p <crate>` where tests
exist, plus required subsystem conformance, generation, allocation, or size
checks. Use the formatter crate's verification commands for formatter changes.
For bindings or JavaScript-facing changes, build and test the affected package.
Reuse passing results while their inputs are unchanged; serialize Cargo
commands sharing a target directory.

Run `just fmt` after modifications and check for unrelated formatting changes.
Do not claim success when a required check failed or could not run. Remove
instrumentation and move scratch projects, reports, and logs into `<triageDir>`
or delete them. Every non-ignored entry left in
`git status --short --untracked-files=all` is committed by the action, so it
must list only the fix's source, regression, generated, and necessary
documentation changes. Do not commit or push.

Append the changed behavior, regression, commands and actual results, and
final diff summary to `report.md`. Return `fixed: true` only when the original
reproduction is resolved and the required checks pass. Use a scoped
Conventional Commit message such as `fix(parser): handle trailing comma`;
otherwise return `commitMessage: null`.
