# Conformance test results - jsx-a11y

Tested against: [jsx-a11y@7f3d698](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/tree/7f3d698f40d697471555e1dae74bfc7ee94d0b86) (6.10.2)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    39 | 100.0% |
| Fully passing     |    34 |  87.2% |
| Partially passing |     5 |  12.8% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests | 36557 | 100.0% |
| Passing     | 36527 |  99.9% |
| Failing     |    30 |   0.1% |
| Skipped     |     0 |   0.0% |

## Fully Passing Rules

- `accessible-emoji` (75 tests)
- `alt-text` (492 tests)
- `anchor-ambiguous-text` (117 tests)
- `anchor-has-content` (48 tests)
- `anchor-is-valid` (762 tests)
- `aria-activedescendant-has-tabindex` (60 tests)
- `aria-props` (84 tests)
- `aria-proptypes` (519 tests)
- `aria-role` (247 tests)
- `click-events-have-key-events` (108 tests)
- `control-has-associated-label` (1467 tests)
- `heading-has-content` (96 tests)
- `iframe-has-title` (48 tests)
- `img-redundant-alt` (177 tests)
- `interactive-supports-focus` (8250 tests)
- `label-has-associated-control` (549 tests)
- `label-has-for` (249 tests)
- `lang` (57 tests)
- `media-has-caption` (165 tests)
- `mouse-events-have-key-events` (117 tests)
- `no-access-key` (42 tests)
- `no-aria-hidden-on-focusable` (39 tests)
- `no-autofocus` (54 tests)
- `no-distracting-elements` (36 tests)
- `no-interactive-element-to-noninteractive-role` (1899 tests)
- `no-noninteractive-element-interactions` (2106 tests)
- `no-noninteractive-element-to-interactive-role` (2337 tests)
- `no-onchange` (51 tests)
- `no-redundant-roles` (81 tests)
- `prefer-tag-over-role` (42 tests)
- `role-has-required-aria-props` (235 tests)
- `role-supports-aria-props` (12663 tests)
- `scope` (30 tests)
- `tabindex-no-positive` (69 tests)

## Rules with Failures

- `aria-unsupported-elements` - 873 / 876 (99.7%)
- `autocomplete-valid` - 66 / 78 (84.6%)
- `html-has-lang` - 30 / 33 (90.9%)
- `no-noninteractive-tabindex` - 120 / 126 (95.2%)
- `no-static-element-interactions` - 2067 / 2073 (99.7%)

## Rules with Failures Detail

### `aria-unsupported-elements`

Pass: 873 / 876 (99.7%)
Fail: 3 / 876 (0.3%)
Skip: 0 / 876 (0.0%)

#### aria-unsupported-elements > valid

```js
<fake aria-hidden />
// features: [], parser: default
```

```json
{
  "errors": [
    {
      "message": "This element does not support ARIA roles, states and properties. Try removing the prop 'aria-hidden'.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### aria-unsupported-elements > valid

```js
<fake aria-hidden />
// features: [], parser: typescript-eslint
```

```json
{
  "errors": [
    {
      "message": "This element does not support ARIA roles, states and properties. Try removing the prop 'aria-hidden'.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### aria-unsupported-elements > valid

```js
<fake aria-hidden />
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "errors": [
    {
      "message": "This element does not support ARIA roles, states and properties. Try removing the prop 'aria-hidden'.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `autocomplete-valid`

Pass: 66 / 78 (84.6%)
Fail: 12 / 78 (15.4%)
Skip: 0 / 78 (0.0%)

#### autocomplete-valid > valid

```js
<input type="date" autocomplete="email" />;
// features: [], parser: default
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="date" autocomplete="email" />;
// features: [], parser: typescript-eslint
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="date" autocomplete="email" />;
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="number" autocomplete="url" />;
// features: [], parser: default
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="number" autocomplete="url" />;
// features: [], parser: typescript-eslint
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="number" autocomplete="url" />;
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="month" autocomplete="tel" />;
// features: [], parser: default
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="month" autocomplete="tel" />;
// features: [], parser: typescript-eslint
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<input type="month" autocomplete="tel" />;
// features: [], parser: @typescript-eslint/parser
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<Foo type="month" autocomplete="tel"></Foo>;
// features: [], parser: default, options: [{"inputComponents":["Foo"]}]
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "inputComponents": [
        "Foo"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<Foo type="month" autocomplete="tel"></Foo>;
// features: [], parser: typescript-eslint, options: [{"inputComponents":["Foo"]}]
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "inputComponents": [
        "Foo"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### autocomplete-valid > valid

```js
<Foo type="month" autocomplete="tel"></Foo>;
// features: [], parser: @typescript-eslint/parser, options: [{"inputComponents":["Foo"]}]
```

```json
{
  "errors": [
    {
      "message": "The autocomplete value is inappropriate for this type of input",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "inputComponents": [
        "Foo"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `html-has-lang`

Pass: 30 / 33 (90.9%)
Fail: 3 / 33 (9.1%)
Skip: 0 / 33 (0.0%)

#### html-has-lang > valid

```js
<HTMLTop lang="en" />
// features: [], parser: default, settings: {"jsx-a11y":{"components":{"HTMLTop":"html"}}}
```

```json
{
  "errors": [
    {
      "message": "<html> elements must have the lang prop.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {
    "jsx-a11y": {
      "components": {
        "HTMLTop": "html"
      }
    }
  }
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### html-has-lang > valid

```js
<HTMLTop lang="en" />
// features: [], parser: typescript-eslint, settings: {"jsx-a11y":{"components":{"HTMLTop":"html"}}}
```

```json
{
  "errors": [
    {
      "message": "<html> elements must have the lang prop.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {
    "jsx-a11y": {
      "components": {
        "HTMLTop": "html"
      }
    }
  }
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### html-has-lang > valid

```js
<HTMLTop lang="en" />
// features: [], parser: @typescript-eslint/parser, settings: {"jsx-a11y":{"components":{"HTMLTop":"html"}}}
```

```json
{
  "errors": [
    {
      "message": "<html> elements must have the lang prop.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {
    "jsx-a11y": {
      "components": {
        "HTMLTop": "html"
      }
    }
  }
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-noninteractive-tabindex`

Pass: 120 / 126 (95.2%)
Fail: 6 / 126 (4.8%)
Skip: 0 / 126 (0.0%)

#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} tabIndex="0" />;
// features: [], parser: default, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} tabIndex="0" />;
// features: [], parser: typescript-eslint, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} tabIndex="0" />;
// features: [], parser: @typescript-eslint/parser, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} tabIndex="0"/>;
// features: [], parser: default, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} tabIndex="0"/>;
// features: [], parser: typescript-eslint, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-noninteractive-tabindex:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} tabIndex="0"/>;
// features: [], parser: @typescript-eslint/parser, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "`tabIndex` should only be declared on interactive elements.",
      "type": "JSXAttribute"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "tags": [],
      "roles": [
        "tabpanel"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `no-static-element-interactions`

Pass: 2067 / 2073 (99.7%)
Fail: 6 / 2073 (0.3%)
Skip: 0 / 2073 (0.0%)

#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} />;
// features: [], parser: default, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} />;
// features: [], parser: typescript-eslint, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? "button" : LINK} onClick={() => {}} />;
// features: [], parser: @typescript-eslint/parser, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} />;
// features: [], parser: default, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} />;
// features: [], parser: typescript-eslint, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-static-element-interactions:recommended > valid

```js
<div role={isButton ? BUTTON : LINK} onClick={() => {}} />;
// features: [], parser: @typescript-eslint/parser, options: [{"allowExpressionValues":true}]
```

```json
{
  "errors": [
    {
      "message": "Avoid non-native interactive elements. If using native HTML is not possible, add an appropriate role and support for tabbing, mouse, keyboard, and touch inputs to an interactive content element.",
      "type": "JSXOpeningElement"
    }
  ],
  "options": [
    {
      "allowExpressionValues": true,
      "handlers": [
        "onClick",
        "onMouseDown",
        "onMouseUp",
        "onKeyPress",
        "onKeyDown",
        "onKeyUp"
      ]
    }
  ],
  "languageOptions": {
    "ecmaVersion": "latest",
    "parserOptions": {
      "ecmaFeatures": {
        "experimentalObjectRestSpread": true,
        "jsx": true
      }
    }
  },
  "settings": {}
}
```

AssertionError [ERR_ASSERTION]: Valid test case must not have `errors` property
    at assertValidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)

