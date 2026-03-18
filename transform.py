#!/usr/bin/env python3
"""Phase 2b: Handle remaining patterns."""
import os

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
    result = []
    for root, dirs, files in os.walk(BASE):
        if 'generated' in root:
            continue
        for f in files:
            if f.endswith('.rs'):
                result.append(os.path.join(root, f))
    return sorted(result)


def has_expr_or_stmt_variant(line):
    """Check if line contains any Expression:: or Statement:: variant pattern."""
    for ty in ['Expression', 'Statement']:
        variants = EXPR if ty == 'Expression' else STMT
        for v in variants:
            if f'{ty}::{v}' in line:
                return True
    return False


def replace_variants_in_match_arm(line):
    """Replace Expression::Variant or Statement::Variant with Kind version in match arms."""
    for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
        for v in variants:
            old = f'{ty}::{v}'
            new = f'{ty}Kind::{v}'
            if old in line:
                line = line.replace(old, new)
    return line


def replace_construction(line):
    """Replace construction patterns: Expression::Variant(x) -> Expression::snake(x)."""
    for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
        for v, snake in variants.items():
            old = f'{ty}::{v}('
            new = f'{ty}::{snake}('
            if old in line:
                line = line.replace(old, new)
    return line


def replace_some_stmt_pattern(line):
    """Replace Some(Statement::Variant(x)) patterns in if-let/let-else."""
    for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
        for v, snake in variants.items():
            # Some(Type::Variant(binding))
            pat = f'Some({ty}::{v}('
            if pat not in line:
                continue

            # Find the full pattern: Some(Type::Variant(BINDING))
            idx = line.index(pat)
            after = line[idx + len(pat):]
            # Find matching paren for the inner binding
            depth = 1
            i = 0
            while i < len(after) and depth > 0:
                if after[i] == '(':
                    depth += 1
                elif after[i] == ')':
                    depth -= 1
                i += 1
            # Now we should be at the ) of Variant(binding)
            binding = after[:i-1]
            # Skip the outer )
            if i < len(after) and after[i] == ')':
                i += 1

            rest_after = after[i:]

            # Find the = expr part before Some(
            before = line[:idx]

            # Determine if this is in a let-else or if-let context
            # Find "= " before "Some("
            eq_idx = before.rfind('= ')
            if eq_idx >= 0:
                expr_part = before[eq_idx+2:].strip()
                # This should be empty since we're at Some(
                # The actual expression is after the = in rest_after
                # Wait, the pattern is: let Some(Statement::Variant(x)) = result.last_mut() else {
                # So "= " is followed by the RHS expression which is on the same line after ") ="

                # Actually, let me re-parse. The full pattern is:
                # BEFORE Some(Type::Variant(BINDING)) = EXPR REST
                # where BEFORE contains "let " or "if let "
                pass

            # Instead of complex parsing, just look for the = after the Some(...) pattern
            full_pat = f'Some({ty}::{v}({binding}))'
            rest = line[idx + len(full_pat):]

            if rest.lstrip().startswith('='):
                # This is: let Some(Ty::Var(x)) = expr
                # Transform to: let Some(x) = expr.and_then(|__s| __s.as_variant_mut())
                eq_pos = rest.index('=')
                after_eq = rest[eq_pos+1:].lstrip()

                # Determine mut vs non-mut
                is_mut = 'last_mut()' in after_eq or '_mut()' in after_eq or '.pop()' in after_eq
                method = f'as_{snake}_mut' if is_mut else f'as_{snake}'

                # Build new line
                new_pat = f'Some({binding})'
                new_after = f' = {after_eq}'
                # We need to insert .and_then(|__s| __s.method())
                # Find the expression end (before "else {" or end of line)
                if ' else {' in after_eq:
                    expr_end_idx = after_eq.index(' else {')
                    expr = after_eq[:expr_end_idx].strip()
                    rest_part = after_eq[expr_end_idx:]
                    new_line = f'{before}{new_pat} = {expr}.and_then(|__s| __s.{method}()){rest_part}'
                elif after_eq.rstrip().endswith('{'):
                    # if let ... = expr {
                    expr = after_eq.rstrip().rstrip('{').strip()
                    new_line = f'{before}{new_pat} = {expr}.and_then(|__s| __s.{method}()) {{'
                else:
                    # Multiline continuation
                    expr = after_eq.strip()
                    new_line = f'{before}{new_pat} = {expr}.and_then(|__s| __s.{method}())'
                line = new_line

            elif rest.lstrip().startswith('=>'):
                # This is a match arm: Some(Ty::Var(x)) =>
                # Transform to use kind_mut() etc
                # Actually in match arm it should be => with the ExpressionKind
                line = line.replace(full_pat, f'Some(__s) if let Some({binding}) = __s.as_{snake}_mut()')

    return line


def process_file(filepath):
    with open(filepath) as f:
        content = f.read()
    original = content
    lines = content.split('\n')
    new_lines = []

    # State: are we in a match block that matches on Expression/Statement?
    match_stack = []  # Stack of (match_type, indent_level)

    for i, line in enumerate(lines):
        stripped = line.lstrip()

        # Skip comments
        if stripped.startswith('//') or stripped.startswith('*') or stripped.startswith('///'):
            new_lines.append(line)
            continue

        # Detect match blocks: "match EXPR {"
        # We need to check if the matched expression is of type Expression or Statement
        if 'match ' in stripped and stripped.rstrip().endswith('{'):
            indent = len(line) - len(stripped)
            # Check what comes next - if match arms use Expression:: or Statement::
            # Look ahead to see
            for j in range(i+1, min(i+5, len(lines))):
                next_stripped = lines[j].lstrip()
                if 'Expression::' in next_stripped and '=>' in next_stripped:
                    # This match block matches on Expression - need to add .kind()
                    # But only if the match target doesn't already have .kind()
                    if '.kind()' not in line and '.kind_mut()' not in line:
                        # Add .kind() or .kind_mut() to the match expression
                        # Heuristic: if any arms use &mut patterns, use kind_mut()
                        # For now, use kind() for shared ref
                        # Find the match expression
                        match_keyword_idx = stripped.index('match ')
                        after_match = stripped[match_keyword_idx + 6:]
                        # The expression is everything before the trailing {
                        match_expr = after_match.rstrip().rstrip('{').strip()
                        # Check if match expr starts with &mut or &
                        if match_expr.startswith('&mut '):
                            new_match_expr = match_expr[5:] + '.kind_mut()'
                        elif match_expr.startswith('&'):
                            new_match_expr = match_expr[1:] + '.kind()'
                        else:
                            new_match_expr = match_expr + '.kind()'
                        indent_str = line[:len(line) - len(stripped)]
                        line = f'{indent_str}match {new_match_expr} {{'
                    match_stack.append(('Expression', indent))
                    break
                elif 'Statement::' in next_stripped and '=>' in next_stripped:
                    if '.kind()' not in line and '.kind_mut()' not in line:
                        match_keyword_idx = stripped.index('match ')
                        after_match = stripped[match_keyword_idx + 6:]
                        match_expr = after_match.rstrip().rstrip('{').strip()
                        if match_expr.startswith('&mut '):
                            new_match_expr = match_expr[5:] + '.kind_mut()'
                        elif match_expr.startswith('&'):
                            new_match_expr = match_expr[1:] + '.kind()'
                        else:
                            new_match_expr = match_expr + '.kind()'
                        indent_str = line[:len(line) - len(stripped)]
                        line = f'{indent_str}match {new_match_expr} {{'
                    match_stack.append(('Statement', indent))
                    break
                elif next_stripped == '' or next_stripped.startswith('//'):
                    continue
                else:
                    break

        # Replace match arms
        if has_expr_or_stmt_variant(line) and '=>' in stripped:
            line = replace_variants_in_match_arm(line)
        # Also handle multi-line match arms with |
        if has_expr_or_stmt_variant(line) and (stripped.startswith('| ') or stripped.endswith('|')):
            line = replace_variants_in_match_arm(line)

        # Handle Some(Statement/Expression::Variant(x)) patterns
        for ty in ['Expression', 'Statement']:
            variants = EXPR if ty == 'Expression' else STMT
            for v in variants:
                if f'Some({ty}::{v}(' in line:
                    line = replace_some_stmt_pattern(line)
                    break
            else:
                continue
            break

        # Handle construction patterns that are clearly not in match arms
        # Only for lines that have Expression::Variant( or Statement::Variant(
        # and are NOT match arms (no =>)
        if has_expr_or_stmt_variant(line) and '=>' not in stripped and '|' not in stripped:
            # Check it's a construction context
            for ty, variants in [('Expression', EXPR), ('Statement', STMT)]:
                for v, snake in variants.items():
                    old = f'{ty}::{v}('
                    if old not in line:
                        continue
                    # Skip if it's in a pattern matching context
                    if 'let ' in stripped and '= ' in stripped and old in stripped.split('= ')[0]:
                        continue
                    if stripped.startswith('if let '):
                        continue
                    # Check it's a construction (after =, push(, etc. or as return value)
                    idx = line.index(old)
                    before = line[:idx].rstrip()
                    if (before.endswith('=') or before.endswith('(') or before.endswith(',')
                        or before.endswith('||') or before.endswith('&&')
                        or before.endswith('return') or before.endswith('=>')
                        or before.endswith('.map(') or before.endswith('.map(|_|')
                        or stripped.startswith(old)  # start of expression
                        or 'push(' in before
                    ):
                        new = f'{ty}::{snake}('
                        line = line.replace(old, new)

        new_lines.append(line)

    result = '\n'.join(new_lines)
    if result != original:
        with open(filepath, 'w') as f:
            f.write(result)
        return True
    return False


for f in get_files():
    if process_file(f):
        print(f"Changed: {os.path.relpath(f, BASE)}")
print("Phase 2b done")
