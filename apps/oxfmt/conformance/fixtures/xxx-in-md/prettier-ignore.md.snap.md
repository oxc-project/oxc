# Input

`````md
# Markdown

## Disabled

<!-- prettier-ignore -->
```ts
const x=1;
const y=2;
console.log(x+y);
```

## Enabled

```jsx
const X = ()=>
 <><div>JSX</div><hr/></>
```

`````

# Output

## `{"printWidth":80}`

`````md
# Markdown

## Disabled

<!-- prettier-ignore -->
```ts
const x=1;
const y=2;
console.log(x+y);
```

## Enabled

```jsx
const X = () => (
  <>
    <div>JSX</div>
    <hr />
  </>
);
```

`````
