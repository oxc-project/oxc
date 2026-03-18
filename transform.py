#!/usr/bin/env python3
"""Transform Expression/Statement enum patterns to tagged pointer struct patterns in oxc_minifier."""

import re
import os
import sys

BASE = "/Users/boshen/oxc/oxc/crates/oxc_minifier/src"

EXPR_VARIANTS = {
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

STMT_VARIANTS = {
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

def apply_line_transforms(lines):
    """Apply line-by-line transformations."""
    new_lines = []
    needs_expr_kind = False
    needs_stmt_kind = False

    # Track if we're inside a match block and what we're matching on
    i = 0
    while i < len(lines):
        line = lines[i]
        transformed = transform_line(line)
        if 'ExpressionKind::' in transformed and 'ExpressionKind::' not in line:
            needs_expr_kind = True
        if 'StatementKind::' in transformed and 'StatementKind::' not in line:
            needs_stmt_kind = True
        new_lines.append(transformed)
        i += 1

    return new_lines, needs_expr_kind, needs_stmt_kind


def transform_line(line):
    """Transform a single line."""
    original = line

    # Skip comments
    stripped = line.strip()
    if stripped.startswith('//') or stripped.startswith('///') or stripped.startswith('*') or stripped.startswith('/*'):
        return line

    all_variants = {}
    for v, s in EXPR_VARIANTS.items():
        all_variants[('Expression', v)] = s
    for v, s in STMT_VARIANTS.items():
        all_variants[('Statement', v)] = s

    for (ty, variant), snake in all_variants.items():
        kind_ty = f'{ty}Kind'

        # =====================================================
        # Pattern 1: matches!(expr, Type::Variant(_)) -> expr.is_snake()
        # =====================================================
        # Simple: matches!(expr, Expression::Variant(_))
        pat = rf'matches!\((\S+),\s*{ty}::{variant}\(_\)\)'
        repl = rf'\1.is_{snake}()'
        line = re.sub(pat, repl, line)

        # With &: matches!(&expr, Expression::Variant(_))
        pat = rf'matches!\(&(\S+),\s*{ty}::{variant}\(_\)\)'
        repl = rf'\1.is_{snake}()'
        line = re.sub(pat, repl, line)

        # =====================================================
        # Pattern 2: matches!(expr, Type::Variant(v) if cond)
        # -> expr.as_snake().is_some_and(|v| cond)
        # =====================================================
        pat = rf'matches!\(&?(\S+),\s*{ty}::{variant}\((\w+)\)\s+if\s+(.+)\)$'
        def mk_repl(m, s=snake):
            return f'{m.group(1)}.as_{s}().is_some_and(|{m.group(2)}| {m.group(3)})'
        line = re.sub(pat, mk_repl, line.rstrip())
        if line != original.rstrip():
            # Restore trailing whitespace
            line = line + '\n' if original.endswith('\n') else line
            # Actually let's just keep going without newline handling

        # =====================================================
        # Pattern 3a: let Type::Variant(x) = expr else { ... };
        # -> let Some(x) = expr.as_variant_mut() else { ... };
        # (assuming mutable context for top-level let-else)
        # =====================================================
        # let Expression::Variant(x) = expr else {
        pat = rf'(\s*)let {ty}::{variant}\((\w+)\) = (\S+) else \{{'
        def mk_let_else(m, s=snake):
            indent = m.group(1)
            var = m.group(2)
            expr = m.group(3)
            return f'{indent}let Some({var}) = {expr}.as_{s}_mut() else {{'
        line = re.sub(pat, mk_let_else, line)

        # let Expression::Variant(mut x) = expr else {
        pat = rf'(\s*)let {ty}::{variant}\(mut (\w+)\) = (\S+) else \{{'
        def mk_let_else_mut(m, s=snake):
            indent = m.group(1)
            var = m.group(2)
            expr = m.group(3)
            # owned: use into_variant()
            # But we need to split into is_ check + into_
            # Actually for .take_in() results (owned), we need into_
            # For &mut refs, we need as_mut
            # Let's use a heuristic: if expr contains .take_in or doesn't start with & it's owned
            if '.take_in(' in expr or '.pop()' in expr:
                return f'{indent}let mut {var} = {expr}.into_{s}(); // TODO: add is_ guard above if needed'
            return f'{indent}let Some(mut {var}) = {expr}.as_{s}_mut() else {{'
        line = re.sub(pat, mk_let_else_mut, line)

        # =====================================================
        # Pattern 3b: let Type::Variant(x) = &expr else { ... };
        # -> let Some(x) = expr.as_variant() else { ... };
        # =====================================================
        pat = rf'(\s*)let {ty}::{variant}\((\w+)\) = &(\S+) else \{{'
        def mk_let_ref(m, s=snake):
            indent = m.group(1)
            var = m.group(2)
            expr = m.group(3)
            return f'{indent}let Some({var}) = {expr}.as_{s}() else {{'
        line = re.sub(pat, mk_let_ref, line)

        # =====================================================
        # Pattern 3c: let Type::Variant(x) = &mut expr else { ... };
        # -> let Some(x) = expr.as_variant_mut() else { ... };
        # =====================================================
        pat = rf'(\s*)let {ty}::{variant}\((\w+)\) = &mut (\S+) else \{{'
        def mk_let_mut_ref(m, s=snake):
            indent = m.group(1)
            var = m.group(2)
            expr = m.group(3)
            return f'{indent}let Some({var}) = {expr}.as_{s}_mut() else {{'
        line = re.sub(pat, mk_let_mut_ref, line)

        # =====================================================
        # Pattern 4a: if let Type::Variant(x) = &expr {
        # -> if let Some(x) = expr.as_variant() {
        # =====================================================
        pat = rf'if let {ty}::{variant}\((\w+)\) = &(\S+) \{{'
        repl = rf'if let Some(\1) = \2.as_{snake}() {{'
        line = re.sub(pat, repl, line)

        # if let Type::Variant(x) = &expr\n (multiline, ends without {)
        pat = rf'if let {ty}::{variant}\((\w+)\) = &(\S+)$'
        repl = rf'if let Some(\1) = \2.as_{snake}()'
        line = re.sub(pat, repl, line)

        # =====================================================
        # Pattern 4b: if let Type::Variant(x) = &mut expr {
        # -> if let Some(x) = expr.as_variant_mut() {
        # =====================================================
        pat = rf'if let {ty}::{variant}\((\w+)\) = &mut (\S+) \{{'
        repl = rf'if let Some(\1) = \2.as_{snake}_mut() {{'
        line = re.sub(pat, repl, line)

        pat = rf'if let {ty}::{variant}\((\w+)\) = &mut (\S+)$'
        repl = rf'if let Some(\1) = \2.as_{snake}_mut()'
        line = re.sub(pat, repl, line)

        # =====================================================
        # Pattern 4c: if let Type::Variant(x) = expr {
        # (expr is &mut Type - most common case in minifier)
        # -> if let Some(x) = expr.as_variant_mut() {
        # =====================================================
        pat = rf'if let {ty}::{variant}\((\w+)\) = (\w+(?:\.\w+)*) \{{'
        repl = rf'if let Some(\1) = \2.as_{snake}_mut() {{'
        line = re.sub(pat, repl, line)

        pat = rf'if let {ty}::{variant}\((\w+)\) = (\w+(?:\.\w+)*)$'
        repl = rf'if let Some(\1) = \2.as_{snake}_mut()'
        line = re.sub(pat, repl, line)

        # =====================================================
        # Pattern 4d: } else if let Type::Variant(x) = &expr {
        # =====================================================
        pat = rf'else if let {ty}::{variant}\((\w+)\) = &(\S+) \{{'
        repl = rf'else if let Some(\1) = \2.as_{snake}() {{'
        line = re.sub(pat, repl, line)

        pat = rf'else if let {ty}::{variant}\((\w+)\) = &mut (\S+) \{{'
        repl = rf'else if let Some(\1) = \2.as_{snake}_mut() {{'
        line = re.sub(pat, repl, line)

        # =====================================================
        # Pattern 5: Some(Type::Variant(x)) in if-let contexts
        # if let Some(Type::Variant(x)) = opt_expr
        # -> if let Some(x) = opt_expr.and_then(|s| s.as_variant())
        # or for &mut:
        # -> if let Some(x) = opt_expr.and_then(|s| s.as_variant_mut())
        # =====================================================
        # These are complex - we'll handle them case by case

        # =====================================================
        # Pattern 6: match arms - Type::Variant(x) => ...
        # In match expr { ... } context, these become ExpressionKind::Variant(x) =>
        # We need to also change `match expr {` to `match expr.kind() {` or `match expr.kind_mut() {`
        # This is too complex for regex - we'll handle with separate logic
        # For now, just transform the arms:
        # =====================================================

        # Match arm: Expression::Variant(x) => (in match context)
        # We can't easily know if it's in a match, but these patterns are distinctive
        # Let's transform Expression::Variant(x) | Expression::Variant2(y) => style patterns
        # that are clearly in match arms

        # =====================================================
        # Pattern 7: Construction: Expression::Variant(alloc_expr) in non-pattern context
        # -> Expression::snake(alloc_expr)
        # =====================================================
        # These appear as: Expression::Variant(ctx.ast.alloc(...))
        # or: Statement::Variant(box_expr)
        # Heuristic: if it's at the beginning of an expression (after =, after push(, etc.)
        # and NOT after "let", "if let", "matches!", "match"

        # Look for construction patterns - after =, push(, etc.
        # Statement::Variant(expr) -> Statement::snake(expr) in construction
        # But NOT in pattern matching contexts

    return line


def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    original = content
    lines = content.split('\n')
    new_lines, needs_expr_kind, needs_stmt_kind = apply_line_transforms(lines)
    content = '\n'.join(new_lines)

    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False


for f in get_files():
    changed = process_file(f)
    if changed:
        print(f"Changed: {f}")
