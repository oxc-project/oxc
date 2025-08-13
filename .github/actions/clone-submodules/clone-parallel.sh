#!/bin/bash

# Clone submodules in parallel for faster setup
# Usage: ./clone-parallel.sh [test262] [babel] [typescript] [prettier] [acorn-test262]
# Arguments: "true" or "false" for each submodule

set -euo pipefail

# Default values
TEST262=${1:-true}
BABEL=${2:-true}
TYPESCRIPT=${3:-true}
PRETTIER=${4:-true}
ACORN_TEST262=${5:-true}

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
            
            # Clone the repository with minimal progress output
            git clone --quiet --no-progress --single-branch --depth 1 \
                "https://github.com/$repo.git" "$path"
            
            # Checkout the specific commit
            cd "$path"
            git fetch --quiet --depth 1 origin "$ref"
            git checkout --quiet "$ref"
            
            echo "✓ Completed clone of $name"
        ) &
        PIDS+=($!)
    else
        echo "Skipping $name"
    fi
}

echo "Cloning submodules in parallel..."

# Start all clone operations in parallel
clone_repo "$TEST262" "tc39/test262" "tasks/coverage/test262" "4b5d36ab6ef2f59d0a8902cd383762547a3a74c4" "test262"
clone_repo "$BABEL" "babel/babel" "tasks/coverage/babel" "98d18aa4f66ce300a6a863bad223ab67b3fdf282" "babel"
clone_repo "$TYPESCRIPT" "microsoft/TypeScript" "tasks/coverage/typescript" "81c951894e93bdc37c6916f18adcd80de76679bc" "typescript"
clone_repo "$PRETTIER" "prettier/prettier" "tasks/prettier_conformance/prettier" "7584432401a47a26943dd7a9ca9a8e032ead7285" "prettier"
clone_repo "$ACORN_TEST262" "oxc-project/acorn-test262" "tasks/coverage/acorn-test262" "d9ba02ddea22800a285c7ad24e3fbfbb00ccbb02" "acorn-test262"

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