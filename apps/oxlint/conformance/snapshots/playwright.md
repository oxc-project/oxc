# Conformance test results - playwright

Tested against: [playwright@7e16bd5](https://github.com/mskelton/eslint-plugin-playwright/tree/7e16bd565cfccd365a6a8f1f7f6fe29a1c868036) (v2.9.0)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    58 | 100.0% |
| Fully passing     |    57 |  98.3% |
| Partially passing |     1 |   1.7% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  3048 | 100.0% |
| Passing     |  3044 |  99.9% |
| Failing     |     4 |   0.1% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

- `consistent-spacing-between-blocks` (45 tests)
- `expect-expect` (40 tests)
- `max-expects` (30 tests)
- `max-nested-describe` (24 tests)
- `missing-playwright-await` (90 tests)
- `no-commented-out-tests` (38 tests)
- `no-conditional-expect` (49 tests)
- `no-conditional-in-test` (57 tests)
- `no-duplicate-hooks` (20 tests)
- `no-duplicate-slow` (24 tests)
- `no-element-handle` (33 tests)
- `no-eval` (25 tests)
- `no-focused-test` (29 tests)
- `no-force-option` (31 tests)
- `no-get-by-title` (5 tests)
- `no-hooks` (11 tests)
- `no-nested-step` (18 tests)
- `no-networkidle` (22 tests)
- `no-nth-methods` (22 tests)
- `no-page-pause` (12 tests)
- `no-raw-locators` (46 tests)
- `no-restricted-locators` (27 tests)
- `no-restricted-matchers` (33 tests)
- `no-restricted-roles` (34 tests)
- `no-skipped-test` (50 tests)
- `no-slowed-test` (33 tests)
- `no-standalone-expect` (40 tests)
- `no-unsafe-references` (46 tests)
- `no-unused-locators` (24 tests)
- `no-useless-await` (56 tests)
- `no-useless-not` (45 tests)
- `no-wait-for-navigation` (32 tests)
- `no-wait-for-selector` (25 tests)
- `no-wait-for-timeout` (25 tests)
- `prefer-comparison-matcher` (626 tests)
- `prefer-equality-matcher` (34 tests)
- `prefer-hooks-in-order` (44 tests)
- `prefer-hooks-on-top` (14 tests)
- `prefer-locator` (32 tests)
- `prefer-lowercase-title` (82 tests)
- `prefer-native-locators` (57 tests)
- `prefer-strict-equal` (15 tests)
- `prefer-to-be` (131 tests)
- `prefer-to-contain` (55 tests)
- `prefer-to-have-count` (32 tests)
- `prefer-to-have-length` (22 tests)
- `prefer-web-first-assertions` (116 tests)
- `require-hook` (40 tests)
- `require-soft-assertions` (18 tests)
- `require-tags` (29 tests)
- `require-to-pass-timeout` (16 tests)
- `require-to-throw-message` (26 tests)
- `require-top-level-describe` (46 tests)
- `valid-describe-callback` (40 tests)
- `valid-expect-in-promise` (141 tests)
- `valid-expect` (57 tests)
- `valid-title` (174 tests)

## Rules with Failures

- `valid-test-tags` - 56 / 60 (93.3%)

## Rules with Failures Detail

### `valid-test-tags`

Pass: 56 / 60 (93.3%)
Fail: 4 / 60 (6.7%)
Skip: 0 / 60 (0.0%)

#### valid-test-tags > valid

```js
test('my test', { tag: '@my-tag-123' }, async ({ page }) => {})
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "parser": {}
  },
  "options": [
    {
      "allowedTags": [
        "@regression",
        {}
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/valid-test-tags',
    message: 'Unknown tag "@my-tag-123"',
    messageId: 'unknownTag',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 0,
    endLine: 1,
    endColumn: 63,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### valid-test-tags > valid

```js
test('@my-tag-123 my test', async ({ page }) => {})
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "parser": {}
  },
  "options": [
    {
      "allowedTags": [
        "@regression",
        {}
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/valid-test-tags',
    message: 'Unknown tag "@my-tag-123"',
    messageId: 'unknownTag',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 0,
    endLine: 1,
    endColumn: 51,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### valid-test-tags > invalid

```js
test('my test', { tag: '@temp-123' }, async ({ page }) => {})
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "data": {
        "tag": "@temp-123"
      },
      "messageId": "disallowedTag"
    }
  ],
  "options": [
    {
      "disallowedTags": [
        "@skip",
        {}
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### valid-test-tags > invalid

```js
test('@temp-123 my test', async ({ page }) => {})
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "data": {
        "tag": "@temp-123"
      },
      "messageId": "disallowedTag"
    }
  ],
  "options": [
    {
      "disallowedTags": [
        "@skip",
        {}
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js

