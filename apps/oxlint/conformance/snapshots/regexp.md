# Conformance test results - regexp

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    82 | 100.0% |
| Fully passing     |    65 |  79.3% |
| Partially passing |    17 |  20.7% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  2370 | 100.0% |
| Passing     |  2312 |  97.6% |
| Failing     |    46 |   1.9% |
| Skipped     |    12 |   0.5% |

## Fully Passing Rules

- `confusing-quantifier` (9 tests)
- `control-character-escape` (19 tests)
- `grapheme-string-literal` (157 tests)
- `hexadecimal-escape` (15 tests)
- `negation` (30 tests)
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
- `no-extra-lookaround-assertions` (13 tests)
- `no-invalid-regexp` (8 tests)
- `no-legacy-features` (40 tests) (1 skipped)
- `no-misleading-capturing-group` (16 tests)
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
- `no-useless-dollar-replacements` (29 tests)
- `no-useless-escape` (143 tests)
- `no-useless-lazy` (23 tests)
- `no-useless-non-capturing-group` (54 tests)
- `no-useless-quantifier` (20 tests)
- `no-useless-range` (13 tests)
- `no-useless-string-literal` (12 tests)
- `no-useless-two-nums-quantifier` (10 tests)
- `no-zero-quantifier` (7 tests)
- `optimal-lookaround-quantifier` (10 tests)
- `prefer-character-class` (54 tests)
- `prefer-d` (25 tests)
- `prefer-escape-replacement-dollar-char` (16 tests)
- `prefer-named-backreference` (6 tests)
- `prefer-named-capture-group` (6 tests)
- `prefer-named-replacement` (16 tests)
- `prefer-plus-quantifier` (13 tests)
- `prefer-predefined-assertion` (14 tests)
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
- `strict` (31 tests)
- `unicode-escape` (11 tests)
- `unicode-property` (10 tests)
- `use-ignore-case` (28 tests)

## Rules with Failures

- `letter-case` - 57 / 58 (98.3%)
- `match-any` - 38 / 40 (95.0%)
- `no-dupe-characters-character-class` - 46 / 59 (78.0%)
- `no-invisible-character` - 15 / 17 (88.2%)
- `no-lazy-ends` - 17 / 18 (94.4%)
- `no-misleading-unicode-character` - 66 / 67 (98.5%)
- `no-unused-capturing-group` - 66 / 67 (98.5%)
- `no-useless-character-class` - 59 / 61 (96.7%)
- `no-useless-flag` - 76 / 82 (92.7%)
- `no-useless-set-operand` - 13 / 14 (92.9%)
- `optimal-quantifier-concatenation` - 55 / 56 (98.2%)
- `prefer-lookaround` - 85 / 88 (96.6%)
- `prefer-quantifier` - 24 / 25 (96.0%)
- `simplify-set-operations` - 33 / 36 (91.7%)
- `sort-alternatives` - 33 / 34 (97.1%)
- `sort-character-class-elements` - 47 / 52 (90.4%)
- `sort-flags` - 16 / 18 (88.9%)

## Rules with Failures Detail

### `letter-case`

Pass: 57 / 58 (98.3%)
Fail: 1 / 58 (1.7%)
Skip: 0 / 58 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[j-z]/i"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\u004a-Z]/i'
- '/[j-z]/i'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `match-any`

Pass: 38 / 40 (95.0%)
Fail: 2 / 40 (5.0%)
Skip: 0 / 40 (0.0%)

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
  "errors": "__unknown__",
  "output": "/. . . ./s"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/. [\\S\\s] [^] ./s'
- '/. . . ./s'

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
  "errors": "__unknown__",
  "output": "/. . . ./s"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/. . [^] ./s'
- '/. . . ./s'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-dupe-characters-character-class`

Pass: 46 / 59 (78.0%)
Fail: 13 / 59 (22.0%)
Skip: 0 / 59 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[a]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect

'/[aa]/' !== '/[a]/'

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
  "errors": "__unknown__",
  "output": "/[\\s\\u180e]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\s\\f\\r\\v\\u1680\\u180e\\u2028\\u202f\\u3000]/'
- '/[\\s\\u180e]/'

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
  "errors": "__unknown__",
  "output": "/[!-z]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[!-zac]/'
- '/[!-z]/'

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
  "errors": "__unknown__",
  "output": "/[\\w-][\\s]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\wac-][\\s\\t\\n\\u3000]/'
- '/[\\w-][\\s]/'

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
  "errors": "__unknown__",
  "output": "/[\\s]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\s ]/'
- '/[\\s]/'

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
  "errors": "__unknown__",
  "output": "/[\\S\\s]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\S \\s]/'
- '/[\\S\\s]/'

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
  "errors": "__unknown__",
  "output": "/[ !-z]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[ _!-z]/'
- '/[ !-z]/'

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
  "errors": "__unknown__",
  "output": "/[\\w\\D]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\W\\w\\d\\D]/'
- '/[\\w\\D]/'

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
  "errors": "__unknown__",
  "output": "/[\\P{ASCII}\\P{Script=Hiragana}]/u"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\P{ASCII}\\P{Script=Hiragana}\\p{Script=Hiragana}]/u'
- '/[\\P{ASCII}\\P{Script=Hiragana}]/u'

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
  "errors": "__unknown__",
  "output": "/[\\p{ASCII}\\P{ASCII}]/u"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\p{ASCII}ac\\P{ASCII}]/u'
- '/[\\p{ASCII}\\P{ASCII}]/u'

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
  "errors": "__unknown__",
  "output": "/[\\P{Script=Hiragana}\\p{Script=Hiragana}]/u"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\P{Script=Hiragana}ac\\p{Script=Hiragana}]/u'
- '/[\\P{Script=Hiragana}\\p{Script=Hiragana}]/u'

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
  "errors": "__unknown__",
  "output": "/[a-c[\\w--b]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[aa-c[\\w--b]]/v'
- '/[a-c[\\w--b]]/v'

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
  "errors": "__unknown__",
  "output": "/[\\q{abc|ab}]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\q{abc|ab}[\\q{abc}&&\\q{abc|ab}]]/v'
- '/[\\q{abc|ab}]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-invisible-character`

Pass: 15 / 17 (88.2%)
Fail: 2 / 17 (11.8%)
Skip: 0 / 17 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[\\t\\xa0\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000\\ufeff\\x85\\u200b]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\t \\u1680᠎\\u2000 \\u2002 \\u2004 \\u2006 \\u2008 \\u200a \\u205f　\\ufeff\x85\\u200b]/'
- '/[\\t\\xa0\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000\\ufeff\\x85\\u200b]/'

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
  "errors": "__unknown__",
  "output": "new RegExp('\\t\\xa0\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000\\ufeff\\x85\\u200b')"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ "new RegExp('\\t \\u1680᠎\\u2000 \\u2002 \\u2004 \\u2006 \\u2008 \\u200a \\u205f　\\ufeff\x85\\u200b')"
- "new RegExp('\\t\\xa0\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000\\ufeff\\x85\\u200b')"

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

Pass: 66 / 67 (98.5%)
Fail: 1 / 67 (1.5%)
Skip: 0 / 67 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[\\q{👶🏻}[\\q{👨‍👩‍👦}]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\q{👶🏻}[👨‍👩‍👦]]/v'
- '/[\\q{👶🏻}[\\q{👨‍👩‍👦}]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


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


### `no-useless-character-class`

Pass: 59 / 61 (96.7%)
Fail: 2 / 61 (3.3%)
Skip: 0 / 61 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[abcdefghi]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[abc[def]ghi]/v'
- '/[abcdefghi]/v'

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
  "errors": "__unknown__",
  "output": "/\\^/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\^]/v'
- '/\\^/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-useless-flag`

Pass: 76 / 82 (92.7%)
Fail: 6 / 82 (7.3%)
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
/\w/ims
```

```json
{
  "languageOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module"
  },
  "errors": "__unknown__",
  "output": "/\\w/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect

'/\\w/ms' !== '/\\w/'

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
  "errors": "__unknown__",
  "output": "/\\w/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect

'/\\w/s' !== '/\\w/'

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


### `no-useless-set-operand`

Pass: 13 / 14 (92.9%)
Fail: 1 / 14 (7.1%)
Skip: 0 / 14 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[[\\d]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\w&&[\\d]]/v'
- '/[[\\d]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `optimal-quantifier-concatenation`

Pass: 55 / 56 (98.2%)
Fail: 1 / 56 (1.8%)
Skip: 0 / 56 (0.0%)

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
  "errors": "__unknown__",
  "output": "/a{0,3}/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/a{0,2}a?/'
- '/a{0,3}/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-lookaround`

Pass: 85 / 88 (96.6%)
Fail: 3 / 88 (3.4%)
Skip: 0 / 88 (0.0%)

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
  "errors": "__unknown__",
  "output": "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(p)(?=t)/g, 'Type$1$2$3$4$5')"
- "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"

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
  "errors": "__unknown__",
  "output": "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(i)(?=pt)/g, 'Type$1$2$3$4')"
- "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"

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
  "errors": "__unknown__",
  "output": "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ "var str = 'JavaScript'.replace(/Java(S)(c)(r)(?=ipt)/g, 'Type$1$2$3')"
- "var str = 'JavaScript'.replace(/Java(?=Script)/g, 'Type')"

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-quantifier`

Pass: 24 / 25 (96.0%)
Fail: 1 / 25 (4.0%)
Skip: 0 / 25 (0.0%)

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
  "errors": "__unknown__",
  "output": "/a{3}.{2}\\s{2}\\S{2}\\p{ASCII}{2}/u"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/a{3}..\\s{2}\\S\\S\\p{ASCII}{2}/u'
- '/a{3}.{2}\\s{2}\\S{2}\\p{ASCII}{2}/u'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `simplify-set-operations`

Pass: 33 / 36 (91.7%)
Fail: 3 / 36 (8.3%)
Skip: 0 / 36 (0.0%)

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
  "errors": "__unknown__",
  "output": "/[b--[[a]\\d]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[[^[a]\\d]&&b]/v'
- '/[b--[[a]\\d]]/v'

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
  "errors": "__unknown__",
  "output": "/[[b]--[[a][c]]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[[^[a][c]]&&[b]]/v'
- '/[[b]--[[a][c]]]/v'

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
  "errors": "__unknown__",
  "output": "/[[a&&d]--[[b][c]]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[[^[b][c]]&&a&&d]/v'
- '/[[a&&d]--[[b][c]]]/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `sort-alternatives`

Pass: 33 / 34 (97.1%)
Fail: 1 / 34 (2.9%)
Skip: 0 / 34 (0.0%)

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
  "errors": "__unknown__",
  "output": "/(?:a|[\\q{blue|green|red}]|c)/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/(?:a|[\\q{red|green|blue}]|c)/v'
- '/(?:a|[\\q{blue|green|red}]|c)/v'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `sort-character-class-elements`

Pass: 47 / 52 (90.4%)
Fail: 5 / 52 (9.6%)
Skip: 0 / 52 (0.0%)

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
  "errors": "__unknown__",
  "output": "\n            const jsxWhitespaceChars = \"\\t\\n\\r \";\n            const matchJsxWhitespaceRegex = new RegExp(\"([\" + jsxWhitespaceChars + \"]+)\");\n            "
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

  '\n' +
+   '            const jsxWhitespaceChars = "\\n \\r\\t";\n' +
-   '            const jsxWhitespaceChars = "\\t\\n\\r ";\n' +
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
  "errors": "__unknown__",
  "output": "/[a\\q{a}[a--b][a]]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\q{a}[a--b][a]a]/v'
- '/[a\\q{a}[a--b][a]]/v'

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
  "errors": "__unknown__",
  "output": "/[\\q{a}\\q{b}\\q{c}]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\q{b}\\q{c}\\q{a}]/v'
- '/[\\q{a}\\q{b}\\q{c}]/v'

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
  "errors": "__unknown__",
  "output": "/[\\q{aa}\\q{ab}\\q{ac}]/v"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\q{ab}\\q{ac}\\q{aa}]/v'
- '/[\\q{aa}\\q{ab}\\q{ac}]/v'

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
  "errors": "__unknown__",
  "output": "/[*\\^~]/"
}
```

AssertionError [ERR_ASSERTION]: Output is incorrect
+ actual - expected

+ '/[\\^~*]/'
- '/[*\\^~]/'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


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
  "output": "/\\w/gimsuy"
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
  "output": "/\\w/gimsvy"
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

