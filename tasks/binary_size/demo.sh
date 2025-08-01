#!/usr/bin/env bash

# Binary Size Optimization Script
# This script demonstrates how to use the binary size analysis tool to
# identify and implement size optimizations

set -euo pipefail

echo "🔍 Oxc Binary Size Optimization Demo"
echo "====================================="
echo

# Function to run analysis and extract key metrics
analyze_size() {
    local features="$1"
    local description="$2"
    
    echo "📊 Analyzing: $description"
    echo "Features: ${features:-none}"
    echo
    
    if [ -n "$features" ]; then
        cargo run -p oxc_binary_size -- --target oxlint --features "$features" --json > /tmp/analysis.json
    else
        cargo run -p oxc_binary_size -- --target oxlint --json > /tmp/analysis.json
    fi
    
    # Extract key metrics using jq (if available) or parse JSON manually
    if command -v jq &> /dev/null; then
        local file_size=$(jq -r '.file_size' /tmp/analysis.json)
        local stripped_size=$(jq -r '.stripped_size' /tmp/analysis.json)
        local text_size=$(jq -r '.text_section_size' /tmp/analysis.json)
        
        echo "  File size: $(numfmt --to=iec $file_size)"
        echo "  Stripped: $(numfmt --to=iec $stripped_size)"  
        echo "  Text section: $(numfmt --to=iec $text_size)"
    else
        echo "  (Install jq for detailed metrics)"
        ls -lh target/release/oxlint | awk '{print "  File size: " $5}'
    fi
    echo
}

# Baseline analysis
echo "1️⃣ Baseline Analysis (no features)"
analyze_size "" "Minimal build"

# Feature impact analysis  
echo "2️⃣ Feature Impact Analysis"
analyze_size "allocator" "With allocator feature"

# Compare builds with different optimization levels
echo "3️⃣ Optimization Level Comparison"

echo "Building with opt-level='z' (size optimization)..."
RUSTFLAGS="-C opt-level=z -C debuginfo=2 -C strip=none" cargo build --release -p oxlint --features allocator --quiet
echo "  Size-optimized build:"
ls -lh target/release/oxlint | awk '{print "  File size: " $5}'

echo "Building with opt-level='3' (speed optimization)..."  
RUSTFLAGS="-C opt-level=3 -C debuginfo=2 -C strip=none" cargo build --release -p oxlint --features allocator --quiet
echo "  Speed-optimized build:"
ls -lh target/release/oxlint | awk '{print "  File size: " $5}'

# Demonstrate stripped vs unstripped
echo "4️⃣ Debug Symbol Impact"
echo "With debug symbols:"
ls -lh target/release/oxlint | awk '{print "  " $5}'

echo "Stripping debug symbols..."
cp target/release/oxlint /tmp/oxlint-stripped
strip /tmp/oxlint-stripped
echo "After stripping:"
ls -lh /tmp/oxlint-stripped | awk '{print "  " $5}'

# Show practical size reduction recommendations
echo "5️⃣ Size Reduction Recommendations"
echo "Based on analysis, here are the most effective optimizations:"
echo
echo "🎯 Production Build (immediate ~92% reduction):"
echo "   RUSTFLAGS='-C strip=symbols' cargo build --release -p oxlint --features allocator"
echo
echo "🎯 Size-Optimized Build:"
echo "   RUSTFLAGS='-C opt-level=z -C strip=symbols' cargo build --release -p oxlint --features allocator"
echo
echo "🎯 Minimal Feature Build:"
echo "   cargo build --release -p oxlint  # No features"
echo
echo "🎯 Advanced Optimization (requires nightly):"
echo "   cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu --release -p oxlint"
echo

# Show monitoring commands
echo "6️⃣ Ongoing Monitoring"
echo "Use these commands to track size over time:"
echo
echo "  just binary-size-baseline      # Save current as baseline"
echo "  just binary-size-compare       # Compare with baseline"  
echo "  just binary-size-detailed      # Full analysis report"
echo

echo "✅ Binary size analysis complete!"
echo "See tasks/binary_size/ANALYSIS_REPORT.md for detailed findings and strategies."