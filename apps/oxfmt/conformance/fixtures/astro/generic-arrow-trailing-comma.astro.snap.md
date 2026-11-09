# Input

`````astro
---
// Astro parses the frontmatter as TSX: the comma is required
const identity = <T,>(value: T) => value
---

<script>
  // Same in `<script>`
  const identity = <T,>(value: T) => value
</script>

`````

# Output

## `{"printWidth":80,"astro":{}}`

`````astro
---
// Astro parses the frontmatter as TSX: the comma is required
const identity = <T,>(value: T) => value;
---

<script>
  // Same in `<script>`
  const identity = <T,>(value: T) => value;
</script>

`````
