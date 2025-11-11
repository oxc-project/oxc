#!/bin/bash

# Clone submodules in parallel for faster setup
# Usage: ./clone-parallel.sh [test262] [babel] [typescript] [prettier] [acorn-test262] [node-compat-table]
# Arguments: "true" or "false" for each submodule

set -euo pipefail

# Submodule commit SHAs - updated automatically by .github/workflows/update_submodules.yml
TEST262_SHA="fd594a077a0a018440f241fdd421a5862f1153f5"
BABEL_SHA="777ded79cd97e872ff607e1a4897036f30939188"
TYPESCRIPT_SHA="48244d89f8ccc803fef4a2f0930100de1c77668d"
PRETTIER_SHA="0df10b1d8425442e754cc34eda66ff31cae9aa50"
ACORN_TEST262_SHA="178572a1d4ef764fd7eb8c7791615fd5a6ca4191"
NODE_COMPAT_TABLE_SHA="d17d348a79f78b3000bc67b7c283723ff07bb9d5"

# Default values for which submodules to clone
TEST262=${1:-true}
BABEL=${2:-true}
TYPESCRIPT=${3:-true}
PRETTIER=${4:-true}
ACORN_TEST262=${5:-true}
NODE_COMPAT_TABLE=${6:-true}

# Array to store background process PIDs
declare -a PIDS=()

# Function to clone a repository in the background
clone_repo() {
    local should_clone="$1"
    local repo="$2"
    local path="$3"
    local ref="$4"
    local name="$5"

    if [[ "$should_clone" == "true" ]]; then
        echo "Starting clone of $name..."
        (
            # Create the directory structure if it doesn't exist
            mkdir -p "$(dirname "$path")"

            # Check if directory exists
            if [ -d "$path/.git" ]; then
                # Directory exists with git repo - update it
                cd "$path"
            elif [ -d "$path" ]; then
                # Directory exists but no git repo - initialize it
                cd "$path"
                git init --quiet
            else
                # Directory doesn't exist - clone it
                git clone --quiet --no-progress --single-branch --depth 1 \
                    "https://github.com/$repo.git" "$path"
                cd "$path"
            fi

            # Add or update remote
            if git remote | grep -q "^origin$"; then
                git remote set-url origin "https://github.com/$repo.git"
            else
                git remote add origin "https://github.com/$repo.git"
            fi

            # Fetch and checkout the specific commit
            git fetch --quiet --depth 1 origin "$ref"
            git reset --hard "$ref"
            git clean -f -q

            echo "✓ Completed clone of $name"
        ) &
        PIDS+=($!)
    else
        echo "Skipping $name"
    fi
}

echo "Cloning submodules in parallel..."

# Start all clone operations in parallel
clone_repo "$TEST262" "tc39/test262" "tasks/coverage/test262" "$TEST262_SHA" "test262"
clone_repo "$BABEL" "babel/babel" "tasks/coverage/babel" "$BABEL_SHA" "babel"
clone_repo "$TYPESCRIPT" "microsoft/TypeScript" "tasks/coverage/typescript" "$TYPESCRIPT_SHA" "typescript"
clone_repo "$PRETTIER" "prettier/prettier" "tasks/prettier_conformance/prettier" "$PRETTIER_SHA" "prettier"
clone_repo "$ACORN_TEST262" "oxc-project/acorn-test262" "tasks/coverage/acorn-test262" "$ACORN_TEST262_SHA" "acorn-test262"
clone_repo "$NODE_COMPAT_TABLE" "williamkapke/node-compat-table" "tasks/coverage/node-compat-table" "$NODE_COMPAT_TABLE_SHA" "node-compat-table"

# Wait for all background processes to complete
echo "Waiting for all clone operations to complete..."
failed_count=0
for pid in "${PIDS[@]}"; do
    if ! wait "$pid"; then
        echo "❌ Clone operation failed (PID: $pid)"
        ((failed_count++))
    fi
done

if [[ $failed_count -eq 0 ]]; then
    echo "✅ All submodule clones completed successfully!"
else
    echo "❌ $failed_count clone operation(s) failed"
    exit 1
fi
