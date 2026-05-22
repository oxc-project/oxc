#!/usr/bin/env bash
#
# Configure crates.io Trusted Publishing for every `publish = true` crate.
#
# For each crate found in `crates/*/Cargo.toml` with `publish = true`:
#   1. Ensure a GitHub Trusted Publishing config exists with environment "crates".
#   2. Delete any other (old) Trusted Publishing configs for that crate.
#   3. Set `trustpub_only = true` so new versions can only be published via
#      Trusted Publishing.
#
# Requirements:
#   - bash, curl, jq
#   - CRATES_IO_TOKEN env var set to a crates.io API token with the
#     `trusted-publishing` scope (and the user must be an owner of every crate).

set -euo pipefail

REPO_OWNER="oxc-project"
REPO_NAME="oxc"
WORKFLOW_FILENAME="release_crates.yml"
ENVIRONMENT="crates"
API="https://crates.io/api/v1"

if [[ -z "${CRATES_IO_TOKEN:-}" ]]; then
  echo "error: CRATES_IO_TOKEN must be set" >&2
  exit 1
fi

for tool in curl jq; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required tool '$tool' not found in PATH" >&2
    exit 1
  fi
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

AUTH_HEADER="Authorization: ${CRATES_IO_TOKEN}"

# Collect crates with `publish = true`.
crates=()
for cargo_toml in crates/*/Cargo.toml; do
  if grep -qE '^[[:space:]]*publish[[:space:]]*=[[:space:]]*false' "$cargo_toml"; then
    continue
  fi
  if ! grep -qE '^[[:space:]]*publish[[:space:]]*=[[:space:]]*true' "$cargo_toml"; then
    # Treat missing `publish` as not-publishable in this repo; only explicit
    # `publish = true` opts in.
    continue
  fi
  name=$(grep -E '^[[:space:]]*name[[:space:]]*=' "$cargo_toml" | head -1 \
    | sed -E 's/.*name[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')
  if [[ -z "$name" ]]; then
    echo "warn: could not determine crate name in $cargo_toml; skipping" >&2
    continue
  fi
  crates+=("$name")
done

echo "Found ${#crates[@]} publishable crates."
echo

api() {
  # api METHOD PATH [DATA]
  local method="$1" path="$2" data="${3:-}"
  if [[ -n "$data" ]]; then
    curl --fail-with-body -sS -X "$method" "$API$path" \
      -H "$AUTH_HEADER" \
      -H "Content-Type: application/json" \
      -d "$data"
  else
    curl --fail-with-body -sS -X "$method" "$API$path" \
      -H "$AUTH_HEADER"
  fi
}

for krate in "${crates[@]}"; do
  echo "==> $krate"

  configs_json=$(api GET "/trusted_publishing/github_configs?crate=$krate")

  matching_id=$(echo "$configs_json" | jq -r --arg owner "$REPO_OWNER" \
    --arg name "$REPO_NAME" --arg wf "$WORKFLOW_FILENAME" --arg env "$ENVIRONMENT" '
      .github_configs[]
      | select(.repository_owner == $owner
        and .repository_name  == $name
        and .workflow_filename == $wf
        and .environment      == $env)
      | .id
    ' | head -1)

  if [[ -n "$matching_id" ]]; then
    echo "    config already exists (id=$matching_id, env=$ENVIRONMENT); skipping create"
  else
    echo "    creating trusted publishing config (env=$ENVIRONMENT)"
    body=$(jq -n \
      --arg c "$krate" \
      --arg o "$REPO_OWNER" \
      --arg n "$REPO_NAME" \
      --arg w "$WORKFLOW_FILENAME" \
      --arg e "$ENVIRONMENT" \
      '{github_config: {crate: $c, repository_owner: $o, repository_name: $n, workflow_filename: $w, environment: $e}}')
    api POST "/trusted_publishing/github_configs" "$body" >/dev/null
    # Refresh after creation so the delete step sees the new config.
    configs_json=$(api GET "/trusted_publishing/github_configs?crate=$krate")
  fi

  # Delete every config that isn't the canonical one.
  old_ids=$(echo "$configs_json" | jq -r --arg owner "$REPO_OWNER" \
    --arg name "$REPO_NAME" --arg wf "$WORKFLOW_FILENAME" --arg env "$ENVIRONMENT" '
      .github_configs[]
      | select(.repository_owner != $owner
        or .repository_name  != $name
        or .workflow_filename != $wf
        or .environment      != $env)
      | .id
    ')
  for id in $old_ids; do
    echo "    deleting old config id=$id"
    api DELETE "/trusted_publishing/github_configs/$id" >/dev/null
  done

  echo "    setting trustpub_only=true"
  api PATCH "/crates/$krate" '{"crate":{"trustpub_only":true}}' >/dev/null
done

echo
echo "Done."
