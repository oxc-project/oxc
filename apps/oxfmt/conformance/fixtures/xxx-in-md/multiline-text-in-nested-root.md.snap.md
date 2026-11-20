# Input

`````md
A newline inside a text resumes at the code block, not at a container the child opens

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
A newline inside a text resumes at the code block, not at a container the child opens

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

### Not idempotent (second pass)

`````md
A newline inside a text resumes at the code block, not at a container the child opens

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
