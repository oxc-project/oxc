# Input

`````md
A newline inside a text resumes at the embedded markdown's column 0: the code block, or a `markdown` template inside it, not a container it opens

````md
- inner

  <!-- prettier-ignore -->
  | a | b |
  |--|--|

  [link]: http://example.com "title
  more"
````

- outer

  ```js
  const doc = markdown`
    - inner

      <!-- prettier-ignore -->
      | a | b |
      |--|--|
  `;
  ```

`````

# Output

## `{"printWidth":80}`

`````md
A newline inside a text resumes at the embedded markdown's column 0: the code block, or a `markdown` template inside it, not a container it opens

```md
- inner

  <!-- prettier-ignore -->
  | a | b |
  |--|--|

  [link]: http://example.com "title
more"
```

- outer

  ```js
  const doc = markdown`
    - inner

      <!-- prettier-ignore -->
      | a | b |
      |--|--|
  `;
  ```

`````
