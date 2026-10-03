# Conformance test results - sdl

Tested against: [sdl@98439dc](https://github.com/microsoft/eslint-plugin-sdl/tree/98439dcebd0d6a1c267d927e1c7d3b7df5675499) (1.1.0)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    17 | 100.0% |
| Fully passing     |    11 |  64.7% |
| Partially passing |     6 |  35.3% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |   139 | 100.0% |
| Passing     |   129 |  92.8% |
| Failing     |    10 |   7.2% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

- `no-angular-bypass-sanitizer` (8 tests)
- `no-angular-sanitization-trusted-urls` (6 tests)
- `no-angularjs-bypass-sce` (18 tests)
- `no-angularjs-enable-svg` (8 tests)
- `no-angularjs-sanitization-whitelist` (6 tests)
- `no-electron-node-integration` (3 tests)
- `no-html-method` (8 tests)
- `no-insecure-url` (19 tests)
- `no-msapp-exec-unsafe` (3 tests)
- `no-unsafe-alloc` (4 tests)
- `no-winjs-html-unsafe` (2 tests)

## Rules with Failures

- `no-cookies` - 6 / 8 (75.0%)
- `no-document-domain` - 3 / 5 (60.0%)
- `no-document-write` - 3 / 4 (75.0%)
- `no-inner-html` - 9 / 10 (90.0%)
- `no-insecure-random` - 15 / 18 (83.3%)
- `no-postmessage-star-origin` - 8 / 9 (88.9%)

## Rules with Failures Detail

### `no-cookies`

Pass: 6 / 8 (75.0%)
Fail: 2 / 8 (25.0%)
Skip: 0 / 8 (0.0%)

#### no-cookies > valid

```js

interface DocumentLikeAPI {
  cookie: string;
}
function documentLikeAPIFunction(): DocumentLikeAPI {
  return null;
}
function X() {
  // These usages are OK because they are not on the DOM document
  var document: DocumentLikeAPI = documentLikeAPIFunction();
  document.cookie = '...';
  document.cookie = '...';
}

documentLikeAPIFunction().cookie = '...';

```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/no-cookies',
    message: 'Do not use HTTP cookies in modern applications',
    messageId: 'doNotUseCookies',
    severity: 1,
    nodeType: 'Identifier',
    line: 11,
    column: 2,
    endLine: 11,
    endColumn: 17,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-cookies',
    message: 'Do not use HTTP cookies in modern applications',
    messageId: 'doNotUseCookies',
    severity: 1,
    nodeType: 'Identifier',
    line: 12,
    column: 2,
    endLine: 12,
    endColumn: 17,
    fixes: null,
    suggestions: null
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-cookies > invalid

```js

function documentFunction(): Document {
  return window.document;
}
documentFunction().cookie = '...';
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "doNotUseCookies"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-document-domain`

Pass: 3 / 5 (60.0%)
Fail: 2 / 5 (40.0%)
Skip: 0 / 5 (0.0%)

#### no-document-domain > valid

```js

interface DocumentLikeAPI {
  domain: string;
}
function documentLikeAPIFunction(): DocumentLikeAPI {
  return null;
}
function main() {
  var document: DocumentLikeAPI = documentLikeAPIFunction();
  document.domain = 'somevalue';
}
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-document-domain',
    message: 'Do not write to document.domain property',
    messageId: 'default',
    severity: 1,
    nodeType: 'Identifier',
    line: 10,
    column: 2,
    endLine: 10,
    endColumn: 31,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-document-domain > invalid

```js
var doc = window.document; doc.domain = 'somevalue';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "default"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-document-write`

Pass: 3 / 4 (75.0%)
Fail: 1 / 4 (25.0%)
Skip: 0 / 4 (0.0%)

#### no-document-write > invalid

```js

        var doc = document; 
        doc.write('...');
        doc.writeln('...');
        function documentFunction() : Document {
          return window.document;
        }
        documentFunction().write('...');
        documentFunction().writeln('...');        
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "default",
      "line": 3
    },
    {
      "messageId": "default",
      "line": 4
    },
    {
      "messageId": "default",
      "line": 8
    },
    {
      "messageId": "default",
      "line": 9
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 4 errors but had 0: []

0 !== 4

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-inner-html`

Pass: 9 / 10 (90.0%)
Fail: 1 / 10 (10.0%)
Skip: 0 / 10 (0.0%)

#### no-inner-html > valid

```js

        class Test {
          innerHTML: string;
          outerHTML: string;
          constructor(test: string) {
              this.innerHTML = test;
              this.outerHTML = test;
          }
        };
        let test = new Test("test");
        test.innerHTML = test;
        test.outerHTML = test;
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 4: [
  {
    ruleId: 'rule-to-test/no-inner-html',
    message: 'Do not write to DOM directly using innerHTML/outerHTML property',
    messageId: 'noInnerHtml',
    severity: 1,
    nodeType: 'ThisExpression',
    line: 6,
    column: 14,
    endLine: 6,
    endColumn: 35,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-inner-html',
    message: 'Do not write to DOM directly using innerHTML/outerHTML property',
    messageId: 'noInnerHtml',
    severity: 1,
    nodeType: 'ThisExpression',
    line: 7,
    column: 14,
    endLine: 7,
    endColumn: 35,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-inner-html',
    message: 'Do not write to DOM directly using innerHTML/outerHTML property',
    messageId: 'noInnerHtml',
    severity: 1,
    nodeType: 'Identifier',
    line: 11,
    column: 8,
    endLine: 11,
    endColumn: 29,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-inner-html',
    message: 'Do not write to DOM directly using innerHTML/outerHTML property',
    messageId: 'noInnerHtml',
    severity: 1,
    nodeType: 'Identifier',
    line: 12,
    column: 8,
    endLine: 12,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

4 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-insecure-random`

Pass: 15 / 18 (83.3%)
Fail: 3 / 18 (16.7%)
Skip: 0 / 18 (0.0%)

#### no-insecure-random > invalid

```js

      Math.random();
      this.Math.random();
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "default",
      "line": 2
    },
    {
      "messageId": "default",
      "line": 3
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/no-insecure-random',
    message: 'Do not use pseudo-random number generators for generating secret values such as tokens, passwords or keys.',
    messageId: 'default',
    severity: 1,
    nodeType: 'Identifier',
    line: 2,
    column: 6,
    endLine: 2,
    endColumn: 17,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-insecure-random > invalid

```js

      function notMath() : Math{
        return Math;
      }
    
      notMath().random();
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "default",
      "line": 6
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-insecure-random > invalid

```js

      function notCrypto() : Crypto{
        return crypto;
      }
    
      notCrypto().pseudoRandomBytes();
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "errors": [
    {
      "messageId": "default",
      "line": 6
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-postmessage-star-origin`

Pass: 8 / 9 (88.9%)
Fail: 1 / 9 (11.1%)
Skip: 0 / 9 (0.0%)

#### no-postmessage-star-origin > valid

```js

class WindowLike {
  postMessage(): void {
  };
}
function main() {
  var w: WindowLike = new WindowLike();
  w.postMessage('test', '*');
}
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/sdl/tests/fixtures/ts",
      "projectService": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-postmessage-star-origin',
    message: 'Do not use * as target origin when sending data to other windows',
    messageId: 'default',
    severity: 1,
    nodeType: 'Identifier',
    line: 8,
    column: 2,
    endLine: 8,
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

