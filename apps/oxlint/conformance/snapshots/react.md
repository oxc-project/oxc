# Conformance test results - react

Tested against: [react@2c98b83](https://github.com/jsx-eslint/eslint-plugin-react/tree/2c98b83c451a4297edf1787d9a616e50687e27e8) (7.37.5)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |   101 | 100.0% |
| Fully passing     |    87 |  86.1% |
| Partially passing |    13 |  12.9% |
| Fully failing     |     1 |   1.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests | 16997 | 100.0% |
| Passing     | 16834 |  99.0% |
| Failing     |   163 |   1.0% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

- `boolean-prop-naming` (184 tests)
- `button-has-type` (173 tests)
- `checked-requires-onchange-or-readonly` (105 tests)
- `default-props-match-prop-types` (228 tests)
- `destructuring-assignment` (190 tests)
- `display-name` (322 tests)
- `forbid-component-props` (118 tests)
- `forbid-dom-props` (46 tests)
- `forbid-elements` (90 tests)
- `forbid-foreign-prop-types` (56 tests)
- `forbid-prop-types` (333 tests)
- `forward-ref-uses-ref` (45 tests)
- `function-component-definition` (262 tests)
- `hook-use-state` (136 tests)
- `iframe-missing-sandbox` (135 tests)
- `jsx-boolean-value` (54 tests)
- `jsx-child-element-spacing` (84 tests)
- `jsx-closing-bracket-location` (294 tests)
- `jsx-closing-tag-location` (52 tests)
- `jsx-curly-newline` (72 tests)
- `jsx-curly-spacing` (888 tests)
- `jsx-equals-spacing` (72 tests)
- `jsx-first-prop-new-line` (89 tests)
- `jsx-handler-names` (132 tests)
- `jsx-key` (167 tests)
- `jsx-max-depth` (78 tests)
- `jsx-max-props-per-line` (121 tests)
- `jsx-newline` (92 tests)
- `jsx-no-comment-textnodes` (66 tests)
- `jsx-no-constructed-context-values` (122 tests)
- `jsx-no-duplicate-props` (55 tests)
- `jsx-no-leaked-render` (177 tests)
- `jsx-no-literals` (324 tests)
- `jsx-no-script-url` (66 tests)
- `jsx-no-target-blank` (339 tests)
- `jsx-no-undef` (29 tests)
- `jsx-one-expression-per-line` (287 tests)
- `jsx-pascal-case` (127 tests)
- `jsx-props-no-spread-multi` (15 tests)
- `jsx-props-no-spreading` (72 tests)
- `jsx-sort-default-props` (90 tests)
- `jsx-sort-props` (281 tests)
- `jsx-space-before-closing` (60 tests)
- `jsx-tag-spacing` (212 tests)
- `jsx-wrap-multilines` (525 tests)
- `no-adjacent-inline-elements` (50 tests)
- `no-array-index-key` (213 tests)
- `no-arrow-function-lifecycle` (218 tests)
- `no-children-prop` (204 tests)
- `no-danger-with-children` (93 tests)
- `no-danger` (38 tests)
- `no-deprecated` (165 tests)
- `no-did-mount-set-state` (79 tests)
- `no-did-update-set-state` (79 tests)
- `no-direct-mutation-state` (63 tests)
- `no-find-dom-node` (24 tests)
- `no-is-mounted` (21 tests)
- `no-multi-comp` (108 tests)
- `no-namespace` (97 tests)
- `no-object-type-as-default-prop` (36 tests)
- `no-redundant-should-component-update` (27 tests)
- `no-render-return-value` (51 tests)
- `no-set-state` (23 tests)
- `no-string-refs` (33 tests)
- `no-this-in-sfc` (86 tests)
- `no-unescaped-entities` (39 tests)
- `no-unknown-property` (360 tests)
- `no-unsafe` (42 tests)
- `no-unstable-nested-components` (231 tests)
- `no-unused-class-component-methods` (168 tests)
- `no-unused-state` (286 tests)
- `no-will-update-set-state` (54 tests)
- `prefer-es6-class` (24 tests)
- `prefer-exact-props` (71 tests)
- `prefer-read-only-props` (26 tests)
- `prefer-stateless-function` (143 tests)
- `react-in-jsx-scope` (50 tests)
- `require-default-props` (381 tests)
- `require-render-return` (49 tests)
- `self-closing-comp` (141 tests)
- `sort-comp` (151 tests)
- `sort-default-props` (90 tests)
- `sort-prop-types` (237 tests)
- `state-in-constructor` (81 tests)
- `static-property-placement` (231 tests)
- `style-prop-object` (105 tests)
- `void-dom-elements-no-children` (72 tests)

## Rules with Failures

- `jsx-curly-brace-presence` - 392 / 393 (99.7%)
- `jsx-filename-extension` - 33 / 54 (61.1%)
- `jsx-fragments` - 60 / 61 (98.4%)
- `jsx-indent-props` - 114 / 120 (95.0%)
- `jsx-indent` - 437 / 449 (97.3%)
- `jsx-no-bind` - 221 / 224 (98.7%)
- `jsx-no-useless-fragment` - 88 / 89 (98.9%)
- `jsx-props-no-multi-spaces` - 0 / 83 (0.0%)
- `no-access-state-in-setstate` - 36 / 57 (63.2%)
- `no-invalid-html-attribute` - 804 / 807 (99.6%)
- `no-typos` - 288 / 291 (99.0%)
- `no-unused-prop-types` - 977 / 979 (99.8%)
- `prop-types` - 1118 / 1121 (99.7%)
- `require-optimization` - 61 / 64 (95.3%)

## Rules with Failures Detail

### `jsx-curly-brace-presence`

Pass: 392 / 393 (99.7%)
Fail: 1 / 393 (0.3%)
Skip: 0 / 393 (0.0%)

#### jsx-curly-brace-presence > invalid

```js

        <MyComponent>
          {'foo'}
          <div>
            {'bar'}
          </div>
          {'baz'}
          {'some-complicated-exp'}
        </MyComponent>
      
// features: [no-default,no-ts-new,no-babel-new], parser: typescript-eslint, options: [{"children":"never"}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "sourceType": "module",
      "ecmaVersion": 2015,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2015,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <MyComponent>\n          foo\n          <div>\n            bar\n          </div>\n          {'baz'}\n          {'some-complicated-exp'}\n        </MyComponent>\n      \n// features: [no-default,no-ts-new,no-babel-new], parser: typescript-eslint, options: [{\"children\":\"never\"}]",
  "options": [
    {
      "children": "never"
    }
  ],
  "errors": [
    {
      "messageId": "unnecessaryCurly",
      "line": 3
    },
    {
      "messageId": "unnecessaryCurly",
      "line": 5
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 4: [
  {
    ruleId: 'rule-to-test/jsx-curly-brace-presence',
    message: 'Curly braces are unnecessary here.',
    messageId: 'unnecessaryCurly',
    severity: 1,
    nodeType: 'JSXExpressionContainer',
    line: 3,
    column: 10,
    endLine: 3,
    endColumn: 17,
    fixes: [ [Object] ],
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/jsx-curly-brace-presence',
    message: 'Curly braces are unnecessary here.',
    messageId: 'unnecessaryCurly',
    severity: 1,
    nodeType: 'JSXExpressionContainer',
    line: 5,
    column: 12,
    endLine: 5,
    endColumn: 19,
    fixes: [ [Object] ],
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/jsx-curly-brace-presence',
    message: 'Curly braces are unnecessary here.',
    messageId: 'unnecessaryCurly',
    severity: 1,
    nodeType: 'JSXExpressionContainer',
    line: 7,
    column: 10,
    endLine: 7,
    endColumn: 17,
    fixes: [ [Object] ],
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/jsx-curly-brace-presence',
    message: 'Curly braces are unnecessary here.',
    messageId: 'unnecessaryCurly',
    severity: 1,
    nodeType: 'JSXExpressionContainer',
    line: 8,
    column: 10,
    endLine: 8,
    endColumn: 34,
    fixes: [ [Object] ],
    suggestions: null
  }
]

4 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `jsx-filename-extension`

Pass: 33 / 54 (61.1%)
Fail: 21 / 54 (38.9%)
Skip: 0 / 54 (0.0%)

#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "<text>"
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "<text>",
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "<text>",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: default, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: typescript-eslint, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: @typescript-eslint/parser, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "<text>"
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "<text>",
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "<text>",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: default, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: typescript-eslint, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > valid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: @typescript-eslint/parser, options: [{"extensions":[".js",".jsx"]}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "options": [
    {
      "extensions": [
        ".js",
        ".jsx"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: default, options: [{"allow":"as-needed"}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "notAComponent.js",
  "options": [
    {
      "allow": "as-needed"
    }
  ],
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: typescript-eslint, options: [{"allow":"as-needed"}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "notAComponent.js",
  "options": [
    {
      "allow": "as-needed"
    }
  ],
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <div>
<div />
</div>; }
// features: [], parser: @typescript-eslint/parser, options: [{"allow":"as-needed"}]
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "notAComponent.js",
  "options": [
    {
      "allow": "as-needed"
    }
  ],
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### jsx-filename-extension > invalid

```js
module.exports = function MyComponent() { return <>
</>; }
// features: [fragment], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "filename": "MyComponent.js",
  "errors": [
    {
      "messageId": "noJSXWithExtension",
      "data": {
        "ext": ".js"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `jsx-fragments`

Pass: 60 / 61 (98.4%)
Fail: 1 / 61 (1.6%)
Skip: 0 / 61 (0.0%)

#### jsx-fragments > invalid

```js
<><Foo /></>
// features: [fragment,no-babel,ts,no-ts-new], parser: typescript-eslint, options: ["element"], settings: {"react":{"version":"16.2","pragma":"Act","fragment":"Frag"}}
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": null,
  "options": [
    "element"
  ],
  "settings": {
    "react": {
      "version": "16.2",
      "pragma": "Act",
      "fragment": "Frag"
    }
  },
  "errors": [
    {
      "messageId": "preferPragma",
      "data": {
        "react": "Act",
        "fragment": "Frag"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Expected no autofixes to be suggested
+ actual - expected

+ '<Act.Frag><Foo /></Act.Frag>\n' +
- '<><Foo /></>\n' +
    '// features: [fragment,no-babel,ts,no-ts-new], parser: typescript-eslint, options: ["element"], settings: {"react":{"version":"16.2","pragma":"Act","fragment":"Frag"}}'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `jsx-indent-props`

Pass: 114 / 120 (95.0%)
Fail: 6 / 120 (5.0%)
Skip: 0 / 120 (0.0%)

#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorFalse
          ? <span
              className="value"
              some={{aaa}}
            />
          : null}
      
// features: [], parser: default, options: [{"indentMode":2,"ignoreTernaryOperator":false}]
```

```json
{
  "output": "\n        {this.props.ignoreTernaryOperatorFalse\n          ? <span\n            className=\"value\"\n            some={{aaa}}\n          />\n          : null}\n      \n// features: [], parser: default, options: [{\"indentMode\":2,\"ignoreTernaryOperator\":false}]",
  "options": [
    {
      "indentMode": 2,
      "ignoreTernaryOperator": false
    }
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorFalse
          ? <span
              className="value"
              some={{aaa}}
            />
          : null}
      
// features: [], parser: typescript-eslint, options: [{"indentMode":2,"ignoreTernaryOperator":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorFalse
          ? <span
              className="value"
              some={{aaa}}
            />
          : null}
      
// features: [], parser: @typescript-eslint/parser, options: [{"indentMode":2,"ignoreTernaryOperator":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorTrue
          ? <span
            className="value"
            some={{aaa}}
            />
          : null}
      
// features: [], parser: default, options: [{"indentMode":2,"ignoreTernaryOperator":true}]
```

```json
{
  "output": "\n        {this.props.ignoreTernaryOperatorTrue\n          ? <span\n            className=\"value\"\n            some={{aaa}}\n          />\n          : null}\n      \n// features: [], parser: default, options: [{\"indentMode\":2,\"ignoreTernaryOperator\":true}]",
  "options": [
    {
      "indentMode": 2,
      "ignoreTernaryOperator": true
    }
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorTrue
          ? <span
            className="value"
            some={{aaa}}
            />
          : null}
      
// features: [], parser: typescript-eslint, options: [{"indentMode":2,"ignoreTernaryOperator":true}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent-props > valid

```js

        {this.props.ignoreTernaryOperatorTrue
          ? <span
            className="value"
            some={{aaa}}
            />
          : null}
      
// features: [], parser: @typescript-eslint/parser, options: [{"indentMode":2,"ignoreTernaryOperator":true}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `jsx-indent`

Pass: 437 / 449 (97.3%)
Fail: 12 / 449 (2.7%)
Skip: 0 / 449 (0.0%)

#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: default, options: [2]
```

```json
{
  "output": "\n        const Component = () => (\n          <View\n            ListFooterComponent={(\n              <View\n                rowSpan={3}\n                placeholder=\"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do\"\n              />\n            )}\n          />\n        );\n      \n// features: [], parser: default, options: [2]",
  "options": [
    2
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: typescript-eslint, options: [2]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: @typescript-eslint/parser, options: [2]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: default, options: ["tab"]
```

```json
{
  "output": "\nconst Component = () => (\n\t<View\n\t\tListFooterComponent={(\n\t\t\t<View\n\t\t\t\trowSpan={3}\n\t\t\t\tplaceholder=\"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do\"\n\t\t\t/>\n\t\t)}\n\t/>\n);\n    \n// features: [], parser: default, options: [\"tab\"]",
  "options": [
    "tab"
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: typescript-eslint, options: ["tab"]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: @typescript-eslint/parser, options: ["tab"]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: default, options: [2,{"checkAttributes":false}]
```

```json
{
  "output": "\n        const Component = () => (\n          <View\n            ListFooterComponent={(\n              <View\n                rowSpan={3}\n                placeholder=\"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do\"\n              />\n            )}\n          />\n        );\n      \n// features: [], parser: default, options: [2,{\"checkAttributes\":false}]",
  "options": [
    2,
    {
      "checkAttributes": false
    }
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: typescript-eslint, options: [2,{"checkAttributes":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

        const Component = () => (
          <View
            ListFooterComponent={(
              <View
                rowSpan={3}
                placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
              />
        )}
          />
        );
      
// features: [], parser: @typescript-eslint/parser, options: [2,{"checkAttributes":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: default, options: ["tab",{"checkAttributes":false}]
```

```json
{
  "output": "\nconst Component = () => (\n\t<View\n\t\tListFooterComponent={(\n\t\t\t<View\n\t\t\t\trowSpan={3}\n\t\t\t\tplaceholder=\"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do\"\n\t\t\t/>\n\t\t)}\n\t/>\n);\n    \n// features: [], parser: default, options: [\"tab\",{\"checkAttributes\":false}]",
  "options": [
    "tab",
    {
      "checkAttributes": false
    }
  ],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: typescript-eslint, options: ["tab",{"checkAttributes":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-indent > valid

```js

const Component = () => (
	<View
		ListFooterComponent={(
			<View
				rowSpan={3}
				placeholder="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do"
			/>
)}
	/>
);
    
// features: [], parser: @typescript-eslint/parser, options: ["tab",{"checkAttributes":false}]
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `output` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `jsx-no-bind`

Pass: 221 / 224 (98.7%)
Fail: 3 / 224 (1.3%)
Skip: 0 / 224 (0.0%)

#### jsx-no-bind > valid

```js

        function click() { return true; }
        class Hello23 extends React.Component {
          renderDiv() {
            return <div onClick={click}>Hello</div>;
          }
        };
      
// features: [], parser: default
```

```json
{
  "errors": [],
  "languageOptions": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-no-bind > valid

```js

        function click() { return true; }
        class Hello23 extends React.Component {
          renderDiv() {
            return <div onClick={click}>Hello</div>;
          }
        };
      
// features: [], parser: typescript-eslint
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### jsx-no-bind > valid

```js

        function click() { return true; }
        class Hello23 extends React.Component {
          renderDiv() {
            return <div onClick={click}>Hello</div>;
          }
        };
      
// features: [], parser: @typescript-eslint/parser
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `jsx-no-useless-fragment`

Pass: 88 / 89 (98.9%)
Fail: 1 / 89 (1.1%)
Skip: 0 / 89 (0.0%)

#### jsx-no-useless-fragment > invalid

```js
<div><>{"a"}{"b"}</></div>
// features: [fragment,ts-old,no-ts-new,no-babel,no-default], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parser": {}
  },
  "output": null,
  "errors": [
    {
      "messageId": "ChildOfHtmlElement",
      "type": "JSXFragment"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Expected no autofixes to be suggested
+ actual - expected

+ '<div>{"a"}{"b"}</div>\n' +
- '<div><>{"a"}{"b"}</></div>\n' +
    '// features: [fragment,ts-old,no-ts-new,no-babel,no-default], parser: typescript-eslint'

    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `jsx-props-no-multi-spaces`

Pass: 0 / 83 (0.0%)
Fail: 83 / 83 (100.0%)
Skip: 0 / 83 (0.0%)

#### jsx-props-no-multi-spaces > valid

```js

        <App />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo="with  spaces   " bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo="with  spaces   " bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App foo="with  spaces   " bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo
          bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo
          bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo
          bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo {...test}
          bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo {...test}
          bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <App
          foo {...test}
          bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<App<T> foo bar />
// features: [ts,no-babel], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of undefined (reading 'end')
    at hasEmptyLines (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:67:44)
    at checkSpacing (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:76:11)
    at apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:138:11
    at Array.reduce (<anonymous>)


#### jsx-props-no-multi-spaces > valid

```js
<App<T> foo bar />
// features: [ts,no-babel], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of undefined (reading 'end')
    at hasEmptyLines (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:67:44)
    at checkSpacing (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:76:11)
    at apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:138:11
    at Array.reduce (<anonymous>)


#### jsx-props-no-multi-spaces > valid

```js
<Foo.Bar baz="quux" />
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<Foo.Bar baz="quux" />
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<Foo.Bar baz="quux" />
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy="thud" />
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy="thud" />
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js
<Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy="thud" />
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          type="button"
        />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          type="button"
        />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          type="button"
        />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          onClick={(value) => {
            console.log(value);
          }}
          type="button"
        />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          onClick={(value) => {
            console.log(value);
          }}
          type="button"
        />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

        <button
          title="Some button"
          onClick={(value) => {
            console.log(value);
          }}
          type="button"
        />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            // this is a second comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            // this is a second comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            // this is a comment
            // this is a second comment
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <App
            foo="Some button" // comment
            // comment
            bar=""
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <App
            foo="Some button" // comment
            // comment
            bar=""
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <App
            foo="Some button" // comment
            // comment
            bar=""
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            /* this is a multiline comment
                ...
                ... */
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            /* this is a multiline comment
                ...
                ... */
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > valid

```js

          <button
            title="Some button"
            /* this is a multiline comment
                ...
                ... */
            onClick={(value) => {
              console.log(value);
            }}
            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <App foo />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo="with  spaces   "   bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <App foo=\"with  spaces   \" bar />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo="with  spaces   "   bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo=\"with  spaces   \" bar />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo="with  spaces   "   bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo=\"with  spaces   \" bar />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo   bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo   bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App  foo   bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo bar />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "App",
        "prop2": "foo"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  {...test}  bar />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <App foo {...test} bar />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "test"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "test",
        "prop2": "bar"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  {...test}  bar />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo {...test} bar />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "test"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "test",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <App foo  {...test}  bar />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <App foo {...test} bar />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "foo",
        "prop2": "test"
      }
    },
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "test",
        "prop2": "bar"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js
<Foo.Bar  baz="quux" />
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "<Foo.Bar baz=\"quux\" />\n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foo.Bar",
        "prop2": "baz"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js
<Foo.Bar  baz="quux" />
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "<Foo.Bar baz=\"quux\" />\n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foo.Bar",
        "prop2": "baz"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js
<Foo.Bar  baz="quux" />
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "<Foo.Bar baz=\"quux\" />\n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foo.Bar",
        "prop2": "baz"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh  xyzzy="thud" />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "output": "\n        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy=\"thud\" />\n      \n// features: [], parser: default",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh",
        "prop2": "xyzzy"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh  xyzzy="thud" />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy=\"thud\" />\n      \n// features: [], parser: typescript-eslint",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh",
        "prop2": "xyzzy"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh  xyzzy="thud" />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "output": "\n        <Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh xyzzy=\"thud\" />\n      \n// features: [], parser: @typescript-eslint/parser",
  "errors": [
    {
      "messageId": "onlyOneSpace",
      "data": {
        "prop1": "Foobar.Foo.Bar.Baz.Qux.Quux.Quuz.Corge.Grault.Garply.Waldo.Fred.Plugh",
        "prop2": "xyzzy"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title='Some button'

          type="button"
        />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "type"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title='Some button'

          type="button"
        />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title='Some button'

          type="button"
        />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title="Some button"

          onClick={(value) => {
            console.log(value);
          }}

          type="button"
        />
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title="Some button"

          onClick={(value) => {
            console.log(value);
          }}

          type="button"
        />
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

        <button
          title="Some button"

          onClick={(value) => {
            console.log(value);
          }}

          type="button"
        />
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            // second comment

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            // second comment

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            // this is a comment
            // second comment

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            /*this is a
              multiline
              comment
            */

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ]
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            /*this is a
              multiline
              comment
            */

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


#### jsx-props-no-multi-spaces > invalid

```js

          <button
            title="Some button"
            /*this is a
              multiline
              comment
            */

            onClick={(value) => {
              console.log(value);
            }}

            type="button"
          />
        
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "title",
        "prop2": "onClick"
      }
    },
    {
      "messageId": "noLineGap",
      "data": {
        "prop1": "onClick",
        "prop2": "type"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

TypeError: Cannot read properties of null (reading 'type')
    at containsGenericType (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:112:32)
    at getGenericNode (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:117:11)
    at JSXOpeningElement (apps/oxlint/conformance/submodules/react_plugin/lib/rules/jsx-props-no-multi-spaces.js:140:12)
    at walkJSXOpeningElement (apps/oxlint/dist/lint.js)


### `no-access-state-in-setstate`

Pass: 36 / 57 (63.2%)
Fail: 21 / 57 (36.8%)
Skip: 0 / 57 (0.0%)

#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState({value: this.state.value + 1})
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState({value: this.state.value + 1})
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState({value: this.state.value + 1})
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(() => ({value: this.state.value + 1}))
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(() => ({value: this.state.value + 1}))
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(() => ({value: this.state.value + 1}))
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            var nextValue = this.state.value + 1
            this.setState({value: nextValue})
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            var nextValue = this.state.value + 1
            this.setState({value: nextValue})
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            var nextValue = this.state.value + 1
            this.setState({value: nextValue})
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        function nextState(state) {
          return {value: state.value + 1}
        }
        var Hello = React.createClass({
          onClick: function() {
            this.setState(nextState(this.state))
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        function nextState(state) {
          return {value: state.value + 1}
        }
        var Hello = React.createClass({
          onClick: function() {
            this.setState(nextState(this.state))
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        function nextState(state) {
          return {value: state.value + 1}
        }
        var Hello = React.createClass({
          onClick: function() {
            this.setState(nextState(this.state))
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => 1 + 1);
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => 1 + 1);
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => 1 + 1);
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => console.log(this.state));
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => console.log(this.state));
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          onClick: function() {
            this.setState(this.state, () => console.log(this.state));
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          nextState: function() {
            return {value: this.state.value + 1}
          },
          onClick: function() {
            this.setState(nextState())
          }
        });
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          nextState: function() {
            return {value: this.state.value + 1}
          },
          onClick: function() {
            this.setState(nextState())
          }
        });
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-access-state-in-setstate > invalid

```js

        var Hello = React.createClass({
          nextState: function() {
            return {value: this.state.value + 1}
          },
          onClick: function() {
            this.setState(nextState())
          }
        });
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true}}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "script",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "useCallback"
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


### `no-invalid-html-attribute`

Pass: 804 / 807 (99.6%)
Fail: 3 / 807 (0.4%)
Skip: 0 / 807 (0.0%)

#### no-invalid-html-attribute > invalid

```js
React.createElement("a", { rel: 1 })
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "neverValid",
      "data": {
        "attributeName": "rel",
        "reportingValue": 1
      },
      "suggestions": 1,
      "type": "Literal"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Error should have undefined suggestion. Instead found 1 suggestion.

1 !== undefined

    at assertSuggestionsAreCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-invalid-html-attribute > invalid

```js
React.createElement("a", { rel: 1 })
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "neverValid",
      "data": {
        "attributeName": "rel",
        "reportingValue": 1
      },
      "suggestions": 1,
      "type": "Literal"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Error should have undefined suggestion. Instead found 1 suggestion.

1 !== undefined

    at assertSuggestionsAreCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-invalid-html-attribute > invalid

```js
React.createElement("a", { rel: 1 })
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "neverValid",
      "data": {
        "attributeName": "rel",
        "reportingValue": 1
      },
      "suggestions": 1,
      "type": "Literal"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Error should have undefined suggestion. Instead found 1 suggestion.

1 !== undefined

    at assertSuggestionsAreCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-typos`

Pass: 288 / 291 (99.0%)
Fail: 3 / 291 (1.0%)
Skip: 0 / 291 (0.0%)

#### no-typos > invalid

```js

        /** @extends React.Component */
        class MyComponent extends BaseComponent {}
        MyComponent.PROPTYPES = {}
      
// features: [], parser: default, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true},"sourceType":"module"}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      },
      "sourceType": "module"
    }
  },
  "errors": [
    {
      "messageId": "typoStaticClassProp"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-typos > invalid

```js

        /** @extends React.Component */
        class MyComponent extends BaseComponent {}
        MyComponent.PROPTYPES = {}
      
// features: [], parser: typescript-eslint, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true},"sourceType":"module"}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      },
      "sourceType": "module"
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "typoStaticClassProp"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### no-typos > invalid

```js

        /** @extends React.Component */
        class MyComponent extends BaseComponent {}
        MyComponent.PROPTYPES = {}
      
// features: [], parser: @typescript-eslint/parser, parserOptions: {"ecmaVersion":2018,"ecmaFeatures":{"jsx":true},"sourceType":"module"}
```

```json
{
  "languageOptions": {
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parserOptions": {
      "ecmaVersion": 2018,
      "ecmaFeatures": {
        "jsx": true
      },
      "sourceType": "module"
    },
    "parser": {}
  },
  "errors": [
    {
      "messageId": "typoStaticClassProp"
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


### `no-unused-prop-types`

Pass: 977 / 979 (99.8%)
Fail: 2 / 979 (0.2%)
Skip: 0 / 979 (0.0%)

#### no-unused-prop-types > valid

```js

        class Test extends Component<{}, {selectedId: string}> {
          constructor(props: *) {
            super(props);
            this.state = {
              selectedId: '',
            };
          }

          onChange = ({id}: {id: string}) => { // This will say: 'id' PropType is defined but prop is never used (react/no-unused-prop-types)
            this.setState({
              selectedId: id,
            });
          };

          render() {
            const {selectedId} = this.state;
            return (
              <div>
                {selectedId}
                <select onChange={() => this.onChange({id: '1'})}>
                  <option value='1'>1</option>
                </select>
              </div>
            );
          }
        }
      
// features: [class fields,types], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "typescript-eslint-parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-unused-prop-types > valid

```js

        class Test extends Component<{}, {selectedId: string}> {
          constructor(props: *) {
            super(props);
            this.state = {
              selectedId: '',
            };
          }

          onChange = ({id}: {id: string}) => { // This will say: 'id' PropType is defined but prop is never used (react/no-unused-prop-types)
            this.setState({
              selectedId: id,
            });
          };

          render() {
            const {selectedId} = this.state;
            return (
              <div>
                {selectedId}
                <select onChange={() => this.onChange({id: '1'})}>
                  <option value='1'>1</option>
                </select>
              </div>
            );
          }
        }
      
// features: [class fields,types], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "tsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `prop-types`

Pass: 1118 / 1121 (99.7%)
Fail: 3 / 1121 (0.3%)
Skip: 0 / 1121 (0.0%)

#### prop-types > invalid

```js

        /** @extends React.Component */
        class Hello extends ChildComponent {
          render() {
            return <div>Hello {this.props.name}</div>;
          }
        }
      
// features: [], parser: default
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module"
  },
  "errors": [
    {
      "messageId": "missingPropType",
      "data": {
        "name": "name"
      },
      "line": 5,
      "column": 43,
      "type": "Identifier"
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### prop-types > invalid

```js

        /** @extends React.Component */
        class Hello extends ChildComponent {
          render() {
            return <div>Hello {this.props.name}</div>;
          }
        }
      
// features: [], parser: typescript-eslint
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "missingPropType",
      "data": {
        "name": "name"
      },
      "line": 5,
      "column": 43,
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "typescript-eslint-parser",
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


#### prop-types > invalid

```js

        /** @extends React.Component */
        class Hello extends ChildComponent {
          render() {
            return <div>Hello {this.props.name}</div>;
          }
        }
      
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2018,
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "ecmaVersion": 2018,
    "sourceType": "module",
    "parser": {}
  },
  "errors": [
    {
      "messageId": "missingPropType",
      "data": {
        "name": "name"
      },
      "line": 5,
      "column": 43,
      "type": "Identifier"
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


### `require-optimization`

Pass: 61 / 64 (95.3%)
Fail: 3 / 64 (4.7%)
Skip: 0 / 64 (0.0%)

#### react-require-optimization > valid

```js

        import React from "react";
        class YourComponent extends React.Component {
          handleClick = () => {}
          shouldComponentUpdate(){
            return true;
          }
          render() {
            return <div onClick={this.handleClick}>123</div>
          }
        }
      
// features: [class fields], parser: default, parserOptions: {"ecmaVersion":2022}
```

```json
{
  "errors": [
    {
      "messageId": "noShouldComponentUpdate"
    }
  ],
  "languageOptions": {
    "parserOptions": {
      "ecmaVersion": 2022
    },
    "ecmaVersion": 2022
  }
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### react-require-optimization > valid

```js

        import React from "react";
        class YourComponent extends React.Component {
          handleClick = () => {}
          shouldComponentUpdate(){
            return true;
          }
          render() {
            return <div onClick={this.handleClick}>123</div>
          }
        }
      
// features: [class fields], parser: typescript-eslint
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### react-require-optimization > valid

```js

        import React from "react";
        class YourComponent extends React.Component {
          handleClick = () => {}
          shouldComponentUpdate(){
            return true;
          }
          render() {
            return <div onClick={this.handleClick}>123</div>
          }
        }
      
// features: [class fields], parser: @typescript-eslint/parser
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)

