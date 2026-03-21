# Conformance test results - mocha

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    24 | 100.0% |
| Fully passing     |     0 |   0.0% |
| Partially passing |    24 | 100.0% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |   825 | 100.0% |
| Passing     |   517 |  62.7% |
| Failing     |   308 |  37.3% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

No rules fully passing

## Rules with Failures

- `consistent-interface` - 6 / 9 (66.7%)
- `consistent-spacing-between-blocks` - 10 / 17 (58.8%)
- `handle-done-callback` - 23 / 42 (54.8%)
- `max-top-level-suites` - 20 / 37 (54.1%)
- `no-async-suite` - 10 / 16 (62.5%)
- `no-empty-title` - 42 / 51 (82.4%)
- `no-exclusive-tests` - 23 / 44 (52.3%)
- `no-exports` - 13 / 31 (41.9%)
- `no-global-tests` - 24 / 40 (60.0%)
- `no-hooks-for-single-case` - 21 / 35 (60.0%)
- `no-hooks` - 23 / 29 (79.3%)
- `no-identical-title` - 18 / 28 (64.3%)
- `no-mocha-arrows` - 11 / 26 (42.3%)
- `no-nested-tests` - 5 / 20 (25.0%)
- `no-pending-tests` - 20 / 45 (44.4%)
- `no-return-and-callback` - 29 / 44 (65.9%)
- `no-return-from-async` - 16 / 27 (59.3%)
- `no-setup-in-describe` - 53 / 65 (81.5%)
- `no-sibling-hooks` - 22 / 32 (68.8%)
- `no-synchronous-tests` - 36 / 45 (80.0%)
- `no-top-level-hooks` - 21 / 31 (67.7%)
- `prefer-arrow-callback` - 41 / 67 (61.2%)
- `valid-suite-title` - 12 / 17 (70.6%)
- `valid-test-title` - 18 / 27 (66.7%)

## Rules with Failures Detail

### `consistent-interface`

Pass: 6 / 9 (66.7%)
Fail: 3 / 9 (33.3%)
Skip: 0 / 9 (0.0%)

#### consistent-interface > invalid

```js
describe('foo', () => {
                test('bar', () => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "module"
  },
  "options": [
    {
      "interface": "BDD"
    }
  ],
  "settings": {
    "mocha": {
      "interface": "BDD"
    }
  },
  "errors": [
    {
      "line": 2,
      "column": 17,
      "message": "Unexpected use of TDD interface instead of BDD"
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


#### consistent-interface > invalid

```js
describe('foo', () => {
                test('bar', () => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "module"
  },
  "options": [
    {
      "interface": "TDD"
    }
  ],
  "settings": {
    "mocha": {
      "interface": "TDD"
    }
  },
  "errors": [
    {
      "line": 1,
      "column": 1,
      "message": "Unexpected use of BDD interface instead of TDD"
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


#### consistent-interface > invalid

```js
import {describe, it} from 'mocha'; describe('foo', () => {
                it('bar', () => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "module"
  },
  "options": [
    {
      "interface": "TDD"
    }
  ],
  "settings": {
    "mocha": {
      "interface": "exports"
    }
  },
  "errors": [
    {
      "line": 1,
      "column": 37,
      "message": "Unexpected use of BDD interface instead of TDD"
    },
    {
      "line": 2,
      "column": 17,
      "message": "Unexpected use of BDD interface instead of TDD"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `consistent-spacing-between-blocks`

Pass: 10 / 17 (58.8%)
Fail: 7 / 17 (41.2%)
Skip: 0 / 17 (0.0%)

#### consistent-spacing-between-mocha-calls > invalid

```js
describe('My Test', function () {
                it('does something', () => {});
                afterEach(() => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe('My Test', function () {\n                it('does something', () => {});\n\n                afterEach(() => {});\n            });",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe('My Test', () => {
                beforeEach(() => {});
                it('does something', () => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe('My Test', () => {\n                beforeEach(() => {});\n\n                it('does something', () => {});\n            });",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe('Variable declaration', () => {
                const a = 1;
                it('uses a variable', () => {});
            });
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe('Variable declaration', () => {\n                const a = 1;\n\n                it('uses a variable', () => {});\n            });",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe('Same line blocks', () => {it('block one', () => {});it('block two', () => {});});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe('Same line blocks', () => {it('block one', () => {});\n\nit('block two', () => {});});",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe('Same line blocks', () => {it('block one', () => {})
.timeout(42);it('block two', () => {});});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe('Same line blocks', () => {it('block one', () => {})\n.timeout(42);\n\nit('block two', () => {});});",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe("", () => {});describe("", () => {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe(\"\", () => {});\n\ndescribe(\"\", () => {});",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


#### consistent-spacing-between-mocha-calls > invalid

```js
describe();
describe();
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2020,
    "sourceType": "script"
  },
  "output": "describe();\n\ndescribe();",
  "errors": [
    {
      "message": "Expected line break before this statement.",
      "type": "CallExpression"
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


### `handle-done-callback`

Pass: 23 / 42 (54.8%)
Fail: 19 / 42 (45.2%)
Skip: 0 / 42 (0.0%)

#### handle-done-callback > invalid

```js
it("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it.skip("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 23,
      "line": 1
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


#### handle-done-callback > invalid

```js
xit("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 19,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (done) { callback(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (callback) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"callback\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (done) { asyncFunction(function (error) { expect(error).to.be.null; }); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it.only("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 23,
      "line": 1
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


#### handle-done-callback > invalid

```js
test("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 20,
      "line": 1
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


#### handle-done-callback > invalid

```js
test.only("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 25,
      "line": 1
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


#### handle-done-callback > invalid

```js
specify("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 23,
      "line": 1
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


#### handle-done-callback > invalid

```js
specify.only("", function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 28,
      "line": 1
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


#### handle-done-callback > invalid

```js
before(function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
after(function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 17,
      "line": 1
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


#### handle-done-callback > invalid

```js
beforeEach(function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 22,
      "line": 1
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


#### handle-done-callback > invalid

```js
afterEach(function (done) { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 21,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", (done) => { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 6
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 9,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (done) { return done; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (done) { done; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


#### handle-done-callback > invalid

```js
it("", function (done) { var foo = done; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Expected \"done\" callback to be handled.",
      "column": 18,
      "line": 1
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


### `max-top-level-suites`

Pass: 20 / 37 (54.1%)
Fail: 17 / 37 (45.9%)
Skip: 0 / 37 (0.0%)

#### max-top-level-suites > invalid

```js
describe("this is a test", function () { });describe("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
describe("", function () { });describe("", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2015
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
describe("this is a test", function () { });describe("this is a different test", function () { });describe("this is an another different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
context("this is a test", function () { });context("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
suite("this is a test", function () { });suite("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
describe("this is a test", function () { }); context("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
suite("this is a test", function () { }); context("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
describe("this is a test", function () { });someOtherFunction();describe("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
someOtherFunction();describe("this is a test", function () { });describe("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
describe.skip("this is a test", function () { });describe.only("this is a different test", function () { });describe("this is a whole different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "limit": 2
    }
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 2."
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


#### max-top-level-suites > invalid

```js
xdescribe("this is a test", function () { });describe.only("this is a different test", function () { });describe("this is a whole different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "limit": 1
    }
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
suite.skip("this is a test", function () { });suite.only("this is a different test", function () { });suite("this is a whole different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "limit": 2
    }
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 2."
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


#### max-top-level-suites > invalid

```js
context.skip("this is a test", function () { });context.only("this is a different test", function () { });context("this is a whole different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "limit": 2
    }
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 2."
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


#### max-top-level-suites > invalid

```js
describe("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "limit": 0
    }
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 0."
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


#### max-top-level-suites > invalid

```js
describe("this is a test", function () { });describe.only("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {}
  ],
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
foo("this is a test", function () { });foo("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "foo",
        "type": "suite",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


#### max-top-level-suites > invalid

```js
foo("this is a test", function () { });foo("this is a different test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "The number of top-level suites is more than 1."
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


### `no-async-suite`

Pass: 10 / 16 (62.5%)
Fail: 6 / 16 (37.5%)
Skip: 0 / 16 (0.0%)

#### no-async-suite > invalid

```js
describe("hello", async function () {})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": "describe(\"hello\", function () {})",
  "errors": [
    {
      "message": "Unexpected async function in describe()",
      "line": 1,
      "column": 19
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


#### no-async-suite > invalid

```js
foo("hello", async function () {})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": "foo(\"hello\", function () {})",
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected async function in foo()",
      "line": 1,
      "column": 14
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


#### no-async-suite > invalid

```js
describe("hello", async () => {})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": "describe(\"hello\", () => {})",
  "errors": [
    {
      "message": "Unexpected async function in describe()",
      "line": 1,
      "column": 19
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


#### no-async-suite > invalid

```js
describe("hello", async () => {await foo;})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": null,
  "errors": [
    {
      "message": "Unexpected async function in describe()",
      "line": 1,
      "column": 19
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


#### no-async-suite > invalid

```js
describe("hello", async () => {async function bar() {await foo;}})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": "describe(\"hello\", () => {async function bar() {await foo;}})",
  "errors": [
    {
      "message": "Unexpected async function in describe()",
      "line": 1,
      "column": 19
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


#### no-async-suite > invalid

```js
describe.foo("bar")("hello", async () => {})
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 8
  },
  "output": "describe.foo(\"bar\")(\"hello\", () => {})",
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "describe.foo()",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected async function in describe.foo()()",
      "line": 1,
      "column": 30
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


### `no-empty-title`

Pass: 42 / 51 (82.4%)
Fail: 9 / 51 (17.6%)
Skip: 0 / 51 (0.0%)

#### no-empty-title > invalid

```js
test()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
test(function() { })
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
test("", function() { })
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
test("      ", function() { })
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
someFunction(function() { })
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "message": "Custom Error"
    }
  ],
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "someFunction",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Custom Error",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
it(` `, function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "column": 1,
      "line": 1
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


#### no-empty-title > invalid

```js
const foo = ""; it(foo);
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "line": 1,
      "column": 17
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


#### no-empty-title > invalid

```js
const foo = { bar: "" }; it(foo.bar);
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "line": 1,
      "column": 26
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


#### no-empty-title > invalid

```js
it(foo?.bar);
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2020
  },
  "errors": [
    {
      "message": "Unexpected empty test description.",
      "line": 1,
      "column": 1
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


### `no-exclusive-tests`

Pass: 23 / 44 (52.3%)
Fail: 21 / 44 (47.7%)
Skip: 0 / 44 (0.0%)

#### no-exclusive-tests > invalid

```js
describe.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 10,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
describe["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 10,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
it.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 4,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
import { it } from "mocha"; it.only()
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2018
  },
  "settings": {
    "mocha": {
      "interface": "exports"
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 32,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
import { it as foo } from "mocha"; foo.only()
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2018
  },
  "settings": {
    "mocha": {
      "interface": "exports"
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 40,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
import { it as foo } from "mocha"; const bar = foo; bar.only()
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2018
  },
  "settings": {
    "mocha": {
      "interface": "exports"
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 57,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
it["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 4,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
suite.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 7,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
suite["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 7,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
test.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 6,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
test["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 6,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
context.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 9,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
context["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 9,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
specify.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 9,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
specify["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 9,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
custom["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "custom",
        "type": "testCase",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 8,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
custom["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "custom",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 8,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
foo.bar["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo.bar",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 9,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
foo["bar"].only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo.bar",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 12,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
foo["bar"]["only"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo.bar",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 12,
      "line": 1
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


#### no-exclusive-tests > invalid

```js
a.b.c.only()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "a.b.c",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected exclusive mocha test.",
      "column": 7,
      "line": 1
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


### `no-exports`

Pass: 13 / 31 (41.9%)
Fail: 18 / 31 (58.1%)
Skip: 0 / 31 (0.0%)

#### no-exports > invalid

```js
describe(function() {}); export default "foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 26,
      "line": 1
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


#### no-exports > invalid

```js
describe(function() {}); export const bar = "foo";
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 26,
      "line": 1
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


#### no-exports > invalid

```js
describe(function() {}); let foo; export { foo }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 35,
      "line": 1
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


#### no-exports > invalid

```js
describe(function() {}); let foo; export { foo as bar }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 35,
      "line": 1
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


#### no-exports > invalid

```js
describe(function() {}); export * from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 26,
      "line": 1
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


#### no-exports > invalid

```js
describe(function() {}); export { bar } from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 26,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); export default "foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 24,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); export const bar = "foo";
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 24,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); let foo; export { foo }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 33,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); let foo; export { foo as bar }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 33,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); export * from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 24,
      "line": 1
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


#### no-exports > invalid

```js
it("", function() {}); export { bar } from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 24,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); export default "foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 28,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); export const bar = "foo";
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 28,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); let foo; export { foo }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 37,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); let foo; export { foo as bar }
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 37,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); export * from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 28,
      "line": 1
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


#### no-exports > invalid

```js
beforeEach(function() {}); export { bar } from "./foo"
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected export from a test file",
      "column": 28,
      "line": 1
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


### `no-global-tests`

Pass: 24 / 40 (60.0%)
Fail: 16 / 40 (40.0%)
Skip: 0 / 40 (0.0%)

#### no-global-tests > invalid

```js
it();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
it.only();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
it["only"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
it.skip();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
it["skip"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
test();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
test.only();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
test["only"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
test.skip();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
test["skip"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
specify();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
specify.only();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
specify["only"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
specify.skip();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
specify["skip"]();
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 1,
      "line": 1
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


#### no-global-tests > invalid

```js
import foo from "bar"; it("");
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2015
  },
  "errors": [
    {
      "message": "Unexpected global mocha test.",
      "column": 24,
      "line": 1
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


### `no-hooks-for-single-case`

Pass: 21 / 35 (60.0%)
Fail: 14 / 35 (40.0%)
Skip: 0 / 35 (0.0%)

#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
    it(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    it(function() {});
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 3
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    after(function() {});
    it(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `after()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    beforeEach(function() {});
    it(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `beforeEach()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    afterEach(function() {});
    it(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `afterEach()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
before(function() {});
it(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 1,
      "line": 1
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
    describe(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
    xdescribe(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
    it.only(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    before(function() {});
    it(function() {});
    describe(function() {
        before(function() {});
        it(function() {});
    });
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 9,
      "line": 5
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


#### no-hooks-for-single-case > invalid

```js
describe(function() {
    after(function() {});
    it(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "allow": [
        "before"
      ]
    }
  ],
  "errors": [
    {
      "message": "Unexpected use of Mocha `after()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
foo(function() {
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


#### no-hooks-for-single-case > invalid

```js
foo(function() {
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "foo",
        "type": "suite",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook for a single test case",
      "column": 5,
      "line": 2
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


### `no-hooks`

Pass: 23 / 29 (79.3%)
Fail: 6 / 29 (20.7%)
Skip: 0 / 29 (0.0%)

#### no-hooks > invalid

```js
describe(function() { before(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook",
      "column": 23,
      "line": 1
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


#### no-hooks > invalid

```js
describe(function() { after(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `after()` hook",
      "column": 23,
      "line": 1
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


#### no-hooks > invalid

```js
describe(function() { beforeEach(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `beforeEach()` hook",
      "column": 23,
      "line": 1
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


#### no-hooks > invalid

```js
describe(function() { afterEach(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `afterEach()` hook",
      "column": 23,
      "line": 1
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


#### no-hooks > invalid

```js
describe(function() { describe(function() { before(function() {}); }); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook",
      "column": 45,
      "line": 1
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


#### no-hooks > invalid

```js
describe(function() { after(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "allow": [
        "before"
      ]
    }
  ],
  "errors": [
    {
      "message": "Unexpected use of Mocha `after()` hook",
      "column": 23,
      "line": 1
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


### `no-identical-title`

Pass: 18 / 28 (64.3%)
Fail: 10 / 28 (35.7%)
Skip: 0 / 28 (0.0%)

#### no-identical-title > invalid

```js
describe("describe1", function() {
   it("it1", function() {});
   it("it1", function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test title is used multiple times in the same test suite.",
      "column": 4,
      "line": 3
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


#### no-identical-title > invalid

```js
it("it1", function() {});
it("it1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test title is used multiple times in the same test suite.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
it.only("it1", function() {});
it("it1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test title is used multiple times in the same test suite.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
it.only("it1", function() {});
it.only("it1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test title is used multiple times in the same test suite.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
it("it1", function() {});
specify("it1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test title is used multiple times in the same test suite.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
describe("describe1", function() {});
describe("describe1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test suite title is used multiple times.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
describe("describe1", function() {});
xdescribe("describe1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test suite title is used multiple times.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
describe("describe1", function() {
   describe("describe2", function() {});
});
describe("describe1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Test suite title is used multiple times.",
      "column": 1,
      "line": 4
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


#### no-identical-title > invalid

```js
foo("describe1", function() {});
foo("describe1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "foo",
        "type": "suite",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Test suite title is used multiple times.",
      "column": 1,
      "line": 2
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


#### no-identical-title > invalid

```js
foo("describe1", function() {});
foo("describe1", function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Test suite title is used multiple times.",
      "column": 1,
      "line": 2
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


### `no-mocha-arrows`

Pass: 11 / 26 (42.3%)
Fail: 15 / 26 (57.7%)
Skip: 0 / 26 (0.0%)

#### no-mocha-arrows > invalid

```js
it(() => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(function() { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(() => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(function() { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(done => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(function(done) { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it("should be false", () => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(\"should be false\", function() { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it.only(() => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it.only()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it.only(function() { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it((done) => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(function(done) { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(done => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(function(done) { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it("should be false", () => {
 assert(something, false);
})
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(\"should be false\", function() {\n assert(something, false);\n})"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(async () => { assert(something, false) })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(async function() { assert(something, false) })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(async () => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(async function() { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(async done => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(async function(done) { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(async (done) => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(async function(done) { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(async() => assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(async function() { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
it(/*one*/async/*two*/(done)/*three*/=>/*four*/assert(something, false))
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 1,
      "line": 1
    }
  ],
  "output": "it(/*one*/async function/*two*/(done)/*three*//*four*/ { return assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-mocha-arrows > invalid

```js
const foo = () => {}; foo("", () => {}); it(() => { assert(something, false); })
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Do not pass arrow functions to it()",
      "column": 42,
      "line": 1
    }
  ],
  "output": "const foo = () => {}; foo(\"\", () => {}); it(function() { assert(something, false); })"
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-nested-tests`

Pass: 5 / 20 (25.0%)
Fail: 15 / 20 (75.0%)
Skip: 0 / 20 (0.0%)

#### no-nested-tests > invalid

```js
it("", function () { it() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it.only("", function () { it() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 27
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


#### no-nested-tests > invalid

```js
it.skip("", function () { it() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 27
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


#### no-nested-tests > invalid

```js
test("", function () { it() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 24
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


#### no-nested-tests > invalid

```js
specify("", function () { it() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 27
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


#### no-nested-tests > invalid

```js
it("", function () { describe() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it("", function () { context() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it("", function () { suite() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it("", function () { describe.skip() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it("", function () { describe("", function () { it(); it(); }); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
    },
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 49
    },
    {
      "message": "Unexpected test nested within another test.",
      "line": 1,
      "column": 55
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 3 errors but had 0: []

0 !== 3

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-nested-tests > invalid

```js
it("", function () { foo() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "foo",
        "type": "suite",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
it("", function () { foo() });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test.",
      "line": 1,
      "column": 22
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


#### no-nested-tests > invalid

```js
beforeEach(function () { it("foo", function () {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test nested within a test hook.",
      "line": 1,
      "column": 26
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


#### no-nested-tests > invalid

```js
beforeEach(function () { describe("foo", function () {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected suite nested within a test hook.",
      "line": 1,
      "column": 26
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


#### no-nested-tests > invalid

```js
beforeEach(function () { beforeEach(function () {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected test hook nested within a test hook.",
      "line": 1,
      "column": 26
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


### `no-pending-tests`

Pass: 20 / 45 (44.4%)
Fail: 25 / 45 (55.6%)
Skip: 0 / 45 (0.0%)

#### no-pending-tests > invalid

```js
it("is pending")
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
test("is pending")
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
specify("is pending")
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
describe.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 10,
      "line": 1
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


#### no-pending-tests > invalid

```js
describe["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 10,
      "line": 1
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


#### no-pending-tests > invalid

```js
xdescribe()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
it.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 4,
      "line": 1
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


#### no-pending-tests > invalid

```js
it["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 4,
      "line": 1
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


#### no-pending-tests > invalid

```js
xit()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
suite.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 7,
      "line": 1
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


#### no-pending-tests > invalid

```js
suite["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 7,
      "line": 1
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


#### no-pending-tests > invalid

```js
test.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 6,
      "line": 1
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


#### no-pending-tests > invalid

```js
test["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 6,
      "line": 1
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


#### no-pending-tests > invalid

```js
context.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 9,
      "line": 1
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


#### no-pending-tests > invalid

```js
context["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 9,
      "line": 1
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


#### no-pending-tests > invalid

```js
xcontext()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
specify.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 9,
      "line": 1
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


#### no-pending-tests > invalid

```js
xspecify()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
custom.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "custom",
        "type": "testCase",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 8,
      "line": 1
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


#### no-pending-tests > invalid

```js
custom["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "custom",
        "type": "testCase",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 8,
      "line": 1
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


#### no-pending-tests > invalid

```js
xcustom()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "custom",
        "type": "testCase",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
custom.skip()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "custom",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 8,
      "line": 1
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


#### no-pending-tests > invalid

```js
custom["skip"]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "custom",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 8,
      "line": 1
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


#### no-pending-tests > invalid

```js
xcustom()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "custom",
          "type": "testCase",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 1,
      "line": 1
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


#### no-pending-tests > invalid

```js
var dynamicOnly = "skip"; suite[dynamicOnly]()
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected pending mocha test.",
      "column": 33,
      "line": 1
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


### `no-return-and-callback`

Pass: 29 / 44 (65.9%)
Fail: 15 / 44 (34.1%)
Skip: 0 / 44 (0.0%)

#### no-return-and-callback > invalid

```js
it("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 30,
      "line": 1
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


#### no-return-and-callback > invalid

```js
it("title", function(done) { return foo.then(function() { done(); }).catch(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 30,
      "line": 1
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


#### no-return-and-callback > invalid

```js
it("title", function(done) { var foo = bar(); return foo.then(function() { done(); }); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 47,
      "line": 1
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


#### no-return-and-callback > invalid

```js
it("title", (done) => { return foo.then(function() { done(); }).catch(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 6
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 25,
      "line": 1
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


#### no-return-and-callback > invalid

```js
it("title", (done) => foo.then(function() { done(); }));
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 6
  },
  "errors": [
    {
      "message": "Confusing implicit return in a test with callback",
      "column": 23,
      "line": 1
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


#### no-return-and-callback > invalid

```js
it.only("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 35,
      "line": 1
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


#### no-return-and-callback > invalid

```js
before("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 34,
      "line": 1
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


#### no-return-and-callback > invalid

```js
beforeEach("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 38,
      "line": 1
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


#### no-return-and-callback > invalid

```js
after("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 33,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(done) { return foo.then(done); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 37,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(done) { return foo; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 37,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(done) { return done; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 37,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(done) { return done.foo(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 37,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(done) { return foo.done(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 37,
      "line": 1
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


#### no-return-and-callback > invalid

```js
afterEach("title", function(end) { return done(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with callback",
      "column": 36,
      "line": 1
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


### `no-return-from-async`

Pass: 16 / 27 (59.3%)
Fail: 11 / 27 (40.7%)
Skip: 0 / 27 (0.0%)

#### no-return-from-async > invalid

```js
it("title", async function() { return foo; });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 32,
      "line": 1
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


#### no-return-from-async > invalid

```js
it("title", async function() { return foo.then(function() {}).catch(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 32,
      "line": 1
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


#### no-return-from-async > invalid

```js
it("title", async function() { var foo = bar(); return foo.then(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 49,
      "line": 1
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


#### no-return-from-async > invalid

```js
it("title", async () => { return foo.then(function() {}).catch(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 27,
      "line": 1
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


#### no-return-from-async > invalid

```js
it("title", async () => foo.then(function() {}));
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Confusing implicit return in a test with an async function",
      "column": 25,
      "line": 1
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


#### no-return-from-async > invalid

```js
it.only("title", async function() { return foo.then(function () {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 37,
      "line": 1
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


#### no-return-from-async > invalid

```js
before("title", async function() { return foo.then(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 36,
      "line": 1
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


#### no-return-from-async > invalid

```js
beforeEach("title", async function() { return foo.then(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 40,
      "line": 1
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


#### no-return-from-async > invalid

```js
after("title", async function() { return foo.then(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 35,
      "line": 1
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


#### no-return-from-async > invalid

```js
afterEach("title", async function() { return foo.then(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 39,
      "line": 1
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


#### no-return-from-async > invalid

```js
afterEach("title", async function() { return foo; });
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 8
  },
  "errors": [
    {
      "message": "Unexpected use of `return` in a test with an async function",
      "column": 39,
      "line": 1
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


### `no-setup-in-describe`

Pass: 53 / 65 (81.5%)
Fail: 12 / 65 (18.5%)
Skip: 0 / 65 (0.0%)

#### no-setup-in-describe > invalid

```js
suite("", function () { this.timeout(42); a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 43
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


#### no-setup-in-describe > invalid

```js
suite("", function () { a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 25
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


#### no-setup-in-describe > invalid

```js
describe("", function () { a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
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


#### no-setup-in-describe > invalid

```js
describe("", () => { a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2015
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 22
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


#### no-setup-in-describe > invalid

```js
foo("", function () { a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 23
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


#### no-setup-in-describe > invalid

```js
foo("", function () { a[b]; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 23
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


#### no-setup-in-describe > invalid

```js
foo("", function () { a["b"]; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 23
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


#### no-setup-in-describe > invalid

```js
describe("", function () { a.b; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 28
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


#### no-setup-in-describe > invalid

```js
describe("", function () { this.a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
    },
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 28
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-setup-in-describe > invalid

```js
foo("", function () { a.b; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 23
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


#### no-setup-in-describe > invalid

```js
describe("", function () { it("", function () {}).a(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
    },
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 28
    },
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 3 errors but had 0: []

0 !== 3

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-setup-in-describe > invalid

```js
describe("", function () { something("", function () {}).timeout(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
    },
    {
      "message": "Unexpected member expression in describe block. Member expressions may call functions via getters.",
      "line": 1,
      "column": 28
    },
    {
      "message": "Unexpected function call in describe block.",
      "line": 1,
      "column": 28
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 3 errors but had 0: []

0 !== 3

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-sibling-hooks`

Pass: 22 / 32 (68.8%)
Fail: 10 / 32 (31.3%)
Skip: 0 / 32 (0.0%)

#### no-sibling-hooks > invalid

```js
describe(function() { before(function() {}); before(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 46,
      "line": 1
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


#### no-sibling-hooks > invalid

```js
describe(function() { after(function() {}); after(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `after()` hook",
      "column": 45,
      "line": 1
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


#### no-sibling-hooks > invalid

```js
describe(function() { beforeEach(function() {}); beforeEach(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `beforeEach()` hook",
      "column": 50,
      "line": 1
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


#### no-sibling-hooks > invalid

```js
describe(function() { afterEach(function() {}); afterEach(function() {}); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `afterEach()` hook",
      "column": 49,
      "line": 1
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


#### no-sibling-hooks > invalid

```js
describe(function() {
    before(function() {});
    describe(function() {
        before(function() {});
        before(function() {});
    });
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 9,
      "line": 5
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


#### no-sibling-hooks > invalid

```js
describe(function() {
    before(function() {});
    describe(function() {
        before(function() {});
    });
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 5,
      "line": 6
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


#### no-sibling-hooks > invalid

```js
foo(function() {
    before(function() {});
    foo(function() {
        before(function() {});
    });
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha/additionalCustomNames": [
      {
        "name": "foo",
        "type": "suite",
        "interface": "BDD"
      }
    ]
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 5,
      "line": 6
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


#### no-sibling-hooks > invalid

```js
foo(function() {
    before(function() {});
    foo(function() {
        before(function() {});
    });
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 5,
      "line": 6
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


#### no-sibling-hooks > invalid

```js
describe.foo(function() {
    before(function() {});
    describe.foo(function() {
        before(function() {});
    });
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "describe.foo",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 5,
      "line": 6
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


#### no-sibling-hooks > invalid

```js
describe.foo()(function() {
    before(function() {});
    describe.foo()(function() {
        before(function() {});
    });
    before(function() {});
});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "settings": {
    "mocha": {
      "additionalCustomNames": [
        {
          "name": "describe.foo()",
          "type": "suite",
          "interface": "BDD"
        }
      ]
    }
  },
  "errors": [
    {
      "message": "Unexpected use of duplicate Mocha `before()` hook",
      "column": 5,
      "line": 6
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


### `no-synchronous-tests`

Pass: 36 / 45 (80.0%)
Fail: 9 / 45 (20.0%)
Skip: 0 / 45 (0.0%)

#### no-synchronous-tests > invalid

```js
it("", function () {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 8,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
it("", function () { callback(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 8,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
it(function () { return; });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 4,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
it("", function () { return "a string" });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 8,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
it("", () => "not-a-promise" );
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 6
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 8,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
specify("", function () {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 13,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
specify.only("", function () {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 18,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
before("", function () {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 12,
      "line": 1
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


#### no-synchronous-tests > invalid

```js
it("", function () { return promise(); });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "allowed": [
        "callback",
        "async"
      ]
    }
  ],
  "errors": [
    {
      "message": "Unexpected synchronous test.",
      "column": 8,
      "line": 1
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


### `no-top-level-hooks`

Pass: 21 / 31 (67.7%)
Fail: 10 / 31 (32.3%)
Skip: 0 / 31 (0.0%)

#### no-top-level-hooks > invalid

```js
before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
after(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `after()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
beforeEach(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `beforeEach()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
afterEach(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `afterEach()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
describe(function() {}); before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 26,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
before(function() {}); describe(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
foo(function() {}); before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 21,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
describe()(function() {}); before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 28,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
describe.foo()(function() {}); before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 32,
      "line": 1
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


#### no-top-level-hooks > invalid

```js
before(function() {});
```

```json
{
  "languageOptions": {
    "sourceType": "module",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Unexpected use of Mocha `before()` hook outside of a test suite",
      "column": 1,
      "line": 1
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


### `prefer-arrow-callback`

Pass: 41 / 67 (61.2%)
Fail: 26 / 67 (38.8%)
Skip: 0 / 67 (0.0%)

#### prefer-arrow-callback > valid

```js
before(function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 7,
    endLine: 1,
    endColumn: 24,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
after(function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 6,
    endLine: 1,
    endColumn: 23,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
beforeEach(function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 11,
    endLine: 1,
    endColumn: 28,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
afterEach(function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 10,
    endLine: 1,
    endColumn: 27,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
describe("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 17,
    endLine: 1,
    endColumn: 34,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
xdescribe("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 35,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
describe.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 39,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
describe.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 39,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
context("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
xcontext("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 17,
    endLine: 1,
    endColumn: 34,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
context.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 38,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
context.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 38,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
suite("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 14,
    endLine: 1,
    endColumn: 31,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
suite.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 36,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
suite.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 36,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
it("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 11,
    endLine: 1,
    endColumn: 28,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
it.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
it.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
xit("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 12,
    endLine: 1,
    endColumn: 29,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
test("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 13,
    endLine: 1,
    endColumn: 30,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
test.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 35,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
test.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 35,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
specify("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
specify.only("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 38,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
specify.skip("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 38,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prefer-arrow-callback > valid

```js
xspecify("name", function bar() {});
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2017,
    "sourceType": "script"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/prefer-arrow-callback',
    message: 'Unexpected function expression.',
    messageId: 'preferArrowCallback',
    severity: 1,
    nodeType: 'FunctionExpression',
    line: 1,
    column: 17,
    endLine: 1,
    endColumn: 34,
    fixes: [ [Object], [Object] ],
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `valid-suite-title`

Pass: 12 / 17 (70.6%)
Fail: 5 / 17 (29.4%)
Skip: 0 / 17 (0.0%)

#### valid-suite-title > invalid

```js
describe("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "^[A-Z]"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"describe()\" description found."
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


#### valid-suite-title > invalid

```js
context("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "^[A-Z]"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"context()\" description found."
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


#### valid-suite-title > invalid

```js
suite("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "^[A-Z]"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"suite()\" description found."
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


#### valid-suite-title > invalid

```js
describe(`this is a test`, function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "options": [
    {
      "pattern": "^[A-Z]"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"describe()\" description found.",
      "line": 1,
      "column": 1
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


#### valid-suite-title > invalid

```js
const foo = "this"; describe(`${foo} is a test`, function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "options": [
    {
      "pattern": "^[A-Z]"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"describe()\" description found.",
      "line": 1,
      "column": 21
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


### `valid-test-title`

Pass: 18 / 27 (66.7%)
Fail: 9 / 27 (33.3%)
Skip: 0 / 27 (0.0%)

#### valid-test-title > invalid

```js
it("does something", function() { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Invalid \"it()\" description found."
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


#### valid-test-title > invalid

```js
specify("does something", function() { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Invalid \"specify()\" description found."
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


#### valid-test-title > invalid

```js
test("does something", function() { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "errors": [
    {
      "message": "Invalid \"test()\" description found."
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


#### valid-test-title > invalid

```js
it("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "required"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"it()\" description found."
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


#### valid-test-title > invalid

```js
specify("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "required"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"specify()\" description found."
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


#### valid-test-title > invalid

```js
test("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {
      "pattern": "required"
    }
  ],
  "errors": [
    {
      "message": "Invalid \"test()\" description found."
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


#### valid-test-title > invalid

```js
it("this is a test", function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script"
  },
  "options": [
    {}
  ],
  "errors": [
    {
      "message": "Invalid \"it()\" description found."
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


#### valid-test-title > invalid

```js
it(`this is a test`, function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Invalid \"it()\" description found.",
      "line": 1,
      "column": 1
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


#### valid-test-title > invalid

```js
const foo = "this"; it(`${foo} is a test`, function () { });
```

```json
{
  "languageOptions": {
    "sourceType": "script",
    "ecmaVersion": 2019
  },
  "errors": [
    {
      "message": "Invalid \"it()\" description found.",
      "line": 1,
      "column": 21
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

