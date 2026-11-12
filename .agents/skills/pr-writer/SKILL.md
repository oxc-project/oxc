---
name: pr-writer
description: Write a scoped Conventional Commit title and minimal PR body for an automated Oxc triage fix.
---

# Write an Oxc fix pull request

Read the fix diff and `triage/gh-<issue number>/report.md`. Return only `title`
and `body` through the active finish schema; do not create or modify a pull
request.

Use a concise, scoped Conventional Commit title, such as
`fix(parser): handle trailing comma`, following `.github/workflows/pr.yml`.

Keep the body to a brief explanation of the user-visible problem and why the
change fixes it. Include `Closes #<issue number>` and disclose AI assistance
with `Automated triagebot fix (AI-assisted).`

Do not add headings or sections such as Tests or Evidence. Keep detailed
validation in the triage report. Never invent test results, performance
numbers, reporter confirmation, or reference behavior.
