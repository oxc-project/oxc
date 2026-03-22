# Conformance test results - regexp

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    82 | 100.0% |
| Fully passing     |    78 |  95.1% |
| Partially passing |     4 |   4.9% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  2370 | 100.0% |
| Passing     |  2349 |  99.1% |
| Failing     |     9 |   0.4% |
| Skipped     |    12 |   0.5% |

## Fully Passing Rules

- `confusing-quantifier` (9 tests)
- `control-character-escape` (19 tests)
- `grapheme-string-literal` (157 tests)
- `hexadecimal-escape` (15 tests)
- `letter-case` (58 tests)
- `match-any` (40 tests)
- `negation` (30 tests)
- `no-contradiction-with-assertion` (13 tests)
- `no-control-character` (23 tests)
- `no-dupe-characters-character-class` (59 tests)
- `no-dupe-disjunctions` (79 tests)
- `no-empty-alternative` (13 tests)
- `no-empty-capturing-group` (10 tests)
- `no-empty-character-class` (21 tests)
- `no-empty-group` (8 tests)
- `no-empty-lookarounds-assertion` (26 tests)
- `no-empty-string-literal` (8 tests)
- `no-escape-backspace` (7 tests)
- `no-extra-lookaround-assertions` (13 tests)
- `no-invalid-regexp` (8 tests)
- `no-invisible-character` (17 tests)
- `no-legacy-features` (40 tests) (1 skipped)
- `no-misleading-capturing-group` (16 tests)
- `no-misleading-unicode-character` (67 tests)
- `no-missing-g-flag` (22 tests)
- `no-non-standard-flag` (6 tests) (6 skipped)
- `no-obscure-range` (15 tests)
- `no-octal` (11 tests)
- `no-optional-assertion` (13 tests)
- `no-potentially-useless-backreference` (12 tests)
- `no-standalone-backslash` (6 tests)
- `no-super-linear-backtracking` (10 tests)
- `no-super-linear-move` (17 tests)
- `no-trivially-nested-assertion` (31 tests)
- `no-trivially-nested-quantifier` (18 tests)
- `no-useless-assertions` (84 tests)
- `no-useless-backreference` (29 tests)
- `no-useless-character-class` (61 tests)
- `no-useless-dollar-replacements` (29 tests)
- `no-useless-escape` (143 tests)
- `no-useless-lazy` (23 tests)
- `no-useless-non-capturing-group` (54 tests)
- `no-useless-quantifier` (20 tests)
- `no-useless-range` (13 tests)
- `no-useless-set-operand` (14 tests)
- `no-useless-string-literal` (12 tests)
- `no-useless-two-nums-quantifier` (10 tests)
- `no-zero-quantifier` (7 tests)
- `optimal-lookaround-quantifier` (10 tests)
- `optimal-quantifier-concatenation` (56 tests)
- `prefer-character-class` (54 tests)
- `prefer-d` (25 tests)
- `prefer-escape-replacement-dollar-char` (16 tests)
- `prefer-lookaround` (88 tests)
- `prefer-named-backreference` (6 tests)
- `prefer-named-capture-group` (6 tests)
- `prefer-named-replacement` (16 tests)
- `prefer-plus-quantifier` (13 tests)
- `prefer-predefined-assertion` (14 tests)
- `prefer-quantifier` (25 tests)
- `prefer-question-quantifier` (27 tests)
- `prefer-range` (38 tests)
- `prefer-regexp-exec` (6 tests)
- `prefer-regexp-test` (16 tests)
- `prefer-result-array-groups` (31 tests) (5 skipped)
- `prefer-set-operation` (8 tests)
- `prefer-star-quantifier` (13 tests)
- `prefer-unicode-codepoint-escapes` (13 tests)
- `prefer-w` (16 tests)
- `require-unicode-regexp` (42 tests)
- `require-unicode-sets-regexp` (28 tests)
- `simplify-set-operations` (36 tests)
- `sort-alternatives` (34 tests)
- `sort-character-class-elements` (52 tests)
- `strict` (31 tests)
- `unicode-escape` (11 tests)
- `unicode-property` (10 tests)
- `use-ignore-case` (28 tests)

## Rules with Failures

- `no-lazy-ends` - 17 / 18 (94.4%)
- `no-unused-capturing-group` - 66 / 67 (98.5%)
- `no-useless-flag` - 77 / 82 (93.9%)
- `sort-flags` - 16 / 18 (88.9%)

## Rules with Failures Detail

### `no-lazy-ends`

Pass: 17 / 18 (94.4%)
Fail: 1 / 18 (5.6%)
Skip: 0 / 18 (0.0%)

#### no-lazy-ends > valid

```js

            /* exported a */
            const a = /a??/
            a.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-lazy-ends',
    message: 'The quantifier and the quantified element can be removed because the quantifier is lazy and has a minimum of 0.',
    messageId: 'uselessElement',
    severity: 1,
    nodeType: 'Literal',
    line: 3,
    column: 23,
    endLine: 3,
    endColumn: 26,
    fixes: null,
    suggestions: [ [Object], [Object] ]
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-unused-capturing-group`

Pass: 66 / 67 (98.5%)
Fail: 1 / 67 (1.5%)
Skip: 0 / 67 (0.0%)

#### no-unused-capturing-group > valid

```js

            /* exported regexp */
            const regexp = /(\d{4})-(\d{2})-(\d{2})/
            const replaced = '2000-12-31'.replace(regexp, 'Date') // "Date"
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 3: [
  {
    ruleId: 'rule-to-test/no-unused-capturing-group',
    message: 'Capturing group number 1 is defined but never used.',
    messageId: 'unusedCapturingGroup',
    severity: 1,
    nodeType: 'Literal',
    line: 3,
    column: 28,
    endLine: 3,
    endColumn: 35,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/no-unused-capturing-group',
    message: 'Capturing group number 2 is defined but never used.',
    messageId: 'unusedCapturingGroup',
    severity: 1,
    nodeType: 'Literal',
    line: 3,
    column: 36,
    endLine: 3,
    endColumn: 43,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/no-unused-capturing-group',
    message: 'Capturing group number 3 is defined but never used.',
    messageId: 'unusedCapturingGroup',
    severity: 1,
    nodeType: 'Literal',
    line: 3,
    column: 44,
    endLine: 3,
    endColumn: 51,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

3 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-useless-flag`

Pass: 77 / 82 (93.9%)
Fail: 5 / 82 (6.1%)
Skip: 0 / 82 (0.0%)

#### no-useless-flag > valid

```js

            /* exported b */
            const a = /foo/y;
            const b = a;
            const regex = b;

            'str'.split(regex)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-useless-flag',
    message: "The 'y' flag is unnecessary because 'String.prototype.split' ignores the 'y' flag.",
    messageId: 'uselessStickyFlag',
    severity: 1,
    nodeType: 'Literal',
    line: 3,
    column: 27,
    endLine: 3,
    endColumn: 28,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-useless-flag > valid

```js

        const orig = /\w/i; // eslint-disable-line
        const clone = new RegExp(orig);
        
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-useless-flag',
    message: "The 'i' flag is unnecessary because the pattern only contains case-invariant characters.",
    messageId: 'uselessIgnoreCaseFlag',
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 25,
    endLine: 2,
    endColumn: 26,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-useless-flag > invalid

```js

            const orig = /\w/i; // eslint-disable-line
            const clone = new RegExp(orig, 'i');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__",
  "output": "\n            const orig = /\\w/i; // eslint-disable-line\n            const clone = new RegExp(orig, '');\n            ",
  "recursive": true
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

  '\n' +
+   '            const orig = /\\w/; // eslint-disable-line\n' +
-   '            const orig = /\\w/i; // eslint-disable-line\n' +
    "            const clone = new RegExp(orig, '');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### Don't conflict even if using the rules together.

```js
/[\s\S]*/s;
```

```json
{}
```

Error: Test case was not run with `RuleTester`
    at it (apps/oxlint/conformance/src/capture.ts:132:13)
    at <anonymous> (apps/oxlint/conformance/submodules/regexp/tests/lib/rules/no-useless-flag.ts:484:9)
    at describe (apps/oxlint/conformance/src/capture.ts:85:17)
    at assert (apps/oxlint/conformance/submodules/regexp/tests/lib/rules/no-useless-flag.ts:398:1)


#### Don't conflict even if using the rules together.

```js
/[\s\S]*/s;
```

```json
{}
```

Error: Test case was not run with `RuleTester`
    at it (apps/oxlint/conformance/src/capture.ts:132:13)
    at <anonymous> (apps/oxlint/conformance/submodules/regexp/tests/lib/rules/no-useless-flag.ts:484:9)
    at describe (apps/oxlint/conformance/src/capture.ts:85:17)
    at assert (apps/oxlint/conformance/submodules/regexp/tests/lib/rules/no-useless-flag.ts:398:1)


### `sort-flags`

Pass: 16 / 18 (88.9%)
Fail: 2 / 18 (11.1%)
Skip: 0 / 18 (0.0%)

#### sort-flags > invalid

```js
/\w/yusimg
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__",
  "output": "/\\w/gimsuy",
  "recursive": true
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/\\w/yusimg'
- '/\\w/gimsuy'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-flags > invalid

```js
/\w/yvsimg
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__",
  "output": "/\\w/gimsvy",
  "recursive": true
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/\\w/yvsimg'
- '/\\w/gimsvy'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)

