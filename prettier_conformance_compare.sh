#!/bin/bash

# Prettier Conformance Comparison Script
# Usage: ./prettier_conformance_compare.sh <dev_analysis.md> <main_analysis.md> <output.md>

DEV_ANALYSIS="${1:-dev_analysis.md}"
MAIN_ANALYSIS="${2:-main_analysis.md}"
OUTPUT="${3:-comparison_report.md}"

echo "# Prettier Conformance Comparison Report" > "$OUTPUT"
echo "" >> "$OUTPUT"
echo "**Generated**: $(date '+%Y-%m-%d %H:%M:%S')" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Extract key metrics from analysis files
extract_failing_count() {
    local file="$1"
    local type="$2"
    grep -A1 "## $type Conformance" "$file" | grep "Failing Tests:" | sed 's/.*Failing Tests: *//' | sed 's/[^0-9]//g' || echo "0"
}

# Get metrics
DEV_JS_FAIL=$(extract_failing_count "$DEV_ANALYSIS" "JavaScript")
MAIN_JS_FAIL=$(extract_failing_count "$MAIN_ANALYSIS" "JavaScript")
DEV_TS_FAIL=$(extract_failing_count "$DEV_ANALYSIS" "TypeScript")
MAIN_TS_FAIL=$(extract_failing_count "$MAIN_ANALYSIS" "TypeScript")

# Executive Summary
echo "## Executive Summary" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Calculate differences
JS_DIFF=$((DEV_JS_FAIL - MAIN_JS_FAIL))
TS_DIFF=$((DEV_TS_FAIL - MAIN_TS_FAIL))

# Check for regressions
REGRESSION_FOUND=false
if [ $JS_DIFF -gt 0 ] || [ $TS_DIFF -gt 0 ]; then
    REGRESSION_FOUND=true
    echo "### 🚨 REGRESSION ALERT 🚨" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

echo "### Overall Statistics" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "| Test Suite | Dev Branch | Main Branch | Difference | Status |" >> "$OUTPUT"
echo "|------------|------------|-------------|------------|--------|" >> "$OUTPUT"

# JavaScript row
if [ $JS_DIFF -gt 0 ]; then
    JS_STATUS="🔴 Regression"
elif [ $JS_DIFF -lt 0 ]; then
    JS_STATUS="✅ Improved"
else
    JS_STATUS="➖ No Change"
fi
echo "| JavaScript | $DEV_JS_FAIL failing | $MAIN_JS_FAIL failing | $JS_DIFF | $JS_STATUS |" >> "$OUTPUT"

# TypeScript row
if [ $TS_DIFF -gt 0 ]; then
    TS_STATUS="🔴 Regression"
elif [ $TS_DIFF -lt 0 ]; then
    TS_STATUS="✅ Improved"
else
    TS_STATUS="➖ No Change"
fi
echo "| TypeScript | $DEV_TS_FAIL failing | $MAIN_TS_FAIL failing | $TS_DIFF | $TS_STATUS |" >> "$OUTPUT"

echo "" >> "$OUTPUT"

# Extract test file lists for comparison
extract_test_files() {
    local file="$1"
    local type="$2"
    awk "/## $type Conformance/,/## [A-Z]/" "$file" | grep "^- " | sed 's/^- //' | cut -d: -f1 | sort
}

# Compare JavaScript tests
echo "## JavaScript Test Comparison" >> "$OUTPUT"
echo "" >> "$OUTPUT"

DEV_JS_FILES=$(extract_test_files "$DEV_ANALYSIS" "JavaScript" | sort | uniq)
MAIN_JS_FILES=$(extract_test_files "$MAIN_ANALYSIS" "JavaScript" | sort | uniq)

if [ -n "$DEV_JS_FILES" ] || [ -n "$MAIN_JS_FILES" ]; then
    # Find new failures (regressions)
    echo "### Tests Failing Only in Development Branch (Regressions)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"

    comm -23 <(echo "$DEV_JS_FILES") <(echo "$MAIN_JS_FILES") > /tmp/js_regressions.txt
    if [ -s /tmp/js_regressions.txt ]; then
        echo "**⚠️ The following JavaScript tests are failing in dev but passing in main:**" >> "$OUTPUT"
        echo "" >> "$OUTPUT"
        while IFS= read -r file; do
            echo "- $file" >> "$OUTPUT"
        done < /tmp/js_regressions.txt
    else
        echo "_No JavaScript regressions found_ ✅" >> "$OUTPUT"
    fi
    echo "" >> "$OUTPUT"

    # Find fixed tests (improvements)
    echo "### Tests Fixed in Development Branch (Improvements)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"

    comm -13 <(echo "$DEV_JS_FILES") <(echo "$MAIN_JS_FILES") > /tmp/js_improvements.txt
    if [ -s /tmp/js_improvements.txt ]; then
        echo "**🎉 The following JavaScript tests were fixed in dev:**" >> "$OUTPUT"
        echo "" >> "$OUTPUT"
        while IFS= read -r file; do
            echo "- $file" >> "$OUTPUT"
        done < /tmp/js_improvements.txt
    else
        echo "_No JavaScript improvements found_" >> "$OUTPUT"
    fi
else
    echo "_All JavaScript tests passing in both branches!_ ✅" >> "$OUTPUT"
fi

echo "" >> "$OUTPUT"

# Compare TypeScript tests
echo "## TypeScript Test Comparison" >> "$OUTPUT"
echo "" >> "$OUTPUT"

DEV_TS_FILES=$(extract_test_files "$DEV_ANALYSIS" "TypeScript" | sort | uniq)
MAIN_TS_FILES=$(extract_test_files "$MAIN_ANALYSIS" "TypeScript" | sort | uniq)

if [ -n "$DEV_TS_FILES" ] || [ -n "$MAIN_TS_FILES" ]; then
    # Find new failures (regressions)
    echo "### Tests Failing Only in Development Branch (Regressions)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"

    comm -23 <(echo "$DEV_TS_FILES") <(echo "$MAIN_TS_FILES") > /tmp/ts_regressions.txt
    if [ -s /tmp/ts_regressions.txt ]; then
        echo "**⚠️ The following TypeScript tests are failing in dev but passing in main:**" >> "$OUTPUT"
        echo "" >> "$OUTPUT"
        while IFS= read -r file; do
            echo "- $file" >> "$OUTPUT"
        done < /tmp/ts_regressions.txt
    else
        echo "_No TypeScript regressions found_ ✅" >> "$OUTPUT"
    fi
    echo "" >> "$OUTPUT"

    # Find fixed tests (improvements)
    echo "### Tests Fixed in Development Branch (Improvements)" >> "$OUTPUT"
    echo "" >> "$OUTPUT"

    comm -13 <(echo "$DEV_TS_FILES") <(echo "$MAIN_TS_FILES") > /tmp/ts_improvements.txt
    if [ -s /tmp/ts_improvements.txt ]; then
        echo "**🎉 The following TypeScript tests were fixed in dev:**" >> "$OUTPUT"
        echo "" >> "$OUTPUT"
        while IFS= read -r file; do
            echo "- $file" >> "$OUTPUT"
        done < /tmp/ts_improvements.txt
    else
        echo "_No TypeScript improvements found_" >> "$OUTPUT"
    fi
else
    echo "_All TypeScript tests passing in both branches!_ ✅" >> "$OUTPUT"
fi

echo "" >> "$OUTPUT"

# Action Items
echo "## Recommended Action Items" >> "$OUTPUT"
echo "" >> "$OUTPUT"

if [ "$REGRESSION_FOUND" = true ]; then
    echo "### 🚨 Priority 1: Fix Regressions" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "**CRITICAL**: The following regressions must be addressed before merging:" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    if [ $JS_DIFF -gt 0 ]; then
        echo "- Fix $JS_DIFF JavaScript test regressions" >> "$OUTPUT"
    fi
    if [ $TS_DIFF -gt 0 ]; then
        echo "- Fix $TS_DIFF TypeScript test regressions" >> "$OUTPUT"
    fi
    echo "" >> "$OUTPUT"
    echo "Review the regression lists above for specific files to investigate." >> "$OUTPUT"
else
    echo "### ✅ No Regressions Detected" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "Safe to proceed with improvements. Consider:" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "1. Review any fixed tests to ensure they're properly documented" >> "$OUTPUT"
    echo "2. Target remaining failures for future improvements" >> "$OUTPUT"
    echo "3. Update PR description with improvement metrics" >> "$OUTPUT"
fi

echo "" >> "$OUTPUT"
echo "## Analysis Files" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "- Development Branch Analysis: \`$DEV_ANALYSIS\`" >> "$OUTPUT"
echo "- Main Branch Analysis: \`$MAIN_ANALYSIS\`" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "---" >> "$OUTPUT"
echo "_Comparison generated on $(date)_" >> "$OUTPUT"

echo "Comparison complete! Report saved to: $OUTPUT"

# Clean up temp files
rm -f /tmp/js_regressions.txt /tmp/js_improvements.txt /tmp/ts_regressions.txt /tmp/ts_improvements.txt