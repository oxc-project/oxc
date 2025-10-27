#!/bin/bash

# Clone submodules in parallel for faster setup
# Usage: ./clone-parallel.sh [test262] [babel] [typescript] [prettier] [acorn-test262]
# Arguments: "true" or "false" for each submodule

set -euo pipefail

# Submodule commit SHAs - updated automatically by .github/workflows/update_submodules.yml
TEST262_SHA="d2940bdbb0e28fd002ec31b89f8182bbf63da092"
BABEL_SHA="4cc3d8888f3e3ac30d38ccfa4557baa7a4a52714"
TYPESCRIPT_SHA="8ea03f88d039759018673f229addb87f579f326c"
PRETTIER_SHA="912342210db02839b9ab7398e50242a16519a8de"
ACORN_TEST262_SHA="994d763f5327046485d985d5596186d3c82f507a"
NODE_COMPAT_TABLE_SHA="17ac85ca289a2d6fea14a991284e631468efe14c"

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
clone_repo "$TEST262" "tc39/test262" "tasks/coverage/test262" "baa48a416c9e9abd698a9010378eccf3d1f4ed1e" "test262"
clone_repo "$BABEL" "babel/babel" "tasks/coverage/babel" "41d96516130ff48f16eca9f387996c0272125f16" "babel"
clone_repo "$TYPESCRIPT" "microsoft/TypeScript" "tasks/coverage/typescript" "261630d650c0c961860187bebc86e25c3707c05d" "typescript"
clone_repo "$PRETTIER" "prettier/prettier" "tasks/prettier_conformance/prettier" "7584432401a47a26943dd7a9ca9a8e032ead7285" "prettier"
clone_repo "$ACORN_TEST262" "oxc-project/acorn-test262" "tasks/coverage/acorn-test262" "090bba4ab63458850b294f55b17f2ca0ee982062" "acorn-test262"
clone_repo "$NODE_COMPAT_TABLE" "williamkapke/node-compat-table" "tasks/coverage/node-compat-table" "ed0d6ba55790519d9ad3f6f776ca2cd303cc1e0b" "node-compat-table"

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
