# TypeScript Optimizations

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

TypeScript-specific minification optimizations that go beyond simple type syntax stripping. These leverage TypeScript semantic information to produce smaller output.

## Why

TypeScript's `const enum` declarations are designed to be inlined at compile time — replacing member accesses with their literal values and removing the enum declaration entirely. This eliminates runtime overhead and reduces code size. Cross-module enum propagation extends this to imported enums.

## Transformations

### Const enum inlining

Replace `const enum` member accesses with their computed literal values and remove the enum declaration.

```ts
// Before
const enum Direction {
  Up = 0,
  Down = 1,
  Left = 2,
  Right = 3,
}
const d = Direction.Up;
move(Direction.Left);

// After
const d = 0;
move(2);
// enum declaration removed entirely
```

### Computed enum values

Handle enum members with computed values (auto-incrementing, expressions).

```ts
// Before
const enum Status {
  Active = 1,
  Pending, // auto-increments to 2
  Disabled, // auto-increments to 3
}
check(Status.Pending);

// After
check(2);
```

### String enum inlining

```ts
// Before
const enum LogLevel {
  Debug = "DEBUG",
  Info = "INFO",
  Error = "ERROR",
}
log(LogLevel.Info);

// After
log("INFO");
```

### Cross-module enum propagation

Resolve imported enum values at compile time when the source module is available.

```ts
// constants.ts
export const enum Color {
  Red = "#ff0000",
  Blue = "#0000ff",
}

// app.ts — Before
import { Color } from "./constants";
element.style.color = Color.Red;

// app.ts — After
element.style.color = "#ff0000";
```

## References

- esbuild: `shouldFoldTypeScriptConstantExpressions`, `EInlinedEnum` in `js_parser.go`
- TypeScript Handbook: [Const enums](https://www.typescriptlang.org/docs/handbook/enums.html#const-enums)
