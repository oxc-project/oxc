A multi-line text lays out as in its standalone file: a non-expanding one keeps its group flat (yaml), an expanding one expands it (toml)

```yaml
k: [aaaaaaaa, bbbbbbbbb,

  c]
```

> ```toml
> a = ["""x
> y""", 2]
> ```
