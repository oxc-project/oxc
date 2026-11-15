# Verify

Verify only; do not edit source. Read `<triageDir>/report.md` and always append
the verdict and its evidence.

## Apply the subsystem's contract

Use the specification, reference revision, fixtures, and documented intentional
differences that govern the affected subsystem:

- `bug`: a supported use case violates that contract, crashes, or changes
  observable behavior incorrectly.
- `intended-behavior`: the behavior follows that contract or an explicit,
  documented intentional difference.
- `unclear`: the available reproduction cannot establish either conclusion.

Confirm that the compared tools used the same source, filename/extension,
configuration, enabled rules or stages, working directory, and relevant flags.
For linter reports, verify the upstream rule and options. For formatter
reports, check the target Prettier revision and divergence policy. Different
minified text alone does not establish a semantic bug.

Use source and existing tests to resolve setup differences. A failing output
already recorded in a conformance snapshot does not make the behavior
intentional; snapshots also track known failures. Do not override documented
compatibility decisions based only on personal expectations.

Append the reporter's claim, verdict, comparison or specification evidence,
and confidence to `report.md`. Use `high` for a direct deterministic result,
`medium` for strong but incomplete evidence, and `low` when ambiguity remains.
Return `verdict` and `confidence`.
