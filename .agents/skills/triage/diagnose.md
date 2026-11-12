# Diagnose

Diagnose only; do not implement a fix. Read `<triageDir>/report.md` first and
always append a diagnosis, even when confidence is low.

If reproduction did not establish a defect, append `DIAGNOSIS SKIPPED: no
reproduced defect` and return `confidence: null`.

## Locate the defect

Trace the reported behavior into the owning crate or package. Read its
`AGENTS.md`, nearby fixtures and tests, and any applicable compatibility or
divergence notes before proposing a change. For a compatibility report,
inspect the reference implementation at the revision used in reproduction.

Narrow the first observable divergence with focused tests, backtraces, or
temporary instrumentation. Keep instrumentation small and remove every
tracked instrumentation change before returning. Compare complete relevant
functions, and check history for an already-known constraint or rejected
approach. Distinguish a rule defect from configuration, parser, semantic,
loader, plugin, and binding problems.

For a performance or memory report, use the repository's relevant measurement
guidance. Do not infer a regression from one timing or add benchmarks to the
fix unless requested. Never measure private code.

## Report and return

Append the responsible functions with paths and lines, the specific defect
that explains the observed behavior, a minimal fix approach, affected
consumers, supporting evidence, remaining uncertainty, and confidence.

Use `high` only when execution evidence identifies the specific defect. Use
`medium` when the cause is well supported but not fully isolated. Use `low`
for a hypothesis. Return that confidence.
