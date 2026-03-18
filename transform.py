#!/usr/bin/env python3
"""
Robust transformation of Expression/Statement enum patterns to tagged pointer patterns.

Categories of transformations:
1. matches!(expr, Expression::Variant(_)) -> expr.is_variant()
2. matches!(expr, Expression::Variant(v) if cond) -> expr.as_variant().is_some_and(|v| cond)
3. let Expression::Variant(x) = expr else { ... } -> let Some(x) = expr.as_variant_mut() else { ... }
4. if let Expression::Variant(x) = &expr -> if let Some(x) = expr.as_variant()
5. Match blocks: match expr { Expression::Variant(x) => } -> match expr.kind() { ExpressionKind::Variant(x) => }
6. Construction: Expression::Variant(alloc) -> Expression::snake(alloc) (outside pattern contexts)
7. Some(Expression::Variant(x)) -> .and_then(|s| s.as_variant())
8. Expression::Variant as function ref -> Expression::snake
"""
import os, re, sys

BASE = "/Users/boshen/oxc/oxc/crates/oxc_minifier/src"

EXPR = {
    'BooleanLiteral': 'boolean_literal',
    'NullLiteral': 'null_literal',
    'NumericLiteral': 'numeric_literal',
    'BigIntLiteral': 'big_int_literal',
    'RegExpLiteral': 'reg_exp_literal',
    'StringLiteral': 'string_literal',
    'TemplateLiteral': 'template_literal',
    'Identifier': 'identifier',
    'MetaProperty': 'meta_property',
    'Super': 'super_expr',
    'ArrayExpression': 'array_expression',
    'ArrowFunctionExpression': 'arrow_function_expression',
    'AssignmentExpression': 'assignment_expression',
    'AwaitExpression': 'await_expression',
    'BinaryExpression': 'binary_expression',
    'CallExpression': 'call_expression',
    'ChainExpression': 'chain_expression',
    'ClassExpression': 'class_expression',
    'ConditionalExpression': 'conditional_expression',
    'FunctionExpression': 'function_expression',
    'ImportExpression': 'import_expression',
    'LogicalExpression': 'logical_expression',
    'NewExpression': 'new_expression',
    'ObjectExpression': 'object_expression',
    'ParenthesizedExpression': 'parenthesized_expression',
    'SequenceExpression': 'sequence_expression',
    'TaggedTemplateExpression': 'tagged_template_expression',
    'ThisExpression': 'this_expression',
    'UnaryExpression': 'unary_expression',
    'UpdateExpression': 'update_expression',
    'YieldExpression': 'yield_expression',
    'PrivateInExpression': 'private_in_expression',
    'JSXElement': 'jsx_element',
    'JSXFragment': 'jsx_fragment',
    'TSAsExpression': 'ts_as_expression',
    'TSSatisfiesExpression': 'ts_satisfies_expression',
    'TSTypeAssertion': 'ts_type_assertion',
    'TSNonNullExpression': 'ts_non_null_expression',
    'TSInstantiationExpression': 'ts_instantiation_expression',
    'V8IntrinsicExpression': 'v8_intrinsic_expression',
    'ComputedMemberExpression': 'computed_member_expression',
    'StaticMemberExpression': 'static_member_expression',
    'PrivateFieldExpression': 'private_field_expression',
}

STMT = {
    'BlockStatement': 'block_statement',
    'BreakStatement': 'break_statement',
    'ContinueStatement': 'continue_statement',
    'DebuggerStatement': 'debugger_statement',
    'DoWhileStatement': 'do_while_statement',
    'EmptyStatement': 'empty_statement',
    'ExpressionStatement': 'expression_statement',
    'ForInStatement': 'for_in_statement',
    'ForOfStatement': 'for_of_statement',
    'ForStatement': 'for_statement',
    'IfStatement': 'if_statement',
    'LabeledStatement': 'labeled_statement',
    'ReturnStatement': 'return_statement',
    'SwitchStatement': 'switch_statement',
    'ThrowStatement': 'throw_statement',
    'TryStatement': 'try_statement',
    'WhileStatement': 'while_statement',
    'WithStatement': 'with_statement',
    'VariableDeclaration': 'variable_declaration',
    'FunctionDeclaration': 'function_declaration',
    'ClassDeclaration': 'class_declaration',
    'TSTypeAliasDeclaration': 'ts_type_alias_declaration',
    'TSInterfaceDeclaration': 'ts_interface_declaration',
    'TSEnumDeclaration': 'ts_enum_declaration',
    'TSModuleDeclaration': 'ts_module_declaration',
    'TSGlobalDeclaration': 'ts_global_declaration',
    'TSImportEqualsDeclaration': 'ts_import_equals_declaration',
    'ImportDeclaration': 'import_declaration',
    'ExportAllDeclaration': 'export_all_declaration',
    'ExportDefaultDeclaration': 'export_default_declaration',
    'ExportNamedDeclaration': 'export_named_declaration',
    'TSExportAssignment': 'ts_export_assignment',
    'TSNamespaceExportDeclaration': 'ts_namespace_export_declaration',
}

ALL_EXPR = set(EXPR.keys())
ALL_STMT = set(STMT.keys())


def get_files():
    result = []
    for root, dirs, files in os.walk(BASE):
        if 'generated' in root:
            continue
        for f in files:
            if f.endswith('.rs'):
                result.append(os.path.join(root, f))
    return sorted(result)


def has_expr_variant(s):
    for v in EXPR:
        if f'Expression::{v}' in s:
            return True
    return False

def has_stmt_variant(s):
    for v in STMT:
        if f'Statement::{v}' in s:
            return True
    return False


def process_file(filepath):
    with open(filepath) as f:
        lines = f.readlines()
    original = ''.join(lines)

    # First pass: identify match blocks that match on Expression/Statement
    # A match block starts with "match EXPR {" and its arms use Expression::/Statement::
    match_blocks = []  # [(header_line_idx, ty, is_mut, arm_line_indices)]

    for i, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith('match ') or not stripped.endswith('{'):
            continue
        # Don't process if this doesn't look like matching on Expression/Statement
        # Look ahead at arms
        ty = None
        is_mut = False
        arm_indices = []

        # Extract match expression
        match_expr_part = stripped[6:].rstrip('{').strip()

        # Check arms for Expression::/Statement:: patterns
        depth = 1
        j = i + 1
        while j < len(lines) and depth > 0:
            arm_stripped = lines[j].strip()
            depth += arm_stripped.count('{') - arm_stripped.count('}')
            if depth <= 0:
                break

            for v in EXPR:
                if f'Expression::{v}' in lines[j] and ('=>' in lines[j] or lines[j].strip().startswith('|') or lines[j].strip().endswith('|')):
                    if ty is None:
                        ty = 'Expression'
                    arm_indices.append(j)
                    break
            for v in STMT:
                if f'Statement::{v}' in lines[j] and ('=>' in lines[j] or lines[j].strip().startswith('|') or lines[j].strip().endswith('|')):
                    if ty is None:
                        ty = 'Statement'
                    arm_indices.append(j)
                    break
            j += 1

        if ty and arm_indices:
            # Determine ref type from match expression
            if match_expr_part.startswith('&mut '):
                is_mut = True
            elif match_expr_part.startswith('&'):
                is_mut = False
            else:
                # Bare expression - need to determine from context
                # Heuristic: if any arm body mutates, it's mut
                # For simplicity, assume non-mut (kind()) unless explicitly &mut
                is_mut = False

            match_blocks.append((i, ty, is_mut, arm_indices))

    # Build set of arm lines for quick lookup
    all_arm_lines = set()
    arm_info = {}  # line_idx -> (ty, is_mut)
    header_lines = {}  # line_idx -> (ty, is_mut)
    for header_idx, ty, is_mut, arm_indices in match_blocks:
        header_lines[header_idx] = (ty, is_mut)
        for arm_idx in arm_indices:
            all_arm_lines.add(arm_idx)
            arm_info[arm_idx] = (ty, is_mut)

    # Second pass: transform lines
    new_lines = []
    for i, line in enumerate(lines):
        stripped = line.strip()

        # Skip comments
        if stripped.startswith('//') or stripped.startswith('///') or stripped.startswith('*') or stripped.startswith('/*'):
            new_lines.append(line)
            continue

        # Transform match headers
        if i in header_lines:
            ty, is_mut = header_lines[i]
            kind_method = 'kind_mut()' if is_mut else 'kind()'
            # Modify the match expression
            s = stripped
            match_expr = s[6:].rstrip('{').strip()
            if match_expr.startswith('&mut '):
                new_match_expr = match_expr[5:].strip() + f'.{kind_method}'
            elif match_expr.startswith('&'):
                new_match_expr = match_expr[1:].strip() + f'.{kind_method}'
            else:
                new_match_expr = match_expr + f'.{kind_method}'
            indent = line[:len(line) - len(line.lstrip())]
            line = f'{indent}match {new_match_expr} {{\n'

        # Transform match arms
        if i in all_arm_lines:
            ty, is_mut = arm_info[i]
            kind_ty = f'{ty}Kind' if not is_mut else f'{ty}KindMut'
            variants = EXPR if ty == 'Expression' else STMT
            for v in variants:
                old = f'{ty}::{v}'
                new = f'{kind_ty}::{v}'
                if old in line:
                    line = line.replace(old, new)

        # Transform non-match patterns (only if not a match arm)
        if i not in all_arm_lines and i not in header_lines:
            line = transform_non_match_patterns(line, stripped)

        new_lines.append(line)

    result = ''.join(new_lines)
    if result != original:
        with open(filepath, 'w') as f:
            f.write(result)
        return True
    return False


def transform_non_match_patterns(line, stripped):
    """Transform non-match patterns: matches!, let-else, if-let, construction, Some()."""

    # Skip if no Expression/Statement variant present
    if not has_expr_variant(line) and not has_stmt_variant(line):
        return line

    for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
        for v, snake in variants.items():
            old = f'{ty}::{v}'
            if old not in line:
                continue

            # === matches! patterns ===
            if 'matches!(' in line and old in line:
                line = transform_matches_pattern(line, ty, v, snake)
                if old not in line:
                    continue

            # === let-else and if-let patterns ===
            if ('let ' in stripped or 'if let ' in stripped or '&& let ' in stripped) and old + '(' in line:
                line = transform_let_pattern(line, ty, v, snake)
                if old not in line:
                    continue

            # === Some(Type::Variant(x)) patterns ===
            if f'Some({old}(' in line:
                line = transform_some_pattern(line, ty, v, snake)
                if old not in line:
                    continue

            # === while let ===
            if 'while let ' in stripped and old + '(' in line:
                line = transform_let_pattern(line, ty, v, snake)
                if old not in line:
                    continue

            # === Tuple destructuring: (Expression::Variant(a), Expression::Variant(b)) ===
            if '(' + old + '(' in line and '), ' in line:
                line = transform_tuple_pattern(line, ty, v, snake)
                if old not in line:
                    continue

            # === Function reference: .map(Statement::VariableDeclaration) ===
            for sep in [')', ',', ';', ' ']:
                ref_pat = old + sep
                if ref_pat in line and '(' + old not in line.replace(ref_pat, ''):
                    # This might be a function reference (no parens after variant name)
                    # Only if there's no ( after the variant name
                    idx = line.index(ref_pat)
                    # Check if this is preceded by map(, .map(, etc
                    before = line[:idx].rstrip()
                    if before.endswith('(') or before.endswith(',') or before.endswith('.map('):
                        line = line.replace(ref_pat, f'{ty}::{snake}{sep}')
                        break

            if old not in line:
                continue

            # === Construction: Expression::Variant(box) -> Expression::snake(box) ===
            if old + '(' in line:
                # Check this is NOT in a pattern context
                if not is_pattern_context(stripped, old):
                    line = line.replace(old + '(', f'{ty}::{snake}(')

    return line


def is_pattern_context(stripped, old):
    """Check if old appears in a pattern matching context."""
    # Pattern contexts:
    # - let X = ...
    # - if let X = ...
    # - match arm (handled separately)
    # - matches!(...)
    idx = stripped.find(old)
    if idx < 0:
        return False
    before = stripped[:idx].rstrip()
    if before.endswith('let') or before.endswith('Some(') or before.endswith('(') and 'let ' in stripped:
        return True
    if 'matches!(' in stripped:
        return True
    return False


def transform_matches_pattern(line, ty, v, snake):
    """Transform matches! patterns."""
    old = f'{ty}::{v}'

    # matches!(expr, Type::Variant(_)) -> expr.is_variant()
    # Handle various expr forms
    pat = f'{old}(_))'
    if pat in line:
        # Find matches!( before it
        idx = line.find('matches!(')
        if idx >= 0:
            after_open = line[idx + 9:]
            comma_idx = after_open.find(',')
            if comma_idx >= 0:
                expr_part = after_open[:comma_idx].strip()
                # Remove & prefix if present
                if expr_part.startswith('&'):
                    expr_part = expr_part[1:]
                rest_after = line[line.find(pat) + len(pat):]
                line = line[:idx] + f'{expr_part}.is_{snake}()' + rest_after
                return line

    # matches!(expr, Type::Variant(_) | Type2::Variant2(_)) - multi-variant
    # These need special handling - convert each one to is_variant() with ||
    # Skip for now, handle manually

    # matches!(expr, Type::Variant(v) if cond) -> expr.as_variant().is_some_and(|v| cond)
    if old + '(' in line and ' if ' in line and 'matches!(' in line:
        idx = line.find('matches!(')
        if idx >= 0:
            after_open = line[idx + 9:]
            comma_idx = after_open.find(',')
            if comma_idx >= 0:
                expr_part = after_open[:comma_idx].strip()
                if expr_part.startswith('&'):
                    expr_part = expr_part[1:]
                # Find the binding and guard
                rest = after_open[comma_idx + 1:].strip()
                # rest should be like: Type::Variant(var) if cond)
                var_start = rest.find(old + '(')
                if var_start >= 0:
                    inner_start = var_start + len(old) + 1
                    inner_end = rest.find(')', inner_start)
                    if inner_end >= 0:
                        binding = rest[inner_start:inner_end]
                        guard_start = rest.find(' if ', inner_end)
                        if guard_start >= 0:
                            # Find matching closing paren
                            guard_content = rest[guard_start + 4:]
                            # Remove trailing )
                            if guard_content.rstrip().endswith(')'):
                                guard_content = guard_content.rstrip()[:-1]
                            rest_after_matches = line[line.rfind(')') + 1:]
                            line = line[:idx] + f'{expr_part}.as_{snake}().is_some_and(|{binding}| {guard_content})' + rest_after_matches
                            return line

    # matches!(expr, Some(Type::Variant(v)) if cond) -> ...
    if f'Some({old}' in line and 'matches!(' in line:
        idx = line.find('matches!(')
        if idx >= 0:
            after_open = line[idx + 9:]
            comma_idx = after_open.find(',')
            if comma_idx >= 0:
                expr_part = after_open[:comma_idx].strip()
                if expr_part.startswith('&'):
                    expr_part = expr_part[1:]
                rest = after_open[comma_idx + 1:].strip()
                # Some(Type::Variant(binding)) if guard
                some_pat = f'Some({old}('
                some_idx = rest.find(some_pat)
                if some_idx >= 0:
                    inner_start = some_idx + len(some_pat)
                    # Find matching ))
                    depth = 2
                    i = inner_start
                    while i < len(rest) and depth > 0:
                        if rest[i] == '(':
                            depth += 1
                        elif rest[i] == ')':
                            depth -= 1
                        i += 1
                    binding = rest[inner_start:i-2]  # -2 for ))
                    after_close = rest[i:]
                    if after_close.strip().startswith('if '):
                        guard = after_close.strip()[3:].rstrip(')')
                        rest_after_matches = line[line.rfind(')') + 1:]
                        line = line[:idx] + f'{expr_part}.as_ref().and_then(|e| e.as_{snake}()).is_some_and(|{binding}| {guard})' + rest_after_matches
                        return line

    return line


def transform_let_pattern(line, ty, v, snake):
    """Transform let/if-let patterns."""
    old = f'{ty}::{v}'
    pat = old + '('

    if pat not in line:
        return line

    idx = line.find(pat)

    # Find the binding
    inner_start = idx + len(pat)
    depth = 1
    i = inner_start
    while i < len(line) and depth > 0:
        if line[i] == '(':
            depth += 1
        elif line[i] == ')':
            depth -= 1
        i += 1
    binding = line[inner_start:i-1]
    after_close = line[i:]
    before = line[:idx]

    # Determine context: is there = before?
    eq_match = before.rstrip()

    # Check for "let Type::Variant(binding) = expr"
    if 'let ' in before:
        # Find = after the pattern
        if after_close.lstrip().startswith('='):
            eq_pos = after_close.index('=')
            rhs = after_close[eq_pos + 1:].lstrip()

            # Determine ref type
            if rhs.startswith('&mut '):
                method = f'as_{snake}_mut()'
                rhs = rhs[5:]
            elif rhs.startswith('&'):
                method = f'as_{snake}()'
                rhs = rhs[1:]
            else:
                # No explicit ref - could be &mut (most common in minifier)
                method = f'as_{snake}_mut()'

            # Split at else { or {
            if ' else {' in rhs:
                expr_part = rhs[:rhs.index(' else {')].strip()
                rest_part = rhs[rhs.index(' else {'):]
            elif rhs.rstrip().endswith('{'):
                expr_part = rhs.rstrip()[:-1].strip()
                rest_part = ' {'
            else:
                expr_part = rhs.strip()
                rest_part = ''

            # Check for 'mut' in binding (owned pattern)
            if binding.startswith('mut '):
                # This is an owned destructure - use into_variant after check
                # But for simplicity in most cases we can use as_variant_mut
                pass

            line = f'{before}Some({binding}) = {expr_part}.{method}{rest_part}\n' if line.endswith('\n') else f'{before}Some({binding}) = {expr_part}.{method}{rest_part}'
            return line

    return line


def transform_some_pattern(line, ty, v, snake):
    """Transform Some(Type::Variant(x)) patterns."""
    old = f'{ty}::{v}'
    pat = f'Some({old}('

    if pat not in line:
        return line

    idx = line.find(pat)
    inner_start = idx + len(pat)
    depth = 1
    i = inner_start
    while i < len(line) and depth > 0:
        if line[i] == '(':
            depth += 1
        elif line[i] == ')':
            depth -= 1
        i += 1
    binding = line[inner_start:i-1]
    # Skip the outer )
    if i < len(line) and line[i] == ')':
        i += 1

    after = line[i:]
    before = line[:idx]

    # Check for = after
    if after.lstrip().startswith('='):
        eq_pos = after.index('=')
        rhs = after[eq_pos + 1:].strip()

        # Determine mut
        is_mut = '_mut()' in rhs or '.pop()' in rhs or '.last_mut()' in rhs
        method = f'as_{snake}_mut' if is_mut else f'as_{snake}'

        if ' else {' in rhs:
            expr_part = rhs[:rhs.index(' else {')].strip()
            rest = rhs[rhs.index(' else {'):]
        elif rhs.rstrip().rstrip('\n').endswith('{'):
            expr_part = rhs.rstrip().rstrip('\n').rstrip('{').strip()
            rest = ' {'
        else:
            expr_part = rhs.strip().rstrip('\n')
            rest = ''

        nl = '\n' if line.endswith('\n') else ''
        line = f'{before}Some({binding}){after[:eq_pos]}= {expr_part}.and_then(|__s| __s.{method}()){rest}{nl}'

    return line


def transform_tuple_pattern(line, ty, v, snake):
    """Transform tuple destructuring patterns like (Expression::Variant(a), Expression::Variant(b))."""
    old = f'{ty}::{v}'
    # Replace Type::Variant(x) with Some(x) and change the = side to use as_variant()
    # This is complex - for now, replace individual parts
    # (Expression::NumericLiteral(left), Expression::NumericLiteral(right)) = (&e.left, &e.right)
    # -> (Some(left), Some(right)) = (e.left.as_numeric_literal(), e.right.as_numeric_literal())
    # But this changes the semantics since now we need both to be Some

    # Actually, the tuple pattern like:
    # if let (Expression::Variant(a), Expression::Variant(b)) = (&e.left, &e.right)
    # should become:
    # if let (Some(a), Some(b)) = (e.left.as_variant(), e.right.as_variant())

    line = line.replace(f'{old}(', f'Some(')
    return line


for f in get_files():
    if process_file(f):
        print(f"Changed: {os.path.relpath(f, BASE)}")

# Count remaining
import subprocess
result = subprocess.run(['grep', '-rn', 'Expression::[A-Z]', BASE, '--include=*.rs'],
                       capture_output=True, text=True)
expr_count = len([l for l in result.stdout.split('\n') if l and 'generated' not in l and '//' not in l.split(':', 2)[2] if len(l.split(':', 2)) > 2])

result = subprocess.run(['grep', '-rn', 'Statement::[A-Z]', BASE, '--include=*.rs'],
                       capture_output=True, text=True)
stmt_count = len([l for l in result.stdout.split('\n') if l and 'generated' not in l and '//' not in l.split(':', 2)[2] if len(l.split(':', 2)) > 2])

print(f"\nRemaining uppercase patterns: Expression={expr_count}, Statement={stmt_count}")
print("Done.")
