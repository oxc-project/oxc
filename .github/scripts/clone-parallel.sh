#!/bin/bash

# Clone submodules in parallel for faster setup
# Usage: ./clone-parallel.sh [test262] [babel] [typescript] [prettier] [estree-conformance] [node-compat-table]
# Arguments: "true" or "false" for each submodule

set -euo pipefail

# Submodule commit SHAs - updated automatically by .github/workflows/update_submodules.yml
# NOTE: Prettier version is now pinned to v3.7.3 (not updated by workflow above), Update manually as needed
TEST262_SHA="947fee33f81e261afd4fc6020b2a1d3ac23efa60"
BABEL_SHA="84d21e4e129468b62ca5e05f8029c18d785f3345"
TYPESCRIPT_SHA="0a071327153b4c386dfcab19a584e0d6224d1354"
PRETTIER_SHA="fdfa6701767f5140a85902ecc9fb6444f5b4e3f8"
ESTREE_CONFORMANCE_SHA="e0aa1b46e2da9b30fb86d429166f6ea4b61999ec"
NODE_COMPAT_TABLE_SHA="499beb6f1daa36f10c26b85a7f3ec3b3448ded23"

# Default values for which submodules to clone
TEST262=${1:-true}
BABEL=${2:-true}
TYPESCRIPT=${3:-true}
PRETTIER=${4:-true}
ESTREE_CONFORMANCE=${5:-true}
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
clone_repo "$ESTREE_CONFORMANCE" "oxc-project/estree-conformance" "tasks/coverage/estree-conformance" "$ESTREE_CONFORMANCE_SHA" "estree-conformance"
clone_repo "$NODE_COMPAT_TABLE" "compat-table/node-compat-table" "tasks/coverage/node-compat-table" "$NODE_COMPAT_TABLE_SHA" "node-compat-table"

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
