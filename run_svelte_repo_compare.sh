#!/usr/bin/env bash
set -euo pipefail

OXC_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILE="auto"
TARGET=""

usage() {
  cat <<USAGE >&2
usage: $0 [--profile auto|svelte-core|generic] /path/to/svelte-repo
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --profile" >&2
        usage
        exit 2
      fi
      PROFILE="$2"
      shift 2
      ;;
    --profile=*)
      PROFILE="${1#*=}"
      if [[ -z "$PROFILE" ]]; then
        echo "missing value for --profile" >&2
        usage
        exit 2
      fi
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "unknown option: $1" >&2
      usage
      exit 2
      ;;
    *)
      if [[ -z "$TARGET" ]]; then
        TARGET="$1"
      else
        echo "unexpected extra positional argument: $1" >&2
        usage
        exit 2
      fi
      shift
      ;;
  esac
done

if [[ -z "$TARGET" ]]; then
  usage
  exit 2
fi

TARGET="$(cd "$TARGET" && pwd)"
if [[ ! -d "$TARGET" ]]; then
  echo "target not found: $TARGET" >&2
  exit 2
fi

detect_profile() {
  if [[ -f "$TARGET/packages/svelte/src/compiler/index.js" ]] || [[ -d "$TARGET/packages/svelte/tests" ]]; then
    echo "svelte-core"
  else
    echo "generic"
  fi
}

if [[ "$PROFILE" == "auto" ]]; then
  PROFILE_RESOLVED="$(detect_profile)"
else
  PROFILE_RESOLVED="$PROFILE"
fi

if [[ "$PROFILE_RESOLVED" != "svelte-core" && "$PROFILE_RESOLVED" != "generic" ]]; then
  echo "invalid profile: $PROFILE_RESOLVED (expected auto|svelte-core|generic)" >&2
  exit 2
fi

WORKDIR="$TARGET/.oxc-compare"
mkdir -p "$WORKDIR"
ESLINT_LOG="$WORKDIR/eslint.log"
PRETTIER_LOG="$WORKDIR/prettier-check.log"
OXLINT_LOG="$WORKDIR/oxlint.log"
OXFMT_LOG="$WORKDIR/oxfmt-check.log"
SUMMARY_TXT="$WORKDIR/summary.txt"

say() {
  printf '\n== %s ==\n' "$1"
}

run_allow_fail() {
  local cmd="$1"
  local log="$2"
  local start_seconds=$SECONDS
  set +e
  bash -lc "$cmd" >"$log" 2>&1
  local code=$?
  set -e
  LAST_RUN_SECONDS=$((SECONDS - start_seconds))
  return "$code"
}

ratio_or_na() {
  local base="$1"
  local candidate="$2"
  awk -v base="$base" -v candidate="$candidate" 'BEGIN {
    if (base == 0) {
      print "n/a"
    } else {
      printf "%.2fx", candidate / base
    }
  }'
}

ignore_patterns_js() {
  if [[ "$PROFILE_RESOLVED" == "svelte-core" ]]; then
    cat <<'PAT'
    '.oxc-compare/**',
    '**/*.d.ts',
    '**/tests/**',
    'packages/svelte/scripts/process-messages/templates/*.js',
    'packages/svelte/scripts/_bundle.js',
    'packages/svelte/src/compiler/errors.js',
    'packages/svelte/src/internal/client/errors.js',
    'packages/svelte/src/internal/client/warnings.js',
    'packages/svelte/src/internal/shared/warnings.js',
    'packages/svelte/src/internal/server/warnings.js',
    'packages/svelte/compiler/index.js',
    'benchmarking/**',
    'coverage/**',
    'playgrounds/sandbox/**',
    '*.config.js',
    'vitest-xhtml-environment.ts',
    'documentation/**',
    'tmp/**',
PAT
  else
    cat <<'PAT'
    '.oxc-compare/**',
    'node_modules/**',
    'dist/**',
    'build/**',
    '.svelte-kit/**',
    '.vercel/**',
    'coverage/**',
PAT
  fi
}

build_if_needed() {
  local head stamp need_build=0
  head="$(git -C "$OXC_ROOT" rev-parse HEAD 2>/dev/null || echo no-git)"
  stamp="$OXC_ROOT/.last_svelte_compare_build_head"

  [[ -f "$OXC_ROOT/apps/oxlint/dist/cli.js" ]] || need_build=1
  [[ -f "$OXC_ROOT/apps/oxfmt/dist/cli.js" ]] || need_build=1
  compgen -G "$OXC_ROOT/apps/oxlint/src-js/*.node" > /dev/null || need_build=1
  compgen -G "$OXC_ROOT/apps/oxfmt/src-js/*.node" > /dev/null || need_build=1
  if [[ ! -f "$stamp" ]] || [[ "$(cat "$stamp")" != "$head" ]]; then
    need_build=1
  fi

  if [[ "$need_build" -eq 1 ]]; then
    say "building oxlint"
    mise -C "$OXC_ROOT" x -- bash -lc "cd '$OXC_ROOT/apps/oxlint' && pnpm run build-test"
    say "building oxfmt"
    mise -C "$OXC_ROOT" x -- bash -lc "cd '$OXC_ROOT/apps/oxfmt' && pnpm run build-test"
    printf '%s\n' "$head" > "$stamp"
  else
    say "using existing oxc build for head $head"
  fi
}

install_if_needed() {
  if [[ ! -d "$TARGET/node_modules" ]]; then
    say "installing target dependencies"
    bash -lc "cd '$TARGET' && pnpm install --frozen-lockfile"
  else
    say "target dependencies already present"
  fi
}

ensure_svelte_shims() {
  if [[ "$PROFILE_RESOLVED" != "svelte-core" ]]; then
    return 0
  fi

  mkdir -p "$TARGET/node_modules"

  if [[ -d "$TARGET/node_modules/.pnpm/node_modules/@typescript-eslint" && ! -e "$TARGET/node_modules/@typescript-eslint" ]]; then
    ln -sfn .pnpm/node_modules/@typescript-eslint "$TARGET/node_modules/@typescript-eslint"
  fi

  if [[ -d "$TARGET/node_modules/.pnpm/node_modules/svelte-eslint-parser" && ! -e "$TARGET/node_modules/svelte-eslint-parser" ]]; then
    ln -sfn .pnpm/node_modules/svelte-eslint-parser "$TARGET/node_modules/svelte-eslint-parser"
  fi

  if [[ -f "$TARGET/packages/svelte/src/compiler/index.js" && ! -e "$TARGET/packages/svelte/compiler/index.js" ]]; then
    mkdir -p "$TARGET/packages/svelte/compiler"
    ln -sfn ../src/compiler/index.js "$TARGET/packages/svelte/compiler/index.js"
  fi
}

find_prettier_config() {
  local candidate
  for candidate in \
    .prettierrc \
    .prettierrc.json \
    .prettierrc.yml \
    .prettierrc.yaml \
    .prettierrc.js \
    .prettierrc.cjs \
    prettier.config.js \
    prettier.config.cjs; do
    if [[ -f "$TARGET/$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

migrate_oxfmt_if_needed() {
  if [[ ! -f "$TARGET/.oxfmtrc.json" ]]; then
    local prettier_config
    prettier_config="$(find_prettier_config || true)"
    if [[ -n "$prettier_config" ]]; then
      say "migrating $prettier_config to .oxfmtrc.json"
      bash -lc "cd '$TARGET' && node '$OXC_ROOT/apps/oxfmt/dist/cli.js' --migrate prettier '$prettier_config'"
    else
      say "no prettier config found; writing default .oxfmtrc.json"
      cat > "$TARGET/.oxfmtrc.json" <<'CFG'
{}
CFG
    fi
  else
    say "using existing .oxfmtrc.json"
  fi
}

generate_oxlint_config_if_needed() {
  local should_write=0
  if [[ ! -f "$TARGET/oxlint.config.js" ]]; then
    should_write=1
    say "creating conservative oxlint.config.js"
  elif grep -q "managed by run_svelte_repo_compare.sh" "$TARGET/oxlint.config.js"; then
    should_write=1
    say "updating managed oxlint.config.js"
  else
    say "using existing oxlint.config.js"
  fi

  if [[ "$should_write" -eq 1 ]]; then
    local ignore_patterns
    ignore_patterns="$(ignore_patterns_js)"
    if [[ "$PROFILE_RESOLVED" == "generic" ]]; then
      cat > "$TARGET/oxlint.config.js" <<CFG
// managed by run_svelte_repo_compare.sh
export default {
  ignorePatterns: [
$ignore_patterns
  ],
};
CFG
    elif [[ -f "$TARGET/svelte.config.js" ]]; then
      cat > "$TARGET/oxlint.config.js" <<CFG
// managed by run_svelte_repo_compare.sh
import sveltePlugin from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import tsParser from '@typescript-eslint/parser';
import svelteConfig from './svelte.config.js';

export default {
  categories: {
    correctness: 'off',
  },
  extends: [sveltePlugin.configs.recommended],
  overrides: [
    {
      files: ['**/*.svelte'],
      languageOptions: {
        parser: svelteParser,
        parserOptions: {
          parser: tsParser,
          extraFileExtensions: ['.svelte'],
          svelteConfig,
        },
      },
    },
  ],
  ignorePatterns: [
$ignore_patterns
  ],
};
CFG
    else
      cat > "$TARGET/oxlint.config.js" <<CFG
// managed by run_svelte_repo_compare.sh
import sveltePlugin from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import tsParser from '@typescript-eslint/parser';

export default {
  categories: {
    correctness: 'off',
  },
  extends: [sveltePlugin.configs.recommended],
  overrides: [
    {
      files: ['**/*.svelte'],
      languageOptions: {
        parser: svelteParser,
        parserOptions: {
          parser: tsParser,
          extraFileExtensions: ['.svelte'],
        },
      },
    },
  ],
  ignorePatterns: [
$ignore_patterns
  ],
};
CFG
    fi
  fi
}

parse_eslint_json_counts() {
  python3 - "$1" <<'PY'
import json, sys
text = open(sys.argv[1]).read().strip()
if not text:
    print('0 0')
    raise SystemExit
# `mise x` can print setup lines before eslint JSON output.
start = text.find('[')
if start == -1:
    print('0 0')
    raise SystemExit
try:
    arr = json.loads(text[start:])
except Exception:
    print('0 0')
    raise SystemExit
errors = 0
warnings = 0
for item in arr:
    errors += int(item.get('errorCount', 0)) + int(item.get('fatalErrorCount', 0))
    warnings += int(item.get('warningCount', 0))
print(f"{errors} {warnings}")
PY
}

parse_oxlint_counts() {
  python3 - "$1" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
m = re.search(r'Found\s+(\d+)\s+warnings?\s+and\s+(\d+)\s+errors?\.', text)
if m:
    print(f"{m.group(2)} {m.group(1)}")
else:
    print('0 0')
PY
}

parse_prettier_check_files() {
  python3 - "$1" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
m = re.search(r'Code style issues found in\s+(\d+)\s+files?\.', text)
if m:
    print(m.group(1))
elif 'All matched files use Prettier code style!' in text:
    print('0')
else:
    print('0')
PY
}

parse_oxfmt_check_files() {
  python3 - "$1" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
m = re.search(r'Format issues found in above\s+(\d+)\s+files?\.', text)
if m:
    print(m.group(1))
elif 'All matched files use the correct format.' in text:
    print('0')
else:
    print('0')
PY
}

build_if_needed
install_if_needed
ensure_svelte_shims

say "running eslint"
ESLINT_EXIT=0
run_allow_fail "cd '$TARGET' && pnpm exec eslint . -f json" "$ESLINT_LOG" || ESLINT_EXIT=$?
ESLINT_TIME_SEC="$LAST_RUN_SECONDS"
read -r ESLINT_ERRORS ESLINT_WARNINGS < <(parse_eslint_json_counts "$ESLINT_LOG")

say "running prettier --check"
PRETTIER_EXIT=0
run_allow_fail "cd '$TARGET' && pnpm exec prettier --check ." "$PRETTIER_LOG" || PRETTIER_EXIT=$?
PRETTIER_TIME_SEC="$LAST_RUN_SECONDS"
PRETTIER_FILES=$(parse_prettier_check_files "$PRETTIER_LOG")

migrate_oxfmt_if_needed
generate_oxlint_config_if_needed
ensure_svelte_shims

say "running oxlint"
OXLINT_EXIT=0
run_allow_fail "cd '$TARGET' && node '$OXC_ROOT/apps/oxlint/dist/cli.js' -c ./oxlint.config.js ." "$OXLINT_LOG" || OXLINT_EXIT=$?
OXLINT_TIME_SEC="$LAST_RUN_SECONDS"
read -r OXLINT_ERRORS OXLINT_WARNINGS < <(parse_oxlint_counts "$OXLINT_LOG")

say "running oxfmt --check"
OXFMT_EXIT=0
run_allow_fail "cd '$TARGET' && node '$OXC_ROOT/apps/oxfmt/dist/cli.js' --check -c ./.oxfmtrc.json ." "$OXFMT_LOG" || OXFMT_EXIT=$?
OXFMT_TIME_SEC="$LAST_RUN_SECONDS"
OXFMT_FILES=$(parse_oxfmt_check_files "$OXFMT_LOG")

LINT_DELTA_SEC=$((OXLINT_TIME_SEC - ESLINT_TIME_SEC))
FMT_DELTA_SEC=$((OXFMT_TIME_SEC - PRETTIER_TIME_SEC))
LINT_RATIO=$(ratio_or_na "$ESLINT_TIME_SEC" "$OXLINT_TIME_SEC")
FMT_RATIO=$(ratio_or_na "$PRETTIER_TIME_SEC" "$OXFMT_TIME_SEC")

cat > "$SUMMARY_TXT" <<SUM
Target: $TARGET
Profile: $PROFILE_RESOLVED
OXC root: $OXC_ROOT
OXC head: $(git -C "$OXC_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)

Standard tools:
  eslint:   exit=$ESLINT_EXIT time=${ESLINT_TIME_SEC}s errors=$ESLINT_ERRORS warnings=$ESLINT_WARNINGS
  prettier: exit=$PRETTIER_EXIT time=${PRETTIER_TIME_SEC}s files_needing_format=$PRETTIER_FILES

OXC tools:
  oxlint:   exit=$OXLINT_EXIT time=${OXLINT_TIME_SEC}s errors=$OXLINT_ERRORS warnings=$OXLINT_WARNINGS
  oxfmt:    exit=$OXFMT_EXIT time=${OXFMT_TIME_SEC}s files_needing_format=$OXFMT_FILES

Differences (OXC - standard):
  errors_delta=$((OXLINT_ERRORS - ESLINT_ERRORS))
  warnings_delta=$((OXLINT_WARNINGS - ESLINT_WARNINGS))
  format_files_delta=$((OXFMT_FILES - PRETTIER_FILES))
  lint_time_delta_sec=$LINT_DELTA_SEC
  lint_time_ratio=$LINT_RATIO
  format_time_delta_sec=$FMT_DELTA_SEC
  format_time_ratio=$FMT_RATIO

Logs:
  $ESLINT_LOG
  $PRETTIER_LOG
  $OXLINT_LOG
  $OXFMT_LOG
SUM

cat <<BRIEF
Brief summary:
  profile: $PROFILE_RESOLVED
  lint:   eslint ${ESLINT_TIME_SEC}s vs oxlint ${OXLINT_TIME_SEC}s (delta ${LINT_DELTA_SEC}s, ratio ${LINT_RATIO})
  format: prettier ${PRETTIER_TIME_SEC}s vs oxfmt ${OXFMT_TIME_SEC}s (delta ${FMT_DELTA_SEC}s, ratio ${FMT_RATIO})
BRIEF

cat "$SUMMARY_TXT"
