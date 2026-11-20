# Input

`````md
```javascript
const  a = 1
```

```mjs
export  const a = 1
```

```postcss
a{color:red}
```

```yml
a:   1
```

```gql
{ a }
```

```md
#  a
```

```JS
const  a = 1
```

`````

# Output

## `{"printWidth":80}`

`````md
```javascript
const a = 1;
```

```mjs
export const a = 1;
```

```postcss
a {
  color: red;
}
```

```yml
a: 1
```

```gql
{
  a
}
```

```md
# a
```

```JS
const  a = 1
```

`````
