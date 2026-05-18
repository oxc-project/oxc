# Conformance test results - regexp

Tested against: [regexp@788787a](https://github.com/ota-meshi/eslint-plugin-regexp/tree/788787a7e9e820c40321fe2f1095d00d4a486866) (v3.1.0)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    82 | 100.0% |
| Fully passing     |    81 |  98.8% |
| Partially passing |     1 |   1.2% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  2370 | 100.0% |
| Passing     |  2349 |  99.1% |
| Failing     |     2 |   0.1% |
| Skipped     |    19 |   0.8% |

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
- `no-lazy-ends` (18 tests) (1 skipped)
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
- `no-unused-capturing-group` (67 tests) (1 skipped)
- `no-useless-assertions` (84 tests)
- `no-useless-backreference` (29 tests)
- `no-useless-character-class` (61 tests)
- `no-useless-dollar-replacements` (29 tests)
- `no-useless-escape` (143 tests)
- `no-useless-flag` (82 tests) (5 skipped)
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

- `sort-flags` - 16 / 18 (88.9%)

## Rules with Failures Detail

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
  "output": "/\\w/gimsuy",
  "errors": [
    "The flags 'yusimg' should be in the order 'gimsuy'."
  ],
  "recursive": true
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


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
  "output": "/\\w/gimsvy",
  "errors": [
    "The flags 'yvsimg' should be in the order 'gimsvy'."
  ],
  "recursive": true
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js

