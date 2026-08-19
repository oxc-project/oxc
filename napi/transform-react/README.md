# Oxc Transform React

Native Node.js bindings for Oxc's React transform pipeline. It combines:

- Oxc's experimental Rust port of [React Compiler]
- TypeScript syntax removal
- JSX transformation with automatic and classic runtimes
- React Fast Refresh
- These transforms can also be run individually

React Compiler runs first, then Oxc removes TypeScript syntax and applies the
configured JSX and Fast Refresh transforms.

> This package and Oxc's React Compiler port are experimental. Review generated
> output before using it in production.

## Usage

```javascript
import { transformSync } from "oxc-transform-react";

const result = transformSync(
  "Component.tsx",
  `
    export function Component({ name }: { name: string }) {
      return <div>Hello {name}</div>;
    }
  `,
);

if (result.fatal) {
  console.error(result.errors);
} else {
  console.log(result.code);
}
```

React Compiler is enabled by default with a React 19 target, and JSX uses the
automatic runtime. Enable React Fast Refresh with `jsx.refresh`. The filename
determines whether the input is parsed as JavaScript, JSX, TypeScript, or TSX.

## API

```typescript
transformSync(
  filename: string,
  sourceText: string,
  options?: TransformOptions,
): TransformResult;

transform(
  filename: string,
  sourceText: string,
  options?: TransformOptions,
): Promise<TransformResult>;
```

The asynchronous `transform` runs on a worker-pool thread. It is useful when
processing files concurrently, but can be slower for a single small file.

`TransformResult` contains `code`, an optional source `map`, `errors`, and a
`fatal` flag. Recoverable React Compiler bail-outs appear in `errors` as
warnings while still producing code, so use `fatal` to determine whether the
output is usable.

## Options

All options are optional. Defaults in the tables below apply when a property is
omitted.

### Transform options

| Option          | Type                                | Default                | Description                                                                                             |
| --------------- | ----------------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------- |
| `lang`          | `string`                            | Inferred from filename | Parse as `"js"`, `"jsx"`, `"ts"`, `"tsx"`, or `"dts"`.                                                  |
| `sourceType`    | `string`                            | Inferred               | Parse as `"script"`, `"module"`, `"commonjs"`, or `"unambiguous"`.                                      |
| `sourcemap`     | `boolean`                           | `false`                | Generate a source map in `result.map`.                                                                  |
| `jsx`           | `"preserve"` or `JsxOptions`        | Automatic runtime      | Configure the JSX transform. Set to `"preserve"` to leave JSX syntax in the output.                     |
| `reactCompiler` | `boolean` or `ReactCompilerOptions` | `true`                 | Configure React Compiler. Set to `false` to disable it or `true` to enable it with the default options. |

### JSX options

| Option             | Type                               | Default                 | Description                                                                                    |
| ------------------ | ---------------------------------- | ----------------------- | ---------------------------------------------------------------------------------------------- |
| `runtime`          | `"classic"` or `"automatic"`       | `"automatic"`           | Select the JSX runtime. The automatic runtime imports JSX factories; the classic one does not. |
| `development`      | `boolean`                          | `false`                 | Emit development information such as `__source` and `__self`.                                  |
| `throwIfNamespace` | `boolean`                          | `true`                  | Report an error for XML namespace syntax such as `<svg:path>`.                                 |
| `pure`             | `boolean`                          | `true`                  | Add pure annotations to JSX and top-level React calls for tree shaking.                        |
| `importSource`     | `string`                           | `"react"`               | Set the package imported by the automatic runtime.                                             |
| `pragma`           | `string`                           | `"React.createElement"` | Set the JSX factory used by the classic runtime.                                               |
| `pragmaFrag`       | `string`                           | `"React.Fragment"`      | Set the JSX fragment used by the classic runtime.                                              |
| `refresh`          | `boolean` or `ReactRefreshOptions` | `false`                 | Enable React Fast Refresh, optionally with custom identifiers and signature output.            |

`refresh: true` uses these defaults:

| Option               | Type      | Default          | Description                                              |
| -------------------- | --------- | ---------------- | -------------------------------------------------------- |
| `refreshReg`         | `string`  | `"$RefreshReg$"` | Set the Refresh registration identifier.                 |
| `refreshSig`         | `string`  | `"$RefreshSig$"` | Set the Refresh signature identifier.                    |
| `emitFullSignatures` | `boolean` | `false`          | Emit readable hook signatures instead of compact hashes. |

### React Compiler options

| Option                   | Type                                  | Default                   | Description                                                                                                                                                                                  |
| ------------------------ | ------------------------------------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compilationMode`        | `string`                              | `"infer"`                 | Select which functions to compile: `"infer"`, `"syntax"`, `"annotation"`, or `"all"`.                                                                                                        |
| `panicThreshold`         | `string`                              | `"none"`                  | Select which diagnostics abort the transform: `"none"`, `"critical_errors"`, or `"all_errors"`.                                                                                              |
| `target`                 | `string` or `ReactCompilerMetaTarget` | `"19"`                    | Target React `"17"`, `"18"`, `"19"`, or a Meta-internal runtime.                                                                                                                             |
| `gating`                 | `ReactCompilerGating`                 | Unset                     | Emit compiled and original functions behind an imported feature flag.                                                                                                                        |
| `dynamicGating`          | `ReactCompilerDynamicGating`          | Unset                     | Resolve flags in `"use memo if(...)"` directives from an imported module. A directive takes precedence over `gating`.                                                                        |
| `noEmit`                 | `boolean`                             | `false`                   | Request lint mode without applying React Compiler output. Deprecated; use `outputMode: "lint"`.                                                                                              |
| `outputMode`             | `string`                              | `"client"`                | Select `"client"`, `"ssr"`, or `"lint"` React Compiler output.                                                                                                                               |
| `eslintSuppressionRules` | `string[]`                            | React Hooks rules         | ESLint or Oxlint rule names whose matching suppression comments opt a function out. Defaults to `react-hooks/exhaustive-deps` and `react-hooks/rules-of-hooks`; `[]` disables this behavior. |
| `flowSuppressions`       | `boolean`                             | `true`                    | Treat `$FlowFixMe...`, `$FlowExpectedError`, or `$FlowIssue` tags immediately followed by `[react-rule` as compiler opt-outs.                                                                |
| `ignoreUseNoForget`      | `boolean`                             | `false`                   | Compile functions carrying `"use no memo"` or `"use no forget"`.                                                                                                                             |
| `customOptOutDirectives` | `string[]`                            | Unset                     | Add directive strings that opt a function or module out of compilation.                                                                                                                      |
| `sources`                | `string[]`                            | All except `node_modules` | Only run React Compiler when the filename contains one of the provided strings. Providing this option replaces the default filter.                                                           |
| `environment`            | `ReactCompilerEnvironmentOptions`     | Compiler defaults         | Override compiler feature flags and validation settings.                                                                                                                                     |

Despite its upstream-compatible name, `eslintSuppressionRules` recognizes both
ESLint and Oxlint comments: `eslint-disable`, `eslint-disable-next-line`,
`eslint-enable`, `oxlint-disable`, `oxlint-disable-next-line`, and
`oxlint-enable`.

`outputMode: "ssr"` takes precedence over `noEmit`. Otherwise, `noEmit: true`
forces lint output, including when `outputMode: "client"` is set. Lint output
suppresses only React Compiler rewrites; the downstream Oxc transform still
removes TypeScript syntax and applies the configured JSX transform.

The object-valued target and gating options have these shapes:

```typescript
interface ReactCompilerMetaTarget {
  kind: "donotuse_meta_internal";
  runtimeModule?: string; // Defaults to "react".
}

interface ReactCompilerGating {
  source: string;
  importSpecifierName: string;
}

interface ReactCompilerDynamicGating {
  source: string;
}
```

### React Compiler environment options

Unset environment properties retain the compiler defaults shown here.

| Option                                          | Type       | Default     | Description                                                                                                     |
| ----------------------------------------------- | ---------- | ----------- | --------------------------------------------------------------------------------------------------------------- |
| `customMacros`                                  | `string[]` | Unset       | Name macro-like functions whose calls and operands must stay together during compilation.                       |
| `enableResetCacheOnSourceFileChanges`           | `boolean`  | Unsupported | Accepted for upstream option compatibility but currently has no effect.                                         |
| `enablePreserveExistingMemoizationGuarantees`   | `boolean`  | `true`      | Use existing `useMemo` and `useCallback` information to preserve referential-equality behavior.                 |
| `validatePreserveExistingMemoizationGuarantees` | `boolean`  | `true`      | Validate that compilation preserves existing manual memoization guarantees.                                     |
| `validateExhaustiveMemoizationDependencies`     | `boolean`  | `false`     | Validate that manual memoization dependency arrays are exhaustive.                                              |
| `validateExhaustiveEffectDependencies`          | `string`   | `"off"`     | Validate effect dependencies with `"off"`, `"all"`, `"missing-only"`, or `"extra-only"`.                        |
| `enableOptionalDependencies`                    | `boolean`  | Unsupported | Accepted for upstream option compatibility but currently has no effect.                                         |
| `enableNameAnonymousFunctions`                  | `boolean`  | `false`     | Give generated or outlined anonymous functions inferred names.                                                  |
| `validateHooksUsage`                            | `boolean`  | `true`      | Validate that components partially satisfy the Rules of Hooks.                                                  |
| `validateRefAccessDuringRender`                 | `boolean`  | `true`      | Validate that ref values are not accessed during render.                                                        |
| `validateNoSetStateInRender`                    | `boolean`  | `true`      | Validate that state setters are not called unconditionally during render.                                       |
| `enableUseKeyedState`                           | `boolean`  | `false`     | Recommend keyed state when reporting render-time state resets.                                                  |
| `validateNoSetStateInEffects`                   | `boolean`  | `false`     | In lint mode, validate that state setters are not called synchronously in effects.                              |
| `validateNoDerivedComputationsInEffects`        | `boolean`  | `false`     | Validate that effects are not used to calculate data that can be derived during render.                         |
| `validateNoDerivedComputationsInEffectsExp`     | `boolean`  | `false`     | In lint mode, enable the experimental form of derived-computation validation.                                   |
| `validateNoJsxInTryStatements`                  | `boolean`  | `false`     | In lint mode, validate against creating JSX inside `try` blocks.                                                |
| `validateStaticComponents`                      | `boolean`  | `false`     | In lint mode, validate against dynamically creating components during render.                                   |
| `validateNoCapitalizedCalls`                    | `string[]` | Unset       | Validate capitalized function calls. The array adds allowed names; `[]` enables validation without extra names. |
| `validateBlocklistedImports`                    | `string[]` | Unset       | Bail out files that import any listed module.                                                                   |
| `validateSourceLocations`                       | `boolean`  | Unsupported | Accepted for upstream option compatibility but currently has no effect.                                         |
| `validateNoImpureFunctionsInRender`             | `boolean`  | `false`     | Validate against impure function calls during render.                                                           |
| `validateNoFreezingKnownMutableFunctions`       | `boolean`  | Unsupported | Accepted for upstream option compatibility, but validation currently runs regardless of its value.              |
| `enableAssumeHooksFollowRulesOfReact`           | `boolean`  | `true`      | Assume hook arguments and return values may be memoized and are therefore frozen.                               |
| `enableTransitivelyFreezeFunctionExpressions`   | `boolean`  | `true`      | Treat values captured by functions passed to React as transitively frozen.                                      |
| `enableFunctionOutlining`                       | `boolean`  | `true`      | Outline anonymous functions that do not capture local variables.                                                |
| `enableJsxOutlining`                            | `boolean`  | `false`     | Outline nested JSX into separately memoizable components.                                                       |
| `assertValidMutableRanges`                      | `boolean`  | Unsupported | Accepted for upstream option compatibility but currently has no effect.                                         |
| `enableCustomTypeDefinitionForReanimated`       | `boolean`  | `false`     | Use React Native Reanimated-aware type definitions.                                                             |
| `enableTreatRefLikeIdentifiersAsRefs`           | `boolean`  | `true`      | Treat `ref`-like identifiers with a `current` property as React refs.                                           |
| `enableTreatSetIdentifiersAsStateSetters`       | `boolean`  | `false`     | Treat called identifiers whose names begin with `set` as state setters.                                         |
| `validateNoVoidUseMemo`                         | `boolean`  | `false`     | Validate that `useMemo` callbacks return a value.                                                               |
| `enableAllowSetStateFromRefsInEffects`          | `boolean`  | `true`      | Allow recognized ref-based state-setting patterns in effects.                                                   |
| `enableVerboseNoSetStateInEffect`               | `boolean`  | `false`     | Emit more detailed diagnostics for state setters called in effects.                                             |
| `enableForest`                                  | `boolean`  | `false`     | Enable the experimental Forest reactive-scope optimization mode.                                                |

The default React 19 target imports `react/compiler-runtime`. Targets 17 and 18
import `react-compiler-runtime`, which must be available to the generated code.

Callback-valued options such as `logger`, function-valued `sources`, and type
provider callbacks are not supported by the native binding. `sources` accepts
an array of filename substrings instead.

## Notes

- TypeScript support removes syntax only. It does not type-check or emit
  declarations.
- `jsx.refresh` emits Fast Refresh registration and signature instrumentation.
  The bundler or development server must provide the Refresh runtime and HMR
  integration.
- The transform stages are independent. `reactCompiler: false` still removes
  TypeScript and transforms JSX, while `jsx: "preserve"` retains JSX without
  disabling React Compiler. Recoverable compiler bail-outs still run the
  downstream TypeScript and JSX transforms.

See [`index.d.ts`] for the generated TypeScript declarations and the [Oxc JSX
documentation] for more JSX examples.

[React Compiler]: https://react.dev/learn/react-compiler
[`index.d.ts`]: https://github.com/oxc-project/oxc/blob/main/napi/transform-react/index.d.ts
[Oxc JSX documentation]: https://oxc.rs/docs/guide/usage/transformer/jsx
