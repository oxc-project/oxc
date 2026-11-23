---
name: triage-bot-pr-writer
description: Write the title and short PR body for an automated Oxc triage fix using triagebot-action's finish schema.
---

# Write an automated triage PR

Read the fix diff and `triage/gh-<issue number>/report.md`. Use the [PR title guidance](../pr-writer/SKILL.md#pr-titles) to write a concise, scoped `fix` title.

Briefly explain the user-visible problem and why the change fixes it. Include `Closes #<issue number>` and `Automated triagebot fix (AI-assisted).` Follow the AI disclosure requirements in the root [AGENTS.md](../../../AGENTS.md). Keep detailed validation in the triage report and omit headings from the body. Use only verified test results, performance numbers, reporter confirmation, and reference behavior.

Return only `title` and `body` through the active finish schema. Leave PR creation and updates to triagebot-action.
