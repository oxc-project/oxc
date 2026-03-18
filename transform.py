#!/usr/bin/env python3
"""
Final transformation script. Handles:
1. matches! patterns
2. let-else / if-let patterns
3. match blocks (arms + headers)
4. construction patterns
5. Some() patterns
6. Function reference patterns

For owned match blocks, marks them with TODO for manual fixing.
"""
import os, sys

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


def get_files():
    r = []
    for root, dirs, files in os.walk(BASE):
        if 'generated' in root: continue
        for f in files:
            if f.endswith('.rs'): r.append(os.path.join(root, f))
    return sorted(r)


def has_variant(line, ty='both'):
    """Check if line has Expression:: or Statement:: variants."""
    types = []
    if ty in ('both', 'Expression'): types.append(('Expression', EXPR))
    if ty in ('both', 'Statement'): types.append(('Statement', STMT))
    for t, variants in types:
        for v in variants:
            if f'{t}::{v}' in line:
                return True
    return False


def which_type(line):
    """Return 'Expression' or 'Statement' based on which is in the line."""
    for v in EXPR:
        if f'Expression::{v}' in line: return 'Expression'
    for v in STMT:
        if f'Statement::{v}' in line: return 'Statement'
    return None


def find_match_blocks(lines):
    """Find match blocks and categorize them.
    Returns: dict of {header_idx: (ty, ref_type, arm_indices)}
    where ref_type is 'ref', 'mut', or 'owned'
    """
    blocks = {}
    for i, line in enumerate(lines):
        s = line.strip()
        if not s.startswith('match ') or not s.endswith('{'):
            continue

        match_expr = s[6:].rstrip('{').strip()

        # Look ahead to find arms with Expression/Statement variants
        ty = None
        arm_indices = []
        depth = 1
        j = i + 1
        while j < len(lines) and depth > 0:
            js = lines[j].strip()
            # Track brace depth
            for ch in js:
                if ch == '{': depth += 1
                elif ch == '}': depth -= 1
            if depth <= 0: break

            # Check if this line is a match arm with our variant
            line_ty = which_type(lines[j])
            if line_ty and ('=>' in lines[j] or js.startswith('| ') or js.endswith('|')):
                if ty is None: ty = line_ty
                arm_indices.append(j)
            j += 1

        if not ty or not arm_indices:
            continue

        # Determine ref type
        if match_expr.startswith('&mut '):
            ref_type = 'mut'
        elif match_expr.startswith('&'):
            ref_type = 'ref'
        else:
            # Need to determine from context
            # If the match expression involves .take_in(), .pop(), it's owned
            if '.take_in(' in match_expr or '.pop()' in match_expr:
                ref_type = 'owned'
            else:
                # Assume the variable is &mut (most common in this codebase)
                ref_type = 'mut'

        blocks[i] = (ty, ref_type, arm_indices)

    return blocks


def transform_match_header(line, ty, ref_type, match_expr):
    """Transform match header line."""
    indent = line[:len(line) - len(line.lstrip())]

    if ref_type == 'mut':
        if match_expr.startswith('&mut '):
            expr = match_expr[5:].strip()
        else:
            expr = match_expr
        return f'{indent}match {expr}.kind_mut() {{\n'
    elif ref_type == 'ref':
        if match_expr.startswith('&'):
            expr = match_expr[1:].strip()
        else:
            expr = match_expr
        return f'{indent}match {expr}.kind() {{\n'
    else:  # owned
        # For owned, we use .kind() but note this borrows
        return f'{indent}match {match_expr}.kind() {{\n'


def transform_match_arm(line, ty, ref_type):
    """Transform match arm: replace Type::Variant with TypeKind[Mut]::Variant."""
    variants = EXPR if ty == 'Expression' else STMT
    if ref_type == 'mut':
        kind_ty = f'{ty}KindMut'
    else:
        kind_ty = f'{ty}Kind'

    for v in variants:
        old = f'{ty}::{v}'
        new = f'{kind_ty}::{v}'
        if old in line:
            line = line.replace(old, new)
    return line


def process_file(filepath):
    with open(filepath) as f:
        lines = f.readlines()
    original = ''.join(lines)

    # Step 1: Find all match blocks
    match_blocks = find_match_blocks(lines)

    # Build lookup tables
    arm_lines = {}  # line_idx -> (ty, ref_type)
    header_lines = {}  # line_idx -> (ty, ref_type, match_expr)
    for h_idx, (ty, ref_type, arms) in match_blocks.items():
        s = lines[h_idx].strip()
        match_expr = s[6:].rstrip('{').strip()
        header_lines[h_idx] = (ty, ref_type, match_expr)
        for a_idx in arms:
            arm_lines[a_idx] = (ty, ref_type)

    # Step 2: Process each line
    new_lines = []
    for i, line in enumerate(lines):
        s = line.strip()

        # Skip comments
        if s.startswith('//') or s.startswith('///') or s.startswith('*') or s.startswith('/*'):
            new_lines.append(line)
            continue

        # Transform match headers
        if i in header_lines:
            ty, ref_type, match_expr = header_lines[i]
            line = transform_match_header(line, ty, ref_type, match_expr)

        # Transform match arms
        if i in arm_lines:
            ty, ref_type = arm_lines[i]
            line = transform_match_arm(line, ty, ref_type)
            new_lines.append(line)
            continue

        # Transform non-match patterns
        if has_variant(line):
            line = transform_other(line, s, lines, i)

        new_lines.append(line)

    result = ''.join(new_lines)
    if result != original:
        with open(filepath, 'w') as f:
            f.write(result)
        return True
    return False


def transform_other(line, s, lines, idx):
    """Transform non-match-arm patterns."""

    for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
        for v, snake in variants.items():
            old = f'{ty}::{v}'
            if old not in line:
                continue

            # === 1. matches! patterns ===
            if 'matches!(' in line:
                # Simple: matches!(X, Type::Variant(_))
                simple_pat = f'{old}(_))'
                if simple_pat in line:
                    # Find the expression being matched
                    mi = line.find('matches!(')
                    rest = line[mi+9:]
                    ci = rest.find(',')
                    if ci >= 0:
                        expr_str = rest[:ci].strip()
                        if expr_str.startswith('&'):
                            expr_str = expr_str[1:]
                        after = line[line.index(simple_pat) + len(simple_pat):]
                        line = line[:mi] + f'{expr_str}.is_{snake}()' + after
                        continue

                # Guard: matches!(..., Type::Variant(var) if cond)
                if f'{old}(' in line and ' if ' in line:
                    mi = line.find('matches!(')
                    if mi >= 0:
                        rest = line[mi+9:]
                        ci = rest.find(',')
                        if ci >= 0:
                            expr_str = rest[:ci].strip()
                            if expr_str.startswith('&'):
                                expr_str = expr_str[1:]
                            after_comma = rest[ci+1:].strip()
                            # Find Type::Variant(binding)
                            vi = after_comma.find(f'{old}(')
                            if vi >= 0:
                                bi = vi + len(old) + 1
                                be = after_comma.find(')', bi)
                                if be >= 0:
                                    binding = after_comma[bi:be]
                                    gi = after_comma.find(' if ', be)
                                    if gi >= 0:
                                        guard = after_comma[gi+4:].rstrip().rstrip(')')
                                        after_matches = line[line.rindex(')') + 1:]
                                        line = line[:mi] + f'{expr_str}.as_{snake}().is_some_and(|{binding}| {guard})' + after_matches
                                        continue

                # Multi-variant matches with |
                if '|' in line and f'{old}(_)' in line:
                    # matches!(expr, Type::A(_) | Type::B(_))
                    # -> expr.is_a() || expr.is_b()
                    # Complex - skip for simple cases, handle in specific fixups
                    pass

                # Some(Type::Variant(v)) if guard inside matches!
                if f'Some({old}(' in line:
                    mi = line.find('matches!(')
                    if mi >= 0:
                        rest = line[mi+9:]
                        ci = rest.find(',')
                        if ci >= 0:
                            expr_str = rest[:ci].strip()
                            if expr_str.startswith('&'):
                                expr_str = expr_str[1:]
                            after_comma = rest[ci+1:].strip()
                            svi = after_comma.find(f'Some({old}(')
                            if svi >= 0:
                                bi = svi + len(f'Some({old}(')
                                # Find matching ))
                                depth = 2
                                k = bi
                                while k < len(after_comma) and depth > 0:
                                    if after_comma[k] == '(': depth += 1
                                    elif after_comma[k] == ')': depth -= 1
                                    k += 1
                                binding = after_comma[bi:k-2]
                                gi = after_comma.find(' if ', k)
                                if gi >= 0:
                                    guard = after_comma[gi+4:].rstrip().rstrip(')')
                                    after_matches = line[line.rindex(')') + 1:]
                                    line = line[:mi] + f'{expr_str}.as_{snake}().is_some_and(|{binding}| {guard})' + after_matches
                                    continue

                continue  # Skip further processing for this variant

            # === 2. let/if-let patterns ===
            if ('let ' in s or 'if let ' in s or '&& let ' in s or 'while let ' in s) and f'{old}(' in line:
                # Find the pattern: Type::Variant(binding) = rhs
                pi = line.find(f'{old}(')
                # Get binding
                bi = pi + len(old) + 1
                depth = 1; k = bi
                while k < len(line) and depth > 0:
                    if line[k] == '(': depth += 1
                    elif line[k] == ')': depth -= 1
                    k += 1
                binding = line[bi:k-1]
                after_pat = line[k:]
                before_pat = line[:pi]

                # Find =
                eq_pos = after_pat.find('= ')
                if eq_pos < 0:
                    # Maybe multiline
                    continue

                rhs = after_pat[eq_pos+2:]

                # Determine ref type from rhs
                rhs_stripped = rhs.lstrip()
                if rhs_stripped.startswith('&mut '):
                    method = f'as_{snake}_mut()'
                    rhs_expr = rhs_stripped[5:]
                elif rhs_stripped.startswith('&'):
                    method = f'as_{snake}()'
                    rhs_expr = rhs_stripped[1:]
                else:
                    # Check if it's owned (.take_in, .pop, .unbox etc)
                    if '.take_in(' in rhs or '.pop()' in rhs or '.unbox()' in rhs:
                        # Owned - need special handling
                        # For let-else with owned, use is_ + into_
                        if ' else ' in rhs:
                            expr_part = rhs_stripped.split(' else ')[0].strip()
                            else_part = ' else ' + rhs_stripped.split(' else ', 1)[1]
                            # Can't use as_ on owned. Need to check + into
                            # If binding has 'mut', it's taking ownership of Box
                            if binding.startswith('mut '):
                                actual_binding = binding[4:]
                                line = f'{before_pat}Some(mut {actual_binding}) = {expr_part}.as_{snake}_mut(){else_part}'
                                continue
                            else:
                                line = f'{before_pat}Some({binding}) = {expr_part}.as_{snake}(){else_part}'
                                continue
                        else:
                            method = f'as_{snake}_mut()'
                            rhs_expr = rhs_stripped
                    else:
                        # Assume mutable context (common in minifier)
                        method = f'as_{snake}_mut()'
                        rhs_expr = rhs_stripped

                # Build new line
                if ' else {' in rhs_expr:
                    expr_part = rhs_expr[:rhs_expr.index(' else {')].strip()
                    rest = rhs_expr[rhs_expr.index(' else {'):]
                    line = f'{before_pat}Some({binding}){after_pat[:eq_pos]}= {expr_part}.{method}{rest}'
                elif rhs_expr.rstrip().rstrip('\n').endswith('{'):
                    expr_part = rhs_expr.rstrip().rstrip('\n').rstrip('{').strip()
                    nl = '\n' if line.endswith('\n') else ''
                    line = f'{before_pat}Some({binding}){after_pat[:eq_pos]}= {expr_part}.{method} {{{nl}'
                else:
                    expr_part = rhs_expr.rstrip()
                    line = f'{before_pat}Some({binding}){after_pat[:eq_pos]}= {expr_part}.{method}'
                continue

            # === 3. Some(Type::Variant(x)) patterns (outside matches!) ===
            if f'Some({old}(' in line and 'matches!(' not in line:
                si = line.find(f'Some({old}(')
                bi = si + len(f'Some({old}(')
                depth = 1; k = bi
                while k < len(line) and depth > 0:
                    if line[k] == '(': depth += 1
                    elif line[k] == ')': depth -= 1
                    k += 1
                binding = line[bi:k-1]
                # Skip outer )
                if k < len(line) and line[k] == ')': k += 1
                after = line[k:]
                before = line[:si]

                if after.lstrip().startswith('='):
                    ei = after.index('=')
                    rhs = after[ei+1:].strip()
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
                    line = f'{before}Some({binding}){after[:ei]}= {expr_part}.and_then(|__s| __s.{method}()){rest}{nl}'
                elif after.lstrip().startswith('=>'):
                    # Match arm with Some() - convert arm
                    pass
                continue

            # === 4. Construction patterns ===
            if f'{old}(' in line:
                # Skip if in pattern context
                pi = line.find(f'{old}(')
                before = line[:pi].rstrip()

                # Pattern context indicators
                if 'let ' in line[:pi] and ' = ' not in line[:pi]:
                    continue
                if s.startswith('let ') and f'{old}(' in s.split(' = ')[0] if ' = ' in s else '':
                    continue
                if 'matches!(' in line:
                    continue

                # Check it's construction: after = , push(, return, , || etc
                if (before.endswith('=') or before.endswith('(') or before.endswith(',')
                    or before.endswith('||') or before.endswith('&&') or before.endswith('return')
                    or before.endswith('=>') or before.endswith('push(')
                    or s.startswith(f'{old}(') or before.endswith('.map(')
                    or before.endswith('Some(')
                ):
                    line = line.replace(f'{old}(', f'{ty}::{snake}(', 1)
                    continue

            # === 5. Function reference: Type::Variant without ( ===
            for sep in [')', ',', ';']:
                ref_pat = f'{old}{sep}'
                if ref_pat in line and f'{old}(' not in line:
                    before = line[:line.index(ref_pat)].rstrip()
                    if before.endswith('(') or before.endswith(',') or before.endswith('.map('):
                        line = line.replace(ref_pat, f'{ty}::{snake}{sep}')
                        break

    return line


for f in get_files():
    if process_file(f):
        print(f"Changed: {os.path.relpath(f, BASE)}")
print("Done.")
