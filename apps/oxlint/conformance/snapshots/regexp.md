# Conformance test results - regexp

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    82 | 100.0% |
| Fully passing     |    30 |  36.6% |
| Partially passing |    52 |  63.4% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  2370 | 100.0% |
| Passing     |  1596 |  67.3% |
| Failing     |   762 |  32.2% |
| Skipped     |    12 |   0.5% |

## Fully Passing Rules

- `confusing-quantifier` (9 tests)
- `grapheme-string-literal` (157 tests)
- `no-contradiction-with-assertion` (13 tests)
- `no-control-character` (23 tests)
- `no-dupe-disjunctions` (79 tests)
- `no-empty-alternative` (13 tests)
- `no-empty-capturing-group` (10 tests)
- `no-empty-character-class` (21 tests)
- `no-empty-group` (8 tests)
- `no-empty-lookarounds-assertion` (26 tests)
- `no-empty-string-literal` (8 tests)
- `no-escape-backspace` (7 tests)
- `no-invalid-regexp` (8 tests)
- `no-legacy-features` (40 tests) (1 skipped)
- `no-misleading-capturing-group` (16 tests)
- `no-non-standard-flag` (6 tests) (6 skipped)
- `no-obscure-range` (15 tests)
- `no-octal` (11 tests)
- `no-optional-assertion` (13 tests)
- `no-potentially-useless-backreference` (12 tests)
- `no-standalone-backslash` (6 tests)
- `no-super-linear-move` (17 tests)
- `no-useless-assertions` (84 tests)
- `no-useless-backreference` (29 tests)
- `no-useless-dollar-replacements` (29 tests)
- `no-zero-quantifier` (7 tests)
- `optimal-lookaround-quantifier` (10 tests)
- `prefer-escape-replacement-dollar-char` (16 tests)
- `prefer-named-capture-group` (6 tests)
- `prefer-regexp-exec` (6 tests)

## Rules with Failures

- `control-character-escape` - 8 / 19 (42.1%)
- `hexadecimal-escape` - 9 / 15 (60.0%)
- `letter-case` - 35 / 58 (60.3%)
- `match-any` - 21 / 40 (52.5%)
- `negation` - 10 / 30 (33.3%)
- `no-dupe-characters-character-class` - 22 / 59 (37.3%)
- `no-extra-lookaround-assertions` - 3 / 13 (23.1%)
- `no-invisible-character` - 11 / 17 (64.7%)
- `no-lazy-ends` - 17 / 18 (94.4%)
- `no-misleading-unicode-character` - 32 / 67 (47.8%)
- `no-missing-g-flag` - 14 / 22 (63.6%)
- `no-super-linear-backtracking` - 6 / 10 (60.0%)
- `no-trivially-nested-assertion` - 7 / 31 (22.6%)
- `no-trivially-nested-quantifier` - 5 / 18 (27.8%)
- `no-unused-capturing-group` - 58 / 67 (86.6%)
- `no-useless-character-class` - 29 / 61 (47.5%)
- `no-useless-escape` - 111 / 143 (77.6%)
- `no-useless-flag` - 55 / 82 (67.1%)
- `no-useless-lazy` - 9 / 23 (39.1%)
- `no-useless-non-capturing-group` - 35 / 54 (64.8%)
- `no-useless-quantifier` - 18 / 20 (90.0%)
- `no-useless-range` - 4 / 13 (30.8%)
- `no-useless-set-operand` - 1 / 14 (7.1%)
- `no-useless-string-literal` - 2 / 12 (16.7%)
- `no-useless-two-nums-quantifier` - 4 / 10 (40.0%)
- `optimal-quantifier-concatenation` - 18 / 56 (32.1%)
- `prefer-character-class` - 10 / 54 (18.5%)
- `prefer-d` - 13 / 25 (52.0%)
- `prefer-lookaround` - 46 / 88 (52.3%)
- `prefer-named-backreference` - 4 / 6 (66.7%)
- `prefer-named-replacement` - 10 / 16 (62.5%)
- `prefer-plus-quantifier` - 7 / 13 (53.8%)
- `prefer-predefined-assertion` - 2 / 14 (14.3%)
- `prefer-quantifier` - 12 / 25 (48.0%)
- `prefer-question-quantifier` - 15 / 27 (55.6%)
- `prefer-range` - 22 / 38 (57.9%)
- `prefer-regexp-test` - 8 / 16 (50.0%)
- `prefer-result-array-groups` - 21 / 31 (67.7%)
- `prefer-set-operation` - 5 / 8 (62.5%)
- `prefer-star-quantifier` - 7 / 13 (53.8%)
- `prefer-unicode-codepoint-escapes` - 8 / 13 (61.5%)
- `prefer-w` - 6 / 16 (37.5%)
- `require-unicode-regexp` - 28 / 42 (66.7%)
- `require-unicode-sets-regexp` - 26 / 28 (92.9%)
- `simplify-set-operations` - 12 / 36 (33.3%)
- `sort-alternatives` - 11 / 34 (32.4%)
- `sort-character-class-elements` - 22 / 52 (42.3%)
- `sort-flags` - 13 / 18 (72.2%)
- `strict` - 24 / 31 (77.4%)
- `unicode-escape` - 6 / 11 (54.5%)
- `unicode-property` - 1 / 10 (10.0%)
- `use-ignore-case` - 20 / 28 (71.4%)

## Rules with Failures Detail

### `control-character-escape`

Pass: 8 / 19 (42.1%)
Fail: 11 / 19 (57.9%)
Skip: 0 / 19 (0.0%)

#### control-character-escape > invalid

```js
/\x00/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\0/' !== '/\\x00/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/\x0a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\n/' !== '/\\x0a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/\cJ/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\n/' !== '/\\cJ/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/\u{a}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\n/u'
- '/\\u{a}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
RegExp("\\cJ")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp("\\\\n")'
- 'RegExp("\\\\cJ")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
RegExp("\\u{a}", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp("\\\\n", "u")'
- 'RegExp("\\\\u{a}", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/\u0009/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\t/'
- '/\\u0009/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/	/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\t/' !== '/\t/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js

            const s = "\\u0009"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "\\\\t"\n            new RegExp(s)\n            '
- '\n            const s = "\\\\u0009"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
RegExp("\t\r\n\0" + /	/.source)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp("\\t\\r\\n\\0" + /\\t/.source)'
- 'RegExp("\\t\\r\\n\\0" + /\t/.source)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### control-character-escape > invalid

```js
/[\q{\x00}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{\\0}]/v'
- '/[\\q{\\x00}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `hexadecimal-escape`

Pass: 9 / 15 (60.0%)
Fail: 6 / 15 (40.0%)
Skip: 0 / 15 (0.0%)

#### hexadecimal-escape > invalid

```js
/\u000a \u{00000a}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\x0a \\x0a/u'
- '/\\u000a \\u{00000a}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### hexadecimal-escape > invalid

```js
/\u000a \u{00000a}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "always"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\x0a \\x0a/u'
- '/\\u000a \\u{00000a}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### hexadecimal-escape > invalid

```js
/\x0f \xff/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "never"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u000f \\u00ff/u'
- '/\\x0f \\xff/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### hexadecimal-escape > invalid

```js
/\x0a \x0b \x41/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "never"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u000a \\u000b \\u0041/u'
- '/\\x0a \\x0b \\x41/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### hexadecimal-escape > invalid

```js
/[\q{\u000a \u{00000a}}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "always"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{\\x0a \\x0a}]/v'
- '/[\\q{\\u000a \\u{00000a}}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### hexadecimal-escape > invalid

```js
/[\q{\x0f \xff}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "never"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{\\u000f \\u00ff}]/v'
- '/[\\q{\\x0f \\xff}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `letter-case`

Pass: 35 / 58 (60.3%)
Fail: 23 / 58 (39.7%)
Skip: 0 / 58 (0.0%)

#### letter-case > invalid

```js
/Regexp/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/regexp/i'
- '/Regexp/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/Regexp/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "caseInsensitive": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/regexp/i'
- '/Regexp/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/ReGeXp/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "caseInsensitive": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/REGEXP/i'
- '/ReGeXp/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[A-Z]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-z]/i'
- '/[A-Z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\u0041-Z]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-z]/i'
- '/[\\u0041-Z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\u004A-Z]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\u004a-Z]/i'
- '/[\\u004A-Z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\u004a-Z]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[j-z]/i'
- '/[\\u004a-Z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u000A/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u000a/'
- '/\\u000A/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u000A/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "unicodeEscape": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u000a/'
- '/\\u000A/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u000a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "unicodeEscape": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u000A/'
- '/\\u000a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u{A}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{a}/u'
- '/\\u{A}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u{A}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "unicodeEscape": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{a}/u'
- '/\\u{A}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\u{a}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "unicodeEscape": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{A}/u'
- '/\\u{a}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\x0A/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\x0a/'
- '/\\x0A/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\x0A/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "hexadecimalEscape": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\x0a/'
- '/\\x0A/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\x0a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "hexadecimalEscape": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\x0A/'
- '/\\x0a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\ca/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\cA/u'
- '/\\ca/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\cA/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "controlEscape": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\ca/u'
- '/\\cA/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/\ca/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "controlEscape": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\cA/u'
- '/\\ca/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
const s = "\\u000A";
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "unicodeEscape": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s = "\\\\u000a";\n            new RegExp(s)'
- 'const s = "\\\\u000A";\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\q{Ab}]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{ab}]/iv'
- '/[\\q{Ab}]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\q{Ab}]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "caseInsensitive": "lowercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{ab}]/iv'
- '/[\\q{Ab}]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### letter-case > invalid

```js
/[\q{Ab}]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "caseInsensitive": "uppercase"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{AB}]/iv'
- '/[\\q{Ab}]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `match-any`

Pass: 21 / 40 (52.5%)
Fail: 19 / 40 (47.5%)
Skip: 0 / 40 (0.0%)

#### match-any > invalid

```js
/[\S\s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[\\S\\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\S\s]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/v'
- '/[\\S\\s]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\S\s\q{a|b|c}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/v'
- '/[\\S\\s\\q{a|b|c}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[[\S\s\q{abc}]--\q{abc}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/v'
- '/[[\\S\\s\\q{abc}]--\\q{abc}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[^]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[^]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\d\D]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[\\d\\D]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\0-\uFFFF]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[\\0-\\uFFFF]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\s\S][\S\s][^]./s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S][\\s\\S][\\s\\S]./s'
- '/[\\s\\S][\\S\\s][^]./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\s\S][\S\s][^]./s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "[^]"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^][^][^][^]/s'
- '/[\\s\\S][\\S\\s][^]./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\s\S] [\S\s] [^] ./s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "dotAll"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/. [\\S\\s] [^] ./s'
- '/[\\s\\S] [\\S\\s] [^] ./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/. [\S\s] [^] ./s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "dotAll"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/. . [^] ./s'
- '/. [\\S\\s] [^] ./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/. . [^] ./s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "dotAll"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/. . . ./s'
- '/. . [^] ./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js

            const s = "[\\s\\S][\\S\\s][^]."
            new RegExp(s, 's')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "[^]"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const s = "[^][^][^][^]"\n' +
-   '            const s = "[\\\\s\\\\S][\\\\S\\\\s][^]."\n' +
    "            new RegExp(s, 's')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js

            const s = "[\\s\\S]"+"[\\S\\s][^]."
            new RegExp(s, 's')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "[^]"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const s = "[^]"+"[^][^][^]"\n' +
-   '            const s = "[\\\\s\\\\S]"+"[\\\\S\\\\s][^]."\n' +
    "            new RegExp(s, 's')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\p{ASCII}\P{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/u'
- '/[\\p{ASCII}\\P{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\p{Script=Hiragana}\P{Script=Hiragana}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/u'
- '/[\\p{Script=Hiragana}\\P{Script=Hiragana}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\s\S\0-\uFFFF]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[\\s\\S\\0-\\uFFFF]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\w\D]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/'
- '/[\\w\\D]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### match-any > invalid

```js
/[\P{ASCII}\w\0-AZ-\xFF]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\S]/u'
- '/[\\P{ASCII}\\w\\0-AZ-\\xFF]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `negation`

Pass: 10 / 30 (33.3%)
Fail: 20 / 30 (66.7%)
Skip: 0 / 30 (0.0%)

#### negation > invalid

```js
/[^\d]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\D/'
- '/[^\\d]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\D]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d/'
- '/[^\\D]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\W/'
- '/[^\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\W]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/'
- '/[^\\W]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\S/'
- '/[^\\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\S]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\s/'
- '/[^\\S]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\p{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\P{ASCII}/u'
- '/[^\\p{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\P{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{ASCII}/u'
- '/[^\\P{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\p{Script=Hiragana}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\P{Script=Hiragana}/u'
- '/[^\\p{Script=Hiragana}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\P{Script=Hiragana}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{Script=Hiragana}/u'
- '/[^\\P{Script=Hiragana}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\P{Ll}]/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{Ll}/u;'
- '/[^\\P{Ll}]/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\P{White_Space}]/iu;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{White_Space}/iu;'
- '/[^\\P{White_Space}]/iu;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
const s ="[^\\w]"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s ="\\\\W"\n            new RegExp(s)'
- 'const s ="[^\\\\w]"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^\P{Lowercase_Letter}]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{Lowercase_Letter}/iv'
- '/[^\\P{Lowercase_Letter}]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^abc]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc]/v'
- '/[^[^abc]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^\q{a|1|A}&&\w]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a|1|A}&&\\w]/v'
- '/[^[^\\q{a|1|A}&&\\w]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^a]]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a]/iv'
- '/[^[^a]]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^\P{Lowercase_Letter}]]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\P{Lowercase_Letter}]/iv'
- '/[^[^\\P{Lowercase_Letter}]]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^[\p{Lowercase_Letter}&&[ABC]]]]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[\\p{Lowercase_Letter}&&[ABC]]]/iv'
- '/[^[^[\\p{Lowercase_Letter}&&[ABC]]]]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### negation > invalid

```js
/[^[^[\p{Lowercase_Letter}&&A]--B]]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[\\p{Lowercase_Letter}&&A]--B]/iv'
- '/[^[^[\\p{Lowercase_Letter}&&A]--B]]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-dupe-characters-character-class`

Pass: 22 / 59 (37.3%)
Fail: 37 / 59 (62.7%)
Skip: 0 / 59 (0.0%)

#### no-dupe-characters-character-class > invalid

```js
var re = /[\\(\\)]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'var re = /[\\\\()]/'
- 'var re = /[\\\\(\\\\)]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
var re = /[a-z\\s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'var re = /[a-z\\\\]/'
- 'var re = /[a-z\\\\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[aaa]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[aa]/'
- '/[aaa]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[0-9\d]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\d]/'
- '/[0-9\\d]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\f\u000C]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\f]/'
- '/[\\f\\u000C]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
RegExp(/[bb]/)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp(/[b]/)'
- 'RegExp(/[bb]/)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\s \f\n\r\t\v\u00a0\u1680\u180e\u2000-\u200a\u2028\u2029\u202f\u205f\u3000\ufeff]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\f\\r\\v\\u1680\\u180e\\u2028\\u202f\\u3000]/'
- '/[\\s \\f\\n\\r\\t\\v\\u00a0\\u1680\\u180e\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000\\ufeff]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\t	 \u0009]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\t ]/'
- '/[\\t\t \\u0009]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\wA-Z a-z:0-9,_]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w :,]/'
- '/[\\wA-Z a-z:0-9,_]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[!-z_abc-]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[!-zac]/'
- '/[!-z_abc-]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\w_abc-][\s \t\r\n\u2000\u3000]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\wac-][\\s\\t\\n\\u3000]/'
- '/[\\w_abc-][\\s \\t\\r\\n\\u2000\\u3000]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a-z a-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-z ]/'
- '/[a-z a-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a-z A-Z]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-z ]/i'
- '/[a-z A-Z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a-d e-h_d-e+c-d]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-d e-h_+]/'
- '/[a-d e-h_d-e+c-d]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[3-6 3-6_2-4+5-7]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ _2-4+5-7]/'
- '/[3-6 3-6_2-4+5-7]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[3-6 3-6_5-7]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[3-6 _5-7]/'
- '/[3-6 3-6_5-7]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\s\s \s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s ]/'
- '/[\\s\\s \\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\S\S \sa]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\S \\s]/'
- '/[\\S\\S \\sa]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\d 0-9_!-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ _!-z]/'
- '/[\\d 0-9_!-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\W\W\w \d\d\D]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\W\\w\\d\\D]/'
- '/[\\W\\W\\w \\d\\d\\D]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\p{ASCII}\P{ASCII}\p{Script=Hiragana}\P{Script=Hiragana}\p{ASCII}\p{Script=Hiragana}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\P{ASCII}\\P{Script=Hiragana}\\p{Script=Hiragana}]/u'
- '/[\\p{ASCII}\\P{ASCII}\\p{Script=Hiragana}\\P{Script=Hiragana}\\p{ASCII}\\p{Script=Hiragana}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\p{ASCII} abc\P{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}ac\\P{ASCII}]/u'
- '/[\\p{ASCII} abc\\P{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\P{Script=Hiragana} abc\p{Script=Hiragana}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\P{Script=Hiragana}ac\\p{Script=Hiragana}]/u'
- '/[\\P{Script=Hiragana} abc\\p{Script=Hiragana}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\w0-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[0-z]/'
- '/[\\w0-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\t-\uFFFF\s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\t-\\uFFFF]/'
- '/[\\t-\\uFFFF\\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\Sa]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\S]/'
- '/[\\Sa]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a-z\p{L}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{L}]/u'
- '/[a-z\\p{L}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\d\p{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}]/u'
- '/[\\d\\p{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\t\s]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s]/'
- '/[\\t\\s]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[A-Z a-\uFFFF]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ a-\\uFFFF]/i'
- '/[A-Z a-\\uFFFF]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a^\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^\\w]/'
- '/[a^\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[0a-a-9a-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[0\\-9a-z]/'
- '/[0a-a-9a-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[a:^\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[:^\\w]/'
- '/[a:^\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\sa-\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s-\\w]/'
- '/[\\sa-\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\x01\d-\x03\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\x01\\-\\x03\\w]/'
- '/[\\x01\\d-\\x03\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\q{a}aa-c[\w--b][\w&&a]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[aa-c[\\w--b]]/v'
- '/[\\q{a}aa-c[\\w--b][\\w&&a]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-dupe-characters-character-class > invalid

```js
/[\q{abc}\q{abc|ab}[\q{abc}--b][\q{abc}&&\q{abc|ab}]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{abc|ab}[\\q{abc}&&\\q{abc|ab}]]/v'
- '/[\\q{abc}\\q{abc|ab}[\\q{abc}--b][\\q{abc}&&\\q{abc|ab}]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-extra-lookaround-assertions`

Pass: 3 / 13 (23.1%)
Fail: 10 / 13 (76.9%)
Skip: 0 / 13 (0.0%)

#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScript'.replace(/Java(?=Scrip(?=t))/u, 'Type'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScript'.replace(/Java(?=Script)/u, 'Type'))"
- "console.log('JavaScript'.replace(/Java(?=Scrip(?=t))/u, 'Type'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScript'.replace(/(?<=(?<=J)ava)Script/u, ''))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScript'.replace(/(?<=Java)Script/u, ''))"
- "console.log('JavaScript'.replace(/(?<=(?<=J)ava)Script/u, ''))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScript Java JavaRuntime'.replace(/Java(?!Scrip(?=t))/gu, 'Python'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScript Java JavaRuntime'.replace(/Java(?!Script)/gu, 'Python'))"
- "console.log('JavaScript Java JavaRuntime'.replace(/Java(?!Scrip(?=t))/gu, 'Python'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScript TypeScript ActionScript'.replace(/(?<!(?<=J)ava)Script/gu, 'ScriptCompiler'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScript TypeScript ActionScript'.replace(/(?<!Java)Script/gu, 'ScriptCompiler'))"
- "console.log('JavaScript TypeScript ActionScript'.replace(/(?<!(?<=J)ava)Script/gu, 'ScriptCompiler'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=Checker|Linter))/gu, 'Type'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?:Checker|Linter))/gu, 'Type'))"
- "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=Checker|Linter))/gu, 'Type'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=(?:Check|Lint)er))/gu, 'Type'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?:Check|Lint)er)/gu, 'Type'))"
- "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=(?:Check|Lint)er))/gu, 'Type'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('ESLint JSLint TSLint'.replace(/(?<=(?<=J|T)S)Lint/gu, '-Runtime'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('ESLint JSLint TSLint'.replace(/(?<=(?:J|T)S)Lint/gu, '-Runtime'))"
- "console.log('ESLint JSLint TSLint'.replace(/(?<=(?<=J|T)S)Lint/gu, '-Runtime'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=Checker)|Script(?=Linter))/gu, 'Type'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=ScriptChecker|ScriptLinter)/gu, 'Type'))"
- "console.log('JavaScriptChecker JavaScriptLinter'.replace(/Java(?=Script(?=Checker)|Script(?=Linter))/gu, 'Type'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('ESLint JSLint TSLint'.replace(/(?<=(?<=J)S|(?<=T)S)Lint/gu, '-Runtime'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('ESLint JSLint TSLint'.replace(/(?<=JS|TS)Lint/gu, '-Runtime'))"
- "console.log('ESLint JSLint TSLint'.replace(/(?<=(?<=J)S|(?<=T)S)Lint/gu, '-Runtime'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-extra-lookaround-assertions > invalid

```js
console.log('JavaScript'.replace(/Java(?=Scrip(?=[\q{t}]))/v, 'Type'))
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "console.log('JavaScript'.replace(/Java(?=Scrip[\\q{t}])/v, 'Type'))"
- "console.log('JavaScript'.replace(/Java(?=Scrip(?=[\\q{t}]))/v, 'Type'))"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-invisible-character`

Pass: 11 / 17 (64.7%)
Fail: 6 / 17 (35.3%)
Skip: 0 / 17 (0.0%)

#### no-invisible-character > invalid

```js
/ /
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\xa0/' !== '/ /'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-invisible-character > invalid

```js
/[	]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\t]/'
- '/[\t]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-invisible-character > invalid

```js
/[	  ᠎             　﻿​]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\t \\u1680᠎\\u2000 \\u2002 \\u2004 \\u2006 \\u2008 \\u200a \\u205f　\\ufeff\x85\\u200b]/'
- '/[\t  ᠎             　﻿\x85​]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-invisible-character > invalid

```js
/[\t \u1680᠎\u2000 \u2002 \u2004 \u2006 \u2008 \u200a \u205f　\ufeff\u200b]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\t\\xa0\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000\\ufeff\\x85\\u200b]/'
- '/[\\t \\u1680᠎\\u2000 \\u2002 \\u2004 \\u2006 \\u2008 \\u200a \\u205f　\\ufeff\x85\\u200b]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-invisible-character > invalid

```js
new RegExp('	  ᠎             　﻿​')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "new RegExp('\\t \\u1680᠎\\u2000 \\u2002 \\u2004 \\u2006 \\u2008 \\u200a \\u205f　\\ufeff\x85\\u200b')"
- "new RegExp('\t  ᠎             　﻿\x85​')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-invisible-character > invalid

```js
/[\q{	}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{\\t}]/v'
- '/[\\q{\t}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


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


### `no-misleading-unicode-character`

Pass: 32 / 67 (47.8%)
Fail: 35 / 67 (52.2%)
Skip: 0 / 67 (0.0%)

#### no-misleading-unicode-character > invalid

```js
/[👍]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/👍/' !== '/[👍]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👍]foo/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/👍foo/'
- '/[👍]foo/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👍]|foo/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/👍|foo/'
- '/[👍]|foo/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👍a]foo/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👍|[a])foo/'
- '/[👍a]foo/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👍a]|foo/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/👍|[a]|foo/'
- '/[👍a]|foo/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[fooo👍bar]baz/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👍|[fooobar])baz/'
- '/[fooo👍bar]baz/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/👍+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👍)+/'
- '/👍+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[Á]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/Á/' !== '/[Á]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[Á]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/Á/u' !== '/[Á]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[❇️]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/❇️/' !== '/[❇️]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[❇️]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/❇️/u' !== '/[❇️]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/❇️+/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:❇️)+/u'
- '/❇️+/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[🇯🇵]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/🇯🇵/'
- '/[🇯🇵]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[🇯🇵]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/🇯🇵/u'
- '/[🇯🇵]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👨‍👩‍👦]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/👨‍👩‍👦/'
- '/[👨‍👩‍👦]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👨‍👩‍👦]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/👨‍👩‍👦/u'
- '/[👨‍👩‍👦]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/👨‍👩‍👦+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👨‍👩‍👦)+/'
- '/👨‍👩‍👦+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/👨‍👩‍👦+/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👨‍👩‍👦)+/u'
- '/👨‍👩‍👦+/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[竈門禰󠄀豆子]|[煉󠄁獄杏寿郎]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/禰󠄀|[竈門豆子]|煉󠄁|[獄杏寿郎]/'
- '/[竈門禰󠄀豆子]|[煉󠄁獄杏寿郎]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[👍]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("👍", "")'
- 'new RegExp("[👍]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[\uD83D\uDC4D]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("👍", "")'
- 'new RegExp("[\\uD83D\\uDC4D]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[Á]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("Á", "")'
- 'new RegExp("[Á]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[Á]", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("Á", "u")'
- 'new RegExp("[Á]", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[❇️]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("❇️", "")'
- 'new RegExp("[❇️]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[❇️]", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("❇️", "u")'
- 'new RegExp("[❇️]", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[🇯🇵]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("🇯🇵", "")'
- 'new RegExp("[🇯🇵]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[🇯🇵]", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("🇯🇵", "u")'
- 'new RegExp("[🇯🇵]", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[👨‍👩‍👦]", "")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("👨‍👩‍👦", "")'
- 'new RegExp("[👨‍👩‍👦]", "")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[👨‍👩‍👦]", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("👨‍👩‍👦", "u")'
- 'new RegExp("[👨‍👩‍👦]", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[[👶🏻]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[\\q{👶🏻}]]/v'
- '/[[👶🏻]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👶🏻[👨‍👩‍👦]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{👶🏻}[👨‍👩‍👦]]/v'
- '/[👶🏻[👨‍👩‍👦]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👶🏻👨‍👩‍👦]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{👶🏻|👨‍👩‍👦}]/v'
- '/[👶🏻👨‍👩‍👦]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/[👶🏻&👨‍👩‍👦]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{👶🏻|👨‍👩‍👦}&]/v'
- '/[👶🏻&👨‍👩‍👦]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
new RegExp("[👨‍👩‍👦]", "v")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("[\\\\q{👨‍👩‍👦}]", "v")'
- 'new RegExp("[👨‍👩‍👦]", "v")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-misleading-unicode-character > invalid

```js
/👨‍👩‍👦+/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:👨‍👩‍👦)+/v'
- '/👨‍👩‍👦+/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-missing-g-flag`

Pass: 14 / 22 (63.6%)
Fail: 8 / 22 (36.4%)
Skip: 0 / 22 (0.0%)

#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.replaceAll(/foo/, 'bar')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   "            const ret = s.replaceAll(/foo/g, 'bar')\n" +
-   "            const ret = s.replaceAll(/foo/, 'bar')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.matchAll(/foo/)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   '            const ret = s.matchAll(/foo/g)\n' +
-   '            const ret = s.matchAll(/foo/)\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.replaceAll(new RegExp('foo'), 'bar')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   `            const ret = s.replaceAll(new RegExp('foo', "g"), 'bar')\n` +
-   "            const ret = s.replaceAll(new RegExp('foo'), 'bar')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.matchAll(new RegExp('foo'))
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   `            const ret = s.matchAll(new RegExp('foo', "g"))\n` +
-   "            const ret = s.matchAll(new RegExp('foo'))\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.matchAll(new RegExp(foo))
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   '            const ret = s.matchAll(new RegExp(foo, "g"))\n' +
-   '            const ret = s.matchAll(new RegExp(foo))\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.matchAll(new RegExp('{', 'u'))
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   "            const ret = s.matchAll(new RegExp('{', 'ug'))\n" +
-   "            const ret = s.matchAll(new RegExp('{', 'u'))\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const ret = s.replaceAll(/foo/, 'bar')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "\n            const ret = s.replaceAll(/foo/g, 'bar')\n            "
- "\n            const ret = s.replaceAll(/foo/, 'bar')\n            "

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-missing-g-flag > invalid

```js

            const s = 'foo'
            const ret = s.replaceAll(/[\q{foo}]/v, 'bar')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const s = 'foo'\n" +
+   "            const ret = s.replaceAll(/[\\q{foo}]/vg, 'bar')\n" +
-   "            const ret = s.replaceAll(/[\\q{foo}]/v, 'bar')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-super-linear-backtracking`

Pass: 6 / 10 (60.0%)
Fail: 4 / 10 (40.0%)
Skip: 0 / 10 (0.0%)

#### no-super-linear-backtracking > invalid

```js
/b(?:a+)+b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/ba+b/'
- '/b(?:a+)+b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-super-linear-backtracking > invalid

```js
/\ba+a+$/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\ba{2,}$/'
- '/\\ba+a+$/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-super-linear-backtracking > invalid

```js
/\b\w+a\w+$/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b\\w[\\dA-Z_b-z]*a\\w+$/'
- '/\\b\\w+a\\w+$/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-super-linear-backtracking > invalid

```js
/[\q{a}]*b?[\q{a}]+$/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\q{a}]+(?:b[\\q{a}]+)?|b[\\q{a}]+)$/v'
- '/[\\q{a}]*b?[\\q{a}]+$/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-trivially-nested-assertion`

Pass: 7 / 31 (22.6%)
Fail: 24 / 31 (77.4%)
Skip: 0 / 31 (0.0%)

#### no-trivially-nested-assertion > invalid

```js
/(?=$)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/$/' !== '/(?=$)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=^)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/^/' !== '/(?=^)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=$)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/$/' !== '/(?<=$)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=^)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/^/' !== '/(?<=^)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b/'
- '/(?=\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?!\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\B/'
- '/(?!\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b/'
- '/(?<=\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<!\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\B/'
- '/(?<!\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=(?=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=a)/'
- '/(?=(?=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=(?!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?!a)/'
- '/(?=(?!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=(?<=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=a)/'
- '/(?=(?<=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?=(?<!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<!a)/'
- '/(?=(?<!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?!(?=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?!a)/'
- '/(?!(?=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?!(?!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=a)/'
- '/(?!(?!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?!(?<=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<!a)/'
- '/(?!(?<=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?!(?<!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=a)/'
- '/(?!(?<!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=(?=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=a)/'
- '/(?<=(?=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=(?!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?!a)/'
- '/(?<=(?!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=(?<=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=a)/'
- '/(?<=(?<=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<=(?<!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<!a)/'
- '/(?<=(?<!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<!(?=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?!a)/'
- '/(?<!(?=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<!(?!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=a)/'
- '/(?<!(?!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<!(?<=a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<!a)/'
- '/(?<!(?<=a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-assertion > invalid

```js
/(?<!(?<!a))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=a)/'
- '/(?<!(?<!a))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-trivially-nested-quantifier`

Pass: 5 / 18 (27.8%)
Fail: 13 / 18 (72.2%)
Skip: 0 / 18 (0.0%)

#### no-trivially-nested-quantifier > invalid

```js
/(?:a?)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a*/'
- '/(?:a?)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{1,2})*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a*/'
- '/(?:a{1,2})*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{1,2})+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+/'
- '/(?:a{1,2})+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{1,2}){3,4}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{3,8}/'
- '/(?:a{1,2}){3,4}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{2,}){4}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{8,}/'
- '/(?:a{2,}){4}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{4,}){5}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{20,}/'
- '/(?:a{4,}){5}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{3}){4}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{12}/'
- '/(?:a{3}){4}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a+|b)*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a|b)*/'
- '/(?:a+|b)*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a?|b)*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a|b)*/'
- '/(?:a?|b)*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{0,4}|b)*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a|b)*/'
- '/(?:a{0,4}|b)*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{0,4}|b)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a?|b)+/'
- '/(?:a{0,4}|b)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:a{0,4}?|b)+?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a??|b)+?/'
- '/(?:a{0,4}?|b)+?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-trivially-nested-quantifier > invalid

```js
/(?:[\q{a}]+)+/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a}]+/v'
- '/(?:[\\q{a}]+)+/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-unused-capturing-group`

Pass: 58 / 67 (86.6%)
Fail: 9 / 67 (13.4%)
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


#### no-unused-capturing-group > invalid

```js
'2000-12-31'.replace(/(\d{4})-(\d{2})-(\d{2})/, 'Date') 
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "'2000-12-31'.replace(/(?:\\d{4})-(?:\\d{2})-(?:\\d{2})/, 'Date') "
- "'2000-12-31'.replace(/(\\d{4})-(\\d{2})-(\\d{2})/, 'Date') "

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js
'2000-12-31'.replace(/(\d{4})-(\d{2})-(\d{2})/, '$1/$2') // "2000/12"
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `'2000-12-31'.replace(/(\\d{4})-(\\d{2})-(?:\\d{2})/, '$1/$2') // "2000/12"`
- `'2000-12-31'.replace(/(\\d{4})-(\\d{2})-(\\d{2})/, '$1/$2') // "2000/12"`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js
'2000-12-31'.replace(/(?<y>\d{4})-(?<m>\d{2})-(?<d>\d{2})/u, '$<y>/$<m>') // "2000/12"
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `'2000-12-31'.replace(/(?<y>\\d{4})-(?<m>\\d{2})-(?:\\d{2})/u, '$<y>/$<m>') // "2000/12"`
- `'2000-12-31'.replace(/(?<y>\\d{4})-(?<m>\\d{2})-(?<d>\\d{2})/u, '$<y>/$<m>') // "2000/12"`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js

            const bs = [...'abc_abc'.matchAll(/a(b)/g)].map(m => m[0])
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const bs = [...'abc_abc'.matchAll(/a(?:b)/g)].map(m => m[0])\n" +
-   "            const bs = [...'abc_abc'.matchAll(/a(b)/g)].map(m => m[0])\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js

            ;[...'abc_abc'.matchAll(/a(b)(c)/g)].forEach(m => console.log(m[1]))
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            ;[...'abc_abc'.matchAll(/a(b)(?:c)/g)].forEach(m => console.log(m[1]))\n" +
-   "            ;[...'abc_abc'.matchAll(/a(b)(c)/g)].forEach(m => console.log(m[1]))\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js

            const text = 'Lorem ipsum dolor sit amet'
            const replaced = text.replace(/([\q{Lorem|ipsum}])/gv, '**Lorem ipsum**')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'Lorem ipsum dolor sit amet'\n" +
+   "            const replaced = text.replace(/(?:[\\q{Lorem|ipsum}])/gv, '**Lorem ipsum**')\n" +
-   "            const replaced = text.replace(/([\\q{Lorem|ipsum}])/gv, '**Lorem ipsum**')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js
const re = /(foo)/d
            console.log(re.exec('foo').indices[2])
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const re = /(?:foo)/d\n' +
- 'const re = /(foo)/d\n' +
    "            console.log(re.exec('foo').indices[2])\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-unused-capturing-group > invalid

```js
const re = /(?<f>foo)/d
            console.log(re.exec('foo').indices.groups.x)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "fixable": true
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const re = /(?:foo)/d\n' +
- 'const re = /(?<f>foo)/d\n' +
    "            console.log(re.exec('foo').indices.groups.x)\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-character-class`

Pass: 29 / 61 (47.5%)
Fail: 32 / 61 (52.5%)
Skip: 0 / 61 (0.0%)

#### no-useless-character-class > invalid

```js
/[a]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/' !== '/[a]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[a-a]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/' !== '/[a-a]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[\d]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\d/' !== '/[\\d]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[=]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/=/' !== '/[=]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[\D]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": [
        "\\d"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\D/' !== '/[\\D]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/(,)[\0]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(,)\\0/'
- '/(,)[\\0]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/(,)[\01]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(,)\\01/'
- '/(,)[\\01]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[.] [*] [+] [?] [\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\]] [/] [\\]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\. \\* \\+ \\? \\^ = ! : \\$ \\{ } \\( \\) \\| \\[ \\] \\/ \\\\/'
- '/[.] [*] [+] [?] [\\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\\]] [/] [\\\\]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[.] [*] [+] [?] [\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\]] [/] [\\]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\. \\* \\+ \\? \\^ = ! : \\$ \\{ \\} \\( \\) \\| \\[ \\] \\/ \\\\/u'
- '/[.] [*] [+] [?] [\\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\\]] [/] [\\\\]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[.] [*] [+] [?] [\^] [=] [!] [:] [$] [\{] [\}] [\(] [\)] [\|] [\[] [\]] [\/] [\\]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\. \\* \\+ \\? \\^ = ! : \\$ \\{ \\} \\( \\) \\| \\[ \\] \\/ \\\\/v'
- '/[.] [*] [+] [?] [\\^] [=] [!] [:] [$] [\\{] [\\}] [\\(] [\\)] [\\|] [\\[] [\\]] [\\/] [\\\\]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[.-.]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\./u'
- '/[.-.]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
new RegExp("[.] [*] [+] [?] [\\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\\]] [/] [\\\\]", "u")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("\\\\. \\\\* \\\\+ \\\\? \\\\^ = ! : \\\\$ \\\\{ \\\\} \\\\( \\\\) \\\\| \\\\[ \\\\] \\\\/ \\\\\\\\", "u")'
- 'new RegExp("[.] [*] [+] [?] [\\\\^] [=] [!] [:] [$] [{] [}] [(] [)] [|] [[] [\\\\]] [/] [\\\\\\\\]", "u")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
new RegExp("[.] [*] [+] [?] [\\^] [=] [!] [:]" + " [$] [{] [}] [(] [)] [|] [[] [\\]] [/] [\\\\]")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("\\\\. \\\\* \\\\+ \\\\? \\\\^ = ! :" + " \\\\$ \\\\{ } \\\\( \\\\) \\\\| \\\\[ \\\\] \\\\/ \\\\\\\\")'
- 'new RegExp("[.] [*] [+] [?] [\\\\^] [=] [!] [:]" + " [$] [{] [}] [(] [)] [|] [[] [\\\\]] [/] [\\\\\\\\]")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
RegExp("[\"]" + '[\']')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `RegExp("\\"" + '\\'')`
- `RegExp("[\\"]" + '[\\']')`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[ [.] [*] [+] [?] [\^] [=] [!] [:] [$] [\{] [\}] [\(] [\)] [\|] [\[] [\]] [\/] [\\] ]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "ignores": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ . * + ? \\^ = ! : $ \\{ \\} \\( \\) \\| \\[ \\] \\/ \\\\ ]/v'
- '/[ [.] [*] [+] [?] [\\^] [=] [!] [:] [$] [\\{] [\\}] [\\(] [\\)] [\\|] [\\[] [\\]] [\\/] [\\\\] ]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[\^]A]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^A]/v'
- '/[[\\^]A]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[A]--[B]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A--B]/v'
- '/[[A]--[B]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[A[&]&B]/v; /[A&&[&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A\\&&B]/v; /[A&&\\&]/v'
- '/[A[&]&B]/v; /[A&&[&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[A[&-&]&B]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A\\&&B]/v'
- '/[A[&-&]&B]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[&]&&[&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\&&&\\&]/v'
- '/[[&]&&[&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[abc]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc]/v'
- '/[[abc]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[A&&B]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A&&B]/v'
- '/[[A&&B]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[\q{abc}]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{abc}]/v'
- '/[[\\q{abc}]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[^\w&&\d]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^\\w&&\\d]/v'
- '/[[^\\w&&\\d]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[^[abc]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^abc]/v'
- '/[^[abc]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[^[abc]d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^abcd]/v'
- '/[^[abc]d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[abc][def]ghi]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc[def]ghi]/v'
- '/[[abc][def]ghi]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[abc[def]ghi]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcdefghi]/v'
- '/[abc[def]ghi]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[abc&]&[&bd]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc\\&&\\&bd]/v'
- '/[[abc&]&[&bd]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[abc!-&]&[&-1bd]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc!-\\&&\\&-1bd]/v'
- '/[[abc!-&]&[&-1bd]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[[]^]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^]/v'
- '/[[]^]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-character-class > invalid

```js
/[&[]&]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[&\\&]/v'
- '/[&[]&]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-escape`

Pass: 111 / 143 (77.6%)
Fail: 32 / 143 (22.4%)
Skip: 0 / 143 (0.0%)

#### no-useless-escape > invalid

```js
/\a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/' !== '/\\a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[ \^ \/ \. \$ \* \+ \? \[ \{ \} \| \( \) \k<title> \B \8 \9]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ ^ / . $ * + ? [ { } | ( ) \\k<title> B 8 9]/'
- '/[ \\^ \\/ \\. \\$ \\* \\+ \\? \\[ \\{ \\} \\| \\( \\) \\k<title> \\B \\8 \\9]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\$]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[$]/v'
- '/[\\$]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\&\&]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[&\\&]/v'
- '/[\\&\\&]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\!\!]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[!\\!]/v'
- '/[\\!\\!]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\#\#]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[#\\#]/v'
- '/[\\#\\#]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\%\%]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[%\\%]/v'
- '/[\\%\\%]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\*\*]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[*\\*]/v'
- '/[\\*\\*]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\+\+]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[+\\+]/v'
- '/[\\+\\+]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\,\,]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[,\\,]/v'
- '/[\\,\\,]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\.\.]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[.\\.]/v'
- '/[\\.\\.]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\:\:]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[:\\:]/v'
- '/[\\:\\:]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\;\;]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[;\\;]/v'
- '/[\\;\\;]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\<\<]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[<\\<]/v'
- '/[\\<\\<]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\=\=]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[=\\=]/v'
- '/[\\=\\=]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\>\>]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[>\\>]/v'
- '/[\\>\\>]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\?\?]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[?\\?]/v'
- '/[\\?\\?]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\@\@]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[@\\@]/v'
- '/[\\@\\@]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\`\`]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[`\\`]/v'
- '/[\\`\\`]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\~\~]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[~\\~]/v'
- '/[\\~\\~]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[^\^\^]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^^\\^]/v'
- '/[^\\^\\^]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[_\^\^]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[_^\\^]/v'
- '/[_\\^\\^]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[^\^]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^^]/v'
- '/[^\\^]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\&\&&\&]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[&\\&&\\&]/v'
- '/[\\&\\&&\\&]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\p{ASCII}--\.]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}--.]/v'
- '/[\\p{ASCII}--\\.]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\p{ASCII}&&\.]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}&&.]/v'
- '/[\\p{ASCII}&&\\.]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\.--[.&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[.--[.&]]/v'
- '/[\\.--[.&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\.&&[.&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[.&&[.&]]/v'
- '/[\\.&&[.&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\.--\.--\.]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[.--.--.]/v'
- '/[\\.--\\.--\\.]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[\.&&\.&&\.]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[.&&.&&.]/v'
- '/[\\.&&\\.&&\\.]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[[\.&]--[\.&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[.&]--[.&]]/v'
- '/[[\\.&]--[\\.&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-escape > invalid

```js
/[[\.&]&&[\.&]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[.&]&&[.&]]/v'
- '/[[\\.&]&&[\\.&]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-flag`

Pass: 55 / 82 (67.1%)
Fail: 27 / 82 (32.9%)
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
/\w \W \s \S \d \D . \b/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w \\W \\s \\S \\d \\D . \\b/'
- '/\\w \\W \\s \\S \\d \\D . \\b/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
/K/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/K/' !== '/K/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
/\w/m
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\w/' !== '/\\w/m'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
/\w/s
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\w/' !== '/\\w/s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
/\w/ims
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/ms'
- '/\\w/ims'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
/\w/ms
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/s'
- '/\\w/ms'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            /* ✓ GOOD */
            const regex1 = /foo/g;
            const str = 'table football, foosball';
            while ((array = regex1.exec(str)) !== null) {
              //
            }

            const regex2 = /foo/g;
            regex2.test(string);
            regex2.test(string);

            str.replace(/foo/g, 'bar');
            str.replaceAll(/foo/g, 'bar');

            /* ✗ BAD */
            /foo/g.test(string);
            const regex3 = /foo/g;
            regex3.test(string); // You have used it only once.

            /foo/g.exec(string);
            const regex4 = /foo/g;
            regex4.exec(string); // You have used it only once.

            new RegExp('foo', 'g').test(string);

            str.search(/foo/g);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected
... Skipped lines

  '\n' +
    '            /* ✓ GOOD */\n' +
    '            const regex1 = /foo/g;\n' +
    "            const str = 'table football, foosball';\n" +
    '            while ((array = regex1.exec(str)) !== null) {\n' +
...
    '            /* ✗ BAD */\n' +
+   '            /foo/.test(string);\n' +
-   '            /foo/g.test(string);\n' +
    '            const regex3 = /foo/g;\n' +
    '            regex3.test(string); // You have used it only once.\n' +
    '\n' +
+   '            /foo/.exec(string);\n' +
-   '            /foo/g.exec(string);\n' +
    '            const regex4 = /foo/g;\n' +
    '            regex4.exec(string); // You have used it only once.\n' +
    '\n' +
+   "            new RegExp('foo', '').test(string);\n" +
-   "            new RegExp('foo', 'g').test(string);\n" +
    '\n' +
+   '            str.search(/foo/);\n' +
-   '            str.search(/foo/g);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            /foo/g.test(bar);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            /foo/.test(bar);\n            '
- '\n            /foo/g.test(bar);\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            /foo/g.exec(bar);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            /foo/.exec(bar);\n            '
- '\n            /foo/g.exec(bar);\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            new RegExp('foo', 'g').test(bar);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "\n            new RegExp('foo', '').test(bar);\n            "
- "\n            new RegExp('foo', 'g').test(bar);\n            "

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            "foo foo".search(/foo/g);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            "foo foo".search(/foo/);\n            '
- '\n            "foo foo".search(/foo/g);\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            const notStr = {}
            notStr.split(/foo/g);

            maybeStr.split(/foo/g);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const notStr = {}\n' +
    '            notStr.split(/foo/g);\n' +
    '\n' +
+   '            maybeStr.split(/foo/);\n' +
-   '            maybeStr.split(/foo/g);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            /** @param {object} obj */
            function fn1 (obj) {
                obj.search(/foo/g);
            }
            function fn2 (maybeStr) {
                maybeStr.split(/foo/g);
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            /** @param {object} obj */\n' +
    '            function fn1 (obj) {\n' +
    '                obj.search(/foo/g);\n' +
    '            }\n' +
    '            function fn2 (maybeStr) {\n' +
+   '                maybeStr.split(/foo/);\n' +
-   '                maybeStr.split(/foo/g);\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            /* ✓ GOOD */
            const regex1 = /foo/y;
            const str = 'table football, foosball';
            regex1.lastIndex = 6
            var array = regex1.exec(str)

            const regex2 = /foo/y;
            regex2.test(string);
            regex2.test(string);

            str.replace(/foo/y, 'bar');
            str.replaceAll(/foo/gy, 'bar');

            const regexp3 = /foo/y
            str.search(regexp3)

            /* ✗ BAD */
            str.split(/foo/y);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected
... Skipped lines

  '\n' +
    '            /* ✓ GOOD */\n' +
    '            const regex1 = /foo/y;\n' +
    "            const str = 'table football, foosball';\n" +
    '            regex1.lastIndex = 6\n' +
...
    '            /* ✗ BAD */\n' +
+   '            str.split(/foo/);\n' +
-   '            str.split(/foo/y);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            "str".split(/foo/y)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            "str".split(/foo/)\n            '
- '\n            "str".split(/foo/y)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            "str".split(new RegExp('foo', 'y'));
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `\n            "str".split(new RegExp('foo', ''));\n            `
- `\n            "str".split(new RegExp('foo', 'y'));\n            `

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            const notStr = {}
            notStr.split(/foo/y);

            maybeStr.split(/foo/y);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const notStr = {}\n' +
    '            notStr.split(/foo/y);\n' +
    '\n' +
+   '            maybeStr.split(/foo/);\n' +
-   '            maybeStr.split(/foo/y);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


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
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const orig = /\\w/i; // eslint-disable-line\n' +
+   "            const clone = new RegExp(orig, '');\n" +
-   "            const clone = new RegExp(orig, 'i');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            const orig = /\w/iy;
            const clone = new RegExp(orig, '');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const orig = /\\w/;\n' +
-   '            const orig = /\\w/iy;\n' +
    "            const clone = new RegExp(orig, '');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            const orig = /\w/i;
            const clone = new RegExp(orig);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const orig = /\\w/;\n' +
-   '            const orig = /\\w/i;\n' +
    '            const clone = new RegExp(orig);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
RegExp(/\w/i);
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp(/\\w/);'
- 'RegExp(/\\w/i);'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js

            const orig = /\w/;
            const clone = new RegExp(orig, 'i');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const orig = /\\w/;\n' +
+   "            const clone = new RegExp(orig, '');\n" +
-   "            const clone = new RegExp(orig, 'i');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-flag > invalid

```js
RegExp(/\w/imu.source);
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp(/\\w/u.source);'
- 'RegExp(/\\w/imu.source);'

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


### `no-useless-lazy`

Pass: 9 / 23 (39.1%)
Fail: 14 / 23 (60.9%)
Skip: 0 / 23 (0.0%)

#### no-useless-lazy > invalid

```js
/a{1}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{1}/'
- '/a{1}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/a{1}?/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{1}/v'
- '/a{1}?/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/a{4}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{4}/'
- '/a{4}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/a{2,2}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{2,2}/'
- '/a{2,2}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
const s = "\\d{1}?"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s = "\\\\d{1}"\n            new RegExp(s)'
- 'const s = "\\\\d{1}?"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
const s = "\\d"+"{1}?"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s = "\\\\d"+"{1}"\n            new RegExp(s)'
- 'const s = "\\\\d"+"{1}?"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/a+?b+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+b+/'
- '/a+?b+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/[\q{aa|ab}]+?b+/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{aa|ab}]+b+/v'
- '/[\\q{aa|ab}]+?b+/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/a*?b+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a*b+/'
- '/a*?b+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/(?:a|cd)+?(?:b+|zzz)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a|cd)+(?:b+|zzz)/'
- '/(?:a|cd)+?(?:b+|zzz)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/\b\w+?(?=\W)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b\\w+(?=\\W)/'
- '/\\b\\w+?(?=\\W)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/\b\w+?(?!\w)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b\\w+(?!\\w)/'
- '/\\b\\w+?(?!\\w)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/\b\w+?\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b\\w+\\b/'
- '/\\b\\w+?\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-lazy > invalid

```js
/\b\w+?$/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b\\w+$/'
- '/\\b\\w+?$/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-non-capturing-group`

Pass: 35 / 54 (64.8%)
Fail: 19 / 54 (35.2%)
Skip: 0 / 54 (0.0%)

#### no-useless-non-capturing-group > invalid

```js
/(?:abcd)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/abcd/.test(str)'
- '/(?:abcd)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:abcd)/v.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/abcd/v.test(str)'
- '/(?:abcd)/v.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:[abcd])/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcd]/.test(str)'
- '/(?:[abcd])/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:ab|cd)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/ab|cd/.test(str)'
- '/(?:ab|cd)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/a(?:ab|(?:.|a|b))/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a(?:ab|.|a|b)/'
- '/a(?:ab|(?:.|a|b))/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:[abcd]+?)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcd]+?/.test(str)'
- '/(?:[abcd]+?)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:0)/.test(str); /\1(?:0)/.test(str); /(?:1)/.test(str); /\1(?:1)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/0/.test(str); /\\1(?:0)/.test(str); /1/.test(str); /\\1(?:1)/.test(str)'
- '/(?:0)/.test(str); /\\1(?:0)/.test(str); /(?:1)/.test(str); /\\1(?:1)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:a\n)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\n/.test(str)'
- '/(?:a\\n)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js

            const s = "(?:a\\n)"
            ;(new RegExp(s)).test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "a\\\\n"\n            ;(new RegExp(s)).test(str)'
- '\n            const s = "(?:a\\\\n)"\n            ;(new RegExp(s)).test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:a)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a/.test(str)'
- '/(?:a)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:a)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a+/' !== '/(?:a)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:\w)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/.test(str)'
- '/(?:\\w)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:[abc])*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc]*/'
- '/(?:[abc])*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/foo(?:[abc]*)bar/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/foo[abc]*bar/'
- '/foo(?:[abc]*)bar/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/foo(?:bar)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/foobar/'
- '/foo(?:bar)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/(?:a|b)/.test(str)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a|b/.test(str)'
- '/(?:a|b)/.test(str)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/a|(?:b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a|b|c/'
- '/a|(?:b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js
/a|(?:b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allowTop": "always"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a|b|c/'
- '/a|(?:b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-non-capturing-group > invalid

```js

            const foo = /(?:a|b)/
            const bar = new RegExp('(?:a|b)' + 'c')
            foo.exec(str)
            bar.exec(str)
            // { allowTop: "partial" }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allowTop": "partial"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const foo = /a|b/\n' +
-   '            const foo = /(?:a|b)/\n' +
    "            const bar = new RegExp('(?:a|b)' + 'c')\n" +
    '            foo.exec(str)\n' +
    '            bar.exec(str)\n' +
    '            // { allowTop: "partial" }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-quantifier`

Pass: 18 / 20 (90.0%)
Fail: 2 / 20 (10.0%)
Skip: 0 / 20 (0.0%)

#### no-useless-quantifier > invalid

```js
/a{1}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/' !== '/a{1}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-quantifier > invalid

```js
/a{1,1}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/' !== '/a{1,1}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-range`

Pass: 4 / 13 (30.8%)
Fail: 9 / 13 (69.2%)
Skip: 0 / 13 (0.0%)

#### no-useless-range > invalid

```js
/[a-a]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/[a]/' !== '/[a-a]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js
/[a-a]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a]/v'
- '/[a-a]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js
/[a-b]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab]/'
- '/[a-b]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js
/[a-b]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab]/v'
- '/[a-b]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js
/[a-a-c-c]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\-c]/'
- '/[a-a-c-c]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js
/[a-abc]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc]/'
- '/[a-abc]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js

            const s = "[a-a-c]"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "[a\\\\-c]"\n            new RegExp(s)'
- '\n            const s = "[a-a-c]"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js

            /[,--b]/;
            /[a-a-z]/;
            /[a-a--z]/;
            /[\c-d]/;
            /[\x6-7]/;
            /[\u002-3]/;
            /[A-\u004-5]/;
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            /[,\\-b]/;\n' +
+   '            /[a\\-z]/;\n' +
+   '            /[a\\--z]/;\n' +
-   '            /[,--b]/;\n' +
-   '            /[a-a-z]/;\n' +
-   '            /[a-a--z]/;\n' +
    '            /[\\c-d]/;\n' +
    '            /[\\x6-7]/;\n' +
    '            /[\\u002-3]/;\n' +
    '            /[A-\\u004-5]/;\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-range > invalid

```js

            /[,-\-b]/;
            /[c-d]/;
            /[x6-7]/;
            /[\x 6-7]/;
            /[u002-3]/;
            /[\u 002-3]/;
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            /[,\\-b]/;\n' +
+   '            /[cd]/;\n' +
+   '            /[x67]/;\n' +
+   '            /[\\x 67]/;\n' +
+   '            /[u0023]/;\n' +
+   '            /[\\u 0023]/;\n' +
-   '            /[,-\\-b]/;\n' +
-   '            /[c-d]/;\n' +
-   '            /[x6-7]/;\n' +
-   '            /[\\x 6-7]/;\n' +
-   '            /[u002-3]/;\n' +
-   '            /[\\u 002-3]/;\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-set-operand`

Pass: 1 / 14 (7.1%)
Fail: 13 / 14 (92.9%)
Skip: 0 / 14 (0.0%)

#### no-useless-set-operand > invalid

```js
/[\w&&\d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\d]/v'
- '/[\\w&&\\d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w&&\s]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[]/v'
- '/[\\w&&\\s]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[^\w&&\s]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^]/v'
- '/[^\\w&&\\s]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w&&[\d\s]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w&&[\\d]]/v'
- '/[\\w&&[\\d\\s]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w&&[^\d\s]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w&&[^\\d]]/v'
- '/[\\w&&[^\\d\\s]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--\s]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w]/v'
- '/[\\w--\\s]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\d--\w]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[]/v'
- '/[\\d--\\w]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[^\d--\w]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^]/v'
- '/[^\\d--\\w]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--[\d\s]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--[\\d]]/v'
- '/[\\w--[\\d\\s]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--[^\d\s]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--[^\\d]]/v'
- '/[\\w--[^\\d\\s]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--[a\q{aa|b}]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--[a\\q{b}]]/v'
- '/[\\w--[a\\q{aa|b}]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--[a\q{aa}]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--[a]]/v'
- '/[\\w--[a\\q{aa}]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-set-operand > invalid

```js
/[\w--\q{a|aa}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--\\q{a}]/v'
- '/[\\w--\\q{a|aa}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-string-literal`

Pass: 2 / 12 (16.7%)
Fail: 10 / 12 (83.3%)
Skip: 0 / 12 (0.0%)

#### no-useless-string-literal > invalid

```js
/[\q{a}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a]/v'
- '/[\\q{a}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{a|bc}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\q{bc}]/v'
- '/[\\q{a|bc}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{ab|c}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[c\\q{ab}]/v'
- '/[\\q{ab|c}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{ab|c|de}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[c\\q{ab|de}]/v'
- '/[\\q{ab|c|de}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[a\q{ab|\-}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\-\\q{ab}]/v'
- '/[a\\q{ab|\\-}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{ab|^}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^\\q{ab}]/v'
- '/[\\q{ab|^}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{ab|c}&&\q{ab}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[c\\q{ab}]&&\\q{ab}]/v'
- '/[\\q{ab|c}&&\\q{ab}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[A&&\q{&}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A&&\\&]/v'
- '/[A&&\\q{&}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[\q{&}&&A]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\&&&A]/v'
- '/[\\q{&}&&A]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-string-literal > invalid

```js
/[A&&\q{^|ab}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A&&[\\^\\q{ab}]]/v'
- '/[A&&\\q{^|ab}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-two-nums-quantifier`

Pass: 4 / 10 (40.0%)
Fail: 6 / 10 (60.0%)
Skip: 0 / 10 (0.0%)

#### no-useless-two-nums-quantifier > invalid

```js
/a{1,1}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{1}/'
- '/a{1,1}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-two-nums-quantifier > invalid

```js
/a{42,42}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{42}/'
- '/a{42,42}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-two-nums-quantifier > invalid

```js
/a{042,42}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{42}/'
- '/a{042,42}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-two-nums-quantifier > invalid

```js
/a{042,042}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{42}/'
- '/a{042,042}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-two-nums-quantifier > invalid

```js
/a{100,100}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{100}?/'
- '/a{100,100}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-two-nums-quantifier > invalid

```js
/a{100,100}?/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{100}?/v'
- '/a{100,100}?/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `optimal-quantifier-concatenation`

Pass: 18 / 56 (32.1%)
Fail: 38 / 56 (67.9%)
Skip: 0 / 56 (0.0%)

#### optimal-quantifier-concatenation > invalid

```js
/[a-fA-F][a-fA-F]?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-fA-F]{1,2}/'
- '/[a-fA-F][a-fA-F]?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a\d*\d*a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\d*a/'
- '/a\\d*\\d*a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\d+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+\\d/'
- '/\\w+\\d+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\d?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+/'
- '/\\w+\\d?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a+\w+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\w+/'
- '/a+\\w+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\d*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+/'
- '/\\w+\\d*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(\d*\w+)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(\\w+)/'
- '/(\\d*\\w+)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/;+.*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/;.*/' !== '/;+.*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a+(?:a|bb)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a(?:a|bb)+/'
- '/a+(?:a|bb)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+(?:a|b)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+(?:a|b)/'
- '/\\w+(?:a|b)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\d{3,5}\w*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d{3}\\w*/'
- '/\\d{3,5}\\w*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w\w*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+/'
- '/\\w\\w*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w*\w/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+/'
- '/\\w*\\w/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\w/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w{2,}/'
- '/\\w+\\w/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\w{4}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w{5,}/'
- '/\\w+\\w{4}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+\w{4}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w{5,}/'
- '/\\w+\\w{4}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w{4}\w{4}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w{8}/'
- '/\\w{4}\\w{4}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/[ab]*(?:a|b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab]+/'
- '/[ab]*(?:a|b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/aa*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a+/' !== '/aa*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a*a*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a*/' !== '/a*a*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a?a?a?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{0,2}a?/'
- '/a?a?a?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a.{1,3}?.{2,4}?c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a.{3,7}?c/'
- '/a.{1,3}?.{2,4}?c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a.{1,3}.{2,4}c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a.{3,7}c/'
- '/a.{1,3}.{2,4}c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/\w+(?:foo|bar)?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w+/'
- '/\\w+(?:foo|bar)?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/[ab]*(?:a|bb)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab]*(?:a|bb)/'
- '/[ab]*(?:a|bb)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(?:\d+|abc)\w+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:\\d|abc)\\w+/'
- '/(?:\\d+|abc)\\w+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(^[ \t]*)[a-z\d].+(?::{2,4}|;;)(?=\s)/im
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(^[ \\t]*)[a-z\\d].+(?::{2}|;;)(?=\\s)/im'
- '/(^[ \\t]*)[a-z\\d].+(?::{2,4}|;;)(?=\\s)/im'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(^[\t ]*)#(?:const|else(?:[\t ]+if)?|end[\t ]+if|error|if).*/im
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(^[\\t ]*)#(?:const|else|end[\\t ]+if|error|if).*/im'
- '/(^[\\t ]*)#(?:const|else(?:[\\t ]+if)?|end[\\t ]+if|error|if).*/im'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(&(?:\r\n?|\n)\s*)!.*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(&(?:\\r|\\n)\\s*)!.*/'
- '/(&(?:\\r\\n?|\\n)\\s*)!.*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a\s*(?=\r?\n|\r)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\s*(?=\\n|\\r)/'
- '/a\\s*(?=\\r?\\n|\\r)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(^|[^\w.])[a-z][\w.]*(?:Error|Exception):.*(?:(?:\r\n?|\n)[ \t]*(?:at[ \t].+|\.{3}.*|Caused by:.*))+(?:(?:\r\n?|\n)[ \t]*\.\.\. .*)?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(^|[^\\w.])[a-z][\\w.]*(?:Error|Exception):.*(?:(?:\\r\\n?|\\n)[ \\t]*(?:at[ \\t].+|\\.{3}.*|Caused by:.*))+/'
- '/(^|[^\\w.])[a-z][\\w.]*(?:Error|Exception):.*(?:(?:\\r\\n?|\\n)[ \\t]*(?:at[ \\t].+|\\.{3}.*|Caused by:.*))+(?:(?:\\r\\n?|\\n)[ \\t]*\\.\\.\\. .*)?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/&key\s+\S+(?:\s+\S+)*(?:\s+&allow-other-keys)?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/&key\\s+\\S+(?:\\s+\\S+)*/'
- '/&key\\s+\\S+(?:\\s+\\S+)*(?:\\s+&allow-other-keys)?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(?:xa+|y[ab]*)a*/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:xa+|y[ab]*)/'
- '/(?:xa+|y[ab]*)a*/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(?:xa+|y[ab]*)(?:a*b)?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:xa+|y[ab]*)(?:b)?/'
- '/(?:xa+|y[ab]*)(?:a*b)?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a+(?:a*z+[ay]*)*b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+(?:z+[ay]*)*b/'
- '/a+(?:a*z+[ay]*)*b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/(?:xa+|y[ab]*)(?:a*z[ac]*|xy[za]+)+b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:xa+|y[ab]*)(?:z[ac]*|xy[za]+)+b/'
- '/(?:xa+|y[ab]*)(?:a*z[ac]*|xy[za]+)+b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/a+[a\q{}]+/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+[a\\q{}]/v'
- '/a+[a\\q{}]+/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### optimal-quantifier-concatenation > invalid

```js
/[ab]*[\q{a|bb}]+/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab]*[\\q{a|bb}]/v'
- '/[ab]*[\\q{a|bb}]+/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-character-class`

Pass: 10 / 54 (18.5%)
Fail: 44 / 54 (81.5%)
Skip: 0 / 54 (0.0%)

#### prefer-character-class > invalid

```js
/a|b|c|\d/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc\\d]/'
- '/a|b|c|\\d/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(a|b|c|\d)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/([abc\\d])/'
- '/(a|b|c|\\d)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|b|c|\d)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[abc\\d])/'
- '/(?:a|b|c|\\d)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?=a|b|c|\d)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=[abc\\d])/'
- '/(?=a|b|c|\\d)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<=a|b|c|\d)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=[abc\\d])/'
- '/(?<=a|b|c|\\d)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|b|c|\d|[d-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc\\dd-f]/'
- '/a|b|c|\\d|[d-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|-|c|\d|c|[-d-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\-c\\dc\\-d-f]/'
- '/a|-|c|\\d|c|[-d-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|[.]|c|\d|c|[-d-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a.c\\dc\\-d-f]/'
- '/a|[.]|c|\\d|c|[-d-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
const s = "a|b|\\d|c"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s = "[ab\\\\dc]"\n            new RegExp(s)'
- 'const s = "a|b|\\\\d|c"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|b|c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abc]/'
- '/a|b|c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/]|a|b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\]ab]/'
- '/]|a|b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/-|a|c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\-ac]/'
- '/-|a|c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|-|c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\-c]/'
- '/a|-|c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/a|[-]|c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\-c]/'
- '/a|[-]|c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[abc])/'
- '/(?:a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/([abc])/'
- '/(a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<name>a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<name>[abc])/'
- '/(?<name>a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|b|c|d\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[abc]|d\\b)/'
- '/(?:a|b|c|d\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|b\b|[c]|d)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[acd]|b\\b)/'
- '/(?:a|b\\b|[c]|d)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|\w|\s|["'])/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `/(?:[a\\w\\s"'])/`
- `/(?:a|\\w|\\s|["'])/`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\w|-|\+|\*|\/)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\w\\-\\+\\*\\/])+/'
- '/(?:\\w|-|\\+|\\*|\\/)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?=a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=[abc])/'
- '/(?=a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?!a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?![abc])/'
- '/(?!a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<=a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=[abc])/'
- '/(?<=a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<!a|b|c)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<![abc])/'
- '/(?<!a|b|c)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?=a|b|c|dd|e)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?=[abce]|dd)/'
- '/(?=a|b|c|dd|e)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?!a|b|c|dd|e)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?![abce]|dd)/'
- '/(?!a|b|c|dd|e)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<=a|b|c|dd|e)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<=[abce]|dd)/'
- '/(?<=a|b|c|dd|e)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?<!a|b|c|dd|e)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<![abce]|dd)/'
- '/(?<!a|b|c|dd|e)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/[abc]|d/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcd]/'
- '/[abc]|d/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/[abc]|d|ee/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcd]|ee/'
- '/[abc]|d|ee/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|\w|b\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[a\\w]|b\\b)/'
- '/(?:a|\\w|b\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\w|a|b\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\wa]|b\\b)/'
- '/(?:\\w|a|b\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\w|b\b|a)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\wa]|b\\b)/'
- '/(?:\\w|b\\b|a)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\s|\S|b\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\s\\S]|b\\b)/'
- '/(?:\\s|\\S|b\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\w|\D|b\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\w\\D]|b\\b)/'
- '/(?:\\w|\\D|b\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:\w|\W|b\b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[\\w\\W]|b\\b)/'
- '/(?:\\w|\\W|b\\b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/--?|-=|\+\+?|\+=|!=?|~|\*\*?|\*=|\/=?|%=?|<<=?|>>=?|<=?|>=?|==?|&&?|&=|\^=?|\|\|?|\|=|\?|:/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/--?|-=|\\+\\+?|\\+=|!=?|[~\\?:]|\\*\\*?|\\*=|\\/=?|%=?|<<=?|>>=?|<=?|>=?|==?|&&?|&=|\\^=?|\\|\\|?|\\|=/'
- '/--?|-=|\\+\\+?|\\+=|!=?|~|\\*\\*?|\\*=|\\/=?|%=?|<<=?|>>=?|<=?|>=?|==?|&&?|&=|\\^=?|\\|\\|?|\\|=|\\?|:/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/--?|\+\+?|!=?=?|<=?|>=?|==?=?|&&|\|\|?|\?|\*|\/|~|\^|%/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/--?|\\+\\+?|!=?=?|<=?|>=?|==?=?|&&|\\|\\|?|[\\?\\*\\/~\\^%]/'
- '/--?|\\+\\+?|!=?=?|<=?|>=?|==?=?|&&|\\|\\|?|\\?|\\*|\\/|~|\\^|%/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/1|2|3|[\w--\d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[123[\\w--\\d]]/v'
- '/1|2|3|[\\w--\\d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/1|&|&|[\w--\d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[1\\&&[\\w--\\d]]/v'
- '/1|&|&|[\\w--\\d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/1|~|~|[\w--\d]|[\q{abc}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[1\\~~[\\w--\\d]]|[\\q{abc}]/v'
- '/1|~|~|[\\w--\\d]|[\\q{abc}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/foo|bar|a|b|c|baz/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/foo|bar|[abc]|baz/'
- '/foo|bar|a|b|c|baz/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-character-class > invalid

```js
/(?:a|b)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "minAlternatives": 2
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[ab])/'
- '/(?:a|b)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-d`

Pass: 13 / 25 (52.0%)
Fail: 12 / 25 (48.0%)
Skip: 0 / 25 (0.0%)

#### prefer-d > invalid

```js
/[0-9]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\d/' !== '/[0-9]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[^0-9]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\D/'
- '/[^0-9]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[^0-9\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "insideCharacterClass": "d"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^\\d\\w]/'
- '/[^0-9\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js

            const s = "[0-9]"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "\\\\d"\n            new RegExp(s)\n            '
- '\n            const s = "[0-9]"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[0-9a-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "insideCharacterClass": "d"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\da-z]/'
- '/[0-9a-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[\da-z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "insideCharacterClass": "range"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[0-9a-z]/'
- '/[\\da-z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[0-9]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d/v'
- '/[0-9]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[[0-9]--[0-7]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\d--[0-7]]/v'
- '/[[0-9]--[0-7]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[[0-:]--:]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d/v'
- '/[[0-:]--:]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[[\da-z]--0]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "insideCharacterClass": "range"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[0-9a-z]--0]/v'
- '/[[\\da-z]--0]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[[0-9a-z]--0]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "insideCharacterClass": "d"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[\\da-z]--0]/v'
- '/[[0-9a-z]--0]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-d > invalid

```js
/[\q{0|1|2|3|4|5|6|7|8|9}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d/v'
- '/[\\q{0|1|2|3|4|5|6|7|8|9}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-lookaround`

Pass: 46 / 88 (52.3%)
Fail: 42 / 88 (47.7%)
Skip: 0 / 88 (0.0%)

#### prefer-lookaround > invalid

```js

            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<before>love )unicorn(?<after>!)/, '$<before>🦄$<after>');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<=love )unicorn(?=!)/, '🦄');\n" +
-   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<before>love )unicorn(?<after>!)/, '$<before>🦄$<after>');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'I love unicorn! I hate unicorn?'.replace(/(love )unicorn(!)/, '$1🦄$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<=love )unicorn(?=!)/, '🦄');\n" +
-   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(love )unicorn(!)/, '$1🦄$2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'A JavaScript linter written in JavaScript.'.replaceAll(/Java(Script)/g, 'Type$1');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'A JavaScript linter written in JavaScript.'.replaceAll(/Java(?=Script)/g, 'Type');\n" +
-   "            const str = 'A JavaScript linter written in JavaScript.'.replaceAll(/Java(Script)/g, 'Type$1');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'A JavaScript formatter written in JavaScript.'.replace(/(Java)Script/g, '$1');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'A JavaScript formatter written in JavaScript.'.replace(/(?<=Java)Script/g, '');\n" +
-   "            const str = 'A JavaScript formatter written in JavaScript.'.replace(/(Java)Script/g, '$1');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/Java(Script)/g, '$1');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/Java(?=Script)/g, '');\n" +
-   "            const str = 'JavaScript'.replace(/Java(Script)/g, '$1');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript.'.replace(/(Java)(Script)/, '$1 and Type$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript.'.replace(/(?<=Java)(?=Script)/, ' and Type');\n" +
-   "            const str = 'JavaScript.'.replace(/(Java)(Script)/, '$1 and Type$2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript.'.replace(/(J)(ava)(Script)/, '$1Query, and Java$3');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript.'.replace(/(?<=J)(ava)(?=Script)/, 'Query, and Java');\n" +
-   "            const str = 'JavaScript.'.replace(/(J)(ava)(Script)/, '$1Query, and Java$3');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const regex = /(Java)(Script)/g
            str.replace(regex, '$1$2');
            str2.replace(regex, '$1 and Java$2');
            str3.replace(regex, '$1 and $2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const regex = /(?<=Java)(?=Script)/g\n' +
+   "            str.replace(regex, '');\n" +
+   "            str2.replace(regex, ' and Java');\n" +
+   "            str3.replace(regex, ' and ');\n" +
-   '            const regex = /(Java)(Script)/g\n' +
-   "            str.replace(regex, '$1$2');\n" +
-   "            str2.replace(regex, '$1 and Java$2');\n" +
-   "            str3.replace(regex, '$1 and $2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const regex = /(Java)(Script)/
            str.replace(regex, '$1$2');
            str2.replace(regex, '$1 and Java$2');
            str3.replace(regex, '$1 and $2');
            regex.test(str);
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const regex = /(?<=Java)(?=Script)/\n' +
+   "            str.replace(regex, '');\n" +
+   "            str2.replace(regex, ' and Java');\n" +
+   "            str3.replace(regex, ' and ');\n" +
-   '            const regex = /(Java)(Script)/\n' +
-   "            str.replace(regex, '$1$2');\n" +
-   "            str2.replace(regex, '$1 and Java$2');\n" +
-   "            str3.replace(regex, '$1 and $2');\n" +
    '            regex.test(str);\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const regex = /(a)b(c)/
            "abc".replace(regex, 'dynami$2');
            "abc".replace(regex, 'dyn$1mi$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const regex = /(a)b(?=c)/\n' +
+   `            "abc".replace(regex, 'dynami');\n` +
+   `            "abc".replace(regex, 'dyn$1mi');\n` +
-   '            const regex = /(a)b(c)/\n' +
-   `            "abc".replace(regex, 'dynami$2');\n` +
-   `            "abc".replace(regex, 'dyn$1mi$2');\n` +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            "aabcaabbcc".replace(/(a+)b+c/g, "$1_")
            // 'aa_aa_c'
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            "aabcaabbcc".replace(/(?<=a+)b+c/g, "_")\n' +
-   '            "aabcaabbcc".replace(/(a+)b+c/g, "$1_")\n' +
    "            // 'aa_aa_c'\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
"aaaaaa".replace(/(^a+)a(a)/, "$1b$2")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '"aaaaaa".replace(/(^a+)a(?=a)/, "$1b")'
- '"aaaaaa".replace(/(^a+)a(a)/, "$1b$2")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Script)$/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Script$)/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Script)$/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Script)\b/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Script\\b)/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Script)\\b/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Script)(?:\b|$)/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Script(?:\\b|$))/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Script)(?:\\b|$)/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Scrip)(?=t)/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Scrip)(?=t)/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScriptLinter'.replace(/Java(Script)(?=Linter|Checker)/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScriptLinter'.replace(/Java(?=Script(?=Linter|Checker))/g, 'Type')"
- "var str = 'JavaScriptLinter'.replace(/Java(Script)(?=Linter|Checker)/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'Java'.replace(/^(J)ava/g, '$1Query')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'Java'.replace(/(?<=^J)ava/g, 'Query')"
- "var str = 'Java'.replace(/^(J)ava/g, '$1Query')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'Java'.replace(/\b(J)ava/g, '$1Query')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'Java'.replace(/(?<=\\bJ)ava/g, 'Query')"
- "var str = 'Java'.replace(/\\b(J)ava/g, '$1Query')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'Java'.replace(/(?:^|\b)(J)ava/g, '$1Query')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'Java'.replace(/(?<=(?:^|\\b)J)ava/g, 'Query')"
- "var str = 'Java'.replace(/(?:^|\\b)(J)ava/g, '$1Query')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'Java'.replace(/(?:^|\b|[\q{}])(J)ava/gv, '$1Query')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'Java'.replace(/(?<=(?:^|\\b|[\\q{}])J)ava/gv, 'Query')"
- "var str = 'Java'.replace(/(?:^|\\b|[\\q{}])(J)ava/gv, '$1Query')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScriptCode'.replace(/(?<=Java)(Script)Code/g, '$1Linter')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScriptCode'.replace(/(?<=JavaScript)Code/g, 'Linter')"
- "var str = 'JavaScriptCode'.replace(/(?<=Java)(Script)Code/g, '$1Linter')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScriptCode'.replace(/(?<=Java|Type)(Script)Code/g, '$1Linter')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScriptCode'.replace(/(?<=(?<=Java|Type)Script)Code/g, 'Linter')"
- "var str = 'JavaScriptCode'.replace(/(?<=Java|Type)(Script)Code/g, '$1Linter')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/^(Java)(Script)$/, '$1 and Type$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=^Java)(?=Script$)/, ' and Type');\n" +
-   "            const str = 'JavaScript'.replace(/^(Java)(Script)$/, '$1 and Type$2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/\b(Java)(Script)\b/, '$1 and Type$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=\\bJava)(?=Script\\b)/, ' and Type');\n" +
-   "            const str = 'JavaScript'.replace(/\\b(Java)(Script)\\b/, '$1 and Type$2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/(?:^|\b)(Java)(Script)(?:\b|$)/, '$1 and Type$2');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=(?:^|\\b)Java)(?=Script(?:\\b|$))/, ' and Type');\n" +
-   "            const str = 'JavaScript'.replace(/(?:^|\\b)(Java)(Script)(?:\\b|$)/, '$1 and Type$2');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/^(J)(ava)(Script)$/, '$1Query, and Java$3');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=^J)(ava)(?=Script$)/, 'Query, and Java');\n" +
-   "            const str = 'JavaScript'.replace(/^(J)(ava)(Script)$/, '$1Query, and Java$3');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/\b(J)(ava)(Script)\b/, '$1Query, and Java$3');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=\\bJ)(ava)(?=Script\\b)/, 'Query, and Java');\n" +
-   "            const str = 'JavaScript'.replace(/\\b(J)(ava)(Script)\\b/, '$1Query, and Java$3');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(t)/g, 'Type$1$2$3$4$5$6')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(?=t)/g, 'Type$1$2$3$4$5')"
- "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(t)/g, 'Type$1$2$3$4$5$6')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(?=t)/g, 'Type$1$2$3$4$5')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(?=pt)/g, 'Type$1$2$3$4')"
- "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(?=t)/g, 'Type$1$2$3$4$5')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(?=pt)/g, 'Type$1$2$3$4')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(?=ipt)/g, 'Type$1$2$3')"
- "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(?=pt)/g, 'Type$1$2$3$4')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            var str1 = 'ESLint ESLint'.replace(/ES(Lint|Tree)$/g, 'TypeScriptES$1');
            var str2 = 'ESTree ESTree'.replace(/ES(Lint|Tree)$/g, 'TypeScriptES$1');
            console.log(str1, str2)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            var str1 = 'ESLint ESLint'.replace(/ES(?=(?:Lint|Tree)$)/g, 'TypeScriptES');\n" +
+   "            var str2 = 'ESTree ESTree'.replace(/ES(?=(?:Lint|Tree)$)/g, 'TypeScriptES');\n" +
-   "            var str1 = 'ESLint ESLint'.replace(/ES(Lint|Tree)$/g, 'TypeScriptES$1');\n" +
-   "            var str2 = 'ESTree ESTree'.replace(/ES(Lint|Tree)$/g, 'TypeScriptES$1');\n" +
    '            console.log(str1, str2)\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            var str1 = 'ESLint ESLint'.replace(/^(E|J)S/g, '$1Script');
            var str2 = 'JSLint JSLint'.replace(/^(E|J)S/g, '$1Script');
            console.log(str1, str2)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            var str1 = 'ESLint ESLint'.replace(/(?<=^(?:E|J))S/g, 'Script');\n" +
+   "            var str2 = 'JSLint JSLint'.replace(/(?<=^(?:E|J))S/g, 'Script');\n" +
-   "            var str1 = 'ESLint ESLint'.replace(/^(E|J)S/g, '$1Script');\n" +
-   "            var str2 = 'JSLint JSLint'.replace(/^(E|J)S/g, '$1Script');\n" +
    '            console.log(str1, str2)\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'foobarbaz'.replace(/foo(bar)(?:^|$|a{0}\b|(?=x)|(?<=y))/, 'test$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'foobarbaz'.replace(/foo(?=bar(?:^|$|a{0}\\b|(?=x)|(?<=y)))/, 'test')"
- "var str = 'foobarbaz'.replace(/foo(bar)(?:^|$|a{0}\\b|(?=x)|(?<=y))/, 'test$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'foobarbaz'.replace(/foo(bar)(?<=z\w+)/, 'test$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'foobarbaz'.replace(/foo(?=bar(?<=z\\w+))/, 'test')"
- "var str = 'foobarbaz'.replace(/foo(bar)(?<=z\\w+)/, 'test$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Script)^/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Script^)/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Script)^/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'Java'.replace(/$(J)ava/g, '$1Query')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'Java'.replace(/(?<=$J)ava/g, 'Query')"
- "var str = 'Java'.replace(/$(J)ava/g, '$1Query')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'JavaScript'.replace(/^\b(J)(ava)(Script)\b$/, '$1Query, and Java$3');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'JavaScript'.replace(/(?<=^\\bJ)(ava)(?=Script\\b$)/, 'Query, and Java');\n" +
-   "            const str = 'JavaScript'.replace(/^\\b(J)(ava)(Script)\\b$/, '$1Query, and Java$3');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'foobar'.replace(/foo(bar)(\b|$)/, 'bar$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'foobar'.replace(/foo(?=bar(\\b|$))/, 'bar')"
- "var str = 'foobar'.replace(/foo(bar)(\\b|$)/, 'bar$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScript'.replace(/Java(Scrip)((?=t))/g, 'Type$1')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(?=Scrip((?=t)))/g, 'Type')"
- "var str = 'JavaScript'.replace(/Java(Scrip)((?=t))/g, 'Type$1')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js
var str = 'JavaScriptCode'.replace(/((?<=Java))(Script)Code/g, '$2Linter')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "var str = 'JavaScriptCode'.replace(/(?<=((?<=Java))Script)Code/g, 'Linter')"
- "var str = 'JavaScriptCode'.replace(/((?<=Java))(Script)Code/g, '$2Linter')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-lookaround > invalid

```js

            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<before>love )unicorn(?<after>!)/, '$<before>🦄$<after>');
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "lookbehind": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<before>love )unicorn(?=!)/, '$<before>🦄');\n" +
-   "            const str = 'I love unicorn! I hate unicorn?'.replace(/(?<before>love )unicorn(?<after>!)/, '$<before>🦄$<after>');\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-named-backreference`

Pass: 4 / 6 (66.7%)
Fail: 2 / 6 (33.3%)
Skip: 0 / 6 (0.0%)

#### prefer-named-backreference > invalid

```js
/(?<foo>a)\1/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<foo>a)\\k<foo>/'
- '/(?<foo>a)\\1/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-backreference > invalid

```js
/(?<foo>a)\1/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?<foo>a)\\k<foo>/v'
- '/(?<foo>a)\\1/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-named-replacement`

Pass: 10 / 16 (62.5%)
Fail: 6 / 16 (37.5%)
Skip: 0 / 16 (0.0%)

#### prefer-named-replacement > invalid

```js
"str".replace(/a(?<foo>b)c/, "_$1_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '"str".replace(/a(?<foo>b)c/, "_$<foo>_")'
- '"str".replace(/a(?<foo>b)c/, "_$1_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-replacement > invalid

```js
"str".replace(/a(?<foo>b)c/v, "_$1_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '"str".replace(/a(?<foo>b)c/v, "_$<foo>_")'
- '"str".replace(/a(?<foo>b)c/v, "_$1_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-replacement > invalid

```js
"str".replaceAll(/a(?<foo>b)c/, "_$1_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '"str".replaceAll(/a(?<foo>b)c/, "_$<foo>_")'
- '"str".replaceAll(/a(?<foo>b)c/, "_$1_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-replacement > invalid

```js
"str".replace(/(a)(?<foo>b)c/, "_$1$2_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '"str".replace(/(a)(?<foo>b)c/, "_$1$<foo>_")'
- '"str".replace(/(a)(?<foo>b)c/, "_$1$2_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-replacement > invalid

```js
unknown.replace(/a(?<foo>b)c/, "_$1_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'unknown.replace(/a(?<foo>b)c/, "_$<foo>_")'
- 'unknown.replace(/a(?<foo>b)c/, "_$1_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-named-replacement > invalid

```js
unknown.replaceAll(/a(?<foo>b)c/, "_$1_")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'unknown.replaceAll(/a(?<foo>b)c/, "_$<foo>_")'
- 'unknown.replaceAll(/a(?<foo>b)c/, "_$1_")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-plus-quantifier`

Pass: 7 / 13 (53.8%)
Fail: 6 / 13 (46.2%)
Skip: 0 / 13 (0.0%)

#### prefer-plus-quantifier > invalid

```js
/a{1,}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a+/' !== '/a{1,}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-plus-quantifier > invalid

```js
/a{1,}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+?/'
- '/a{1,}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-plus-quantifier > invalid

```js
/(a){1,}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)+/'
- '/(a){1,}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-plus-quantifier > invalid

```js
/(a){1,}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)+/v'
- '/(a){1,}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-plus-quantifier > invalid

```js
/(a){1,}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)+?/'
- '/(a){1,}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-plus-quantifier > invalid

```js

            const s = "a{1,}"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "a+"\n            new RegExp(s)\n            '
- '\n            const s = "a{1,}"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-predefined-assertion`

Pass: 2 / 14 (14.3%)
Fail: 12 / 14 (85.7%)
Skip: 0 / 14 (0.0%)

#### prefer-predefined-assertion > invalid

```js
/a(?=\w)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\B/'
- '/a(?=\\w)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/a(?!\w)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\b/'
- '/a(?!\\w)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/(?<=\w)a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\Ba/'
- '/(?<=\\w)a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/(?<!\w)a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\ba/'
- '/(?<!\\w)a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/a(?=\W)./
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\b./'
- '/a(?=\\W)./'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/a(?!\W)./
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a\\B./'
- '/a(?!\\W)./'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/.(?<=\W)a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/.\\ba/'
- '/.(?<=\\W)a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/.(?<!\W)a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/.\\Ba/'
- '/.(?<!\\W)a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/.(?<!\W)a/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/.\\Ba/v'
- '/.(?<!\\W)a/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/a+(?!\w)(?:\s|bc+)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a+\\b(?:\\s|bc+)+/'
- '/a+(?!\\w)(?:\\s|bc+)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/(?!.)(?![^])/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?!.)$/'
- '/(?!.)(?![^])/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-predefined-assertion > invalid

```js
/(?<!.)(?<![^])/m
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/^(?<![^])/m'
- '/(?<!.)(?<![^])/m'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-quantifier`

Pass: 12 / 25 (48.0%)
Fail: 13 / 25 (52.0%)
Skip: 0 / 25 (0.0%)

#### prefer-quantifier > invalid

```js
/\d\d/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d{2}/'
- '/\\d\\d/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/  /
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/ {2}/' !== '/  /'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/  /v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/ {2}/v' !== '/  /v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/\p{ASCII}\p{ASCII}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{ASCII}{2}/u'
- '/\\p{ASCII}\\p{ASCII}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/\p{Basic_Emoji}\p{Basic_Emoji}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{Basic_Emoji}{2}/v'
- '/\\p{Basic_Emoji}\\p{Basic_Emoji}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/(\d\d\d*)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(\\d{2}\\d*)/'
- '/(\\d\\d\\d*)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/\d\d\d\d-\d\d-\d\d/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d{4}-\\d{2}-\\d{2}/'
- '/\\d\\d\\d\\d-\\d\\d-\\d\\d/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/aaa..\s\s\S\S\p{ASCII}\p{ASCII}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{3}..\\s{2}\\S\\S\\p{ASCII}{2}/u'
- '/aaa..\\s\\s\\S\\S\\p{ASCII}\\p{ASCII}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/aaaa(aaa)/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a{4}(a{3})/u'
- '/aaaa(aaa)/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/(b)?aaaa(b)?/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(b)?a{4}(b)?/u'
- '/(b)?aaaa(b)?/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js

            const s = "\\d\\d"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "\\\\d{2}"\n            new RegExp(s)\n            '
- '\n            const s = "\\\\d\\\\d"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/wwww/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "www"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/w{4}/' !== '/wwww/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-quantifier > invalid

```js
/\d\d-\d\d\d\d/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "allows": [
        "\\d\\d"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\d\\d-\\d{4}/'
- '/\\d\\d-\\d\\d\\d\\d/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-question-quantifier`

Pass: 15 / 27 (55.6%)
Fail: 12 / 27 (44.4%)
Skip: 0 / 27 (0.0%)

#### prefer-question-quantifier > invalid

```js
/a{0,1}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a?/' !== '/a{0,1}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/a{0,1}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a??/'
- '/a{0,1}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(a){0,1}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)?/'
- '/(a){0,1}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(a){0,1}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)?/v'
- '/(a){0,1}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(a){0,1}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)??/'
- '/(a){0,1}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(?:abc|)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:abc)?/'
- '/(?:abc|)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(?:abc|def|)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:abc|def)?/'
- '/(?:abc|def|)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(?:abc||def|)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:abc||def)?/'
- '/(?:abc||def|)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(?:abc|def||)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:abc|def)?/'
- '/(?:abc|def||)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js
/(?:abc|def|)?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:abc|def)?/'
- '/(?:abc|def|)?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js

            const s = "a{0,1}"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "a?"\n            new RegExp(s)\n            '
- '\n            const s = "a{0,1}"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-question-quantifier > invalid

```js

            const s = "(?:abc|def|)"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "(?:abc|def)?"\n            new RegExp(s)\n            '
- '\n            const s = "(?:abc|def|)"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-range`

Pass: 22 / 38 (57.9%)
Fail: 16 / 38 (42.1%)
Skip: 0 / 38 (0.0%)

#### prefer-range > invalid

```js
/[abcd]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-d]/'
- '/[abcd]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[ABCD abcd]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A-D a-d]/'
- '/[ABCD abcd]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[ABCD abcd]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[A-D a-d]/v'
- '/[ABCD abcd]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[abc-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f]/'
- '/[abc-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[a-cd-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f]/'
- '/[a-cd-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[d-fa-c]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f]/'
- '/[d-fa-c]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[abc_d-f]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f_]/'
- '/[abc_d-f]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[abc_d-f_h-j_k-m]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f__h-m_]/'
- '/[abc_d-f_h-j_k-m]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[a-d_d-f_h-k_j-m]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-f__h-m_]/'
- '/[a-d_d-f_h-k_j-m]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[0-2\d3-4]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[0-4\\d]/'
- '/[0-2\\d3-4]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[3-4560-2]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[0-6]/'
- '/[3-4560-2]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
const s = "[0-23-4\\d]"
            new RegExp(s)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'const s = "[0-4\\\\d]"\n            new RegExp(s)'
- 'const s = "[0-23-4\\\\d]"\n            new RegExp(s)'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[ !"#$]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "target": "all"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ -$]/'
- '/[ !"#$]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[abcd ①②③④⑤⑥⑦⑧⑨10⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "target": [
        "alphanumeric",
        "①-⑳"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-d ①-⑨10⑪-⑳]/'
- '/[abcd ①②③④⑤⑥⑦⑧⑨10⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[😀😁😂😃😄 😆😇😈😉😊]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "target": [
        "alphanumeric",
        "😀-😏"
      ]
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[😀-😄 😆-😊]/u'
- '/[😀😁😂😃😄 😆😇😈😉😊]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-range > invalid

```js
/[😀😁😂😃😄 😆😇😈😉😊]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "settings": {
    "regexp": {
      "allowedCharacterRanges": [
        "alphanumeric",
        "😀-😏"
      ]
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[😀-😄 😆-😊]/u'
- '/[😀😁😂😃😄 😆😇😈😉😊]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-regexp-test`

Pass: 8 / 16 (50.0%)
Fail: 8 / 16 (50.0%)
Skip: 0 / 16 (0.0%)

#### prefer-regexp-test > invalid

```js

            const text = 'something';
            const pattern = /thing/;
            if (pattern.exec(text)) {}
            if (text.match(pattern)) {}
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'something';\n" +
    '            const pattern = /thing/;\n' +
+   '            if (pattern.test(text)) {}\n' +
+   '            if (pattern.test(text)) {}\n' +
-   '            if (pattern.exec(text)) {}\n' +
-   '            if (text.match(pattern)) {}\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

            const text = 'something';
            const pattern = ()=>/thing/;
            if (pattern().exec(text)) {}
            if (text.match(pattern())) {}
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'something';\n" +
    '            const pattern = ()=>/thing/;\n' +
+   '            if (pattern().test(text)) {}\n' +
-   '            if (pattern().exec(text)) {}\n' +
    '            if (text.match(pattern())) {}\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

            const text = 'something';
            const pattern = /thing/;
            const b1 = Boolean(pattern.exec(text))
            const b2 = Boolean(text.match(pattern))
            const b3 = !pattern.exec(text)
            const b4 = !text.match(pattern)
            const b5 = !(foo && pattern.exec(text))
            const b6 = !(foo || text.match(pattern))
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'something';\n" +
    '            const pattern = /thing/;\n' +
+   '            const b1 = Boolean(pattern.test(text))\n' +
+   '            const b2 = Boolean(pattern.test(text))\n' +
+   '            const b3 = !pattern.test(text)\n' +
+   '            const b4 = !pattern.test(text)\n' +
+   '            const b5 = !(foo && pattern.test(text))\n' +
+   '            const b6 = !(foo || pattern.test(text))\n' +
-   '            const b1 = Boolean(pattern.exec(text))\n' +
-   '            const b2 = Boolean(text.match(pattern))\n' +
-   '            const b3 = !pattern.exec(text)\n' +
-   '            const b4 = !text.match(pattern)\n' +
-   '            const b5 = !(foo && pattern.exec(text))\n' +
-   '            const b6 = !(foo || text.match(pattern))\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

            const re = /a/g;
            const str = 'abc';

            console.log(!!str.match(re)); // ignore
            console.log(!!str.match(re)); // ignore
            console.log(!!re.exec(str));
            console.log(!!re.exec(str));
            console.log(re.test(str));
            console.log(re.test(str));
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const re = /a/g;\n' +
    "            const str = 'abc';\n" +
    '\n' +
    '            console.log(!!str.match(re)); // ignore\n' +
    '            console.log(!!str.match(re)); // ignore\n' +
+   '            console.log(!!re.test(str));\n' +
+   '            console.log(!!re.test(str));\n' +
-   '            console.log(!!re.exec(str));\n' +
-   '            console.log(!!re.exec(str));\n' +
    '            console.log(re.test(str));\n' +
    '            console.log(re.test(str));\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

            const text = 'something';
            /** @type {RegExp} */
            const pattern = getPattern();
            if (text.match(pattern)) {}
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'something';\n" +
    '            /** @type {RegExp} */\n' +
    '            const pattern = getPattern();\n' +
+   '            if (pattern.test(text)) {}\n' +
-   '            if (text.match(pattern)) {}\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

            const text = 'something';
            const pattern = /thin[[g]]/v;
            if (pattern.exec(text)) {}
            if (text.match(pattern)) {}
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "            const text = 'something';\n" +
    '            const pattern = /thin[[g]]/v;\n' +
+   '            if (pattern.test(text)) {}\n' +
+   '            if (pattern.test(text)) {}\n' +
-   '            if (pattern.exec(text)) {}\n' +
-   '            if (text.match(pattern)) {}\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

        const text = 'something';
        const pattern = /thing/;
        const a = pattern.exec(test) === null;
        const b = pattern.exec(test) !== null;
        const c = text.match(pattern) === null;
        const d = text.match(pattern) !== null;
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "        const text = 'something';\n" +
    '        const pattern = /thing/;\n' +
+   '        const a = !pattern.test(test);\n' +
+   '        const b = pattern.test(test);\n' +
+   '        const c = !pattern.test(text);\n' +
+   '        const d = pattern.test(text);\n' +
-   '        const a = pattern.exec(test) === null;\n' +
-   '        const b = pattern.exec(test) !== null;\n' +
-   '        const c = text.match(pattern) === null;\n' +
-   '        const d = text.match(pattern) !== null;\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-regexp-test > invalid

```js

        const text = 'something';
        const pattern = /thing/;
        const a = pattern.exec(test)=== null;
        const b = pattern.exec(test) /* Comment */ !== null;
        const c = text.match(pattern) === /** Comment */ null;
        const d = (text.match(pattern)) !== null;
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    "        const text = 'something';\n" +
    '        const pattern = /thing/;\n' +
+   '        const a = !pattern.test(test);\n' +
+   '        const b = pattern.test(test) /* Comment */;\n' +
+   '        const c = !pattern.test(text);\n' +
+   '        const d = (pattern.test(text));\n' +
-   '        const a = pattern.exec(test)=== null;\n' +
-   '        const b = pattern.exec(test) /* Comment */ !== null;\n' +
-   '        const c = text.match(pattern) === /** Comment */ null;\n' +
-   '        const d = (text.match(pattern)) !== null;\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-result-array-groups`

Pass: 16 / 31 (51.6%)
Fail: 10 / 31 (32.3%)
Skip: 5 / 31 (16.1%)

#### prefer-result-array-groups > invalid

```js

            const regex = /a(?<foo>b)c/
            let match
            while (match = regex.exec(foo)) {
                const p1 = match[1]
                // ...
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const regex = /a(?<foo>b)c/\n' +
    '            let match\n' +
    '            while (match = regex.exec(foo)) {\n' +
+   '                const p1 = match.groups.foo\n' +
-   '                const p1 = match[1]\n' +
    '                // ...\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const regex = /a(?<foo>b)c/
            let match
            while (match = regex.exec(foo)) {
                const p1 = match?.[1]
                // ...
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const regex = /a(?<foo>b)c/\n' +
    '            let match\n' +
    '            while (match = regex.exec(foo)) {\n' +
+   '                const p1 = match?.groups.foo\n' +
-   '                const p1 = match?.[1]\n' +
    '                // ...\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const regex = /a(?<foo>b)c/
            let match
            while (match = regex.exec(foo)) {
                const p1 = match?.[(1)]
                // ...
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const regex = /a(?<foo>b)c/\n' +
    '            let match\n' +
    '            while (match = regex.exec(foo)) {\n' +
+   '                const p1 = match?.groups.foo\n' +
-   '                const p1 = match?.[(1)]\n' +
    '                // ...\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const regex = /(a)(?<foo>b)c/
            let match
            while (match = regex.exec(foo)) {
                const p1 = match[1]
                const p2 = match[2]
                // ...
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const regex = /(a)(?<foo>b)c/\n' +
    '            let match\n' +
    '            while (match = regex.exec(foo)) {\n' +
    '                const p1 = match[1]\n' +
+   '                const p2 = match.groups.foo\n' +
-   '                const p2 = match[2]\n' +
    '                // ...\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const regex = /a(?<foo>[[b]])c/v
            let match
            while (match = regex.exec(foo)) {
                const p1 = match[1]
                // ...
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const regex = /a(?<foo>[[b]])c/v\n' +
    '            let match\n' +
    '            while (match = regex.exec(foo)) {\n' +
+   '                const p1 = match.groups.foo\n' +
-   '                const p1 = match[1]\n' +
    '                // ...\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const arr = "str".match(/a(?<foo>b)c/)
            const p1 = arr[1]
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const arr = "str".match(/a(?<foo>b)c/)\n' +
+   '            const p1 = arr.groups.foo\n' +
-   '            const p1 = arr[1]\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const arr = unknown.match(/a(?<foo>b)c/)
            const p1 = arr[1]
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const arr = unknown.match(/a(?<foo>b)c/)\n' +
+   '            const p1 = arr.groups.foo\n' +
-   '            const p1 = arr[1]\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const matches = "str".matchAll(/a(?<foo>b)c/);
            for (const match of matches) {
                const p1 = match[1]
                // ..
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const matches = "str".matchAll(/a(?<foo>b)c/);\n' +
    '            for (const match of matches) {\n' +
+   '                const p1 = match.groups.foo\n' +
-   '                const p1 = match[1]\n' +
    '                // ..\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const matches = unknown.matchAll(/a(?<foo>b)c/);
            for (const match of matches) {
                const p1 = match[1]
                // ..
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "strictTypes": false
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const matches = unknown.matchAll(/a(?<foo>b)c/);\n' +
    '            for (const match of matches) {\n' +
+   '                const p1 = match.groups.foo\n' +
-   '                const p1 = match[1]\n' +
    '                // ..\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-result-array-groups > invalid

```js

            const match = /(?<foo>foo)/u.exec(str)
            if (match) {
                match[1] // <-
            }

            const match2 = /(?<bar>bar)/u.exec(str)
            match2
                ? match2[1] // <-
                : null;

            const match3 = /(?<baz>baz)/u.exec(str)
            match3 && match3[1] // <-

            const match4 = /(?<qux>qux)/u.exec(str)
            if (!match4) {
            } else {
                match4[1] // <-
            }
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
    '            const match = /(?<foo>foo)/u.exec(str)\n' +
    '            if (match) {\n' +
+   '                match.groups.foo // <-\n' +
-   '                match[1] // <-\n' +
    '            }\n' +
    '\n' +
    '            const match2 = /(?<bar>bar)/u.exec(str)\n' +
    '            match2\n' +
+   '                ? match2.groups.bar // <-\n' +
-   '                ? match2[1] // <-\n' +
    '                : null;\n' +
    '\n' +
    '            const match3 = /(?<baz>baz)/u.exec(str)\n' +
+   '            match3 && match3.groups.baz // <-\n' +
-   '            match3 && match3[1] // <-\n' +
    '\n' +
    '            const match4 = /(?<qux>qux)/u.exec(str)\n' +
    '            if (!match4) {\n' +
    '            } else {\n' +
+   '                match4.groups.qux // <-\n' +
-   '                match4[1] // <-\n' +
    '            }\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-set-operation`

Pass: 5 / 8 (62.5%)
Fail: 3 / 8 (37.5%)
Skip: 0 / 8 (0.0%)

#### prefer-set-operation > invalid

```js
/(?!a)\w/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w--a]/v'
- '/(?!a)\\w/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-set-operation > invalid

```js
/\w(?<=\d)/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w&&\\d]/v'
- '/\\w(?<=\\d)/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-set-operation > invalid

```js
/(?!-)&/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\&--\\-]/v'
- '/(?!-)&/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-star-quantifier`

Pass: 7 / 13 (53.8%)
Fail: 6 / 13 (46.2%)
Skip: 0 / 13 (0.0%)

#### prefer-star-quantifier > invalid

```js
/a{0,}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a*/' !== '/a{0,}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-star-quantifier > invalid

```js
/a{0,}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a*?/'
- '/a{0,}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-star-quantifier > invalid

```js
/(a){0,}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)*/'
- '/(a){0,}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-star-quantifier > invalid

```js
/(a){0,}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)*/v'
- '/(a){0,}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-star-quantifier > invalid

```js
/(a){0,}?/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(a)*?/'
- '/(a){0,}?/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-star-quantifier > invalid

```js

            const s = "a{0,}"
            new RegExp(s)
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '\n            const s = "a*"\n            new RegExp(s)\n            '
- '\n            const s = "a{0,}"\n            new RegExp(s)\n            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-unicode-codepoint-escapes`

Pass: 8 / 13 (61.5%)
Fail: 5 / 13 (38.5%)
Skip: 0 / 13 (0.0%)

#### prefer-unicode-codepoint-escapes > invalid

```js
/\ud83d\ude00/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{1f600}/u'
- '/\\ud83d\\ude00/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-unicode-codepoint-escapes > invalid

```js
/[\ud83d\ude00]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\u{1f600}]/u'
- '/[\\ud83d\\ude00]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-unicode-codepoint-escapes > invalid

```js
/\uD83D\uDE00/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{1F600}/u'
- '/\\uD83D\\uDE00/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-unicode-codepoint-escapes > invalid

```js

            const s = "\\ud83d\\ude00"
            new RegExp(s, 'u')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const s = "\\\\u{1f600}"\n' +
-   '            const s = "\\\\ud83d\\\\ude00"\n' +
    "            new RegExp(s, 'u')\n" +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-unicode-codepoint-escapes > invalid

```js
/\ud83d\ude00/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\u{1f600}/v'
- '/\\ud83d\\ude00/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-w`

Pass: 6 / 16 (37.5%)
Fail: 10 / 16 (62.5%)
Skip: 0 / 16 (0.0%)

#### prefer-w > invalid

```js
/[0-9a-zA-Z_]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/'
- '/[0-9a-zA-Z_]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[0-9a-zA-Z_#]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w#]/'
- '/[0-9a-zA-Z_#]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[\da-zA-Z_#]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w#]/'
- '/[\\da-zA-Z_#]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[0-9a-z_[\s&&\p{ASCII}]]/iv
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w[\\s&&\\p{ASCII}]]/iv'
- '/[0-9a-z_[\\s&&\\p{ASCII}]]/iv'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[0-9a-z_]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\w/i'
- '/[0-9a-z_]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[^0-9a-zA-Z_]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\W/'
- '/[^0-9a-zA-Z_]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js
/[^0-9A-Z_]/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\W/i'
- '/[^0-9A-Z_]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js

            const s = "[0-9A-Z_]"
            new RegExp(s, 'i')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `\n            const s = "\\\\w"\n            new RegExp(s, 'i')\n            `
- '\n' +
-   '            const s = "[0-9A-Z_]"\n' +
-   "            new RegExp(s, 'i')\n" +
-   '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js

            const s = "[0-9A-Z_c]"
            new RegExp(s, 'i')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `\n            const s = "\\\\w"\n            new RegExp(s, 'i')\n            `
- '\n' +
-   '            const s = "[0-9A-Z_c]"\n' +
-   "            new RegExp(s, 'i')\n" +
-   '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### prefer-w > invalid

```js

            const s = "[0-9A-Z_-]"
            new RegExp(s, 'i')
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `\n            const s = "[\\\\w-]"\n            new RegExp(s, 'i')\n            `
- '\n' +
-   '            const s = "[0-9A-Z_-]"\n' +
-   "            new RegExp(s, 'i')\n" +
-   '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `require-unicode-regexp`

Pass: 28 / 42 (66.7%)
Fail: 14 / 42 (33.3%)
Skip: 0 / 42 (0.0%)

#### require-unicode-regexp > invalid

```js
/foo/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/foo/u' !== '/foo/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/foo/gimy
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/foo/gimyu'
- '/foo/gimy'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
RegExp('foo')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `RegExp('foo', "u")`
- "RegExp('foo')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
RegExp('foo', '')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "RegExp('foo', 'u')"
- "RegExp('foo', '')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
RegExp('foo', 'gimy')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "RegExp('foo', 'gimyu')"
- "RegExp('foo', 'gimy')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
new RegExp('foo')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `new RegExp('foo', "u")`
- "new RegExp('foo')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
new RegExp('foo', '')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "new RegExp('foo', 'u')"
- "new RegExp('foo', '')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
new RegExp('foo', 'gimy')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "new RegExp('foo', 'gimyu')"
- "new RegExp('foo', 'gimy')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
const flags = 'gi'; new RegExp('foo', flags)
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "const flags = 'giu'; new RegExp('foo', flags)"
- "const flags = 'gi'; new RegExp('foo', flags)"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/❤️/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/❤️/u' !== '/❤️/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/foo/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/foo/iu'
- '/foo/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/ab+c/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/ab+c/u'
- '/ab+c/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/a.*b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a.*b/u'
- '/a.*b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-regexp > invalid

```js
/<[^<>]+>/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "globals": {
      "globalThis": "off"
    }
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/<[^<>]+>/u'
- '/<[^<>]+>/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `require-unicode-sets-regexp`

Pass: 26 / 28 (92.9%)
Fail: 2 / 28 (7.1%)
Skip: 0 / 28 (0.0%)

#### require-unicode-sets-regexp > invalid

```js
/a/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/a/v' !== '/a/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### require-unicode-sets-regexp > invalid

```js
/[\p{ASCII}]/iu
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}]/iv'
- '/[\\p{ASCII}]/iu'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `simplify-set-operations`

Pass: 12 / 36 (33.3%)
Fail: 24 / 36 (66.7%)
Skip: 0 / 36 (0.0%)

#### simplify-set-operations > invalid

```js
/[a&&[^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a--[b]]/v'
- '/[a&&[^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a&&b&&[^c]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a&&b]--[c]]/v'
- '/[a&&b&&[^c]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a&&[^b]&&c]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a&&c]--[b]]/v'
- '/[a&&[^b]&&c]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a&&b&&[^c]&&d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a&&b&&d]--[c]]/v'
- '/[a&&b&&[^c]&&d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a]&&b&&c]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[b&&c]--[a]]/v'
- '/[[^a]&&b&&c]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^b]&&a]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a--[b]]/v'
- '/[[^b]&&a]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[abc]&&[^def]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[abc]--[def]]/v'
- '/[[abc]&&[^def]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a--[^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a&&[b]]/v'
- '/[a--[^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a--[^b]--c]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a&&[b]]--c]/v'
- '/[a--[^b]--c]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a--b--[^c]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a--b]&&[c]]/v'
- '/[a--b--[^c]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[abc]--[^def]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[abc]&&[def]]/v'
- '/[[abc]--[^def]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a]&&[^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^[a][b]]/v'
- '/[[^a]&&[^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[^[^a]&&[^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a][b]]/v'
- '/[^[^a]&&[^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a]&&[^b]&&\D]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^[a][b]\\d]/v'
- '/[[^a]&&[^b]&&\\D]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[^[^a]&&[^b]&&\D]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a][b]\\d]/v'
- '/[^[^a]&&[^b]&&\\D]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a]&&\D&&b]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[^[a]\\d]&&b]/v'
- '/[[^a]&&\\D&&b]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^abc]&&[^def]&&\D]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^[abc][def]\\d]/v'
- '/[[^abc]&&[^def]&&\\D]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a]&&[b]&&[^c]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[^[a][c]]&&[b]]/v'
- '/[[^a]&&[b]&&[^c]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^a][^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^[a]&&[b]]/v'
- '/[[^a][^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^abc][^def]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[^[abc]&&[def]]/v'
- '/[[^abc][^def]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[^[^a][^b]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a]&&[b]]/v'
- '/[^[^a][^b]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[^\S\P{ASCII}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s&&\\p{ASCII}]/v'
- '/[^\\S\\P{ASCII}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[a&&[^b]&&[^c]&&d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[^[b][c]]&&a&&d]/v'
- '/[a&&[^b]&&[^c]&&d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### simplify-set-operations > invalid

```js
/[[^bc]&&a&&d]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a&&d]--[bc]]/v'
- '/[[^bc]&&a&&d]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `sort-alternatives`

Pass: 11 / 34 (32.4%)
Fail: 23 / 34 (67.6%)
Skip: 0 / 34 (0.0%)

#### sort-alternatives > invalid

```js
/c|b|a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a|b|c/'
- '/c|b|a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/c|bb|a/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a|bb|c/'
- '/c|bb|a/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:c|b|a)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:a|b|c)\\b/'
- '/\\b(?:c|b|a)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:A|a|C|c|B|b)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:A|B|C|a|b|c)\\b/'
- '/\\b(?:A|a|C|c|B|b)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:aa|aA|aB|ab)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:aA|aB|aa|ab)\\b/'
- '/\\b(?:aa|aA|aB|ab)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:A|a|C|c|B|b)\b/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:A|a|B|b|C|c)\\b/i'
- '/\\b(?:A|a|C|c|B|b)\\b/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:a|A|c|C|b|B)\b/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:A|a|B|b|C|c)\\b/i'
- '/\\b(?:a|A|c|C|b|B)\\b/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:aa|aA|aB|ab)\b/i
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:aA|aa|aB|ab)\\b/i'
- '/\\b(?:aa|aA|aB|ab)\\b/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:1|2|4|8|16|32|64|128|256|0)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:0|1|2|4|8|16|32|64|128|256)\\b/'
- '/\\b(?:1|2|4|8|16|32|64|128|256|0)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:[Nn]umber|[Ss]tring|[Bb]oolean|Function|any|mixed|null|void)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:[Bb]oolean|Function|[Nn]umber|[Ss]tring|any|mixed|null|void)\\b/'
- '/\\b(?:[Nn]umber|[Ss]tring|[Bb]oolean|Function|any|mixed|null|void)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/_(?:SERVER|GET|POST|FILES|REQUEST|SESSION|ENV|COOKIE)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/_(?:COOKIE|ENV|FILES|GET|POST|REQUEST|SERVER|SESSION)\\b/'
- '/_(?:SERVER|GET|POST|FILES|REQUEST|SESSION|ENV|COOKIE)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b[ui](?:128|16|32|64|8|size)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b[ui](?:8|16|32|64|128|size)\\b/'
- '/\\b[ui](?:128|16|32|64|8|size)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\((?:TM|R|C)\)/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\((?:C|R|TM)\\)/'
- '/\\((?:TM|R|C)\\)/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:\^|c|b|a)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:\\^|a|b|c)\\b/'
- '/\\b(?:\\^|c|b|a)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:green|gr[ae]y)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:gr[ae]y|green)\\b/'
- '/\\b(?:green|gr[ae]y)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:(?:script|source)_foo|sample)\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:sample|(?:script|source)_foo)\\b/'
- '/\\b(?:(?:script|source)_foo|sample)\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/[\q{red|green|blue}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{blue|green|red}]/v'
- '/[\\q{red|green|blue}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/(?:c|[\q{red|green|blue}]|a)/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:a|[\\q{red|green|blue}]|c)/v'
- '/(?:c|[\\q{red|green|blue}]|a)/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/[\q{ac|ab|aa}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{aa|ab|ac}]/v'
- '/[\\q{ac|ab|aa}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/(?:b|[a-b])/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:[a-b]|b)/v'
- '/(?:b|[a-b])/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:a[\q{bd}]|abc)\b/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:abc|a[\\q{bd}])\\b/v'
- '/\\b(?:a[\\q{bd}]|abc)\\b/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:abb|[\q{a|aba}]bb)\b/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:[\\q{a|aba}]bb|abb)\\b/v'
- '/\\b(?:abb|[\\q{a|aba}]bb)\\b/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-alternatives > invalid

```js
/\b(?:c|b_|[\q{b|da}]_|b_2)\b/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b(?:b_|[\\q{b|da}]_|b_2|c)\\b/v'
- '/\\b(?:c|b_|[\\q{b|da}]_|b_2)\\b/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `sort-character-class-elements`

Pass: 22 / 52 (42.3%)
Fail: 30 / 52 (57.7%)
Skip: 0 / 52 (0.0%)

#### sort-character-class-elements > invalid

```js
/[acdb]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[abcd]/'
- '/[acdb]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[b-da]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab-d]/'
- '/[b-da]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[b-da]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[ab-d]/'
- '/[b-da]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[da-c]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-cd]/'
- '/[da-c]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[da-c]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-cd]/'
- '/[da-c]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[abcd\d]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\dabcd]/'
- '/[abcd\\d]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\s\d\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\w\\d]/'
- '/[\\s\\d\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\s\d\w]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\w\\d]/'
- '/[\\s\\d\\w]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\p{ASCII}\w]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w\\p{ASCII}]/u'
- '/[\\p{ASCII}\\w]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\p{ASCII}\w]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w\\p{ASCII}]/u'
- '/[\\p{ASCII}\\w]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\p{Script=Hiragana}\p{ASCII}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{ASCII}\\p{Script=Hiragana}]/u'
- '/[\\p{Script=Hiragana}\\p{ASCII}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\p{Script=Hiragana}\p{Script=Han}]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\p{Script=Han}\\p{Script=Hiragana}]/u'
- '/[\\p{Script=Hiragana}\\p{Script=Han}]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\d\w]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\w\\d]/u'
- '/[\\d\\w]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\da-b-]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\d\\-a-b]/u'
- '/[\\da-b-]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[a-b-]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[-a-b]/u'
- '/[a-b-]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[-$a]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[$\\-a]/u'
- '/[-$a]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[-_\s]+/gu
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\s\\-_]+/gu'
- '/[-_\\s]+/gu'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[-_-]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[-\\-_]/u'
- '/[-_-]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
const s = "[\\d\\w]"
            new RegExp(s, 'u')
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `const s = "[\\\\w\\\\d]"\n            new RegExp(s, 'u')`
- `const s = "[\\\\d\\\\w]"\n            new RegExp(s, 'u')`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js

            const jsxWhitespaceChars = " \n\r\t";
            const matchJsxWhitespaceRegex = new RegExp("([" + jsxWhitespaceChars + "]+)");
            
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '\n' +
+   '            const jsxWhitespaceChars = "\\n \\r\\t";\n' +
-   '            const jsxWhitespaceChars = " \\n\\r\\t";\n' +
    '            const matchJsxWhitespaceRegex = new RegExp("([" + jsxWhitespaceChars + "]+)");\n' +
    '            '

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[[a--b][a]\q{a}a]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a}[a--b][a]a]/v'
- '/[[a--b][a]\\q{a}a]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\q{a}[a--b][a]a]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a\\q{a}[a--b][a]]/v'
- '/[\\q{a}[a--b][a]a]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[[b--c][a]]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[[a][b--c]]/v'
- '/[[b--c][a]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[[a]\q{a}]/v; /[\q{a}a]/v; /[[b-c]\q{a}]/v; /[[b-c][a]]/v;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "order": []
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a}[a]]/v; /[a\\q{a}]/v; /[\\q{a}[b-c]]/v; /[[a][b-c]]/v;'
- '/[[a]\\q{a}]/v; /[\\q{a}a]/v; /[[b-c]\\q{a}]/v; /[[b-c][a]]/v;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\q{c}\q{b}\q{a}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{b}\\q{c}\\q{a}]/v'
- '/[\\q{c}\\q{b}\\q{a}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\q{b}\q{c}\q{a}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a}\\q{b}\\q{c}]/v'
- '/[\\q{b}\\q{c}\\q{a}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\q{ac}\q{ab}\q{aa}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{ab}\\q{ac}\\q{aa}]/v'
- '/[\\q{ac}\\q{ab}\\q{aa}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[\q{ab}\q{ac}\q{aa}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{aa}\\q{ab}\\q{ac}]/v'
- '/[\\q{ab}\\q{ac}\\q{aa}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[~^*]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^~*]/'
- '/[~^*]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-character-class-elements > invalid

```js
/[~^]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\^~]/'
- '/[~^]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `sort-flags`

Pass: 13 / 18 (72.2%)
Fail: 5 / 18 (27.8%)
Skip: 0 / 18 (0.0%)

#### sort-flags > invalid

```js
new RegExp("\\w", "yusimg")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("\\\\w", "gimsuy")'
- 'new RegExp("\\\\w", "yusimg")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-flags > invalid

```js
new RegExp("\\w", "yusimgd")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("\\\\w", "dgimsuy")'
- 'new RegExp("\\\\w", "yusimgd")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-flags > invalid

```js
new RegExp("\\w)", "ui")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'new RegExp("\\\\w)", "iu")'
- 'new RegExp("\\\\w)", "ui")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-flags > invalid

```js
RegExp('a' + b, 'us');
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "RegExp('a' + b, 'su');"
- "RegExp('a' + b, 'us');"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### sort-flags > invalid

```js
var a = "foo"; RegExp(foo, 'us'); RegExp(foo, 'u');
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ `var a = "foo"; RegExp(foo, 'su'); RegExp(foo, 'u');`
- `var a = "foo"; RegExp(foo, 'us'); RegExp(foo, 'u');`

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `strict`

Pass: 24 / 31 (77.4%)
Fail: 7 / 31 (22.6%)
Skip: 0 / 31 (0.0%)

#### strict > invalid

```js
/]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\]/' !== '/]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/{/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\{/' !== '/{/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.

'/\\}/' !== '/}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/\p{H}/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p\\{H\\}/'
- '/\\p{H}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/\; \_ \a \- \'/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "/; _ a - '/"
- "/\\; \\_ \\a \\- \\'/"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/[\; \_ \a \']/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ "/[; _ a ']/"
- "/[\\; \\_ \\a \\']/"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### strict > invalid

```js
/(?!a)+/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:(?!a))+/'
- '/(?!a)+/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `unicode-escape`

Pass: 6 / 11 (54.5%)
Fail: 5 / 11 (45.5%)
Skip: 0 / 11 (0.0%)

#### unicode-escape > invalid

```js
/a \x0a \cM \0 \u0100 \u00ff \ud83d\ude00 \u{1f600}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a \\x0a \\cM \\0 \\u{100} \\u{ff} \\ud83d\\ude00 \\u{1f600}/u'
- '/a \\x0a \\cM \\0 \\u0100 \\u00ff \\ud83d\\ude00 \\u{1f600}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-escape > invalid

```js
/a \x0a \cM \0 \u0100 \u00ff \ud83d\ude00 \u{1f600}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "unicodeCodePointEscape"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a \\x0a \\cM \\0 \\u{100} \\u{ff} \\ud83d\\ude00 \\u{1f600}/u'
- '/a \\x0a \\cM \\0 \\u0100 \\u00ff \\ud83d\\ude00 \\u{1f600}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-escape > invalid

```js
/a \x0a \cM \0 \u{ff} \u{100} \ud83d\ude00 \u{1f600}/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "unicodeEscape"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a \\x0a \\cM \\0 \\u00ff \\u0100 \\ud83d\\ude00 \\u{1f600}/u'
- '/a \\x0a \\cM \\0 \\u{ff} \\u{100} \\ud83d\\ude00 \\u{1f600}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-escape > invalid

```js
/a \x0a \cM \0 \u0100 \u00ff \ud83d\ude00 \u{1f600}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "unicodeCodePointEscape"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a \\x0a \\cM \\0 \\u{100} \\u{ff} \\ud83d\\ude00 \\u{1f600}/v'
- '/a \\x0a \\cM \\0 \\u0100 \\u00ff \\ud83d\\ude00 \\u{1f600}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-escape > invalid

```js
/a \x0a \cM \0 \u{ff} \u{100} \ud83d\ude00 \u{1f600}/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    "unicodeEscape"
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/a \\x0a \\cM \\0 \\u00ff \\u0100 \\ud83d\\ude00 \\u{1f600}/v'
- '/a \\x0a \\cM \\0 \\u{ff} \\u{100} \\ud83d\\ude00 \\u{1f600}/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `unicode-property`

Pass: 1 / 10 (10.0%)
Fail: 9 / 10 (90.0%)
Skip: 0 / 10 (0.0%)

#### unicode-property > invalid

```js
test default configuration
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "name": "test default configuration",
  "code": "/\\P{ASCII}/u;\n/\\P{Hex_Digit}/u;\n/\\P{Hex}/u;\n\n/\\p{L}/u;\n/\\p{Letter}/u;\n/\\p{gc=L}/u;\n/\\p{gc=Letter}/u;\n/\\p{General_Category=L}/u;\n/\\p{General_Category=Letter}/u;\n\n/\\p{sc=Grek}/u;\n/\\p{sc=Greek}/u;\n/\\p{Script=Grek}/u;\n/\\p{Script=Greek}/u;\n\n/\\p{scx=Grek}/u;\n/\\p{scx=Greek}/u;\n/\\p{Script_Extensions=Grek}/u;\n/\\p{Script_Extensions=Greek}/u;\n\n// Binary Properties\n// https://github.com/tc39/ecma262/issues/3286\n// /\\p{White_Space} \\p{space} \\p{WSpace}/u;\n\n// General_Category\n/\\p{Control} \\p{Cc} \\p{cntrl}/u;\n/\\p{Mark} \\p{M} \\p{Combining_Mark}/u;\n/\\p{Decimal_Number} \\p{Nd} \\p{digit}/u;\n/\\p{Punctuation} \\p{P} \\p{punct}/u;",
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '/\\P{ASCII}/u;\n' +
    '/\\P{Hex_Digit}/u;\n' +
    '/\\P{Hex}/u;\n' +
    '\n' +
    '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
+   '/\\p{L}/u;\n' +
+   '/\\p{Letter}/u;\n' +
+   '/\\p{L}/u;\n' +
+   '/\\p{Letter}/u;\n' +
-   '/\\p{gc=L}/u;\n' +
-   '/\\p{gc=Letter}/u;\n' +
-   '/\\p{General_Category=L}/u;\n' +
-   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
-   '/\\p{sc=Grek}/u;\n' +
    '/\\p{sc=Greek}/u;\n' +
+   '/\\p{sc=Greek}/u;\n' +
-   '/\\p{Script=Grek}/u;\n' +
    '/\\p{Script=Greek}/u;\n' +
+   '/\\p{Script=Greek}/u;\n' +
    '\n' +
-   '/\\p{scx=Grek}/u;\n' +
    '/\\p{scx=Greek}/u;\n' +
+   '/\\p{scx=Greek}/u;\n' +
-   '/\\p{Script_Extensions=Grek}/u;\n' +
    '/\\p{Script_Extensions=Greek}/u;\n' +
+   '/\\p{Script_Extensions=Greek}/u;\n' +
    '\n' +
    '// Binary Properties\n' +
    '// https://github.com/tc39/ecma262/issues/3286\n' +
    '// /\\p{White_Space} \\p{space} \\p{WSpace}/u;\n' +
    '\n' +

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "always",
      "key": "ignore",
      "property": "ignore"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{gc=L}/u;\n' +
+   '/\\p{gc=Letter}/u;\n' +
- '/\\p{L}/u;\n' +
-   '/\\p{Letter}/u;\n' +
    '/\\p{gc=L}/u;\n' +
    '/\\p{gc=Letter}/u;\n' +
    '/\\p{General_Category=L}/u;\n' +
    '/\\p{General_Category=Letter}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "never",
      "key": "ignore",
      "property": "ignore"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\p{L}/u;\n/\\p{Letter}/u;\n/\\p{L}/u;\n/\\p{Letter}/u;\n/\\p{L}/u;\n/\\p{Letter}/u;'
- '/\\p{L}/u;\n' +
-   '/\\p{Letter}/u;\n' +
-   '/\\p{gc=L}/u;\n' +
-   '/\\p{gc=Letter}/u;\n' +
-   '/\\p{General_Category=L}/u;\n' +
-   '/\\p{General_Category=Letter}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "long",
      "property": "ignore"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
-   '/\\p{gc=L}/u;\n' +
-   '/\\p{gc=Letter}/u;\n' +
    '/\\p{General_Category=L}/u;\n' +
    '/\\p{General_Category=Letter}/u;\n' +
+   '/\\p{General_Category=L}/u;\n' +
+   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
-   '/\\p{sc=Grek}/u;\n' +
-   '/\\p{sc=Greek}/u;\n' +
    '/\\p{Script=Grek}/u;\n' +
    '/\\p{Script=Greek}/u;\n' +
+   '/\\p{Script=Grek}/u;\n' +
+   '/\\p{Script=Greek}/u;\n' +
    '\n' +
-   '/\\p{scx=Grek}/u;\n' +
-   '/\\p{scx=Greek}/u;\n' +
    '/\\p{Script_Extensions=Grek}/u;\n' +
+   '/\\p{Script_Extensions=Greek}/u;\n' +
+   '/\\p{Script_Extensions=Grek}/u;\n' +
    '/\\p{Script_Extensions=Greek}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "short",
      "property": "ignore"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
    '/\\p{gc=L}/u;\n' +
    '/\\p{gc=Letter}/u;\n' +
+   '/\\p{gc=L}/u;\n' +
+   '/\\p{gc=Letter}/u;\n' +
-   '/\\p{General_Category=L}/u;\n' +
-   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
    '/\\p{sc=Grek}/u;\n' +
    '/\\p{sc=Greek}/u;\n' +
+   '/\\p{sc=Grek}/u;\n' +
+   '/\\p{sc=Greek}/u;\n' +
-   '/\\p{Script=Grek}/u;\n' +
-   '/\\p{Script=Greek}/u;\n' +
    '\n' +
    '/\\p{scx=Grek}/u;\n' +
    '/\\p{scx=Greek}/u;\n' +
+   '/\\p{scx=Grek}/u;\n' +
+   '/\\p{scx=Greek}/u;'
-   '/\\p{Script_Extensions=Grek}/u;\n' +
-   '/\\p{Script_Extensions=Greek}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\P{ASCII}/u;
/\P{Hex_Digit}/u;
/\P{Hex}/u;

/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;

// Binary Properties
// https://github.com/tc39/ecma262/issues/3286
// /\p{White_Space} \p{space} \p{WSpace}/u;

// General_Category
/\p{Control} \p{Cc} \p{cntrl}/u;
/\p{Mark} \p{M} \p{Combining_Mark}/u;
/\p{Decimal_Number} \p{Nd} \p{digit}/u;
/\p{Punctuation} \p{P} \p{punct}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "ignore",
      "property": "long"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '/\\P{ASCII}/u;\n' +
    '/\\P{Hex_Digit}/u;\n' +
+   '/\\P{Hex_Digit}/u;\n' +
-   '/\\P{Hex}/u;\n' +
    '\n' +
-   '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
+   '/\\p{Letter}/u;\n' +
-   '/\\p{gc=L}/u;\n' +
    '/\\p{gc=Letter}/u;\n' +
+   '/\\p{gc=Letter}/u;\n' +
-   '/\\p{General_Category=L}/u;\n' +
    '/\\p{General_Category=Letter}/u;\n' +
+   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
-   '/\\p{sc=Grek}/u;\n' +
    '/\\p{sc=Greek}/u;\n' +
+   '/\\p{sc=Greek}/u;\n' +
-   '/\\p{Script=Grek}/u;\n' +
    '/\\p{Script=Greek}/u;\n' +
+   '/\\p{Script=Greek}/u;\n' +
    '\n' +
-   '/\\p{scx=Grek}/u;\n' +
    '/\\p{scx=Greek}/u;\n' +
+   '/\\p{scx=Greek}/u;\n' +
-   '/\\p{Script_Extensions=Grek}/u;\n' +
    '/\\p{Script_Extensions=Greek}/u;\n' +
+   '/\\p{Script_Extensions=Greek}/u;\n' +
    '\n' +
    '// Binary Properties\n' +
    '// https://github.com/tc39/ecma262/issues/3286\n' +
    '// /\\p{White_Space} \\p{space} \\p{WSpace}/u;\n' +
    '\n' +
    '// General_Category\n' +
+   '/\\p{Control} \\p{Control} \\p{Control}/u;\n' +
+   '/\\p{Mark} \\p{Mark} \\p{Mark}/u;\n' +
+   '/\\p{Decimal_Number} \\p{Decimal_Number} \\p{Decimal_Number}/u;\n' +
+   '/\\p{Punctuation} \\p{Punctuation} \\p{Punctuation}/u;'
-   '/\\p{Control} \\p{Cc} \\p{cntrl}/u;\n' +
-   '/\\p{Mark} \\p{M} \\p{Combining_Mark}/u;\n' +
-   '/\\p{Decimal_Number} \\p{Nd} \\p{digit}/u;\n' +
-   '/\\p{Punctuation} \\p{P} \\p{punct}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\P{ASCII}/u;
/\P{Hex_Digit}/u;
/\P{Hex}/u;

/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;

// Binary Properties
// https://github.com/tc39/ecma262/issues/3286
// /\p{White_Space} \p{space} \p{WSpace}/u;

// General_Category
/\p{Control} \p{Cc} \p{cntrl}/u;
/\p{Mark} \p{M} \p{Combining_Mark}/u;
/\p{Decimal_Number} \p{Nd} \p{digit}/u;
/\p{Punctuation} \p{P} \p{punct}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "ignore",
      "property": "short"
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

  '/\\P{ASCII}/u;\n' +
-   '/\\P{Hex_Digit}/u;\n' +
    '/\\P{Hex}/u;\n' +
+   '/\\P{Hex}/u;\n' +
    '\n' +
    '/\\p{L}/u;\n' +
+   '/\\p{L}/u;\n' +
-   '/\\p{Letter}/u;\n' +
    '/\\p{gc=L}/u;\n' +
+   '/\\p{gc=L}/u;\n' +
-   '/\\p{gc=Letter}/u;\n' +
    '/\\p{General_Category=L}/u;\n' +
+   '/\\p{General_Category=L}/u;\n' +
-   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
    '/\\p{sc=Grek}/u;\n' +
+   '/\\p{sc=Grek}/u;\n' +
-   '/\\p{sc=Greek}/u;\n' +
    '/\\p{Script=Grek}/u;\n' +
+   '/\\p{Script=Grek}/u;\n' +
-   '/\\p{Script=Greek}/u;\n' +
    '\n' +
    '/\\p{scx=Grek}/u;\n' +
+   '/\\p{scx=Grek}/u;\n' +
-   '/\\p{scx=Greek}/u;\n' +
    '/\\p{Script_Extensions=Grek}/u;\n' +
+   '/\\p{Script_Extensions=Grek}/u;\n' +
-   '/\\p{Script_Extensions=Greek}/u;\n' +
    '\n' +
    '// Binary Properties\n' +
    '// https://github.com/tc39/ecma262/issues/3286\n' +
    '// /\\p{White_Space} \\p{space} \\p{WSpace}/u;\n' +
    '\n' +
    '// General_Category\n' +
+   '/\\p{Cc} \\p{Cc} \\p{Cc}/u;\n' +
+   '/\\p{M} \\p{M} \\p{M}/u;\n' +
+   '/\\p{Nd} \\p{Nd} \\p{Nd}/u;\n' +
+   '/\\p{P} \\p{P} \\p{P}/u;'
-   '/\\p{Control} \\p{Cc} \\p{cntrl}/u;\n' +
-   '/\\p{Mark} \\p{M} \\p{Combining_Mark}/u;\n' +
-   '/\\p{Decimal_Number} \\p{Nd} \\p{digit}/u;\n' +
-   '/\\p{Punctuation} \\p{P} \\p{punct}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\P{ASCII}/u;
/\P{Hex_Digit}/u;
/\P{Hex}/u;

/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;

// Binary Properties
// https://github.com/tc39/ecma262/issues/3286
// /\p{White_Space} \p{space} \p{WSpace}/u;

// General_Category
/\p{Control} \p{Cc} \p{cntrl}/u;
/\p{Mark} \p{M} \p{Combining_Mark}/u;
/\p{Decimal_Number} \p{Nd} \p{digit}/u;
/\p{Punctuation} \p{P} \p{punct}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "ignore",
      "property": {
        "binary": "short",
        "generalCategory": "long",
        "script": "ignore"
      }
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected
... Skipped lines

  '/\\P{ASCII}/u;\n' +
-   '/\\P{Hex_Digit}/u;\n' +
    '/\\P{Hex}/u;\n' +
+   '/\\P{Hex}/u;\n' +
    '\n' +
-   '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
+   '/\\p{Letter}/u;\n' +
-   '/\\p{gc=L}/u;\n' +
    '/\\p{gc=Letter}/u;\n' +
+   '/\\p{gc=Letter}/u;\n' +
-   '/\\p{General_Category=L}/u;\n' +
    '/\\p{General_Category=Letter}/u;\n' +
+   '/\\p{General_Category=Letter}/u;\n' +
    '\n' +
    '/\\p{sc=Grek}/u;\n' +
    '/\\p{sc=Greek}/u;\n' +
    '/\\p{Script=Grek}/u;\n' +
    '/\\p{Script=Greek}/u;\n' +
...
    '// General_Category\n' +
+   '/\\p{Control} \\p{Control} \\p{Control}/u;\n' +
+   '/\\p{Mark} \\p{Mark} \\p{Mark}/u;\n' +
+   '/\\p{Decimal_Number} \\p{Decimal_Number} \\p{Decimal_Number}/u;\n' +
+   '/\\p{Punctuation} \\p{Punctuation} \\p{Punctuation}/u;'
-   '/\\p{Control} \\p{Cc} \\p{cntrl}/u;\n' +
-   '/\\p{Mark} \\p{M} \\p{Combining_Mark}/u;\n' +
-   '/\\p{Decimal_Number} \\p{Nd} \\p{digit}/u;\n' +
-   '/\\p{Punctuation} \\p{P} \\p{punct}/u;'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### unicode-property > invalid

```js
/\P{ASCII}/u;
/\P{Hex_Digit}/u;
/\P{Hex}/u;

/\p{L}/u;
/\p{Letter}/u;
/\p{gc=L}/u;
/\p{gc=Letter}/u;
/\p{General_Category=L}/u;
/\p{General_Category=Letter}/u;

/\p{sc=Grek}/u;
/\p{sc=Greek}/u;
/\p{Script=Grek}/u;
/\p{Script=Greek}/u;

/\p{scx=Grek}/u;
/\p{scx=Greek}/u;
/\p{Script_Extensions=Grek}/u;
/\p{Script_Extensions=Greek}/u;

// Binary Properties
// https://github.com/tc39/ecma262/issues/3286
// /\p{White_Space} \p{space} \p{WSpace}/u;

// General_Category
/\p{Control} \p{Cc} \p{cntrl}/u;
/\p{Mark} \p{M} \p{Combining_Mark}/u;
/\p{Decimal_Number} \p{Nd} \p{digit}/u;
/\p{Punctuation} \p{P} \p{punct}/u;
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "options": [
    {
      "generalCategory": "ignore",
      "key": "ignore",
      "property": {
        "binary": "long",
        "generalCategory": "ignore",
        "script": "short"
      }
    }
  ],
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected
... Skipped lines

  '/\\P{ASCII}/u;\n' +
    '/\\P{Hex_Digit}/u;\n' +
+   '/\\P{Hex_Digit}/u;\n' +
-   '/\\P{Hex}/u;\n' +
    '\n' +
    '/\\p{L}/u;\n' +
    '/\\p{Letter}/u;\n' +
    '/\\p{gc=L}/u;\n' +
    '/\\p{gc=Letter}/u;\n' +
...
    '/\\p{sc=Grek}/u;\n' +
+   '/\\p{sc=Grek}/u;\n' +
-   '/\\p{sc=Greek}/u;\n' +
    '/\\p{Script=Grek}/u;\n' +
+   '/\\p{Script=Grek}/u;\n' +
-   '/\\p{Script=Greek}/u;\n' +
    '\n' +
    '/\\p{scx=Grek}/u;\n' +
+   '/\\p{scx=Grek}/u;\n' +
-   '/\\p{scx=Greek}/u;\n' +
    '/\\p{Script_Extensions=Grek}/u;\n' +
+   '/\\p{Script_Extensions=Grek}/u;\n' +
-   '/\\p{Script_Extensions=Greek}/u;\n' +
    '\n' +
    '// Binary Properties\n' +
    '// https://github.com/tc39/ecma262/issues/3286\n' +
    '// /\\p{White_Space} \\p{space} \\p{WSpace}/u;\n' +
    '\n' +

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `use-ignore-case`

Pass: 20 / 28 (71.4%)
Fail: 8 / 28 (28.6%)
Skip: 0 / 28 (0.0%)

#### use-ignore-case > invalid

```js
/[a-zA-Z]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a-z]/i'
- '/[a-zA-Z]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/[aA][aA][aA][aA][aA]/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a][a][a][a][a]/i'
- '/[aA][aA][aA][aA][aA]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/[aA]/u
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a]/iu'
- '/[aA]/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/[aA]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[a]/iv'
- '/[aA]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/\b0[xX][a-fA-F0-9]+\b/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/\\b0[x][a-f0-9]+\\b/i'
- '/\\b0[xX][a-fA-F0-9]+\\b/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
RegExp("[a-zA-Z]")
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ 'RegExp("[a-z]", "i")'
- 'RegExp("[a-zA-Z]")'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/[\q{a|A}]/v
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/[\\q{a}]/iv'
- '/[\\q{a|A}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### use-ignore-case > invalid

```js
/(?:(?<foo>[aA])|(?<foo>[bB]))\k<foo>/
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__"
}
```

AssertionError [ERR_ASSERTION]: The rule fixed the code. Please add `output` property.
+ actual - expected

+ '/(?:(?<foo>[a])|(?<foo>[b]))\\k<foo>/i'
- '/(?:(?<foo>[aA])|(?<foo>[bB]))\\k<foo>/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)

