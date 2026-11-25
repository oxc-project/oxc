# Input

`````md
A multi-line text expands its group as in its standalone file, its next lines resuming at the code block

> ```toml
> a = ["""x
> y""", 2]
> ```

`````

# Output

## `{"printWidth":80}`

`````md
A multi-line text expands its group as in its standalone file, its next lines resuming at the code block

> ```toml
> a = [
>   """x
> y""",
>   2,
> ]
> ```

`````
