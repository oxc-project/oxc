# Input

`````md
- item

  ```js
  const  s = `a
  b`
  /* x
     y */
  ```

> - quoted
>
>   ```ts
>   function   f(){return `a
>   b`}
>   ```

```jsx
<Button   onClick={() => { a(); b() }} />
```

```css
a{color:red}
```

```yaml
a:   1
```

`````

# Output

## `{"printWidth":80}`

`````md
- item

  ```js
  const s = `a
  b`;
  /* x
     y */
  ```

> - quoted
>
>   ```ts
>   function f() {
>     return `a
>   b`;
>   }
>   ```

```jsx
<Button
  onClick={() => {
    a();
    b();
  }}
/>
```

```css
a {
  color: red;
}
```

```yaml
a: 1
```

`````
