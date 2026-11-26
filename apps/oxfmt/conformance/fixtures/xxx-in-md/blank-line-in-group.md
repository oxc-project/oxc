A blank line in a group is dropped when it fits, and kept without indention when it breaks, in a list and a blockquote

```yaml
k: [aaaaaaaa, bbbbbbbbb,

  c]
```

- item

  ```yaml
  k: [aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,

    c]
  ```

  ```toml
  a = ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",

    "c"]
  ```

> ```yaml
> k: [aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
>
>   c]
> ```
>
> ```toml
> a = ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
>
>   "c"]
> ```
