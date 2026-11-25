# Input

`````astro
---
import b from "b"
import a from "a"
const   items=[1,2,3]
if (!items) return Astro.redirect('/404')
---
<div class="p-4 flex bg-red-500"><Foo   title={ "x" } />
{items.map((i)=><p>{i}</p>)}
</div>
<script>
const x   = 1
</script>
<style>
div{color:red}
</style>

`````

# Output

## `{"astro":{}}`

`````astro
---
import b from "b";
import a from "a";
const items = [1, 2, 3];
if (!items) return Astro.redirect("/404");
---

<div class="p-4 flex bg-red-500">
  <Foo title={"x"} />
  {items.map((i) => (
    <p>{i}</p>
  ))}
</div>
<script>
  const x = 1;
</script>
<style>
  div {
    color: red;
  }
</style>

`````

## `{"astro":{"skipFrontmatter":true}}`

`````astro
---
import b from "b"
import a from "a"
const   items=[1,2,3]
if (!items) return Astro.redirect('/404')
---

<div class="p-4 flex bg-red-500">
  <Foo title={"x"} />
  {items.map((i) => (
    <p>{i}</p>
  ))}
</div>
<script>
  const x = 1;
</script>
<style>
  div {
    color: red;
  }
</style>

`````

## `{"astro":{"allowShorthand":true,"skipFrontmatter":false,"compressHTML":"html"}}`

`````astro
---
import b from "b";
import a from "a";
const items = [1, 2, 3];
if (!items) return Astro.redirect("/404");
---

<div class="p-4 flex bg-red-500">
  <Foo title={"x"} />
  {items.map((i) => <p>{i}</p>)}
</div>
<script>
  const x = 1;
</script>
<style>
  div {
    color: red;
  }
</style>

`````

## `{}`

### Errors

`````
Cannot format `.astro`: `astro` plugin is not enabled in resolved config: fixtures/astro/each-option/basic.astro
`````

## `{"astro":false}`

### Errors

`````
Cannot format `.astro`: `astro` plugin is not enabled in resolved config: fixtures/astro/each-option/basic.astro
`````

## `{"astro":{},"sortImports":{}}`

`````astro
---
import a from "a";
import b from "b";
const items = [1, 2, 3];
if (!items) return Astro.redirect("/404");
---

<div class="p-4 flex bg-red-500">
  <Foo title={"x"} />
  {items.map((i) => (
    <p>{i}</p>
  ))}
</div>
<script>
  const x = 1;
</script>
<style>
  div {
    color: red;
  }
</style>

`````

## `{"astro":{},"sortTailwindcss":{}}`

`````astro
---
import b from "b";
import a from "a";
const items = [1, 2, 3];
if (!items) return Astro.redirect("/404");
---

<div class="flex bg-red-500 p-4">
  <Foo title={"x"} />
  {items.map((i) => (
    <p>{i}</p>
  ))}
</div>
<script>
  const x = 1;
</script>
<style>
  div {
    color: red;
  }
</style>

`````
