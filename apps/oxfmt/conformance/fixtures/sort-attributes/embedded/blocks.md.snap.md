# Input

`````md
```html
<div z="last" a="first"></div>
```

```tsx
const element = <Widget z="last" a="first" />;
```

`````

# Output

## `{"sortAttributes":true}`

`````md
```html
<div a="first" z="last"></div>
```

```tsx
const element = <Widget a="first" z="last" />;
```

`````
