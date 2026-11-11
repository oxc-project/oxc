# Input

`````md
+++
title="x"
+++

- a

  ```toml
  arr=[1,
  2]
  s = """
  x
    y"""
  ```

```toml
bad = = 1
```

`````

# Output

## `{"printWidth":80}`

`````md
+++
title = "x"
+++

- a

  ```toml
  arr = [1, 2]
  s = """
  x
    y"""
  ```

```toml
bad = = 1
```

`````
