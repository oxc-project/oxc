#!/usr/bin/env bash
# The pinned triagebot-action revision assumes these labels exist. Keep existing
# labels untouched and create only the ones the action needs but cannot bootstrap.
set -euo pipefail

existing="$(gh api --paginate "repos/$GITHUB_REPOSITORY/labels?per_page=100" --jq '.[].name')"

while IFS=$'\t' read -r name color description; do
  if grep -Fqx -- "$name" <<<"$existing"; then
    continue
  fi

  gh label create \
    --repo "$GITHUB_REPOSITORY" \
    --color "$color" \
    --description "$description" \
    -- "$name"
  existing+=$'\n'"$name"
done <<'LABELS'
triage: needs triage	fbca04	Waiting for automated investigation
triage: not actionable	ededed	Not an actionable bug report
triage: needs reproduction	d876e3	More information is needed to reproduce the report
triage: skipped	cfd3d7	Cannot be investigated in the triage environment
triage: unable to reproduce	e4e669	Automated investigation could not reproduce the report
triage: unable to fix	b60205	Reproduced, but no verified automated fix was found
triage: failed	d73a4a	Automated investigation failed unexpectedly
triage: fix pending	0e8a16	A proposed fix is waiting for reporter verification
triage: fix rejected	d93f0b	The reporter says the proposed fix does not work
triage: fix verified	0e8a16	The proposed fix was verified
- P0	b60205	Release-blocking or broadly catastrophic
- P1	d93f0b	High-impact correctness or reliability problem
- P2	fbca04	Confirmed issue with normal priority
- P3	0e8a16	Narrow or low-impact issue
LABELS
