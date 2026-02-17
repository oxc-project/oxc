# Conformance test results - react-hooks

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |     3 | 100.0% |
| Fully passing     |     0 |   0.0% |
| Partially passing |     2 |  66.7% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     1 |  33.3% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  5018 | 100.0% |
| Passing     |  4812 |  95.9% |
| Failing     |   192 |   3.8% |
| Skipped     |    14 |   0.3% |

## Fully Passing Rules

No rules fully passing

## Rules with Failures

- `exhaustive-deps` - 3570 / 3642 (98.0%)
- `rules-of-hooks` - 1256 / 1376 (91.3%)

## Rules with Failures Detail

### `exhaustive-deps`

Pass: 3564 / 3642 (97.9%)
Fail: 72 / 3642 (2.0%)
Skip: 6 / 3642 (0.2%)

#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: babel-eslint > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: babel-eslint':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: babel-eslint > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: babel-eslint',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: babel-eslint',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: babel-eslint > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: babel-eslint > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @babel/eslint-parser > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @babel/eslint-parser':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @babel/eslint-parser > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @babel/eslint-parser',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @babel/eslint-parser',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @babel/eslint-parser > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @babel/eslint-parser > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: hermes-eslint > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: hermes-eslint':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: hermes-eslint > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: hermes-eslint',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: hermes-eslint',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: hermes-eslint > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: hermes-eslint > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: hermes-eslint > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: hermes-eslint':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: hermes-eslint > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: hermes-eslint',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: hermes-eslint',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: hermes-eslint > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: hermes-eslint > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@2.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@3.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@4.x':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/exhaustive-deps > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > valid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber);
  })
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > valid

```js

function MyComponent(props) {
  useSpecialEffect(() => {
    console.log(props.foo);
  }, null);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > valid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, []);
  React.useEffect(() => {
    onStuff();
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0',
    message: "React Hook useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 8,
    column: 5,
    endLine: 8,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  },
  {
    ruleId: 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0',
    message: "React Hook React.useEffect has a missing dependency: 'onStuff'. Either include it or remove the dependency array.",
    messageId: null,
    severity: 1,
    nodeType: 'ArrayExpression',
    line: 11,
    column: 5,
    endLine: 11,
    endColumn: 7,
    fixes: null,
    suggestions: [ [Object] ]
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function MyComponent() {
  const [state, setState] = React.useState<number>(0);

  useSpecialEffect(() => {
    const someNumber: typeof state = 2;
    setState(prevState => prevState + someNumber + state);
  }, [])
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "additionalHooks": "useSpecialEffect",
      "experimental_autoDependenciesHooks": [
        "useSpecialEffect"
      ]
    }
  ],
  "errors": [
    {
      "message": "React Hook useSpecialEffect has a missing dependency: 'state'. Either include it or remove the dependency array. You can also do a functional update 'setState(s => ...)' if you only need 'state' in the 'setState' call.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [state]",
          "output": "\nfunction MyComponent() {\n  const [state, setState] = React.useState<number>(0);\n\n  useSpecialEffect(() => {\n    const someNumber: typeof state = 2;\n    setState(prevState => prevState + someNumber + state);\n  }, [state])\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0':
Options:
[
  {
    "additionalHooks": "useSpecialEffect",
    "experimental_autoDependenciesHooks": [
      "useSpecialEffect"
    ]
  }
]
Errors:
	Value {"additionalHooks":"useSpecialEffect","experimental_autoDependenciesHooks":["useSpecialEffect"]} should NOT have additional properties.
		Unexpected property "experimental_autoDependenciesHooks". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function MyComponent(props) {
  useEffect(() => {
    console.log(props.foo);
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "options": [
    {
      "requireExplicitEffectDeps": true
    }
  ],
  "errors": [
    {
      "message": "React Hook useEffect always requires dependencies. Please add a dependency array or an explicit `undefined`"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/eslint: v9, parser: @typescript-eslint/parser@^5.0.0':
Options:
[
  {
    "requireExplicitEffectDeps": true
  }
]
Errors:
	Value {"requireExplicitEffectDeps":true} should NOT have additional properties.
		Unexpected property "requireExplicitEffectDeps". Expected properties: "additionalHooks", "enableDangerousAutofixThisMayCauseInfiniteLoops".
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function MyComponent(props) {
  useCustomEffect(() => {
    console.log(props.foo);
  }, []);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useCustomEffect"
    }
  },
  "errors": [
    {
      "message": "React Hook useCustomEffect has a missing dependency: 'props.foo'. Either include it or remove the dependency array.",
      "suggestions": [
        {
          "desc": "Update the dependencies array to be: [props.foo]",
          "output": "\nfunction MyComponent(props) {\n  useCustomEffect(() => {\n    console.log(props.foo);\n  }, [props.foo]);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/exhaustive-deps > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function MyComponent({ theme }) {
  const onStuff = useEffectEvent(() => {
    showNotification(theme);
  });
  useEffect(() => {
    onStuff();
  }, [onStuff]);
  React.useEffect(() => {
    onStuff();
  }, [onStuff]);
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, []);\n  React.useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n}\n"
        }
      ]
    },
    {
      "message": "Functions returned from `useEffectEvent` must not be included in the dependency array. Remove `onStuff` from the list.",
      "suggestions": [
        {
          "desc": "Remove the dependency `onStuff`",
          "output": "\nfunction MyComponent({ theme }) {\n  const onStuff = useEffectEvent(() => {\n    showNotification(theme);\n  });\n  useEffect(() => {\n    onStuff();\n  }, [onStuff]);\n  React.useEffect(() => {\n    onStuff();\n  }, []);\n}\n"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `rules-of-hooks`

Pass: 1248 / 1376 (90.7%)
Fail: 120 / 1376 (8.7%)
Skip: 8 / 1376 (0.6%)

#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: babel-eslint > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/node_modules/babel-eslint/lib/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "babel-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 4 errors but had 0: []

0 !== 4

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @babel/eslint-parser > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 4 errors but had 0: []

0 !== 4

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: hermes-eslint > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/hermes-eslint/dist/index.js",
  "parserOptions": {
    "sourceType": "module",
    "enableExperimentalComponentSyntax": true
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 4 errors but had 0: []

0 !== 4

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 0: []

0 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: hermes-eslint > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "sourceType": "module",
      "enableExperimentalComponentSyntax": true
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "hermes-eslint",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 4 errors but had 0: []

0 !== 4

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v2/dist/parser.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@2.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v2",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v3/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@3.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v3",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v4/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@4.x > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v4",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v7, parser: @typescript-eslint/parser@^5.0.0-0 > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "parser": "/Users/cameron/github/Boshen/oxc2/apps/oxlint/conformance/submodules/react/node_modules/@typescript-eslint/parser-v5/dist/index.js",
  "parserOptions": {
    "ecmaFeatures": {
      "jsx": true
    },
    "ecmaVersion": 6,
    "sourceType": "module"
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function App({p1, p2}) {
  try {
    use(p1);
  } catch (error) {
    console.error(error);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function App({p1, p2}) {
  try {
    doSomething();
  } catch {
    use(p1);
  }
  use(p2);
  return <div>App</div>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "React Hook \"use\" cannot be called in a try/catch block."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// Invalid: useEffectEvent should not be callable in regular custom hooks without additional configuration
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useCustomHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// Invalid: useEffectEvent should not be callable in hooks not matching the settings regex
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  useWrongHook(() => {
    onClick();
  });
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "settings": {
    "react-hooks": {
      "additionalEffectHooks": "useMyEffect"
    }
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// This should error even though it shares an identifier name with the below
function MyComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={onClick} />
}

// The useEffectEvent function shares an identifier name with the above
function MyOtherComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  return <Child onClick={() => onClick()} />
}

// The useEffectEvent function shares an identifier name with the above
function MyLastComponent({theme}) {
  const onClick = useEffectEvent(() => {
    showNotification(theme)
  });
  useEffect(() => {
    onClick(); // No error here, errors on all other uses
    onClick;
  })
  return <Child />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    },
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
      "line": 15
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
    "lang": "tsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

const MyComponent = ({ theme }) => {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  return <Child onClick={onClick}></Child>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// Invalid because onClick is being aliased to foo but not invoked
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  let foo = onClick;
  return <Bar onClick={foo} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
      "line": 7
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// Should error because it's being passed down to JSX, although it's been referenced once
// in an effect
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(them);
  });
  useEffect(() => {
    setTimeout(onClick, 100);
  });
  return <Child onClick={onClick} />
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    {
      "message": "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


#### rules-of-hooks/rules-of-hooks > eslint: v9, parser: @typescript-eslint/parser@^5.0.0 > invalid

```js

// Invalid because functions created with useEffectEvent cannot be called in arbitrary closures.
function MyComponent({ theme }) {
  const onClick = useEffectEvent(() => {
    showNotification(theme);
  });
  // error message 1
  const onClick2 = () => { onClick() };
  // error message 2
  const onClick3 = useCallback(() => onClick(), []);
  // error message 3
  const onClick4 = onClick;
  return <>
    {/** error message 4 */}
    <Child onClick={onClick}></Child>
    <Child onClick={onClick2}></Child>
    <Child onClick={onClick3}></Child>
  </>;
}

```

```json
{
  "languageOptions": {
    "ecmaVersion": 6,
    "sourceType": "module",
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    },
    "parser": {}
  },
  "errors": [
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down.",
    "`onClick` is a function created with React Hook \"useEffectEvent\", and can only be called from Effects and Effect Events in the same component. It cannot be assigned to a variable or passed down."
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser-v5",
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


## Load Errors

### `compiler`

TypeError: Cannot read properties of undefined (reading 'rule')
    at <anonymous> (apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/__tests__/ReactCompilerRuleTypescript-test.ts:77:61)
    at Object.<anonymous> (apps/oxlint/conformance/submodules/react/packages/eslint-plugin-react-hooks/__tests__/ReactCompilerRuleTypescript-test.ts:77:72)
    at Module._compile (node:internal/modules/cjs/loader:1761:14)
    at Object.transformer (node_modules/.pnpm/tsx@4.21.0/node_modules/tsx/dist/register-D46fvsV_.cjs:3:1104)


