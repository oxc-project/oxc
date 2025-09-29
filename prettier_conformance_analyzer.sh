#!/bin/bash

# Prettier Conformance Test Analyzer
# Usage: ./prettier_conformance_analyzer.sh <output_file> <branch_name>

OUTPUT_FILE="${1:-analysis.md}"
BRANCH_NAME="${2:-$(git branch --show-current)}"
SNAPSHOT_DIR="tasks/prettier_conformance/snapshots"

echo "# Prettier Conformance Analysis Report" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "**Branch**: $BRANCH_NAME" >> "$OUTPUT_FILE"
echo "**Generated**: $(date '+%Y-%m-%d %H:%M:%S')" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Function to count tests in a snapshot file
count_tests() {
    local file="$1"
    if [ -f "$file" ]; then
        # Count the number of test entries (lines starting with "source:")
        grep -c "^source:" "$file" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to extract test names from snapshot file
extract_test_names() {
    local file="$1"
    if [ -f "$file" ]; then
        # Extract test names (lines between "source:" and next separator)
        grep -A1 "^source:" "$file" | grep -v "^--$" | grep -v "^source:" | head -20
    fi
}

# Analyze JavaScript files
echo "## JavaScript Conformance" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

JS_SNAPSHOT="$SNAPSHOT_DIR/prettier_javascript.snap.md"
if [ -f "$JS_SNAPSHOT" ]; then
    TOTAL_JS_TESTS=$(count_tests "$JS_SNAPSHOT")
    echo "### Overview" >> "$OUTPUT_FILE"
    echo "- **Failing Tests**: $TOTAL_JS_TESTS" >> "$OUTPUT_FILE"

    # Extract detailed failure information
    echo "" >> "$OUTPUT_FILE"
    echo "### Failing Test Files" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Parse the snapshot to get file-level statistics
    awk '/^source:/ {
        getline file
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", file)
        files[file]++
    }
    END {
        for (file in files) {
            print "- " file ": " files[file] " tests"
        }
    }' "$JS_SNAPSHOT" | sort -t: -k2 -rn | head -50 >> "$OUTPUT_FILE"

    echo "" >> "$OUTPUT_FILE"
    echo "### Sample Failing Test Names" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
    extract_test_names "$JS_SNAPSHOT" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
else
    echo "- **Status**: All JavaScript tests passing! ✅" >> "$OUTPUT_FILE"
fi

echo "" >> "$OUTPUT_FILE"

# Analyze TypeScript files
echo "## TypeScript Conformance" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

TS_SNAPSHOT="$SNAPSHOT_DIR/prettier_typescript.snap.md"
if [ -f "$TS_SNAPSHOT" ]; then
    TOTAL_TS_TESTS=$(count_tests "$TS_SNAPSHOT")
    echo "### Overview" >> "$OUTPUT_FILE"
    echo "- **Failing Tests**: $TOTAL_TS_TESTS" >> "$OUTPUT_FILE"

    # Extract detailed failure information
    echo "" >> "$OUTPUT_FILE"
    echo "### Failing Test Files" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Parse the snapshot to get file-level statistics
    awk '/^source:/ {
        getline file
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", file)
        files[file]++
    }
    END {
        for (file in files) {
            print "- " file ": " files[file] " tests"
        }
    }' "$TS_SNAPSHOT" | sort -t: -k2 -rn | head -50 >> "$OUTPUT_FILE"

    echo "" >> "$OUTPUT_FILE"
    echo "### Sample Failing Test Names" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
    extract_test_names "$TS_SNAPSHOT" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
else
    echo "- **Status**: All TypeScript tests passing! ✅" >> "$OUTPUT_FILE"
fi

echo "" >> "$OUTPUT_FILE"

# Summary statistics
echo "## Summary Statistics" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Get pass rates from the test output if available
LAST_RUN_LOG="prettier_conformance_output.log"
if [ -f "$LAST_RUN_LOG" ]; then
    echo "### Test Pass Rates" >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
    grep -E "(Passed|Failed|prettier_javascript|prettier_typescript)" "$LAST_RUN_LOG" | tail -10 >> "$OUTPUT_FILE"
    echo "\`\`\`" >> "$OUTPUT_FILE"
else
    echo "Run \`cargo run -p oxc_prettier_conformance 2>&1 | tee prettier_conformance_output.log\` to capture pass rates" >> "$OUTPUT_FILE"
fi

echo "" >> "$OUTPUT_FILE"
echo "---" >> "$OUTPUT_FILE"
echo "_Report generated for branch: $BRANCH_NAME on $(date)_" >> "$OUTPUT_FILE"

echo "Analysis complete! Report saved to: $OUTPUT_FILE"