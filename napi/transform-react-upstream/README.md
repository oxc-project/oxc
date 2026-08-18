# oxc-transform-react-upstream

Private, host-only N-API benchmark harness for `oxc_react_compiler_upstream`. It exposes only:

```ts
transformSync(filename: string, sourceText: string): {
  fatal: boolean;
  code: string;
  errors: OxcError[];
};
```

The pipeline targets React 19, preserves JSX, strips TypeScript, and omits source maps. These fixed
settings match the transform-only comparison in `bench-transformer`.
