# Angular Plugin for Oxlint

This plugin provides Angular-specific linting rules for oxlint, ported from [angular-eslint](https://github.com/angular-eslint/angular-eslint).

## Implementation Status

The Angular plugin implements **48 rules** covering component/directive conventions, lifecycle methods, signals, inputs/outputs, pipes, and more. This represents near-complete parity with angular-eslint's TypeScript rules.

## Implemented Rules

### Component/Directive Rules

| Rule                                        | Category    | Description                                                           |
| ------------------------------------------- | ----------- | --------------------------------------------------------------------- |
| `component-class-suffix`                    | correctness | Ensures component classes end with 'Component' or a custom suffix     |
| `component-max-inline-declarations`         | style       | Enforces maximum lines for inline template/styles/animations          |
| `component-selector`                        | correctness | Validates component selector naming conventions (prefix, style, type) |
| `consistent-component-styles`               | style       | Enforces consistent usage of `styles`/`styleUrl`/`styleUrls`          |
| `directive-class-suffix`                    | correctness | Ensures directive classes end with 'Directive' or a custom suffix     |
| `directive-selector`                        | correctness | Validates directive selector naming conventions                       |
| `no-host-metadata-property`                 | style       | Disallows `host` metadata property in decorators                      |
| `prefer-host-metadata-property`             | style       | Prefers `host` metadata over `@HostBinding`/`@HostListener`           |
| `prefer-on-push-component-change-detection` | style       | Enforces OnPush change detection strategy                             |
| `prefer-standalone`                         | style       | Enforces standalone components (Angular 14+)                          |
| `relative-url-prefix`                       | correctness | Ensures relative URLs start with `./` or `../`                        |
| `use-component-selector`                    | correctness | Ensures components have a selector defined                            |
| `use-component-view-encapsulation`          | style       | Disallows ViewEncapsulation.None                                      |

### Lifecycle Rules

| Rule                             | Category    | Description                                           |
| -------------------------------- | ----------- | ----------------------------------------------------- |
| `contextual-lifecycle`           | correctness | Ensures lifecycle methods match the decorator context |
| `no-async-lifecycle-method`      | correctness | Disallows async lifecycle methods                     |
| `no-conflicting-lifecycle`       | correctness | Disallows implementing both DoCheck and OnChanges     |
| `no-empty-lifecycle-method`      | correctness | Disallows empty lifecycle methods                     |
| `no-lifecycle-call`              | correctness | Disallows explicit lifecycle method calls             |
| `require-lifecycle-on-prototype` | correctness | Ensures lifecycle methods on prototype, not instance  |
| `sort-lifecycle-methods`         | style       | Ensures lifecycle methods are in canonical order      |
| `use-lifecycle-interface`        | correctness | Ensures lifecycle interface implementation            |

### Signal Rules

| Rule                        | Category    | Description                                   |
| --------------------------- | ----------- | --------------------------------------------- |
| `computed-must-return`      | correctness | Ensures `computed()` signals return a value   |
| `prefer-output-emitter-ref` | style       | Prefers `output()` over `@Output()` decorator |
| `prefer-signal-model`       | style       | Suggests `model()` for input/output pairs     |
| `prefer-signals`            | style       | Enforces signal-based APIs over decorators    |

### Input/Output Rules

| Rule                           | Category    | Description                             |
| ------------------------------ | ----------- | --------------------------------------- |
| `no-input-prefix`              | style       | Disallows certain input name prefixes   |
| `no-input-rename`              | correctness | Disallows input aliasing                |
| `no-inputs-metadata-property`  | style       | Disallows `inputs` metadata property    |
| `no-output-native`             | correctness | Disallows native event names as outputs |
| `no-output-on-prefix`          | style       | Disallows "on" prefix for outputs       |
| `no-output-rename`             | correctness | Disallows output aliasing               |
| `no-outputs-metadata-property` | style       | Disallows `outputs` metadata property   |
| `prefer-output-readonly`       | correctness | Enforces readonly on outputs            |

### Pipe Rules

| Rule                           | Category    | Description                                    |
| ------------------------------ | ----------- | ---------------------------------------------- |
| `no-pipe-impure`               | style       | Disallows impure pipes                         |
| `pipe-prefix`                  | style       | Enforces pipe name prefix conventions          |
| `use-pipe-transform-interface` | correctness | Ensures PipeTransform interface implementation |

### Decorator Rules

| Rule                               | Category    | Description                                           |
| ---------------------------------- | ----------- | ----------------------------------------------------- |
| `contextual-decorator`             | correctness | Ensures decorators used in appropriate class contexts |
| `no-attribute-decorator`           | style       | Disallows `@Attribute` decorator                      |
| `no-duplicates-in-metadata-arrays` | correctness | Disallows duplicates in metadata arrays               |
| `no-queries-metadata-property`     | style       | Disallows `queries` metadata property                 |
| `sort-keys-in-type-decorator`      | style       | Orders decorator properties consistently              |

### Dependency Injection Rules

| Rule                         | Category    | Description                                   |
| ---------------------------- | ----------- | --------------------------------------------- |
| `no-forward-ref`             | style       | Disallows `forwardRef()` usage                |
| `prefer-inject`              | style       | Prefers `inject()` over constructor injection |
| `use-injectable-provided-in` | correctness | Ensures `providedIn` in `@Injectable`         |

### Localization Rules

| Rule                        | Category | Description                              |
| --------------------------- | -------- | ---------------------------------------- |
| `require-localize-metadata` | pedantic | Ensures `$localize` has metadata         |
| `runtime-localize`          | pedantic | Prevents `$localize` at module load time |

### Other Rules

| Rule                               | Category    | Description                                    |
| ---------------------------------- | ----------- | ---------------------------------------------- |
| `no-implicit-take-until-destroyed` | correctness | Requires DestroyRef for `takeUntilDestroyed()` |
| `prefer-control-flow`              | style       | Enforces `@if`/`@for`/`@switch` syntax         |

## Rules Not Implemented

The following angular-eslint rules are **not implemented** due to requiring TypeScript type information:

| Rule                   | Reason                                                          |
| ---------------------- | --------------------------------------------------------------- |
| `no-developer-preview` | Requires JSDoc/type analysis to detect `@developerPreview` tags |
| `no-experimental`      | Requires JSDoc/type analysis to detect `@experimental` tags     |
| `no-uncalled-signals`  | Requires type information to detect Signal types                |

These rules cannot be implemented without full TypeScript semantic analysis capabilities.

## Known Limitations

### General Limitations

1. **No type information**: Oxlint operates on AST only, without TypeScript's type checker. Rules requiring type inference cannot be implemented.

2. **No auto-fix**: Most rules do not support automatic fixing yet. Angular-eslint provides auto-fix for many rules.

### Rule-Specific Limitations

#### `component-selector` / `directive-selector`

- **Complex compound selectors** (e.g., `app-[custom]`, `button[appHighlight]`) are not fully supported
  - **Why**: Angular-eslint uses `CssSelector.parse()` from `@angular-eslint/bundled-angular-compiler` which contains Angular's full CSS selector parser. This parser handles all CSS selector syntax including attribute selectors, pseudo-classes, and compound selectors. Oxlint uses a simpler regex-based approach that handles common cases (element selectors, attribute selectors) but cannot parse arbitrary CSS selector combinations.
- **Multiple selector configurations in array format** not supported
  - **Why**: The angular-eslint rule supports passing an array of selector configurations (different prefixes/types per selector). The oxlint implementation only supports a single configuration object.

#### `require-lifecycle-on-prototype`

- Only detects PropertyDefinition patterns (`ngOnInit = () => {}`)
- Does not detect AssignmentExpression patterns (`this.ngOnInit = () => {}` in constructor)
  - **Why**: Angular-eslint uses ESLint's selector query system which can match AST patterns declaratively (e.g., `AssignmentExpression > MemberExpression[property.name=ngOnInit]`). Oxlint uses a visitor pattern that processes nodes individually. Detecting constructor assignments requires traversing into constructor bodies and matching specific assignment patterns, which is more complex to implement correctly and was deferred.

## Usage

To enable Angular rules in oxlint, add them to your configuration:

```json
{
  "rules": {
    "angular/component-class-suffix": "error",
    "angular/prefer-standalone": "warn",
    "angular/use-lifecycle-interface": "error"
  }
}
```

Or enable all Angular rules:

```json
{
  "plugins": ["angular"]
}
```

## Configuration Examples

### Component Selector Validation

```json
{
  "rules": {
    "angular/component-selector": [
      "error",
      {
        "type": "element",
        "prefix": "app",
        "style": "kebab-case"
      }
    ]
  }
}
```

### Directive Selector Validation

```json
{
  "rules": {
    "angular/directive-selector": [
      "error",
      {
        "type": "attribute",
        "prefix": "app",
        "style": "camelCase"
      }
    ]
  }
}
```

### Custom Class Suffixes

```json
{
  "rules": {
    "angular/component-class-suffix": [
      "error",
      {
        "suffixes": ["Component", "Page", "View"]
      }
    ]
  }
}
```

### Input Prefix Restrictions

```json
{
  "rules": {
    "angular/no-input-prefix": [
      "error",
      {
        "prefixes": ["on", "is", "can"]
      }
    ]
  }
}
```

## Contributing

When adding new Angular rules:

1. Create the rule file in `crates/oxc_linter/src/rules/angular/`
2. Register the rule in `crates/oxc_linter/src/rules/angular.rs`
3. Add comprehensive test cases (pass and fail)
4. Update this README with the rule documentation

## Reference

- [angular-eslint](https://github.com/angular-eslint/angular-eslint) - Original ESLint implementation
- [Angular Style Guide](https://angular.io/guide/styleguide) - Official Angular style guidelines
