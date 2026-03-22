# Conformance test results - cypress

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    12 | 100.0% |
| Fully passing     |    11 |  91.7% |
| Partially passing |     1 |   8.3% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |   150 | 100.0% |
| Passing     |   147 |  98.0% |
| Failing     |     3 |   2.0% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

- `assertion-before-screenshot` (18 tests)
- `no-assigning-return-values` (19 tests)
- `no-async-before` (8 tests)
- `no-async-tests` (8 tests)
- `no-chained-get` (4 tests)
- `no-debug` (6 tests)
- `no-force` (22 tests)
- `no-pause` (6 tests)
- `no-xpath` (3 tests)
- `require-data-selectors` (22 tests)
- `unsafe-to-chain-command` (8 tests)

## Rules with Failures

- `no-unnecessary-waiting` - 23 / 26 (88.5%)

## Rules with Failures Detail

### `no-unnecessary-waiting`

Pass: 23 / 26 (88.5%)
Fail: 3 / 26 (11.5%)
Skip: 0 / 26 (0.0%)

#### no-unnecessary-waiting > invalid

```js
function customWait (ms = 1) { cy.wait(ms) }
```

```json
{
  "errors": [
    {
      "messageId": "unexpected"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-unnecessary-waiting > invalid

```js
const customWait = (ms = 1) => { cy.wait(ms) }
```

```json
{
  "errors": [
    {
      "messageId": "unexpected"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-unnecessary-waiting > invalid

```js
const customWait = (ms = 1) => { cy.get(".some-element").wait(ms) }
```

```json
{
  "errors": [
    {
      "messageId": "unexpected"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js

