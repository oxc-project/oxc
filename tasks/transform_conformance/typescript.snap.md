# typescript/tests/cases/conformance/enums/awaitAndYield.ts
```error

  × `await` is only allowed within async functions and at the top levels of modules
   ╭─[typescript/tests/cases/conformance/enums/awaitAndYield.ts:5:15]
 4 │     enum E {
 5 │         foo = await x,
   ·               ─────
 6 │         baz = yield 1,
   ╰────


```

# typescript/tests/cases/conformance/enums/enumBasics.ts
```typescript
var x = E1.A;
var e = E1;
var e;
var e;
var s = E1[e.A];
var s;
var doNotPropagate = [E8.B, E7.A, E4.Z, E3.X, E3.Y, E3.Z];
var doPropagate = [E9.A, E9.B, E6.B, E6.C, E6.A, E5.A, E5.B, E5.C];

```

# typescript/tests/cases/conformance/enums/enumClassification.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithString.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithStringEmitDeclaration.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithTemplateLiterals.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumConstantMemberWithTemplateLiteralsEmitDeclaration.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumConstantMembers.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumErrorOnConstantBindingWithInitializer.ts
```typescript
const {value='123'} = thing;

```

# typescript/tests/cases/conformance/enums/enumErrors.ts
```error

  × Expected `,` but found `;`
    ╭─[typescript/tests/cases/conformance/enums/enumErrors.ts:48:18]
 47 │ 
 48 │     postSemicolon;
    ·                  ┬
    ·                  ╰── `,` expected
 49 │     postColonValueComma: 2,
    ╰────


```

# typescript/tests/cases/conformance/enums/enumExportMergingES6.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumMerging.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumMergingErrors.ts
```typescript

```

# typescript/tests/cases/conformance/enums/enumShadowedInfinityNaN.ts
```typescript
{
	let Infinity = {};
}
{
	let NaN = {};
}

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/boolean-value/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/const/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/constant-folding/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/enum-merging-inner-references/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/enum-merging-inner-references-shadow/input.ts
```typescript
const Cat = 10;
const Dog = 20;

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/export/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/inferred/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/inner-references/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/mix-references/input.ts
```typescript
var x = 10;

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/non-foldable-constant/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/non-scoped/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/outer-references/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/scoped/input.ts
```typescript
{
}

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-value/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-value-template/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/string-values-computed/input.ts
```typescript

```

# babel/packages/babel-plugin-transform-typescript/test/fixtures/enum/ts5.0-const-foldable/input.ts
```typescript
const BaseValue = 10;
const Prefix = '/data';
const xxx = 100 + Values.First;
const yyy = xxx;

```

