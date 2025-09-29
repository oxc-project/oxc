#!/usr/bin/env python3
"""Identify specific test regressions between main and development branch."""

def parse_snapshot(content):
    """Parse snapshot content and return list of failing tests."""
    failing_tests = []
    lines = content.strip().split('\n')

    for line in lines:
        if line.startswith('| ') and '💥' in line:
            # Extract test path from markdown table row
            parts = line.split('|')
            if len(parts) >= 2:
                test_path = parts[1].strip()
                if test_path and test_path != 'Spec path':
                    failing_tests.append(test_path)

    return set(failing_tests)

# JavaScript tests - Main branch (663/699 passing = 36 failing)
main_js_failing = {
    'js/comments/15661.js',
    'js/comments/empty-statements.js',
    'js/comments/function-declaration.js',
    'js/comments/return-statement.js',
    'js/comments/html-like/comment.js',
    'js/comments-closure-typecast/comment-in-the-middle.js',
    'js/conditional/comments.js',
    'js/conditional/new-ternary-examples.js',
    'js/conditional/new-ternary-spec.js',
    'js/conditional/postfix-ternary-regressions.js',
    'js/explicit-resource-management/valid-await-using-comments.js',
    'js/for/for-in-with-initializer.js',
    'js/for/parentheses.js',
    'js/identifier/for-of/let.js',
    'js/identifier/parentheses/let.js',
    'js/last-argument-expansion/dangling-comment-in-arrow-function.js',
    'js/object-multiline/multiline.js',
    'js/quote-props/classes.js',
    'js/quote-props/objects.js',
    'js/quote-props/with_numbers.js',
    'js/quotes/objects.js',
    'js/ternaries/binary.js',
    'js/ternaries/func-call.js',
    'js/ternaries/indent-after-paren.js',
    'js/ternaries/indent.js',
    'js/ternaries/nested-in-condition.js',
    'js/ternaries/nested.js',
    'js/ternaries/parenthesis.js',
    'js/ternaries/test.js',
    'js/ternaries/parenthesis/await-expression.js',
    'js/test-declarations/angularjs_inject.js',
    'jsx/expression-with-types/expression.js',
    'jsx/fbt/test.js',
    'jsx/jsx/quotes.js',
    'jsx/single-attribute-per-line/single-attribute-per-line.js',
    'jsx/text-wrap/test.js',
}

# JavaScript tests - Dev branch (647/699 passing = 52 failing)
dev_js_failing = {
    'js/arrows/call.js',
    'js/arrows/chain-as-arg.js',
    'js/arrows/curried.js',
    'js/arrows/currying-2.js',
    'js/arrows/currying-4.js',
    'js/comments/15661.js',
    'js/comments/empty-statements.js',
    'js/comments/function-declaration.js',
    'js/comments/return-statement.js',
    'js/comments/html-like/comment.js',
    'js/comments-closure-typecast/comment-in-the-middle.js',
    'js/conditional/comments.js',
    'js/conditional/new-ternary-examples.js',
    'js/conditional/new-ternary-spec.js',
    'js/conditional/postfix-ternary-regressions.js',
    'js/decorators/member-expression.js',
    'js/decorators/parens.js',
    'js/explicit-resource-management/valid-await-using-comments.js',
    'js/for/for-in-with-initializer.js',
    'js/for/parentheses.js',
    'js/functional-composition/pipe-function-calls.js',
    'js/identifier/for-of/let.js',
    'js/identifier/parentheses/let.js',
    'js/last-argument-expansion/dangling-comment-in-arrow-function.js',
    'js/method-chain/print-width-120/constructor.js',
    'js/new-expression/call.js',
    'js/new-expression/new_expression.js',
    'js/object-multiline/multiline.js',
    'js/quote-props/classes.js',
    'js/quote-props/objects.js',
    'js/quote-props/with_numbers.js',
    'js/quotes/objects.js',
    'js/require/require.js',
    'js/strings/template-literals.js',
    'js/ternaries/binary.js',
    'js/ternaries/func-call.js',
    'js/ternaries/indent-after-paren.js',
    'js/ternaries/indent.js',
    'js/ternaries/nested-in-condition.js',
    'js/ternaries/nested.js',
    'js/ternaries/parenthesis.js',
    'js/ternaries/test.js',
    'js/ternaries/parenthesis/await-expression.js',
    'js/test-declarations/angular_async.js',
    'js/test-declarations/angular_fakeAsync.js',
    'js/test-declarations/angular_waitForAsync.js',
    'js/test-declarations/angularjs_inject.js',
    'jsx/expression-with-types/expression.js',
    'jsx/fbt/test.js',
    'jsx/jsx/quotes.js',
    'jsx/single-attribute-per-line/single-attribute-per-line.js',
    'jsx/text-wrap/test.js',
}

# TypeScript tests - Main branch (533/573 passing = 40 failing)
main_ts_failing = {
    'jsx/expression-with-types/expression.js',
    'jsx/fbt/test.js',
    'jsx/jsx/quotes.js',
    'jsx/single-attribute-per-line/single-attribute-per-line.js',
    'jsx/text-wrap/test.js',
    'typescript/angular-component-examples/15934-computed.component.ts',
    'typescript/angular-component-examples/15934.component.ts',
    'typescript/angular-component-examples/test.component.ts',
    'typescript/arrow/comments.ts',
    'typescript/cast/tuple-and-record.ts',
    'typescript/chain-expression/call-expression.ts',
    'typescript/chain-expression/member-expression.ts',
    'typescript/chain-expression/test.ts',
    'typescript/class/empty-method-body.ts',
    'typescript/class/quoted-property.ts',
    'typescript/comments/method_types.ts',
    'typescript/conditional-types/comments.ts',
    'typescript/conditional-types/conditonal-types.ts',
    'typescript/conditional-types/infer-type.ts',
    'typescript/conditional-types/nested-in-condition.ts',
    'typescript/conditional-types/new-ternary-spec.ts',
    'typescript/conditional-types/parentheses.ts',
    'typescript/conformance/types/functions/functionOverloadErrorsSyntax.ts',
    'typescript/decorators-ts/angular.ts',
    'typescript/definite/without-annotation.ts',
    'typescript/enum/computed-members.ts',
    'typescript/interface/ignore.ts',
    'typescript/intersection/intersection-parens.ts',
    'typescript/intersection/consistent-with-flow/intersection-parens.ts',
    'typescript/last-argument-expansion/decorated-function.tsx',
    'typescript/multiparser-css/issue-6259.ts',
    'typescript/non-null/optional-chain.ts',
    'typescript/object-multiline/multiline.ts',
    'typescript/prettier-ignore/mapped-types.ts',
    'typescript/prettier-ignore/prettier-ignore-nested-unions.ts',
    'typescript/type-arguments-bit-shift-left-like/3.ts',
    'typescript/type-arguments-bit-shift-left-like/5.tsx',
    'typescript/union/union-parens.ts',
    'typescript/union/consistent-with-flow/prettier-ignore.ts',
    'typescript/union/single-type/single-type.ts',
}

# TypeScript tests - Dev branch (526/573 passing = 47 failing)
dev_ts_failing = {
    'jsx/expression-with-types/expression.js',
    'jsx/fbt/test.js',
    'jsx/jsx/quotes.js',
    'jsx/single-attribute-per-line/single-attribute-per-line.js',
    'jsx/text-wrap/test.js',
    'typescript/angular-component-examples/15934-computed.component.ts',
    'typescript/angular-component-examples/15934.component.ts',
    'typescript/angular-component-examples/test.component.ts',
    'typescript/arrow/16067.ts',
    'typescript/arrow/comments.ts',
    'typescript/as/nested-await-and-as.ts',
    'typescript/cast/generic-cast.ts',
    'typescript/cast/tuple-and-record.ts',
    'typescript/chain-expression/call-expression.ts',
    'typescript/chain-expression/member-expression.ts',
    'typescript/chain-expression/test.ts',
    'typescript/class/empty-method-body.ts',
    'typescript/class/quoted-property.ts',
    'typescript/comments/method_types.ts',
    'typescript/comments/type-parameters.ts',
    'typescript/conditional-types/comments.ts',
    'typescript/conditional-types/conditonal-types.ts',
    'typescript/conditional-types/infer-type.ts',
    'typescript/conditional-types/nested-in-condition.ts',
    'typescript/conditional-types/new-ternary-spec.ts',
    'typescript/conditional-types/parentheses.ts',
    'typescript/conformance/types/functions/functionOverloadErrorsSyntax.ts',
    'typescript/decorators-ts/angular.ts',
    'typescript/decorators-ts/typeorm.ts',
    'typescript/definite/without-annotation.ts',
    'typescript/enum/computed-members.ts',
    'typescript/functional-composition/pipe-function-calls.ts',
    'typescript/interface/ignore.ts',
    'typescript/intersection/intersection-parens.ts',
    'typescript/intersection/consistent-with-flow/intersection-parens.ts',
    'typescript/last-argument-expansion/decorated-function.tsx',
    'typescript/multiparser-css/issue-6259.ts',
    'typescript/non-null/optional-chain.ts',
    'typescript/object-multiline/multiline.ts',
    'typescript/prettier-ignore/mapped-types.ts',
    'typescript/prettier-ignore/prettier-ignore-nested-unions.ts',
    'typescript/satisfies-operators/nested-await-and-satisfies.ts',
    'typescript/type-arguments-bit-shift-left-like/3.ts',
    'typescript/type-arguments-bit-shift-left-like/5.tsx',
    'typescript/union/union-parens.ts',
    'typescript/union/consistent-with-flow/prettier-ignore.ts',
    'typescript/union/single-type/single-type.ts',
}

# Calculate regressions
js_regressions = dev_js_failing - main_js_failing
ts_regressions = dev_ts_failing - main_ts_failing

# Calculate improvements
js_improvements = main_js_failing - dev_js_failing
ts_improvements = main_ts_failing - dev_ts_failing

print("# Prettier Conformance Test Regression Analysis\n")
print(f"## Summary")
print(f"- JavaScript: {len(dev_js_failing)} failing (dev) vs {len(main_js_failing)} failing (main)")
print(f"- TypeScript: {len(dev_ts_failing)} failing (dev) vs {len(main_ts_failing)} failing (main)")
print(f"- **Total Regressions: {len(js_regressions) + len(ts_regressions)}**\n")

if js_regressions:
    print(f"## JavaScript Regressions ({len(js_regressions)} tests)")
    print("Tests that are failing in dev branch but passing in main:\n")
    for test in sorted(js_regressions):
        print(f"- `{test}`")
    print()

if ts_regressions:
    print(f"## TypeScript Regressions ({len(ts_regressions)} tests)")
    print("Tests that are failing in dev branch but passing in main:\n")
    for test in sorted(ts_regressions):
        print(f"- `{test}`")
    print()

if js_improvements:
    print(f"## JavaScript Improvements ({len(js_improvements)} tests)")
    print("Tests that are passing in dev branch but failing in main:\n")
    for test in sorted(js_improvements):
        print(f"- `{test}`")
    print()

if ts_improvements:
    print(f"## TypeScript Improvements ({len(ts_improvements)} tests)")
    print("Tests that are passing in dev branch but failing in main:\n")
    for test in sorted(ts_improvements):
        print(f"- `{test}`")
    print()

print("## Analysis by Category\n")

# Group regressions by category
categories = {}
for test in js_regressions.union(ts_regressions):
    parts = test.split('/')
    if len(parts) >= 2:
        category = parts[0] if parts[0] != 'js' else parts[1]
        if category not in categories:
            categories[category] = []
        categories[category].append(test)

if categories:
    print("### Regressions by Test Category:")
    for category in sorted(categories.keys()):
        tests = categories[category]
        print(f"\n**{category}** ({len(tests)} tests):")
        for test in sorted(tests):
            print(f"  - `{test}`")