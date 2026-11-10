# Input

`````md
- a

  ```toml
  a = ["aaaa", "bbbb"]
  ```

  ```yaml
  a: [aaaa, bbbbbbbbb]
  ```

`````

# Output

## `{"printWidth":20}`

`````md
- a

  ```toml
  a = ["aaaa", "bbbb"]
  ```

  ```yaml
  a:
    [
      aaaa,
      bbbbbbbbb,
    ]
  ```

`````
