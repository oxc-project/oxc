# Input

`````md
A multi-line text inside a group expands it here, but not in its standalone file

```yaml
k: [aaaaaaaa, bbbbbbbbb,

  c]
```

```toml
a = ["""x
y""", 2]
```

`````

# Output

## `{"printWidth":80}`

`````md
A multi-line text inside a group expands it here, but not in its standalone file

```yaml
k: [
    aaaaaaaa,
    bbbbbbbbb,

    c,
  ]
```

```toml
a = [
  """x
y""",
  2,
]
```

`````
