# Conformance test results - import-x

Tested against: [import-x@b767b8c](https://github.com/un-ts/eslint-plugin-import-x/tree/b767b8c16cae017ac7b092a161684e31b262e5ae) (4.17.1)

## Summary

### Rules

| Status            | Count | %      |
| ----------------- | ----- | ------ |
| Total rules       |    45 | 100.0% |
| Fully passing     |    17 |  37.8% |
| Partially passing |    28 |  62.2% |
| Fully failing     |     0 |   0.0% |
| Load errors       |     0 |   0.0% |
| No tests run      |     0 |   0.0% |

### Tests

| Status      | Count | %      |
| ----------- | ----- | ------ |
| Total tests |  2670 | 100.0% |
| Passing     |  1936 |  72.5% |
| Failing     |   609 |  22.8% |
| Skipped     |   125 |   4.7% |

## Fully Passing Rules

- `dynamic-import-chunkname` (168 tests)
- `exports-last` (13 tests)
- `newline-after-import` (95 tests)
- `no-absolute-path` (35 tests) (2 skipped)
- `no-amd` (15 tests)
- `no-anonymous-default-export` (49 tests) (1 skipped)
- `no-commonjs` (45 tests)
- `no-dynamic-require` (46 tests) (14 skipped)
- `no-import-module-exports` (14 tests)
- `no-namespace` (11 tests)
- `no-nodejs-modules` (34 tests)
- `no-relative-packages` (11 tests)
- `no-relative-parent-imports` (18 tests)
- `no-self-import` (24 tests)
- `no-unassigned-import` (34 tests)
- `no-webpack-loader-syntax` (19 tests)
- `prefer-namespace-import` (12 tests)

## Rules with Failures

- `consistent-type-specifier-style` - 83 / 133 (62.4%)
- `default` - 38 / 65 (58.5%)
- `export` - 33 / 68 (48.5%)
- `extensions` - 77 / 89 (86.5%)
- `first` - 13 / 14 (92.9%)
- `group-exports` - 34 / 39 (87.2%)
- `max-dependencies` - 8 / 13 (61.5%)
- `named` - 110 / 144 (76.4%)
- `namespace` - 33 / 101 (32.7%)
- `no-cycle` - 25 / 71 (35.2%)
- `no-default-export` - 26 / 30 (86.7%)
- `no-deprecated` - 20 / 55 (36.4%)
- `no-duplicates` - 50 / 79 (63.3%)
- `no-empty-named-blocks` - 12 / 29 (41.4%)
- `no-extraneous-dependencies` - 122 / 128 (95.3%)
- `no-internal-modules` - 45 / 46 (97.8%)
- `no-mutable-exports` - 28 / 30 (93.3%)
- `no-named-as-default-member` - 22 / 29 (75.9%)
- `no-named-as-default` - 26 / 34 (76.5%)
- `no-named-default` - 23 / 25 (92.0%)
- `no-named-export` - 22 / 26 (84.6%)
- `no-rename-default` - 32 / 116 (27.6%)
- `no-restricted-paths` - 55 / 59 (93.2%)
- `no-unresolved` - 143 / 152 (94.1%)
- `no-useless-path-segments` - 98 / 106 (92.5%)
- `order` - 191 / 283 (67.5%)
- `prefer-default-export` - 39 / 51 (76.5%)
- `unambiguous` - 10 / 12 (83.3%)

## Rules with Failures Detail

### `consistent-type-specifier-style`

Pass: 34 / 133 (25.6%)
Fail: 50 / 133 (37.6%)
Skip: 49 / 133 (36.8%)

#### TypeScript > consistent-type-specifier-style > valid

```js
import type Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type {} from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type { Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type { Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type { Foo, Bar, Baz, Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type {} from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import { type Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import { type Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import { type Foo, type Bar, Baz, Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type * as Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type {Foo} from 'some-package' with {'resolution-mode': 'import'};
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > consistent-type-specifier-style > valid

```js
import type {Foo} from 'some-package' with {'resolution-mode': 'import'};
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type {} from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type { Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type { Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type { Foo, Bar, Baz, Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import type {} from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { type Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { type Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { type Foo, type Bar, Baz, Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import typeof Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import typeof { Foo, Bar, Baz, Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-top-level"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import typeof Foo from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { typeof Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { typeof Foo, typeof Bar, typeof Baz, typeof Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > valid

```js
import { type Foo, type Bar, typeof Baz, typeof Bam } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "options": [
    "prefer-inline"
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { type Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import type {Foo} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "data": {
        "kind": "type"
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { type Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import type {Foo as Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { type Foo, type Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import type {Foo, Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { Foo, type Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import { Foo  } from 'Foo';\nimport type {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { type Foo, Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import {  Bar } from 'Foo';\nimport type {Foo} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import Foo, { type Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import Foo from 'Foo';\nimport type {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import Foo, { type Bar, Baz } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import Foo, {  Baz } from 'Foo';\nimport type {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { Component, type ComponentProps } from "package-1";
import {
  Component1,
  Component2,
  Component3,
  Component4,
  Component5,
} from "package-2";
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import { Component  } from \"package-1\";\nimport type {ComponentProps} from \"package-1\";\nimport {\n  Component1,\n  Component2,\n  Component3,\n  Component4,\n  Component5,\n} from \"package-2\";",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import type { Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import  { type Foo } from 'Foo';",
  "options": [
    "prefer-inline"
  ],
  "errors": [
    {
      "messageId": "inline",
      "data": {
        "kind": "type"
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import type { Foo, Bar, Baz } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import  { type Foo, type Bar, type Baz } from 'Foo';",
  "options": [
    "prefer-inline"
  ],
  "errors": [
    {
      "messageId": "inline",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { typeof Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import typeof {Foo} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "data": {
        "kind": "typeof"
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { typeof Foo as Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import typeof {Foo as Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { type Foo, typeof Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import type {Foo} from 'Foo';\nimport typeof {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "data": {
        "kind": "type/typeof"
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { typeof Foo, typeof Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import typeof {Foo, Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { Foo, typeof Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import { Foo  } from 'Foo';\nimport typeof {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { typeof Foo, Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import {  Bar } from 'Foo';\nimport typeof {Foo} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import { Foo, type Bar, typeof Baz } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import { Foo   } from 'Foo';\nimport type {Bar} from 'Foo';\nimport typeof {Baz} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "data": {
        "kind": "type"
      },
      "type": "ImportSpecifier"
    },
    {
      "messageId": "topLevel",
      "data": {
        "kind": "typeof"
      },
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import Foo, { typeof Bar } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import Foo from 'Foo';\nimport typeof {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import Foo, { typeof Bar, Baz } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import Foo, {  Baz } from 'Foo';\nimport typeof {Bar} from 'Foo';",
  "options": [
    "prefer-top-level"
  ],
  "errors": [
    {
      "messageId": "topLevel",
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import typeof { Foo } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import  { typeof Foo } from 'Foo';",
  "options": [
    "prefer-inline"
  ],
  "errors": [
    {
      "messageId": "inline",
      "data": {
        "kind": "typeof"
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Babel/Flow > consistent-type-specifier-style > invalid

```js
import typeof { Foo, Bar, Baz } from 'Foo';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module",
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "output": "import  { typeof Foo, typeof Bar, typeof Baz } from 'Foo';",
  "options": [
    "prefer-inline"
  ],
  "errors": [
    {
      "messageId": "inline",
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `default`

Pass: 36 / 65 (55.4%)
Fail: 27 / 65 (41.5%)
Skip: 2 / 65 (3.1%)

#### default > valid

```js
import foo from "./default-export";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 34,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import foo from "./mixed-exports";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './mixed-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import bar from "./default-export";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 34,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import CoolClass from "./default-class";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-class': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 39,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import bar, { baz } from "./default-export";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
export bar from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > valid

```js
export bar, { foo } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > valid

```js
export bar, * as names from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > valid

```js
import twofer from "./trampoline"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './trampoline': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import foo from "./named-default-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './named-default-export': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import connectedApp from "./redux"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './redux': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 34,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import App from "./jsx/App"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true,
        "modules": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './jsx/App': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 27,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import Foo from './jsx/FooES7.js';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './jsx/FooES7.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import bar from './default-export-from.js';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export-from.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 42,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import bar from './default-export-from-named.js';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export-from-named.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 48,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
import bar from './default-export-from-ignored.js';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": [
      "common"
    ]
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: "Parse errors in imported module './default-export-from-ignored.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 50,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### default > valid

```js
export bar from './default-export-from-ignored.js';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": [
      "common"
    ]
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
import baz from "./named-exports";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./named-exports"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'noDefaultExport'
+ actual - expected

+ null
- 'noDefaultExport'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
export baz from "./named-exports"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./named-exports"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
export baz, { bar } from "./named-exports"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./named-exports"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
export baz, * as names from "./named-exports"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./named-exports"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
import twofer from "./broken-trampoline"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./broken-trampoline"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'noDefaultExport'
+ actual - expected

+ null
- 'noDefaultExport'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### default > invalid

```js
import barDefault from "./re-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noDefaultExport",
      "data": {
        "module": "./re-export"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'noDefaultExport'
+ actual - expected

+ null
- 'noDefaultExport'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > default > valid

```js
import React from "./typescript-export-assign-default-namespace"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/typescript-export-assign-default-namespace"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: 'No default export found in imported module "./typescript-export-assign-default-namespace".',
    messageId: 'noDefaultExport',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 7,
    endLine: 1,
    endColumn: 12,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### TypeScript > default > valid

```js
import Foo from "./typescript-export-as-default-namespace"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/typescript-export-as-default-namespace"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: 'No default export found in imported module "./typescript-export-as-default-namespace".',
    messageId: 'noDefaultExport',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 7,
    endLine: 1,
    endColumn: 10,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### TypeScript > default > valid

```js
import Foo from "./typescript-export-react-test-renderer"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/typescript-export-react-test-renderer"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: 'No default export found in imported module "./typescript-export-react-test-renderer".',
    messageId: 'noDefaultExport',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 7,
    endLine: 1,
    endColumn: 10,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### TypeScript > default > valid

```js
import Foo from "./typescript-extended-config"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "tsconfigRootDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/typescript-extended-config"
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/default',
    message: 'No default export found in imported module "./typescript-extended-config".',
    messageId: 'noDefaultExport',
    severity: 1,
    nodeType: 'Identifier',
    line: 1,
    column: 7,
    endLine: 1,
    endColumn: 10,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `export`

Pass: 30 / 68 (44.1%)
Fail: 35 / 68 (51.5%)
Skip: 3 / 68 (4.4%)

#### export > valid

```js
let bar; export { bar }; export * from "./export-all"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "Parse errors in imported module './export-all': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 39,
    endLine: 1,
    endColumn: 53,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### export > valid

```js
export * from "./export-all"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "Parse errors in imported module './export-all': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 14,
    endLine: 1,
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


#### export > valid

```js
export default foo; export * from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "Parse errors in imported module './bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 34,
    endLine: 1,
    endColumn: 41,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### export > valid

```js
export * from "./issue-370-commonjs-namespace/bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": [
      "foo"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "Parse errors in imported module './issue-370-commonjs-namespace/bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 14,
    endLine: 1,
    endColumn: 50,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### export > invalid

```js
let foo; export { foo }; export * from "./export-all"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "foo"
      }
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "foo"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "Parse errors in imported module './export-all': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 39,
    endLine: 1,
    endColumn: 53,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### export > invalid

```js
export * from "./default-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noNamed",
      "data": {
        "module": "./default-export"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'noNamed'

null !== 'noNamed'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### export > invalid

```js

        export default function a(): void;
        export default function a() {}
        export { x as default };
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiDefault"
    },
    {
      "messageId": "multiDefault"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export const Foo = 1;
          export type Foo = number;
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export const Foo = 1;
          export interface Foo {}
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export function fff(a: string);
          export function fff(a: number);
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export function fff(a: string);
          export function fff(a: number);
          export function fff(a: string|number) {};
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export const Bar = 1;
          export namespace Foo {
            export const Bar = 1;
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export type Bar = string;
          export namespace Foo {
            export type Bar = string;
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export const Bar = 1;
          export type Bar = string;
          export namespace Foo {
            export const Bar = 1;
            export type Bar = string;
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export namespace Foo {
            export const Foo = 1;
            export namespace Bar {
              export const Foo = 2;
            }
            export namespace Baz {
              export const Foo = 3;
            }
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

            export class Foo { }
            export namespace Foo { }
            export namespace Foo {
              export class Bar {}
            }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

            export function Foo();
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

            export function Foo(a: string);
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

            export function Foo(a: string);
            export function Foo(a: number);
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

            export enum Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          declare module "a" {
            const Foo = 1;
            export {Foo as default};
          }
          declare module "b" {
            const Bar = 2;
            export {Bar as default};
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          declare module "a" {
            const Foo = 1;
            export {Foo as default};
          }
          const Bar = 2;
          export {Bar as default};
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > valid

```js

          export * from './module';
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/export-star-4/index.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    },
    "import-x/extensions": [
      ".js",
      ".ts",
      ".jsx"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/export',
    message: "No named exports found in module './module'.",
    messageId: 'noNamed',
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 24,
    endLine: 2,
    endColumn: 34,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### TypeScript > export > invalid

```js

          export type Foo = string;
          export type Foo = number;
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

          export const a = 1
          export namespace Foo {
            export const a = 2;
            export const a = 3;
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "a"
      },
      "line": 4
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "a"
      },
      "line": 5
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

          declare module 'foo' {
            const Foo = 1;
            export default Foo;
            export default Foo;
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiDefault",
      "line": 4
    },
    {
      "messageId": "multiDefault",
      "line": 5
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

          export namespace Foo {
            export namespace Bar {
              export const Foo = 1;
              export const Foo = 2;
            }
            export namespace Baz {
              export const Bar = 3;
              export const Bar = 4;
            }
          }
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 4
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 5
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Bar"
      },
      "line": 8
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Bar"
      },
      "line": 9
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export class Foo { }
            export class Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export enum Foo { }
            export enum Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export enum Foo { }
            export class Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export const Foo = 'bar';
            export class Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export function Foo() { };
            export class Foo { }
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export const Foo = 'bar';
            export function Foo() { };
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

            export const Foo = 'bar';
            export namespace Foo { }
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 2
    },
    {
      "messageId": "multiNamed",
      "data": {
        "name": "Foo"
      },
      "line": 3
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > export > invalid

```js

          declare module "a" {
            const Foo = 1;
            export {Foo as default};
          }
          const Bar = 2;
          export {Bar as default};
          const Baz = 3;
          export {Baz as default};
        
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "multiDefault",
      "line": 7
    },
    {
      "messageId": "multiDefault",
      "line": 9
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `extensions`

Pass: 77 / 89 (86.5%)
Fail: 12 / 89 (13.5%)
Skip: 0 / 89 (0.0%)

#### TypeScript > typescript - extensions ignore type-only > valid

```js
import type T from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ts": "never",
      "tsx": "never",
      "js": "never",
      "jsx": "never"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > typescript - extensions ignore type-only > valid

```js
export type { MyType } from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ts": "never",
      "tsx": "never",
      "js": "never",
      "jsx": "never"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > typescript - extensions ignore type-only > invalid

```js
import type T from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "missing",
      "data": {
        "importPath": "./typescript-declare"
      }
    }
  ],
  "options": [
    "always",
    {
      "pattern": {
        "ts": "never",
        "tsx": "never",
        "js": "never",
        "jsx": "never"
      },
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > typescript - extensions ignore type-only > invalid

```js
export type { MyType } from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "missing",
      "data": {
        "importPath": "./typescript-declare"
      }
    }
  ],
  "options": [
    "always",
    {
      "pattern": {
        "ts": "never",
        "tsx": "never",
        "js": "never",
        "jsx": "never"
      },
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > valid

```js
import type { MyType } from "./typescript-declare.ts";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > valid

```js
export type { MyType } from "./typescript-declare.ts";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > valid

```js

              import { ErrorMessage as UpstreamErrorMessage } from '@black-flag/core/util';

              import { $instances } from 'rootverse+debug:src.ts';
              import { $exists } from 'rootverse+bfe:src/symbols.ts';

              import type { Entries } from 'type-fest';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ignorePackages": true,
      "checkTypeImports": true,
      "pathGroupOverrides": [
        {
          "pattern": "multiverse{*,*/**}",
          "action": "enforce"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > valid

```js

              import { ErrorMessage as UpstreamErrorMessage } from '@black-flag/core/util';

              import { $instances } from 'rootverse+debug:src.ts';
              import { $exists } from 'rootverse+bfe:src/symbols.ts';

              import type { Entries } from 'type-fest';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ignorePackages": true,
      "checkTypeImports": true,
      "pathGroupOverrides": [
        {
          "pattern": "rootverse{*,*/**}",
          "action": "enforce"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > valid

```js

              import { ErrorMessage as UpstreamErrorMessage } from '@black-flag/core/util';

              import { $instances } from 'rootverse+debug:src';
              import { $exists } from 'rootverse+bfe:src/symbols';

              import type { Entries } from 'type-fest';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ignorePackages": true,
      "checkTypeImports": true,
      "pathGroupOverrides": [
        {
          "pattern": "multiverse{*,*/**}",
          "action": "enforce"
        },
        {
          "pattern": "rootverse{*,*/**}",
          "action": "ignore"
        }
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > invalid

```js
import type { MyType } from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "missing",
      "data": {
        "importPath": "./typescript-declare"
      }
    }
  ],
  "options": [
    "always",
    {
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > invalid

```js
export type { MyType } from "./typescript-declare";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "missing",
      "data": {
        "importPath": "./typescript-declare"
      }
    }
  ],
  "options": [
    "always",
    {
      "checkTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > extensions > invalid

```js

              import { ErrorMessage as UpstreamErrorMessage } from '@black-flag/core/util';

              import { $instances } from 'rootverse+debug:src';
              import { $exists } from 'rootverse+bfe:src/symbols';

              import type { Entries } from 'type-fest';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    "always",
    {
      "ignorePackages": true,
      "checkTypeImports": true,
      "pathGroupOverrides": [
        {
          "pattern": "rootverse{*,*/**}",
          "action": "enforce"
        },
        {
          "pattern": "universe{*,*/**}",
          "action": "ignore"
        }
      ]
    }
  ],
  "errors": [
    {
      "messageId": "missing",
      "data": {
        "importPath": "rootverse+debug:src"
      },
      "line": 4
    },
    {
      "messageId": "missing",
      "data": {
        "importPath": "rootverse+bfe:src/symbols"
      },
      "line": 5
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `first`

Pass: 12 / 14 (85.7%)
Fail: 1 / 14 (7.1%)
Skip: 1 / 14 (7.1%)

#### first > invalid

```js
var a = 1;              import { y } from './bar';              if (true) { x() };              import { x } from './foo';              import { z } from './baz';
```

```json
{
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "order"
    },
    {
      "messageId": "order"
    },
    {
      "messageId": "order"
    }
  ],
  "output": [
    "import { y } from './bar';              var a = 1;              if (true) { x() };              import { x } from './foo';              import { z } from './baz';",
    "import { y } from './bar';              import { x } from './foo';              var a = 1;              if (true) { x() };              import { z } from './baz';",
    "import { y } from './bar';              import { x } from './foo';              import { z } from './baz';              var a = 1;              if (true) { x() };"
  ]
}
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `group-exports`

Pass: 34 / 39 (87.2%)
Fail: 5 / 39 (12.8%)
Skip: 0 / 39 (0.0%)

#### group-exports > valid

```js

      type firstType = {
        propType: string
      };
      const first = {};
      export type { firstType };
      export { first };
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/preset-flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "skip": false,
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### group-exports > valid

```js

      type firstType = {
        propType: string
      };
      type secondType = {
        propType: string
      };
      export type { firstType, secondType };
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/preset-flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "skip": false,
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### group-exports > valid

```js

      export type { type1A, type1B } from './module-1'
      export { method1 } from './module-1'
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/preset-flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "skip": false,
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### group-exports > invalid

```js

        type firstType = {
          propType: string
        };
        type secondType = {
          propType: string
        };
        const first = {};
        export type { firstType };
        export type { secondType };
        export { first };
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/preset-flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "skip": false,
  "errors": [
    {
      "messageId": "ExportNamedDeclaration"
    },
    {
      "messageId": "ExportNamedDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### group-exports > invalid

```js

        export type { type1 } from './module-1'
        export type { type2 } from './module-1'
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/preset-flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "skip": false,
  "errors": [
    {
      "messageId": "ExportNamedDeclaration"
    },
    {
      "messageId": "ExportNamedDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `max-dependencies`

Pass: 8 / 13 (61.5%)
Fail: 5 / 13 (38.5%)
Skip: 0 / 13 (0.0%)

#### max-dependencies > invalid

```js
import type { x } from './foo'; import type { y } from './bar'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "max": 1
    }
  ],
  "errors": [
    {
      "messageId": "max",
      "data": {
        "max": 1
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### max-dependencies > invalid

```js
import type { x } from './foo'; import type { y } from './bar'; import type { z } from './baz'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "max": 2,
      "ignoreTypeImports": false
    }
  ],
  "errors": [
    {
      "messageId": "max",
      "data": {
        "max": 2
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > max-dependencies > valid

```js
import type { x } from './foo'; import { y } from './bar';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "max": 1,
      "ignoreTypeImports": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > max-dependencies > invalid

```js
import type { x } from './foo'; import type { y } from './bar'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "max": 1
    }
  ],
  "errors": [
    {
      "messageId": "max",
      "data": {
        "max": 1
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > max-dependencies > invalid

```js
import type { x } from './foo'; import type { y } from './bar'; import type { z } from './baz'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "max": 2,
      "ignoreTypeImports": false
    }
  ],
  "errors": [
    {
      "messageId": "max",
      "data": {
        "max": 2
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `named`

Pass: 104 / 144 (72.2%)
Fail: 34 / 144 (23.6%)
Skip: 6 / 144 (4.2%)

#### named > valid

```js
export bar, { foo } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import type { MissingType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import typeof { MissingType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import type { MyOpaqueType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import typeof { MyOpaqueType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import { type MyOpaqueType, MyClass } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import { typeof MyOpaqueType, MyClass } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import typeof MissingType from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
import typeof * as MissingType from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
export type { MissingType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > valid

```js
export type { MyOpaqueType } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > invalid

```js
import { somethingElse } from "./test-module"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "somethingElse",
        "path": "./test-module"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { baz } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./bar"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { baz, bop } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./bar"
      },
      "type": "Identifier"
    },
    {
      "messageId": "notFound",
      "data": {
        "name": "bop",
        "path": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 0: []

0 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### named > invalid

```js
import {a, b, c} from "./named-exports"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "c",
        "path": "./named-exports"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { a } from "./default-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "a",
        "path": "./default-export"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { ActionTypess } from "./qc"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "ActionTypess",
        "path": "./qc"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import {a, b, c, d, e} from "./re-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "e",
        "path": "./re-export"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { a } from "./re-export-names"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "a",
        "path": "./re-export-names"
      },
      "type": "Identifier"
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


#### named > invalid

```js
export { bar } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "bar",
        "path": "./bar"
      },
      "type": "Identifier"
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


#### named > invalid

```js
export bar2, { bar } from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "bar",
        "path": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > invalid

```js
import { foo, bar, baz } from "./named-trampoline"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./named-trampoline"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { baz } from "./broken-trampoline"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundDeep",
      "data": {
        "name": "baz",
        "deepPath": "broken-trampoline.js -> named-exports.js"
      },
      "type": "Identifier"
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


#### named > invalid

```js
const { baz } = require("./bar")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "options": [
    {
      "commonjs": true
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


#### named > invalid

```js
let { baz } = require("./bar")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "options": [
    {
      "commonjs": true
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


#### named > invalid

```js
const { baz: bar, bop } = require("./bar"), { a } = require("./re-export-names")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "./bar"
      },
      "type": "Identifier"
    },
    {
      "messageId": "notFound",
      "data": {
        "name": "bop",
        "path": "./bar"
      },
      "type": "Identifier"
    },
    {
      "messageId": "notFound",
      "data": {
        "name": "a",
        "path": "./re-export-names"
      },
      "type": "Identifier"
    }
  ],
  "options": [
    {
      "commonjs": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 3 errors but had 0: []

0 !== 3

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### named > invalid

```js
const { default: defExport } = require("./named-exports")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "default",
        "path": "./named-exports"
      },
      "type": "Identifier"
    }
  ],
  "options": [
    {
      "commonjs": true
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


#### named > invalid

```js
import  { type MyOpaqueType, MyMissingClass } from "./flowtypes"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "MyMissingClass",
        "path": "./flowtypes"
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### named > invalid

```js
/*jsnext*/ import { createSnorlax } from "redux"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": []
  },
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "createSnorlax",
        "path": "redux"
      },
      "type": "Identifier"
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


#### named > invalid

```js
/*jsnext*/ import { createSnorlax } from "redux"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "createSnorlax",
        "path": "redux"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { baz } from "es6-module"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "baz",
        "path": "es6-module"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { foo, bar, bap } from "./re-export-default"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "bap",
        "path": "./re-export-default"
      },
      "type": "Identifier"
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


#### named > invalid

```js
import { default as barDefault } from "./re-export"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "default",
        "path": "./re-export"
      },
      "type": "Identifier"
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


#### export * > named > invalid

```js
import { bar } from "./export-all"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFound",
      "data": {
        "name": "bar",
        "path": "./export-all"
      },
      "type": "Identifier"
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


### `namespace`

Pass: 24 / 101 (23.8%)
Fail: 68 / 101 (67.3%)
Skip: 9 / 101 (8.9%)

#### namespace > valid

```js
import * as names from "./named-exports"; console.log((names.b).c); 
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; console.log(names.a);
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./re-export-names"; console.log(names.foo);
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './re-export-names': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 42,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as elements from './jsx';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "sourceType": "module",
      "ecmaFeatures": {
        "jsx": true
      },
      "ecmaVersion": 2015
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './jsx': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 26,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js

      import * as foo from "./jsx/re-export.js";
      console.log(foo.jsxFoo);
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/extensions": [
      ".js",
      ".jsx"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './jsx/re-export.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 27,
    endLine: 2,
    endColumn: 47,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js

      import * as foo from "./jsx/bar/index.js";
      console.log(foo.Baz1);
      console.log(foo.Baz2);
      console.log(foo.Qux1);
      console.log(foo.Qux2);
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/extensions": [
      ".js",
      ".jsx"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './jsx/bar/index.js': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 27,
    endLine: 2,
    endColumn: 47,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; const { a } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; const { d: c } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js

      import * as names from "./named-exports";
      const { c } = foo,
        { length } = "names",
        alt = names;
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 29,
    endLine: 2,
    endColumn: 46,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; const { ExportedClass: { length } } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; function b(names) { const { c } = names }
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; function b() { let names = null; const { c } = names }
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from "./named-exports"; const x = function names() { const { c } = names }
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
export defport, * as names from "./named-exports"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > valid

```js
import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Users)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './issue-195/Endpoints': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 27,
    endLine: 1,
    endColumn: 50,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
function x() { console.log((names.b).c); } import * as names from "./named-exports"; 
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 66,
    endLine: 1,
    endColumn: 83,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from './default-export';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './default-export': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 41,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
export defport, * as names from "./default-export"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > valid

```js
import * as names from './named-exports'; console.log(names['a']);
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "allowComputed": true
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
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from './named-exports'; const {a, b, ...rest} = names;
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaVersion": 2018
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from './named-exports'; const {a, b, ...rest} = names;
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './named-exports': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as ns from './re-export-common'; const {foo} = ns;
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './re-export-common': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 40,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as Names from "./named-exports"; const Foo = <Names.a/>
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > valid

```js
export = function name() {}
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > valid

```js
import { foo } from "./issue-370-commonjs-namespace/bar"
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": [
      "foo"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './issue-370-commonjs-namespace/bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 56,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./commonjs-namespace/a"; a.b
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './commonjs-namespace/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js

    import * as color from './color';
    export const getBackgroundFromColor = (color) => color.bg;
    export const getExampleColor = () => color.example
    
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './color': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 27,
    endLine: 2,
    endColumn: 36,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js

    import * as middle from './middle';

    console.log(middle.myName);
  
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaVersion": 2020
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/export-star-2/downstream.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './middle': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 28,
    endLine: 2,
    endColumn: 38,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from './default-export-string';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaVersion": 2022
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './default-export-string': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 48,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as names from './default-export-string'; console.log(names.default)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaVersion": 2022
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './default-export-string': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 48,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep/a"; console.log(a.b.c.d.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import { b } from "./deep/a"; console.log(b.c.d.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
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


#### namespace > valid

```js
import * as a from "./deep/a"; console.log(a.b.c.d.e.f)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep/a"; var {b:{c:{d:{e}}}} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import { b } from "./deep/a"; var {c:{d:{e}}} = b
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
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


#### namespace > valid

```js
import * as a from "./deep/a"; console.log(a.b.default)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep-es7/a"; console.log(a.b.c.d.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import { b } from "./deep-es7/a"; console.log(b.c.d.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 32,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep-es7/a"; console.log(a.b.c.d.e.f)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep-es7/a"; var {b:{c:{d:{e}}}} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import { b } from "./deep-es7/a"; var {c:{d:{e}}} = b
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 32,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > valid

```js
import * as a from "./deep-es7/a"; console.log(a.b.default)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './deep-es7/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > invalid

```js
import * as names from './named-exports'; console.log(names.c)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as names from './named-exports'; console.log(names['a']);
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "computedReference",
      "data": {
        "namespace": "names"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'computedReference'
+ actual - expected

+ null
- 'computedReference'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as foo from './bar'; foo.foo = 'y';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "namespaceMember",
      "data": {
        "namespace": "foo"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'namespaceMember'
+ actual - expected

+ null
- 'namespaceMember'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as foo from './bar'; foo.x = 'y';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "namespaceMember",
      "data": {
        "namespace": "foo"
      }
    },
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "x",
        "namepath": "foo"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/namespace',
    message: "Parse errors in imported module './bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 28,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### namespace > invalid

```js
import * as names from "./named-exports"; const { c } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      },
      "type": "Property"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as names from "./named-exports"; function b() { const { c } = names }
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      },
      "type": "Property"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as names from "./named-exports"; const { c: d } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      },
      "type": "Property"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as names from "./named-exports"; const { c: { d } } = names
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      },
      "type": "Property"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Foo)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "Foo",
        "namepath": "Endpoints"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import b from './deep/default'; console.log(b.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "e",
        "namepath": "b"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
console.log(names.c); import * as names from './named-exports';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
function x() { console.log(names.c) } import * as names from './named-exports';
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "c",
        "namepath": "names"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as ree from "./re-export"; console.log(ree.default)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "default",
        "namepath": "ree"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as Names from "./named-exports"; const Foo = <Names.e/>
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      },
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "e",
        "namepath": "Names"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep/a"; console.log(a.b.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import { b } from "./deep/a"; console.log(b.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "e",
        "namepath": "b"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep/a"; console.log(a.b.c.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import { b } from "./deep/a"; console.log(b.c.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep/a"; var {b:{ e }} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep/a"; var {b:{c:{ e }}} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep-es7/a"; console.log(a.b.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import { b } from "./deep-es7/a"; console.log(b.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespace",
      "data": {
        "name": "e",
        "namepath": "b"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespace'
+ actual - expected

+ null
- 'notFoundInNamespace'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep-es7/a"; console.log(a.b.c.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import { b } from "./deep-es7/a"; console.log(b.c.e)
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep-es7/a"; var {b:{ e }} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### namespace > invalid

```js
import * as a from "./deep-es7/a"; var {b:{c:{ e }}} = a
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "env": {
        "es6": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "notFoundInNamespaceDeep",
      "data": {
        "name": "e",
        "namepath": "a.b.c"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'notFoundInNamespaceDeep'
+ actual - expected

+ null
- 'notFoundInNamespaceDeep'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-cycle`

Pass: 25 / 71 (35.2%)
Fail: 46 / 71 (64.8%)
Skip: 0 / 71 (0.0%)

#### no-cycle > valid

```js
import type { FooType } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-cycle > valid

```js
import type { FooType, BarType } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-cycle > invalid

```js
import { bar } from "./flow-types-some-type-imports"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "cycles/external/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
    }
  ],
  "settings": {
    "import-x/resolver": "webpack",
    "import-x/external-module-folders": [
      "cycles/external"
    ]
  },
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


#### no-cycle > invalid

```js
import { foo } from "./external-depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "cycles/external/depth-one:1"
      }
    }
  ],
  "settings": {
    "import-x/resolver": "webpack",
    "import-x/external-module-folders": [
      "cycles/external"
    ]
  },
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": 1
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
const { foo } = require("./es6/depth-one")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
require(["./es6/depth-one"], d1 => {})
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "amd": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
define(["./es6/depth-one"], d1 => {})
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "amd": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one-reexport"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": 2
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
const { foo } = require("./es6/depth-two")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
    }
  ],
  "options": [
    {
      "commonjs": true
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


#### no-cycle > invalid

```js
import { two } from "./es6/depth-three-star"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import one, { two, three } from "./es6/depth-three-star"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { bar } from "./es6/depth-three-indirect"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { bar } from "./es6/depth-three-indirect"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {}
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": null
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/no-cycle':
Options:
[
  {
    "maxDepth": null,
    "ignoreExternal": false,
    "allowUnsafeDynamicCyclicDependency": false
  }
]
Errors:
	Value null should be integer.
	Value null should be string.
	Value null should be equal to one of the allowed values.
	Value null should match some schema in anyOf.
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": "∞"
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "maxDepth": 1
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
const { foo } = require("./es6/depth-one")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
require(["./es6/depth-one"], d1 => {})
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "amd": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
define(["./es6/depth-one"], d1 => {})
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "amd": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one-reexport"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "maxDepth": 2
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
const { foo } = require("./es6/depth-two")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
    }
  ],
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "commonjs": true
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


#### no-cycle > invalid

```js
import { two } from "./es6/depth-three-star"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import one, { two, three } from "./es6/depth-three-star"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { bar } from "./es6/depth-three-indirect"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { bar } from "./es6/depth-three-indirect"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "maxDepth": null
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/no-cycle':
Options:
[
  {
    "allowUnsafeDynamicCyclicDependency": true,
    "maxDepth": null,
    "ignoreExternal": false
  }
]
Errors:
	Value null should be integer.
	Value null should be string.
	Value null should be equal to one of the allowed values.
	Value null should match some schema in anyOf.
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-two"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "allowUnsafeDynamicCyclicDependency": true,
      "maxDepth": "∞"
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import("./es6/depth-three-star")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import("./es6/depth-three-indirect")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-two:1=>./depth-one:1"
      }
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


#### no-cycle > invalid

```js
import("./es6/depth-two")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": null
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Options validation failed for rule 'rule-to-test/no-cycle':
Options:
[
  {
    "maxDepth": null,
    "ignoreExternal": false,
    "allowUnsafeDynamicCyclicDependency": false
  }
]
Errors:
	Value null should be integer.
	Value null should be string.
	Value null should be equal to one of the allowed values.
	Value null should match some schema in anyOf.
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### no-cycle > invalid

```js
import("./es6/depth-two")
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "options": [
    {
      "maxDepth": "∞"
    }
  ],
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./depth-one:1"
      }
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


#### no-cycle > invalid

```js
function bar(){ return import("./es6/depth-one"); } // #2265 5
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one-dynamic"; // #2265 6
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
function bar(){ return import("./es6/depth-one"); } // #2265 7
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { foo } from "./es6/depth-one-dynamic"; // #2265 8
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle"
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


#### no-cycle > invalid

```js
import { bar } from "./flow-types-depth-one"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./flow-types-depth-two:4=>./es6/depth-one:1"
      }
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


#### no-cycle > invalid

```js
import { foo } from "./intermediate-ignore"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycleSource",
      "data": {
        "source": "./ignore:1"
      },
      "line": 1
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


#### no-cycle > invalid

```js
import { foo } from "./ignore"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/cycles/depth-zero.js",
  "errors": [
    {
      "messageId": "cycle",
      "line": 1
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


### `no-default-export`

Pass: 25 / 30 (83.3%)
Fail: 4 / 30 (13.3%)
Skip: 1 / 30 (3.3%)

#### no-default-export > valid

```js
export type UserId = number;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-default-export > valid

```js
export foo from "foo.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-default-export > valid

```js
export Memory, { MemoryValue } from './Memory'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-default-export > invalid

```js
export default from "foo.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "type": "ExportNamedDeclaration",
      "messageId": "preferNamed"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-deprecated`

Pass: 20 / 55 (36.4%)
Fail: 35 / 55 (63.6%)
Skip: 0 / 55 (0.0%)

#### no-deprecated > valid

```js
import bar from './bar'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 23,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { fine } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 35,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { _undocumented } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 30,
    endLine: 1,
    endColumn: 44,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { fn } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 33,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { fine } from './tomdoc-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './tomdoc-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 42,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { _undocumented } from './tomdoc-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './tomdoc-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 30,
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


#### no-deprecated > valid

```js
import * as depd from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 36,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import * as depd from './deprecated'; console.log(depd.fine())
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 36,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { deepDep } from './deep-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deep-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { deepDep } from './deep-deprecated'; console.log(deepDep.fine())
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deep-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { deepDep } from './deep-deprecated'; function x(deepDep) { console.log(deepDep.MY_TERRIBLE_ACTION) }
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deep-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import { foo } from "./issue-370-commonjs-namespace/bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/ignore": [
      "foo"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './issue-370-commonjs-namespace/bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 56,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > valid

```js
import * as a from "./commonjs-namespace/a"; a.b
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './commonjs-namespace/a': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 43,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > invalid

```js
import { _deprecatedNoDescription } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecated"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecated'
+ actual - expected

+ null
- 'deprecated'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { fn } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please use 'x' instead."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import TerribleClass from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "This is awful, use NotAsBadClass."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { fn } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "jsdoc",
      "tomdoc"
    ]
  },
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please use 'x' instead."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { fn } from './tomdoc-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "This function is terrible."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import TerribleClass from './tomdoc-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "this is awful, use NotAsBadClass."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './tomdoc-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/docstyle": [
      "tomdoc"
    ]
  },
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'; function shadow(MY_TERRIBLE_ACTION) { console.log(MY_TERRIBLE_ACTION); }
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION, fine } from './deprecated'; console.log(fine)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "ImportSpecifier"
    },
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 35,
    endLine: 1,
    endColumn: 49,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(someOther.MY_TERRIBLE_ACTION)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION.whatever())
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "ImportSpecifier"
    },
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 35,
    endLine: 1,
    endColumn: 49,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > invalid

```js
import { MY_TERRIBLE_ACTION } from './deprecated'; console.log(MY_TERRIBLE_ACTION(this, is, the, worst))
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "ImportSpecifier"
    },
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 35,
    endLine: 1,
    endColumn: 49,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated > invalid

```js
import Thing from './deprecated-file'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "This module is the worst."
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import Thing from './deprecated-file'; console.log(other.Thing)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "This module is the worst."
      },
      "type": "ImportDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import * as depd from './deprecated'; console.log(depd.MY_TERRIBLE_ACTION)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import * as deep from './deep-deprecated'; console.log(deep.deepDep.MY_TERRIBLE_ACTION)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { deepDep } from './deep-deprecated'; console.log(deepDep.MY_TERRIBLE_ACTION)
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated > invalid

```js
import { deepDep } from './deep-deprecated'; function x(deepNDep) { console.log(deepDep.MY_TERRIBLE_ACTION) }
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'deprecatedDesc'
+ actual - expected

+ null
- 'deprecatedDesc'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-deprecated: hoisting > valid

```js
function x(deepDep) { console.log(deepDep.MY_TERRIBLE_ACTION) } import { deepDep } from './deep-deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated: hoisting',
    message: "Parse errors in imported module './deep-deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 88,
    endLine: 1,
    endColumn: 107,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-deprecated: hoisting > invalid

```js
console.log(MY_TERRIBLE_ACTION); import { MY_TERRIBLE_ACTION } from './deprecated'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "Identifier"
    },
    {
      "messageId": "deprecatedDesc",
      "data": {
        "description": "Please stop sending/handling this action type."
      },
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 1: [
  {
    ruleId: 'rule-to-test/no-deprecated: hoisting',
    message: "Parse errors in imported module './deprecated': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 68,
    endLine: 1,
    endColumn: 82,
    fixes: null,
    suggestions: null
  }
]

1 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-duplicates`

Pass: 47 / 79 (59.5%)
Fail: 29 / 79 (36.7%)
Skip: 3 / 79 (3.8%)

#### no-duplicates > valid

```js
import { x } from './foo'; import type { y } from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-duplicates > invalid

```js
import type { x } from './foo'; import type { y } from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "import type { x, y  } from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      }
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-duplicates > invalid

```js

import {
  DEFAULT_FILTER_KEYS,
  BULK_DISABLED,
} from '../constants';
import React from 'react';
import {
  BULK_ACTIONS_ENABLED
} from '../constants';

const TestComponent = () => {
  return <div>
  </div>;
}

export default TestComponent;
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "ecmaFeatures": {
        "jsx": true
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\nimport {\n  DEFAULT_FILTER_KEYS,\n  BULK_DISABLED,\n  BULK_ACTIONS_ENABLED\n\n} from '../constants';\nimport React from 'react';\n\nconst TestComponent = () => {\n  return <div>\n  </div>;\n}\n\nexport default TestComponent;\n      ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "../constants"
      }
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "../constants"
      }
    }
  ],
  "settings": {
    "import-x/extensions": [
      ".js",
      ".jsx",
      ".mjs",
      ".cjs"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type { x } from './foo'; import y from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type { x } from './foo'; import type * as y from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type x from './foo'; import type y from './bar'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type {x} from './foo'; import type {y} from './bar'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type x from './foo'; import type {y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js

        import type {} from './module';
        import {} from './module2';
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js

        import type { Identifier } from 'module';

        declare module 'module2' {
          import type { Identifier } from 'module';
        }

        declare module 'module3' {
          import type { Identifier } from 'module';
        }
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import { type x } from './foo'; import y from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import { type x } from './foo'; import { y } from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import { type x } from './foo'; import type y from 'foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type { A } from 'foo';import type B from 'foo';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import { type A } from 'foo';import type B from 'foo';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > valid

```js
import type A from 'foo';import { B } from 'foo';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import type x from './foo'; import type y from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 20
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 48
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import type x from './foo'; import type x from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "import type x from './foo'; ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 20
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 48
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import type {x} from './foo'; import type {y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "output": "import type {x,y} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 52
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {type x} from './foo'; import type {y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": false
    }
  ],
  "output": "import {type x,y} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 52
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import type {x} from 'foo'; import {type y} from 'foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "import {type x,type y} from 'foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 50
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {type x} from 'foo'; import type {y} from 'foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "import {type x,type y} from 'foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 50
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {type x} from 'foo'; import type {y} from 'foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "output": "import {type x,y} from 'foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "foo"
      },
      "line": 1,
      "column": 50
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {type x} from './foo'; import {type y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "import {type x,type y} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 52
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {type x} from './foo'; import {type y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "output": "import {type x,type y} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 52
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {AValue, type x, BValue} from './foo'; import {type y} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "output": "import {AValue, type x, BValue,type y} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 38
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 68
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import {AValue} from './foo'; import type {AType} from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "import {AValue,type AType} from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 22
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 56
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js
import type { AType as BType } from './foo'; import { CValue } from './foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "import { type AType as BType, CValue  } from './foo'; ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 37
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 1,
      "column": 69
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-duplicates > invalid

```js

            import {
              a
            } from './foo';
            import type {
              b,
              c,
            } from './foo';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "prefer-inline": true
    }
  ],
  "output": "\n            import {\n              a,\n              type b,\n              type c\n            } from './foo';\n            ",
  "errors": [
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 4,
      "column": 20
    },
    {
      "messageId": "duplicate",
      "data": {
        "module": "./foo"
      },
      "line": 8,
      "column": 20
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-empty-named-blocks`

Pass: 12 / 29 (41.4%)
Fail: 17 / 29 (58.6%)
Skip: 0 / 29 (0.0%)

#### no-empty-named-blocks > valid

```js
import type Default from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import type { Named } from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import type Default, { Named } from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import type * as Namespace from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import typeof Default from 'mod'; // babel old
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import typeof { Named } from 'mod'; // babel old
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > valid

```js
import typeof Default, { Named } from 'mod'; // babel old
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import type {} from 'mod';
```

```json
{
  "languageOptions": {},
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
        }
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import type {}from 'mod';
```

```json
{
  "languageOptions": {},
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
        }
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import type{}from 'mod';
```

```json
{
  "languageOptions": {},
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
        }
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import type {}from'mod';
```

```json
{
  "languageOptions": {},
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
        }
      ]
    }
  ]
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import type Default, {} from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "import type Default from 'mod';",
  "errors": [
    {
      "messageId": "emptyNamed"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import typeof {} from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
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

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import typeof {}from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
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

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import typeof {} from'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
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

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import typeof{}from'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "emptyNamed",
      "suggestions": [
        {
          "messageId": "unused",
          "output": ""
        },
        {
          "messageId": "emptyImport",
          "output": "import 'mod';"
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

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-empty-named-blocks > invalid

```js
import typeof Default, {} from 'mod';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "import typeof Default from 'mod';",
  "errors": [
    {
      "messageId": "emptyNamed"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-extraneous-dependencies`

Pass: 122 / 128 (95.3%)
Fail: 6 / 128 (4.7%)
Skip: 0 / 128 (0.0%)

#### no-extraneous-dependencies > valid

```js
import type MyType from "myflowtyped";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-flow-typed"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-extraneous-dependencies > valid

```js

        // @flow
        import typeof TypeScriptModule from 'typescript';
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-flow-typed"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-extraneous-dependencies > valid

```js
import type T from "a";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-typescript-dev-dependencies",
      "devDependencies": false
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": [
      "node",
      "typescript"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-extraneous-dependencies > valid

```js
import type { T } from "a"; export type { T };
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-typescript-dev-dependencies",
      "devDependencies": false
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": [
      "node",
      "typescript"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-extraneous-dependencies > valid

```js
export type { T } from "a";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-typescript-dev-dependencies",
      "devDependencies": false
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": [
      "node",
      "typescript"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-extraneous-dependencies > invalid

```js
import type T from "a";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "packageDir": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/with-typescript-dev-dependencies",
      "devDependencies": false,
      "includeTypes": true
    }
  ],
  "errors": [
    {
      "messageId": "devDep",
      "data": {
        "packageName": "a"
      }
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": [
      "node",
      "typescript"
    ]
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-internal-modules`

Pass: 45 / 46 (97.8%)
Fail: 1 / 46 (2.2%)
Skip: 0 / 46 (0.0%)

#### no-internal-modules > valid

```js

        export class AuthHelper {

          public static checkAuth(auth?: string): boolean {
          }
        }
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-mutable-exports`

Pass: 26 / 30 (86.7%)
Fail: 2 / 30 (6.7%)
Skip: 2 / 30 (6.7%)

#### no-mutable-exports > valid

```js
export Something from "./something";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-mutable-exports > valid

```js
type Foo = {}
export type {Foo}
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-named-as-default-member`

Pass: 20 / 29 (69.0%)
Fail: 7 / 29 (24.1%)
Skip: 2 / 29 (6.9%)

#### no-named-as-default-member > valid

```js
import bar, {foo} from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-named-as-default-member',
    message: "Parse errors in imported module './bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 30,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-named-as-default-member > valid

```js
import bar from "./bar"; const baz = bar.baz
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-named-as-default-member',
    message: "Parse errors in imported module './bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
    endLine: 1,
    endColumn: 23,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-named-as-default-member > valid

```js
import foo from "./default-export-default-property"; const a = foo.default
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/no-named-as-default-member',
    message: "Parse errors in imported module './default-export-default-property': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 16,
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


#### no-named-as-default-member > invalid

```js
import bar from "./bar"; const foo = bar.foo;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "member",
      "data": {
        "objectName": "bar",
        "propName": "foo",
        "sourcePath": "./bar"
      },
      "type": "MemberExpression"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'member'

null !== 'member'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default-member > invalid

```js
import bar from "./bar"; bar.foo();
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "member",
      "data": {
        "objectName": "bar",
        "propName": "foo",
        "sourcePath": "./bar"
      },
      "type": "MemberExpression"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'member'

null !== 'member'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default-member > invalid

```js
import bar from "./bar"; const {foo} = bar;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "member",
      "data": {
        "objectName": "bar",
        "propName": "foo",
        "sourcePath": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'member'

null !== 'member'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default-member > invalid

```js
import bar from "./bar"; const {foo: foo2, baz} = bar;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "member",
      "data": {
        "objectName": "bar",
        "propName": "foo",
        "sourcePath": "./bar"
      },
      "type": "Identifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'member'

null !== 'member'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-named-as-default`

Pass: 21 / 34 (61.8%)
Fail: 8 / 34 (23.5%)
Skip: 5 / 34 (14.7%)

#### no-named-as-default > valid

```js
export bar, { foo } from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > valid

```js
export bar from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > valid

```js
export default from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > invalid

```js
import foo from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "default",
      "data": {
        "name": "foo"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'default'

null !== 'default'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > invalid

```js
import foo, { foo as bar } from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "default",
      "data": {
        "name": "foo"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'default'

null !== 'default'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > invalid

```js
export foo from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "default",
      "data": {
        "name": "foo"
      },
      "type": "ExportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > invalid

```js
export foo, { foo as bar } from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "default",
      "data": {
        "name": "foo"
      },
      "type": "ExportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-as-default > invalid

```js
import importX from 'eslint-plugin-import-x';
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "default",
      "data": {
        "name": "importX"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'default'

null !== 'default'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-named-default`

Pass: 22 / 25 (88.0%)
Fail: 2 / 25 (8.0%)
Skip: 1 / 25 (4.0%)

#### no-named-default > valid

```js
import { type default as Foo } from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-default > valid

```js
import { typeof default as Foo } from "./bar";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-named-export`

Pass: 21 / 26 (80.8%)
Fail: 4 / 26 (15.4%)
Skip: 1 / 26 (3.8%)

#### no-named-export > valid

```js
export default from "foo.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-export > invalid

```js
export type UserId = number;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noAllowed",
      "type": "ExportNamedDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-export > invalid

```js
export foo from "foo.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noAllowed",
      "type": "ExportNamedDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-named-export > invalid

```js
export Memory, { MemoryValue } from './Memory'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "noAllowed",
      "type": "ExportNamedDeclaration"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-rename-default`

Pass: 32 / 116 (27.6%)
Fail: 84 / 116 (72.4%)
Skip: 0 / 116 (0.0%)

#### no-rename-default > valid

```js
const _ = require('./no-rename-default/anonymous-arrow')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/anonymous-arrow': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 55,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const _ = require('./no-rename-default/anonymous-arrow-async')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/anonymous-arrow-async': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 61,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const _ = require('./no-rename-default/anonymous-class')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/anonymous-class': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 55,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const _ = require('./no-rename-default/anonymous-object')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/anonymous-object': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 56,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const _ = require('./no-rename-default/anonymous-primitive')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/anonymous-primitive': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 59,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
import myArrow from './no-rename-default/assign-arrow'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-arrow.js",
        "defaultExportName": "arrow",
        "requiresOrImports": "imports",
        "importName": "myArrow",
        "suggestion": "import arrow from './no-rename-default/assign-arrow'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myArrowAsync from './no-rename-default/assign-arrow-async'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-arrow-async.js",
        "defaultExportName": "arrowAsync",
        "requiresOrImports": "imports",
        "importName": "myArrowAsync",
        "suggestion": "import arrowAsync from './no-rename-default/assign-arrow-async'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import MyUser from './no-rename-default/assign-class'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-class.js",
        "defaultExportName": "User",
        "requiresOrImports": "imports",
        "importName": "MyUser",
        "suggestion": "import User from './no-rename-default/assign-class'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import MyUser from './no-rename-default/assign-class-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-class-named.js",
        "defaultExportName": "User",
        "requiresOrImports": "imports",
        "importName": "MyUser",
        "suggestion": "import User from './no-rename-default/assign-class-named'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myFn from './no-rename-default/assign-fn'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-fn.js",
        "defaultExportName": "fn",
        "requiresOrImports": "imports",
        "importName": "myFn",
        "suggestion": "import fn from './no-rename-default/assign-fn'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myFn from './no-rename-default/assign-fn-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-fn-named.js",
        "defaultExportName": "fn",
        "requiresOrImports": "imports",
        "importName": "myFn",
        "suggestion": "import fn from './no-rename-default/assign-fn-named'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myGenerator from './no-rename-default/assign-generator'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-generator.js",
        "defaultExportName": "generator",
        "requiresOrImports": "imports",
        "importName": "myGenerator",
        "suggestion": "import generator from './no-rename-default/assign-generator'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myGenerator from './no-rename-default/assign-generator-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-generator-named.js",
        "defaultExportName": "generator",
        "requiresOrImports": "imports",
        "importName": "myGenerator",
        "suggestion": "import generator from './no-rename-default/assign-generator-named'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > valid

```js
import myArrow from './no-rename-default/assign-arrow'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 54,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import myArrowAsync from './no-rename-default/assign-arrow-async'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow-async': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 65,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import MyUser from './no-rename-default/assign-class'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-class': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 53,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import MyUser from './no-rename-default/assign-class-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-class-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 59,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import myFn from './no-rename-default/assign-fn'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 17,
    endLine: 1,
    endColumn: 48,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import myFn from './no-rename-default/assign-fn-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 17,
    endLine: 1,
    endColumn: 54,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import myGenerator from './no-rename-default/assign-generator'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 62,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import myGenerator from './no-rename-default/assign-generator-named'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 68,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const arrow = require('./no-rename-default/assign-arrow')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 22,
    endLine: 1,
    endColumn: 56,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const arrowAsync = require('./no-rename-default/assign-arrow-async')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow-async': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 27,
    endLine: 1,
    endColumn: 67,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const User = require('./no-rename-default/assign-class')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-class': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 55,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const User = require('./no-rename-default/assign-class-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-class-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 61,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const fn = require('./no-rename-default/assign-fn')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 50,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const fn = require('./no-rename-default/assign-fn-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 19,
    endLine: 1,
    endColumn: 56,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const generator = require('./no-rename-default/assign-generator')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 26,
    endLine: 1,
    endColumn: 64,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const generator = require('./no-rename-default/assign-generator-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 26,
    endLine: 1,
    endColumn: 70,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
const myArrow = require('./no-rename-default/assign-arrow')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-arrow.js",
        "defaultExportName": "arrow",
        "requiresOrImports": "requires",
        "importName": "myArrow",
        "suggestion": "const arrow = require('./no-rename-default/assign-arrow')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const myArrowAsync = require('./no-rename-default/assign-arrow-async')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-arrow-async.js",
        "defaultExportName": "arrowAsync",
        "requiresOrImports": "requires",
        "importName": "myArrowAsync",
        "suggestion": "const arrowAsync = require('./no-rename-default/assign-arrow-async')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const MyUser = require('./no-rename-default/assign-class')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-class.js",
        "defaultExportName": "User",
        "requiresOrImports": "requires",
        "importName": "MyUser",
        "suggestion": "const User = require('./no-rename-default/assign-class')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const MyUser = require('./no-rename-default/assign-class-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-class-named.js",
        "defaultExportName": "User",
        "requiresOrImports": "requires",
        "importName": "MyUser",
        "suggestion": "const User = require('./no-rename-default/assign-class-named')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const myFn = require('./no-rename-default/assign-fn')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-fn.js",
        "defaultExportName": "fn",
        "requiresOrImports": "requires",
        "importName": "myFn",
        "suggestion": "const fn = require('./no-rename-default/assign-fn')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const myFn = require('./no-rename-default/assign-fn-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-fn-named.js",
        "defaultExportName": "fn",
        "requiresOrImports": "requires",
        "importName": "myFn",
        "suggestion": "const fn = require('./no-rename-default/assign-fn-named')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const myGenerator = require('./no-rename-default/assign-generator')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-generator.js",
        "defaultExportName": "generator",
        "requiresOrImports": "requires",
        "importName": "myGenerator",
        "suggestion": "const generator = require('./no-rename-default/assign-generator')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const myGenerator = require('./no-rename-default/assign-generator-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "assign-generator-named.js",
        "defaultExportName": "generator",
        "requiresOrImports": "requires",
        "importName": "myGenerator",
        "suggestion": "const generator = require('./no-rename-default/assign-generator-named')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-renamed-default > valid

```js
const myArrow = require('./no-rename-default/assign-arrow')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 24,
    endLine: 1,
    endColumn: 58,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const myArrowAsync = require('./no-rename-default/assign-arrow-async')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-arrow-async': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 29,
    endLine: 1,
    endColumn: 69,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const MyUser = require('./no-rename-default/assign-class')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-class': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 57,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const MyUser = require('./no-rename-default/assign-class-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-class-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
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


#### no-renamed-default > valid

```js
const myFn = require('./no-rename-default/assign-fn')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 52,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const myFn = require('./no-rename-default/assign-fn-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-fn-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 58,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const myGenerator = require('./no-rename-default/assign-generator')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 28,
    endLine: 1,
    endColumn: 66,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-renamed-default > valid

```js
const myGenerator = require('./no-rename-default/assign-generator-named')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-renamed-default',
    message: "Parse errors in imported module './no-rename-default/assign-generator-named': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 28,
    endLine: 1,
    endColumn: 72,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
import MyUser from './no-rename-default/class-user'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "class-user.js",
        "defaultExportName": "User",
        "requiresOrImports": "imports",
        "importName": "MyUser",
        "suggestion": "import User from './no-rename-default/class-user'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > valid

```js
const User = require('./no-rename-default/class-user')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/class-user': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 21,
    endLine: 1,
    endColumn: 53,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
const MyUser = require('./no-rename-default/class-user')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "class-user.js",
        "defaultExportName": "User",
        "requiresOrImports": "requires",
        "importName": "MyUser",
        "suggestion": "const User = require('./no-rename-default/class-user')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import bar from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import foo from './no-rename-default/const-foo'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import { default as bar } from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import { default as foo } from './no-rename-default/const-foo'"
      },
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import { default as bar, fooNamed1 } from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import { default as foo } from './no-rename-default/const-foo'"
      },
      "type": "ImportSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 2: [
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 42,
    endLine: 1,
    endColumn: 73,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 42,
    endLine: 1,
    endColumn: 73,
    fixes: null,
    suggestions: null
  }
]

2 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
import bar, { fooNamed1 } from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import foo from './no-rename-default/const-foo'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 1 error but had 2: [
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 31,
    endLine: 1,
    endColumn: 62,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 31,
    endLine: 1,
    endColumn: 62,
    fixes: null,
    suggestions: null
  }
]

2 !== 1

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
import foo from './no-rename-default/const-bar'
        import bar from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-bar.js",
        "defaultExportName": "bar",
        "requiresOrImports": "imports",
        "importName": "foo",
        "suggestion": "import bar from './no-rename-default/const-bar'"
      },
      "type": "ImportDefaultSpecifier"
    },
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import foo from './no-rename-default/const-foo'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import findUsers from './no-rename-default/fn-get-users'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users.js",
        "defaultExportName": "getUsers",
        "requiresOrImports": "imports",
        "importName": "findUsers",
        "suggestion": "import getUsers from './no-rename-default/fn-get-users'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import findUsersSync from './no-rename-default/fn-get-users-sync'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users-sync.js",
        "defaultExportName": "getUsersSync",
        "requiresOrImports": "imports",
        "importName": "findUsersSync",
        "suggestion": "import getUsersSync from './no-rename-default/fn-get-users-sync'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import foo, { barNamed1 } from './no-rename-default/const-bar'
        import bar, { fooNamed1 } from './no-rename-default/const-foo'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-bar.js",
        "defaultExportName": "bar",
        "requiresOrImports": "imports",
        "importName": "foo",
        "suggestion": "import bar from './no-rename-default/const-bar'"
      },
      "type": "ImportDefaultSpecifier"
    },
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "imports",
        "importName": "bar",
        "suggestion": "import foo from './no-rename-default/const-foo'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have 2 errors but had 4: [
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 31,
    endLine: 1,
    endColumn: 62,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 31,
    endLine: 1,
    endColumn: 62,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 39,
    endLine: 2,
    endColumn: 70,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 39,
    endLine: 2,
    endColumn: 70,
    fixes: null,
    suggestions: null
  }
]

4 !== 2

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const foo = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
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


#### no-rename-default > valid

```js
const { default: foo } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 33,
    endLine: 1,
    endColumn: 64,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const { default: foo, fooNamed1 } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 44,
    endLine: 1,
    endColumn: 75,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const foo = require('./no-rename-default/const-foo')
        const { fooNamed1 } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
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


#### no-rename-default > valid

```js
const getUsers = require('./no-rename-default/fn-get-users')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/fn-get-users': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 59,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const getUsersSync = require('./no-rename-default/fn-get-users-sync')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/fn-get-users-sync': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 29,
    endLine: 1,
    endColumn: 68,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js

        const bar = require('./no-rename-default/const-bar')
        const { barNamed1 } = require('./no-rename-default/const-bar')
        const foo = require('./no-rename-default/const-foo')
        const { fooNamed1 } = require('./no-rename-default/const-foo')
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 2: [
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-bar': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 2,
    column: 28,
    endLine: 2,
    endColumn: 59,
    fixes: null,
    suggestions: null
  },
  {
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/const-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 4,
    column: 28,
    endLine: 4,
    endColumn: 59,
    fixes: null,
    suggestions: null
  }
]

2 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
const bar = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const foo = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const bar = require('./no-rename-default/const-foo')
        const { fooNamed1 } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const foo = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const { default: bar } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const { default: foo } = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const { default: bar, fooNamed1 } = require('./no-rename-default/const-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const { default: foo } = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js

        const foo = require('./no-rename-default/const-bar')
        const bar = require('./no-rename-default/const-foo')
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-bar.js",
        "defaultExportName": "bar",
        "requiresOrImports": "requires",
        "importName": "foo",
        "suggestion": "const bar = require('./no-rename-default/const-bar')"
      },
      "type": "VariableDeclarator"
    },
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const foo = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js

        const foo = require('./no-rename-default/const-bar')
        const { barNamed1 } = require('./no-rename-default/const-bar')
        const bar = require('./no-rename-default/const-foo')
        const { fooNamed1 } = require('./no-rename-default/const-foo')
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-bar.js",
        "defaultExportName": "bar",
        "requiresOrImports": "requires",
        "importName": "foo",
        "suggestion": "const bar = require('./no-rename-default/const-bar')"
      },
      "type": "VariableDeclarator"
    },
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "const-foo.js",
        "defaultExportName": "foo",
        "requiresOrImports": "requires",
        "importName": "bar",
        "suggestion": "const foo = require('./no-rename-default/const-foo')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import findUsers from './no-rename-default/fn-get-users'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users.js",
        "defaultExportName": "getUsers",
        "requiresOrImports": "imports",
        "importName": "findUsers",
        "suggestion": "import getUsers from './no-rename-default/fn-get-users'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import findUsersSync from './no-rename-default/fn-get-users-sync'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users-sync.js",
        "defaultExportName": "getUsersSync",
        "requiresOrImports": "imports",
        "importName": "findUsersSync",
        "suggestion": "import getUsersSync from './no-rename-default/fn-get-users-sync'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > valid

```js
const getUsers = require('./no-rename-default/fn-get-users')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/fn-get-users': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 59,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const getUsersSync = require('./no-rename-default/fn-get-users-sync')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/fn-get-users-sync': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 29,
    endLine: 1,
    endColumn: 68,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
const findUsers = require('./no-rename-default/fn-get-users')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users.js",
        "defaultExportName": "getUsers",
        "requiresOrImports": "requires",
        "importName": "findUsers",
        "suggestion": "const getUsers = require('./no-rename-default/fn-get-users')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
const findUsersSync = require('./no-rename-default/fn-get-users-sync')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "fn-get-users-sync.js",
        "defaultExportName": "getUsersSync",
        "requiresOrImports": "requires",
        "importName": "findUsersSync",
        "suggestion": "const getUsersSync = require('./no-rename-default/fn-get-users-sync')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > invalid

```js
import myReader from './no-rename-default/generator-reader'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "generator-reader.js",
        "defaultExportName": "reader",
        "requiresOrImports": "imports",
        "importName": "myReader",
        "suggestion": "import reader from './no-rename-default/generator-reader'"
      },
      "type": "ImportDefaultSpecifier"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > valid

```js
const reader = require('./no-rename-default/generator-reader')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/generator-reader': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 23,
    endLine: 1,
    endColumn: 61,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > invalid

```js
const myReader = require('./no-rename-default/generator-reader')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
    }
  ],
  "errors": [
    {
      "messageId": "renameDefault",
      "data": {
        "importBasename": "generator-reader.js",
        "defaultExportName": "reader",
        "requiresOrImports": "requires",
        "importName": "myReader",
        "suggestion": "const reader = require('./no-rename-default/generator-reader')"
      },
      "type": "VariableDeclarator"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: messageId 'null' does not match expected messageId 'renameDefault'
+ actual - expected

+ null
- 'renameDefault'

    at assertMessageIdIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCaseMessageIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-rename-default > valid

```js
const foo = require('./no-rename-default/pr-3006-feedback/binding-const-rename-fn')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-const-rename-fn': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 82,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const foo = require('./no-rename-default/pr-3006-feedback/binding-hoc-with-logger-for-foo')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-hoc-with-logger-for-foo': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 20,
    endLine: 1,
    endColumn: 90,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const getUsers = require('./no-rename-default/pr-3006-feedback/binding-hoc-with-logger-for-get-users')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-hoc-with-logger-for-get-users': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 101,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const getUsers = require('./no-rename-default/pr-3006-feedback/binding-hoc-with-logger-with-auth-for-get-users')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-hoc-with-logger-with-auth-for-get-users': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 25,
    endLine: 1,
    endColumn: 111,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
import _ from './no-rename-default/pr-3006-feedback/binding-fn-rename'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-fn-rename': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 14,
    endLine: 1,
    endColumn: 70,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### no-rename-default > valid

```js
const _ = require('./no-rename-default/pr-3006-feedback/binding-fn-rename')
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "commonjs": true,
      "preventRenamingBindings": false
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
    ruleId: 'rule-to-test/no-rename-default',
    message: "Parse errors in imported module './no-rename-default/pr-3006-feedback/binding-fn-rename': `context.languageOptions.parser.parse` not implemented yet. (undefined:undefined)",
    messageId: null,
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 18,
    endLine: 1,
    endColumn: 74,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


### `no-restricted-paths`

Pass: 55 / 59 (93.2%)
Fail: 4 / 59 (6.8%)
Skip: 0 / 59 (0.0%)

#### Typescript > no-restricted-paths > valid

```js
import type b from "../server/b.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Typescript > no-restricted-paths > valid

```js
import type * as b from "../server/b.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Typescript > no-restricted-paths > invalid

```js
import type b from "../server/b"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/restricted-paths/client/a",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "zones": [
        {
          "target": "./client",
          "from": "./server"
        }
      ],
      "basePath": "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/test/fixtures/restricted-paths"
    }
  ],
  "errors": [
    {
      "messageId": "zone",
      "data": {
        "importPath": "../server/b",
        "extra": ""
      },
      "line": 1,
      "column": 20
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### Typescript > no-restricted-paths > invalid

```js
import type b from "../two/a"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/restricted-paths/server/one/a",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "zones": [
        {
          "target": "./test/fixtures/restricted-paths/server/one",
          "from": "./test/fixtures/restricted-paths/server",
          "except": [
            "./one"
          ],
          "message": "Custom message"
        }
      ]
    }
  ],
  "errors": [
    {
      "messageId": "zone",
      "data": {
        "importPath": "../two/a",
        "extra": " Custom message"
      },
      "line": 1,
      "column": 20
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-unresolved`

Pass: 141 / 152 (92.8%)
Fail: 9 / 152 (5.9%)
Skip: 2 / 152 (1.3%)

#### no-unresolved (node) > valid

```js
export bar from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/resolver": "node",
    "import-x/cache": {
      "lifetime": 0
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-unresolved (node) > invalid

```js
export bar from "./does-not-exist"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/resolver": "node",
    "import-x/cache": {
      "lifetime": 0
    }
  },
  "errors": [
    {
      "messageId": "unresolved",
      "data": {
        "module": "./does-not-exist"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-unresolved (webpack) > valid

```js
export bar from "./bar"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/resolver": "webpack",
    "import-x/cache": {
      "lifetime": 0
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-unresolved (webpack) > invalid

```js
export bar from "./does-not-exist"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/resolver": "webpack",
    "import-x/cache": {
      "lifetime": 0
    }
  },
  "errors": [
    {
      "messageId": "unresolved",
      "data": {
        "module": "./does-not-exist"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### no-unresolved ignore list > valid

```js
import "./test.GIF"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "ignore": [
        {},
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

Error: Options validation failed for rule 'rule-to-test/no-unresolved ignore list':
Options:
[
  {
    "ignore": [
      {},
      {}
    ],
    "caseSensitive": true
  }
]
Errors:
	Value [{},{}] should NOT have duplicate items (items ## 0 and 1 are identical).
    at <anonymous> (apps/oxlint/dist/lint.js)
    at processOptions (apps/oxlint/dist/lint.js)
    at setOptions (apps/oxlint/dist/lint.js)
    at setupOptions (apps/oxlint/dist/plugins-dev.js)


#### no-unresolved ignore list > invalid

```js
import "./test.gif"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "ignore": [
        {}
      ]
    }
  ],
  "errors": [
    {
      "messageId": "unresolved",
      "data": {
        "module": "./test.gif"
      }
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


#### no-unresolved ignore list > invalid

```js
import "./test.png"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "ignore": [
        {}
      ]
    }
  ],
  "errors": [
    {
      "messageId": "unresolved",
      "data": {
        "module": "./test.png"
      }
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


#### TypeScript > no-unresolved (ignore type-only) > valid

```js
import type { JSONSchema7Type } from "@types/json-schema";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > no-unresolved (ignore type-only) > valid

```js
export type { JSONSchema7Type } from "@types/json-schema";
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `no-useless-path-segments`

Pass: 88 / 106 (83.0%)
Fail: 8 / 106 (7.5%)
Skip: 10 / 106 (9.4%)

#### no-useless-path-segments (node) > invalid

```js
require("./../fixtures/malformed.js")
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (node) > invalid

```js
require("./../fixtures/malformed")
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (node) > invalid

```js
import "./../fixtures/malformed.js"
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (node) > invalid

```js
import "./../fixtures/malformed"
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (webpack) > invalid

```js
require("./../fixtures/malformed.js")
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (webpack) > invalid

```js
require("./../fixtures/malformed")
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (webpack) > invalid

```js
import "./../fixtures/malformed.js"
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### no-useless-path-segments (webpack) > invalid

```js
import "./../fixtures/malformed"
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `order`

Pass: 182 / 283 (64.3%)
Fail: 92 / 283 (32.5%)
Skip: 9 / 283 (3.2%)

#### order > valid

```js

        import { a } from "./a";
        export namespace SomeNamespace {
            export import a2 = a;
        }
      
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order > invalid

```js

    var sibling = require('./sibling');
    var async = require('async');
    var fs = require('fs');
  
```

```json
{
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": [
    "\n    var async = require('async');\n    var sibling = require('./sibling');\n    var fs = require('fs');\n  ",
    "\n    var fs = require('fs');\n    var async = require('async');\n    var sibling = require('./sibling');\n  "
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`async` import",
        "order": "before",
        "firstImport": "import of `./sibling`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`fs` import",
        "order": "before",
        "firstImport": "import of `./sibling`"
      }
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### order > invalid

```js

        import path from 'path'; /* 1
        2 */
        import _ from 'lodash';
      
```

```json
{
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": [
    "\n        import path from 'path';\n /* 1\n        2 */\n        import _ from 'lodash';\n      ",
    "\n        import path from 'path';\n\n /* 1\n        2 */\n        import _ from 'lodash';\n      "
  ],
  "options": [
    {
      "newlines-between": "always"
    }
  ],
  "errors": [
    {
      "messageId": "oneLineBetweenGroups",
      "line": 2
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### order > invalid

```js

          const { cello } = require('./cello');
          import { int } from './int';
          const blah = require('./blah');
          import { hello } from './hello';
        
```

```json
{
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": [
    "\n          import { int } from './int';\n          const { cello } = require('./cello');\n          const blah = require('./blah');\n          import { hello } from './hello';\n        ",
    "\n          import { int } from './int';\n          import { hello } from './hello';\n          const { cello } = require('./cello');\n          const blah = require('./blah');\n        "
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`./int` import",
        "order": "before",
        "firstImport": "import of `./cello`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`./hello` import",
        "order": "before",
        "firstImport": "import of `./cello`"
      }
    }
  ]
}
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import type { C } from 'Bar';
              import b from 'bar';
              import a from 'foo';
              import type { A } from 'foo';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import a from 'foo';
              import type { A } from 'foo';
              import b from 'bar';
              import c from 'Bar';
              import type { C } from 'Bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import b from 'bar';
              import a from 'foo';

              import index from './';

              import type { C } from 'Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import a from 'foo';

              import b from 'dirA/bar';

              import index from './';

              import type { C } from 'dirA/Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [
        "type"
      ]
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import type { A } from 'foo';
              import a from 'foo';

              import type { C } from 'dirA/Bar';
              import b from 'dirA/bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import a from 'foo';
              import b from 'bar';
              import c from 'Bar';

              import index from './';

              import type { A } from 'foo';
              import type { C } from 'Bar';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import { serialize, parse, mapFieldErrors } from '@vtaits/form-schema';
              import type { GetFieldSchema } from '@vtaits/form-schema';
              import { useMemo, useCallback } from 'react';
              import type { ReactElement, ReactNode } from 'react';
              import { Form } from 'react-final-form';
              import type { FormProps as FinalFormProps } from 'react-final-form';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import type { CopyOptions } from 'fs';
              import type { ParsedPath } from 'path';

              declare module 'my-module' {
                import type { CopyOptions } from 'fs';
                import type { ParsedPath } from 'path';
              }
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": false
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [
        "type"
      ],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import type { AA } from 'abc';
            import a from 'foo';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import type { AA } from 'abc';
            import a from 'foo';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';
            import type { AA } from 'abc';
            import type { A } from 'foo';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';
            import type { AA } from 'abc';

            import type { A } from 'foo';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "ignore",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA } from 'abc';

            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';

            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "alphabetize": {
        "order": "asc",
        "caseInsensitive": true
      },
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";

            import c from "../foo.js";

            import d from "./bar.js";

            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always",
      "newlines-between-types": "ignore"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";

            import type C from "../foo.js";

            import type D from "./bar.js";

            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "sortTypesGroup": true,
      "newlines-between": "never",
      "newlines-between-types": "always"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import fs from 'fs';

            import '@scoped/package';
            import type { B } from 'fs';

            import type { A1 } from '/bad/bad/bad/bad';
            import './a/b/c';
            import type { A2 } from '/bad/bad/bad/bad';
            import type { A3 } from '/bad/bad/bad/bad';
            import type { D1 } from '/bad/bad/not/good';
            import type { D2 } from '/bad/bad/not/good';
            import type { D3 } from '/bad/bad/not/good';

            import type { C } from '@something/else';

            import type { E } from './index.js';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "builtin",
        "type",
        "unknown",
        "external"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "never"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import makeVanillaYargs from 'yargs/yargs';

            import { createDebugLogger } from 'multiverse+rejoinder';

            import { globalDebuggerNamespace } from 'rootverse+bfe:src/constant.ts';
            import { ErrorMessage, type KeyValueEntry } from 'rootverse+bfe:src/error.ts';

            import {
              $artificiallyInvoked,
              $canonical,
              $exists,
              $genesis
            } from 'rootverse+bfe:src/symbols.ts';

            import type {
              Entries,
              LiteralUnion,
              OmitIndexSignature,
              Promisable,
              StringKeyOf
            } from 'type-fest';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "multiverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "universe{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import makeVanillaYargs from 'yargs/yargs';
            import { createDebugLogger } from 'multiverse+rejoinder';
            import { globalDebuggerNamespace } from 'rootverse+bfe:src/constant.ts';
            import { ErrorMessage, type KeyValueEntry } from 'rootverse+bfe:src/error.ts';
            import { $artificiallyInvoked } from 'rootverse+bfe:src/symbols.ts';

            import type {
              Entries,
              LiteralUnion,
              OmitIndexSignature,
              Promisable,
              StringKeyOf
            } from 'type-fest';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "multiverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "universe{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import assert from 'assert';
            import { isNativeError } from 'util/types';

            import { runNoRejectOnBadExit } from '@-xun/run';
            import { TrialError } from 'named-app-errors';
            import { resolve as resolverLibrary } from 'resolve.exports';

            import { toAbsolutePath, type AbsolutePath } from 'rootverse+project-utils:src/fs.ts';

            import type { PackageJson } from 'type-fest';
            // Some comment about remembering to do something
            import type { XPackageJson } from 'rootverse:src/assets/config/_package.json.ts';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      },
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";

            import c from "../foo.js";

            import d from "./bar.js";

            import e from "./";

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always",
      "newlines-between-types": "ignore"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";

            import type A from "fs";
            import type B from "path";

            import type C from "../foo.js";

            import type D from "./bar.js";

            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true,
      "newlines-between": "never",
      "newlines-between-types": "always"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';
            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';
            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';
            import type { F } from './index.js';
            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';
            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';
            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';
            import type { F } from './index.js';
            import type { G } from './aaa.js';
            import type { H } from './bbb';

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "type",
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import b from 'bar';
              import c from 'Bar';
              import type { C } from 'Bar';
              import a from 'foo';
              import type { A } from 'foo';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import c from 'Bar';\n              import type { C } from 'Bar';\n              import b from 'bar';\n              import a from 'foo';\n              import type { A } from 'foo';\n\n              import index from './';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "after",
        "firstImport": "type import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import a from 'foo';
              import type { A } from 'foo';
              import c from 'Bar';
              import type { C } from 'Bar';
              import b from 'bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import a from 'foo';\n              import type { A } from 'foo';\n              import b from 'bar';\n              import c from 'Bar';\n              import type { C } from 'Bar';\n\n              import index from './';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "before",
        "firstImport": "import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import b from 'bar';
              import c from 'Bar';
              import a from 'foo';

              import index from './';

              import type { A } from 'foo';
              import type { C } from 'Bar';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import c from 'Bar';\n              import b from 'bar';\n              import a from 'foo';\n\n              import index from './';\n\n              import type { C } from 'Bar';\n              import type { A } from 'foo';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`Bar` import",
        "order": "before",
        "firstImport": "import of `bar`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`Bar` type import",
        "order": "before",
        "firstImport": "type import of `foo`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import a from 'foo';
              import c from 'Bar';
              import b from 'bar';

              import index from './';

              import type { C } from 'Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import a from 'foo';\n              import b from 'bar';\n              import c from 'Bar';\n\n              import index from './';\n\n              import type { A } from 'foo';\n              import type { C } from 'Bar';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "before",
        "firstImport": "import of `Bar`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`foo` type import",
        "order": "before",
        "firstImport": "type import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import type { ParsedPath } from 'path';
              import type { CopyOptions } from 'fs';

              declare module 'my-module' {
                import type { ParsedPath } from 'path';
                import type { CopyOptions } from 'fs';
              }
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import type { CopyOptions } from 'fs';\n              import type { ParsedPath } from 'path';\n\n              declare module 'my-module' {\n                import type { CopyOptions } from 'fs';\n                import type { ParsedPath } from 'path';\n              }\n            ",
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`fs` type import",
        "order": "before",
        "firstImport": "type import of `path`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`fs` type import",
        "order": "before",
        "firstImport": "type import of `path`"
      }
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

            import { type Z, A } from "./Z";
            import type N, { E, D } from "./Z";
            import type { L, G } from "./Z";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n            import { A, type Z } from \"./Z\";\n            import type N, { D, E } from \"./Z\";\n            import type { G, L } from \"./Z\";\n          ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "named": true,
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`A` import",
        "order": "before",
        "firstImport": "type import of `Z`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`D` import",
        "order": "before",
        "firstImport": "import of `E`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`G` import",
        "order": "before",
        "firstImport": "import of `L`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              export { type B, A };
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              export { A, type B };\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "named": {
        "enabled": true,
        "types": "mixed"
      },
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`A` export",
        "order": "before",
        "firstImport": "type export of `B`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import { type B, A, default as C } from "./Z";
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import { A, default as C, type B } from \"./Z\";\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "named": {
        "import": true,
        "types": "types-last"
      },
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`B` type import",
        "order": "after",
        "firstImport": "import of `default`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              export { A, type Z } from "./Z";
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              export { type Z, A } from \"./Z\";\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "named": {
        "enabled": true,
        "types": "types-first"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`Z` type export",
        "order": "before",
        "firstImport": "export of `A`"
      }
    }
  ],
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import type { C } from 'Bar';
              import b from 'bar';
              import a from 'foo';
              import type { A } from 'foo';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import a from 'foo';
              import type { A } from 'foo';
              import b from 'bar';
              import c from 'Bar';
              import type { C } from 'Bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import b from 'bar';
              import a from 'foo';

              import index from './';

              import type { C } from 'Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import a from 'foo';

              import b from 'dirA/bar';

              import index from './';

              import type { C } from 'dirA/Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [
        "type"
      ]
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import c from 'Bar';
              import type { A } from 'foo';
              import a from 'foo';

              import type { C } from 'dirA/Bar';
              import b from 'dirA/bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import a from 'foo';
              import b from 'bar';
              import c from 'Bar';

              import index from './';

              import type { A } from 'foo';
              import type { C } from 'Bar';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import { serialize, parse, mapFieldErrors } from '@vtaits/form-schema';
              import type { GetFieldSchema } from '@vtaits/form-schema';
              import { useMemo, useCallback } from 'react';
              import type { ReactElement, ReactNode } from 'react';
              import { Form } from 'react-final-form';
              import type { FormProps as FinalFormProps } from 'react-final-form';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

              import type { CopyOptions } from 'fs';
              import type { ParsedPath } from 'path';

              declare module 'my-module' {
                import type { CopyOptions } from 'fs';
                import type { ParsedPath } from 'path';
              }
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": false
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
            import type { A } from 'foo';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [
        "type"
      ],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import type { AA } from 'abc';
            import a from 'foo';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import type { AA } from 'abc';
            import a from 'foo';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import b from 'dirA/bar';
            import type { D } from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": []
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA } from 'abc';
            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';
            import type { AA } from 'abc';
            import type { A } from 'foo';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';
            import type { AA } from 'abc';

            import type { A } from 'foo';
            import type { C } from 'dirA/Bar';
            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "ignore",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA } from 'abc';

            import type { A } from 'foo';

            import type { C } from 'dirA/Bar';

            import type { D } from 'dirA/bar';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "alphabetize": {
        "order": "asc",
        "caseInsensitive": true
      },
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";

            import c from "../foo.js";

            import d from "./bar.js";

            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always",
      "newlines-between-types": "ignore"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";

            import type C from "../foo.js";

            import type D from "./bar.js";

            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "sortTypesGroup": true,
      "newlines-between": "never",
      "newlines-between-types": "always"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import fs from 'fs';

            import '@scoped/package';
            import type { B } from 'fs';

            import type { A1 } from '/bad/bad/bad/bad';
            import './a/b/c';
            import type { A2 } from '/bad/bad/bad/bad';
            import type { A3 } from '/bad/bad/bad/bad';
            import type { D1 } from '/bad/bad/not/good';
            import type { D2 } from '/bad/bad/not/good';
            import type { D3 } from '/bad/bad/not/good';

            import type { C } from '@something/else';

            import type { E } from './index.js';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "builtin",
        "type",
        "unknown",
        "external"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "never"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import makeVanillaYargs from 'yargs/yargs';

            import { createDebugLogger } from 'multiverse+rejoinder';

            import { globalDebuggerNamespace } from 'rootverse+bfe:src/constant.ts';
            import { ErrorMessage, type KeyValueEntry } from 'rootverse+bfe:src/error.ts';

            import {
              $artificiallyInvoked,
              $canonical,
              $exists,
              $genesis
            } from 'rootverse+bfe:src/symbols.ts';

            import type {
              Entries,
              LiteralUnion,
              OmitIndexSignature,
              Promisable,
              StringKeyOf
            } from 'type-fest';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "multiverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "universe{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import makeVanillaYargs from 'yargs/yargs';
            import { createDebugLogger } from 'multiverse+rejoinder';
            import { globalDebuggerNamespace } from 'rootverse+bfe:src/constant.ts';
            import { ErrorMessage, type KeyValueEntry } from 'rootverse+bfe:src/error.ts';
            import { $artificiallyInvoked } from 'rootverse+bfe:src/symbols.ts';

            import type {
              Entries,
              LiteralUnion,
              OmitIndexSignature,
              Promisable,
              StringKeyOf
            } from 'type-fest';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "multiverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        },
        {
          "pattern": "universe{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';
            import b from 'dirA/bar';
            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';

            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';

            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';

            import type { F } from './index.js';

            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "never",
      "newlines-between-types": "always-and-inside-groups",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import assert from 'assert';
            import { isNativeError } from 'util/types';

            import { runNoRejectOnBadExit } from '@-xun/run';
            import { TrialError } from 'named-app-errors';
            import { resolve as resolverLibrary } from 'resolve.exports';

            import { toAbsolutePath, type AbsolutePath } from 'rootverse+project-utils:src/fs.ts';

            import type { PackageJson } from 'type-fest';
            // Some comment about remembering to do something
            import type { XPackageJson } from 'rootverse:src/assets/config/_package.json.ts';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc",
        "caseInsensitive": true
      },
      "named": {
        "enabled": true,
        "types": "types-last"
      },
      "groups": [
        "builtin",
        "external",
        "internal",
        [
          "parent",
          "sibling",
          "index"
        ],
        [
          "object",
          "type"
        ]
      ],
      "pathGroups": [
        {
          "pattern": "rootverse{*,*/**}",
          "group": "external",
          "position": "after"
        }
      ],
      "distinctGroup": true,
      "pathGroupsExcludedImportTypes": [
        "builtin",
        "object"
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "type",
        "builtin",
        "parent",
        "sibling",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      },
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";

            import c from "../foo.js";

            import d from "./bar.js";

            import e from "./";

            import type A from "fs";
            import type B from "path";
            import type C from "../foo.js";
            import type D from "./bar.js";
            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true,
      "newlines-between": "always",
      "newlines-between-types": "ignore"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import a from "fs";
            import b from "path";
            import c from "../foo.js";
            import d from "./bar.js";
            import e from "./";

            import type A from "fs";
            import type B from "path";

            import type C from "../foo.js";

            import type D from "./bar.js";

            import type E from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "builtin",
        "parent",
        "sibling",
        "index",
        "type"
      ],
      "sortTypesGroup": true,
      "newlines-between": "never",
      "newlines-between-types": "always"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';
            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';
            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';
            import type { F } from './index.js';
            import type { G } from './aaa.js';
            import type { H } from './bbb';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "external",
        "internal",
        "index",
        "type"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > valid

```js

            import type { AA,
              BB, CC } from 'abc';

            import type { Z } from 'fizz';

            import type {
              A,
              B
            } from 'foo';

            import type { C2 } from 'dirB/Bar';

            import type {
              D2,
              X2,
              Y2
            } from 'dirB/bar';

            import type { E2 } from 'dirB/baz';
            import type { C3 } from 'dirC/Bar';

            import type {
              D3,
              X3,
              Y3
            } from 'dirC/bar';

            import type { E3 } from 'dirC/baz';
            import type { F3 } from 'dirC/caz';
            import type { C1 } from 'dirA/Bar';

            import type {
              D1,
              X1,
              Y1
            } from 'dirA/bar';

            import type { E1 } from 'dirA/baz';
            import type { F } from './index.js';
            import type { G } from './aaa.js';
            import type { H } from './bbb';

            import c from 'Bar';
            import d from 'bar';

            import {
              aa,
              bb,
              cc,
              dd,
              ee,
              ff,
              gg
            } from 'baz';

            import {
              hh,
              ii,
              jj,
              kk,
              ll,
              mm,
              nn
            } from 'fizz';

            import a from 'foo';

            import b from 'dirA/bar';

            import index from './';
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      },
      "groups": [
        "type",
        "external",
        "internal",
        "index"
      ],
      "pathGroups": [
        {
          "pattern": "dirA/**",
          "group": "internal",
          "position": "after"
        },
        {
          "pattern": "dirB/**",
          "group": "internal",
          "position": "before"
        },
        {
          "pattern": "dirC/**",
          "group": "internal"
        }
      ],
      "newlines-between": "always-and-inside-groups",
      "newlines-between-types": "never",
      "pathGroupsExcludedImportTypes": [],
      "sortTypesGroup": true,
      "consolidateIslands": "inside-groups"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import b from 'bar';
              import c from 'Bar';
              import type { C } from 'Bar';
              import a from 'foo';
              import type { A } from 'foo';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import c from 'Bar';\n              import type { C } from 'Bar';\n              import b from 'bar';\n              import a from 'foo';\n              import type { A } from 'foo';\n\n              import index from './';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "after",
        "firstImport": "type import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import a from 'foo';
              import type { A } from 'foo';
              import c from 'Bar';
              import type { C } from 'Bar';
              import b from 'bar';

              import index from './';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import a from 'foo';\n              import type { A } from 'foo';\n              import b from 'bar';\n              import c from 'Bar';\n              import type { C } from 'Bar';\n\n              import index from './';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "before",
        "firstImport": "import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import b from 'bar';
              import c from 'Bar';
              import a from 'foo';

              import index from './';

              import type { A } from 'foo';
              import type { C } from 'Bar';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import c from 'Bar';\n              import b from 'bar';\n              import a from 'foo';\n\n              import index from './';\n\n              import type { C } from 'Bar';\n              import type { A } from 'foo';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`Bar` import",
        "order": "before",
        "firstImport": "import of `bar`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`Bar` type import",
        "order": "before",
        "firstImport": "type import of `foo`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import a from 'foo';
              import c from 'Bar';
              import b from 'bar';

              import index from './';

              import type { C } from 'Bar';
              import type { A } from 'foo';
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import a from 'foo';\n              import b from 'bar';\n              import c from 'Bar';\n\n              import index from './';\n\n              import type { A } from 'foo';\n              import type { C } from 'Bar';\n            ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "groups": [
        "external",
        "index",
        "type"
      ],
      "alphabetize": {
        "order": "desc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`bar` import",
        "order": "before",
        "firstImport": "import of `Bar`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`foo` type import",
        "order": "before",
        "firstImport": "type import of `Bar`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

              import type { ParsedPath } from 'path';
              import type { CopyOptions } from 'fs';

              declare module 'my-module' {
                import type { ParsedPath } from 'path';
                import type { CopyOptions } from 'fs';
              }
            
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n              import type { CopyOptions } from 'fs';\n              import type { ParsedPath } from 'path';\n\n              declare module 'my-module' {\n                import type { CopyOptions } from 'fs';\n                import type { ParsedPath } from 'path';\n              }\n            ",
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`fs` type import",
        "order": "before",
        "firstImport": "type import of `path`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`fs` type import",
        "order": "before",
        "firstImport": "type import of `path`"
      }
    }
  ],
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > order > invalid

```js

            import { type Z, A } from "./Z";
            import type N, { E, D } from "./Z";
            import type { L, G } from "./Z";
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "output": "\n            import { A, type Z } from \"./Z\";\n            import type N, { D, E } from \"./Z\";\n            import type { G, L } from \"./Z\";\n          ",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "options": [
    {
      "named": true,
      "alphabetize": {
        "order": "asc"
      }
    }
  ],
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`A` import",
        "order": "before",
        "firstImport": "type import of `Z`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`D` import",
        "order": "before",
        "firstImport": "import of `E`"
      }
    },
    {
      "messageId": "order",
      "data": {
        "secondImport": "`G` import",
        "order": "before",
        "firstImport": "import of `L`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order: flow > order > valid

```js

        import type {Bar} from 'common';
        import typeof {foo} from 'common';
        import {bar} from 'common';
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order: flow > order > invalid

```js

        import type {Bar} from 'common';
        import {bar} from 'common';
        import typeof {foo} from 'common';
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc"
      }
    }
  ],
  "output": "\n        import type {Bar} from 'common';\n        import typeof {foo} from 'common';\n        import {bar} from 'common';\n      ",
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`common` typeof import",
        "order": "before",
        "firstImport": "import of `common`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order: flow > order > invalid

```js

        import type {Bar} from 'common';
        import {bar} from 'common';
        import typeof {foo} from 'common';
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "desc"
      }
    }
  ],
  "output": "\n        import {bar} from 'common';\n        import typeof {foo} from 'common';\n        import type {Bar} from 'common';\n      ",
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`common` type import",
        "order": "after",
        "firstImport": "typeof import of `common`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order: flow > order > invalid

```js

        import type {Bar} from './local/sub';
        import {bar} from './local/sub';
        import {baz} from './local-sub';
        import typeof {foo} from './local/sub';
      
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "requireConfigFile": false,
      "babelOptions": {
        "configFile": false,
        "babelrc": false,
        "presets": [
          "@babel/flow"
        ]
      }
    }
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "alphabetize": {
        "order": "asc",
        "orderImportKind": "asc"
      }
    }
  ],
  "output": "\n        import type {Bar} from './local/sub';\n        import typeof {foo} from './local/sub';\n        import {bar} from './local/sub';\n        import {baz} from './local-sub';\n      ",
  "errors": [
    {
      "messageId": "order",
      "data": {
        "secondImport": "`./local/sub` typeof import",
        "order": "before",
        "firstImport": "import of `./local/sub`"
      }
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertInvalidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)


#### order: flow > order > invalid

```js

        import { cfg } from 'path/path/path/src/Cfg';
        import { l10n } from 'path/src/l10n';
        import { helpers } from 'path/path/path/helpers';
        import { tip } from 'path/path/tip';

        import { controller } from '../../../../path/path/path/controller';
        import { component } from '../../../../path/path/path/component';
      
```

AssertionError [ERR_ASSERTION]: Test property `output`, if specified, must be a string or null. If no autofix is expected, then omit the `output` property or set it to null.
    at assertInvalidTestCaseIsWellFormed (apps/oxlint/dist/plugins-dev.js)
    at runInvalidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js
    at it (apps/oxlint/conformance/src/capture.ts:123:5)


### `prefer-default-export`

Pass: 37 / 51 (72.5%)
Fail: 12 / 51 (23.5%)
Skip: 2 / 51 (3.9%)

#### prefer-default-export > valid

```js
export Memory, { MemoryValue } from './Memory'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### prefer-default-export > valid

```js
export type UserId = number;
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### prefer-default-export > valid

```js
export default from "foo.js"
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### prefer-default-export > valid

```js
export Memory, { MemoryValue } from './Memory'
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "options": [
    {
      "target": "any"
    }
  ],
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js

            export type foo = string;
            export type bar = number;
            /* $PWD/node_modules/@typescript-eslint/parser/dist/index.js */
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export type foo = string /* $PWD/node_modules/@typescript-eslint/parser/dist/index.js*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export interface foo { bar: string; } /* $PWD/node_modules/@typescript-eslint/parser/dist/index.js*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export interface foo { bar: string; }; export function goo() {} /* $PWD/node_modules/@typescript-eslint/parser/dist/index.js*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js

            export type foo = string;
            export type bar = number;
            /* $PWD/node_modules/@babel/eslint-parser/lib/index.cjs */
          
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export type foo = string /* $PWD/node_modules/@babel/eslint-parser/lib/index.cjs*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export interface foo { bar: string; } /* $PWD/node_modules/@babel/eslint-parser/lib/index.cjs*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


#### TypeScript > prefer-default-export > valid

```js
export interface foo { bar: string; }; export function goo() {} /* $PWD/node_modules/@babel/eslint-parser/lib/index.cjs*/
```

```json
{
  "languageOptions": {
    "parser": {}
  },
  "filename": "apps/oxlint/conformance/submodules/import_x/test/fixtures/foo.js",
  "settings": {
    "import-x/parsers": {
      "/home/david/workspace/oxc/apps/oxlint/conformance/submodules/import_x/node_modules/@typescript-eslint/parser/dist/index.js": [
        ".ts"
      ]
    },
    "import-x/resolver": {
      "eslint-import-resolver-typescript": true
    }
  },
  "_parser": {
    "specifier": "@babel/eslint-parser",
    "lang": "jsx"
  }
}
```

Error: Parsing failed
    at parse (apps/oxlint/dist/plugins-dev.js)
    at lint (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)


### `unambiguous`

Pass: 10 / 12 (83.3%)
Fail: 2 / 12 (16.7%)
Skip: 0 / 12 (0.0%)

#### unambiguous > valid

```js
function x() {}
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "sourceType": "commonjs"
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/unambiguous',
    message: 'This module could be parsed as a valid script.',
    messageId: 'module',
    severity: 1,
    nodeType: 'FunctionDeclaration',
    line: 1,
    column: 0,
    endLine: 1,
    endColumn: 15,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js


#### unambiguous > valid

```js
"use strict"; function y() {}
```

```json
{
  "languageOptions": {
    "parser": {},
    "parserOptions": {
      "sourceType": "commonjs"
    }
  },
  "_parser": {
    "specifier": "@typescript-eslint/parser",
    "lang": "ts"
  }
}
```

AssertionError [ERR_ASSERTION]: Should have no errors but had 1: [
  {
    ruleId: 'rule-to-test/unambiguous',
    message: 'This module could be parsed as a valid script.',
    messageId: 'module',
    severity: 1,
    nodeType: 'Literal',
    line: 1,
    column: 0,
    endLine: 1,
    endColumn: 29,
    fixes: null,
    suggestions: null
  }
]

1 !== 0

    at assertErrorCountIsCorrect (apps/oxlint/dist/plugins-dev.js)
    at assertValidTestCasePasses (apps/oxlint/dist/plugins-dev.js)
    at runValidTestCase (apps/oxlint/dist/plugins-dev.js)
    at apps/oxlint/dist/plugins-dev.js

