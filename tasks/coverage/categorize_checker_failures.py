#!/usr/bin/env python3
"""
Comprehensive analysis of checker_typescript conformance test failures.

Produces:
1. Mismatch kind distribution (from actual checker runs)
2. Actionable root causes ranked by test count
3. Feature enrichment analysis (failing vs passing tests)

Prerequisites:
  Run the dump_checker_mismatches example first:
    cargo run --release -p oxc_coverage --example dump_checker_mismatches > /tmp/mismatches.tsv

Usage:
  python3 tasks/coverage/categorize_checker_failures.py
"""

import re
import os
import sys
import math
from collections import Counter

WORKSPACE = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TSV_FILE = "/tmp/mismatches.tsv"
SNAP_FILE = os.path.join(WORKSPACE, "tasks/coverage/snapshots/checker_typescript.snap")

# ============================================================
# Part 1: Load mismatch data from dump_checker_mismatches
# ============================================================

rows = []
if os.path.exists(TSV_FILE):
    with open(TSV_FILE, 'rb') as f:
        content = f.read().replace(b'\x00', b'').decode('utf-8', errors='replace')
    for line in content.split('\n'):
        line = line.strip()
        if not line or line.startswith('file\t'):
            continue
        parts = line.split('\t')
        if len(parts) >= 5:
            rows.append({
                'file': parts[0],
                'expr': parts[1],
                'actual': parts[2],
                'expected': parts[3],
                'kind': parts[4],
            })

if not rows:
    print("ERROR: No mismatch data found. Run the dumper first:")
    print("  cargo run --release -p oxc_coverage --example dump_checker_mismatches > /tmp/mismatches.tsv")
    sys.exit(1)

total = len(rows)
print(f"Total failing tests with mismatch data: {total}")
print()

# ============================================================
# Part 2: Actionable Root Cause Analysis
# ============================================================

actionable = Counter()
for r in rows:
    actual = r['actual']
    expected = r['expected']
    kind = r['kind']
    expr = r['expr']

    if kind == 'parse_error':
        actionable['Parse errors (test directives/unsupported syntax)'] += 1
        continue

    if kind == 'missing_expression':
        if '"use strict"' in expr:
            actionable['Walker: directive prologue ("use strict")'] += 1
        elif '...' in expr:
            actionable['Walker: spread element'] += 1
        elif len(expr) > 80:
            actionable['Walker: complex/multiline expression'] += 1
        elif '.' in expr and '(' not in expr:
            actionable['Walker: member expression'] += 1
        elif re.match(r'^[a-zA-Z_]\w*$', expr):
            actionable['Walker: simple identifier not emitted'] += 1
        elif '(' in expr:
            actionable['Walker: call/paren expression'] += 1
        else:
            actionable['Walker: other missing expression'] += 1
        continue

    if kind == 'got_any':
        if expected.startswith('typeof '):
            actionable['Namespace/module: typeof binding not resolved'] += 1
        elif 'this' in expr:
            actionable['this type resolution'] += 1
        elif 'Symbol' in expr or expected == 'unique symbol':
            actionable['Symbol/unique symbol support'] += 1
        elif '(' in expr:
            actionable['Call expression return type (any)'] += 1
        elif '.' in expr:
            actionable['Member access type resolution (any)'] += 1
        elif '[' in expr:
            actionable['Element access type resolution (any)'] += 1
        elif expected.startswith('typeof '):
            actionable['Namespace/module: typeof binding not resolved'] += 1
        else:
            actionable['Variable type: got any for declared binding'] += 1
        continue

    if kind in ('missing_typeof', 'missing_union', 'missing_intersection', 'missing_function_type'):
        actionable[f'Type construction: {kind.replace("missing_", "")}'] += 1
        continue

    if kind == 'widened_literal':
        actionable['Literal widening: should keep literal in const ctx'] += 1
        continue

    if kind == 'got_never':
        actionable['Control flow: incorrect narrowing to never'] += 1
        continue

    if kind == 'got_error':
        actionable['Error type in unexpected place'] += 1
        continue

    # type_mismatch: the big bucket
    if kind == 'type_mismatch':
        # typeof confusion
        if actual.startswith('typeof ') and not expected.startswith('typeof '):
            if expected == actual.replace('typeof ', ''):
                actionable['Class binding: typeof X vs X (constructor vs instance)'] += 1
            else:
                actionable['typeof on value but expected different type'] += 1
            continue
        if not actual.startswith('typeof ') and expected.startswith('typeof '):
            actionable['Missing typeof for namespace/class binding'] += 1
            continue

        # Literal widening
        if re.match(r'^-?\d+(\.\d+)?$', actual) and expected == 'number':
            actionable['Literal widening: num literal should be number'] += 1
            continue
        if (actual.startswith('"') or actual.startswith("'")) and expected == 'string':
            actionable['Literal widening: str literal should be string'] += 1
            continue
        if actual in ('true', 'false') and expected == 'boolean':
            actionable['Literal widening: bool literal should be boolean'] += 1
            continue

        # Reverse widening (narrowing context not applied)
        if actual == 'number' and re.match(r'^-?\d+$', expected):
            actionable['Missing narrowing: number should be numeric literal'] += 1
            continue
        if actual == 'string' and (expected.startswith('"') or expected.startswith("'")):
            actionable['Missing narrowing: string should be string literal'] += 1
            continue

        # Function signatures
        if '=>' in actual and '=>' in expected:
            act_parts = actual.rsplit('=>', 1)
            exp_parts = expected.rsplit('=>', 1)
            if len(act_parts) == 2 and len(exp_parts) == 2:
                act_ret = act_parts[1].strip()
                exp_ret = exp_parts[1].strip()
                act_params = act_parts[0].strip()
                exp_params = exp_parts[0].strip()
                if act_params != exp_params and act_ret == exp_ret:
                    actionable['Function params wrong (return type correct)'] += 1
                elif act_params == exp_params and act_ret != exp_ret:
                    if act_ret == 'any':
                        actionable['Function return type: any -> correct'] += 1
                    elif act_ret == 'void':
                        actionable['Function return type: void -> correct'] += 1
                    else:
                        actionable['Function return type: wrong'] += 1
                else:
                    actionable['Function sig: both params & return differ'] += 1
            else:
                actionable['Function sig: complex mismatch'] += 1
            continue

        # Union issues
        if '|' in actual and '|' not in expected:
            actionable['Union: producing spurious union'] += 1
            continue
        if '|' not in actual and '|' in expected:
            actionable['Union: not producing expected union'] += 1
            continue
        if '|' in actual and '|' in expected:
            actionable['Union: different union types'] += 1
            continue

        # Object types
        if '{' in actual and '{' in expected:
            actionable['Object type: wrong shape/properties'] += 1
            continue

        # Enum member types
        if re.match(r'^-?\d+$', actual) and '.' in expected:
            actionable['Enum member: got numeric literal instead of enum value'] += 1
            continue
        if (actual.startswith('"') or actual.startswith("'")) and '.' in expected:
            actionable['Enum member: got string literal instead of enum value'] += 1
            continue

        # Everything else
        actionable['Other type computation error'] += 1

# Print actionable root causes
print("=" * 95)
print("ACTIONABLE ROOT CAUSES (ranked by number of failing tests)")
print("=" * 95)
print(f"{'Rank':<5}{'Root Cause':<68}{'Tests':>7}{'%':>7}")
print("-" * 87)
for rank, (cause, count) in enumerate(actionable.most_common(40), 1):
    pct = 100.0 * count / total
    print(f"{rank:<5}{cause:<68}{count:>7}{pct:>6.1f}%")

# Grouped summary
print()
print("=" * 95)
print("GROUPED ROOT CAUSES (actionable implementation areas)")
print("=" * 95)

groups = {
    'Namespace/module typeof': [],
    'AST Walker gaps': [],
    'Literal widening/narrowing': [],
    'Function signatures': [],
    'typeof vs value type': [],
    'Union type construction': [],
    'Object type shape': [],
    'Variable resolution (any)': [],
    'Enum members': [],
    'Control flow': [],
    'Other': [],
}

for cause, count in actionable.items():
    if 'Namespace/module' in cause or 'namespace' in cause.lower():
        groups['Namespace/module typeof'].append((cause, count))
    elif 'Walker' in cause:
        groups['AST Walker gaps'].append((cause, count))
    elif 'Literal' in cause or 'literal' in cause or 'widening' in cause or 'narrowing' in cause:
        groups['Literal widening/narrowing'].append((cause, count))
    elif 'Function' in cause or 'function' in cause:
        groups['Function signatures'].append((cause, count))
    elif 'typeof' in cause.lower() or 'Class binding' in cause:
        groups['typeof vs value type'].append((cause, count))
    elif 'Union' in cause or 'union' in cause:
        groups['Union type construction'].append((cause, count))
    elif 'Object' in cause:
        groups['Object type shape'].append((cause, count))
    elif 'Variable' in cause or 'Member' in cause or 'Call' in cause or 'Element' in cause or 'this' in cause or 'Symbol' in cause:
        groups['Variable resolution (any)'].append((cause, count))
    elif 'Enum' in cause:
        groups['Enum members'].append((cause, count))
    elif 'Control' in cause or 'never' in cause:
        groups['Control flow'].append((cause, count))
    else:
        groups['Other'].append((cause, count))

group_totals = []
for group, items in groups.items():
    total_count = sum(c for _, c in items)
    if total_count > 0:
        group_totals.append((group, total_count, items))

group_totals.sort(key=lambda x: -x[1])

print(f"{'Group':<40}{'Total':>8}{'% of All':>10}")
print("-" * 58)
for group, total_count, items in group_totals:
    pct = 100.0 * total_count / total
    print(f"{group:<40}{total_count:>8}{pct:>9.1f}%")
    for cause, count in sorted(items, key=lambda x: -x[1]):
        print(f"    {cause:<55}{count:>5}")

# Cumulative impact
print()
print("=" * 95)
print("CUMULATIVE IMPACT (if groups fixed in priority order)")
print("=" * 95)
cumulative = 0
print(f"{'Priority':<5}{'Group':<50}{'Added':>7}{'Cumul':>7}{'% Fixed':>9}")
print("-" * 78)
for priority, (group, total_count, _) in enumerate(group_totals, 1):
    cumulative += total_count
    pct = 100.0 * cumulative / total
    print(f"{priority:<5}{group:<50}{total_count:>7}{cumulative:>7}{pct:>8.1f}%")
