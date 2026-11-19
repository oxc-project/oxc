# Input

`````md
# Markdown

In Prettier, using `typescript` parser inside `markdown` parser duplicates lines after `await { then }`.

```ts
await Promise.resolve(  "value"  );
await {
  then() {
  },
};
// This comment and below will be duplicated
await getPromise()
```

But we are not!

`````

# Output

## `{"printWidth":80}`

`````md
# Markdown

In Prettier, using `typescript` parser inside `markdown` parser duplicates lines after `await { then }`.

```ts
await Promise.resolve("value");
await {
  then() {},
};
// This comment and below will be duplicated
await getPromise();
```

But we are not!

`````
