# Exit code
1

# stdout
```
  x whole-file-svelte-type-aware(options-visible): parser: true; services: true; projectService: true; extraFileExtensions: true; nestedParserFn: true; svelteRunes: true; preprocessFn: true; mergedBaseOption: true; scriptCount: 2; styleCount: 1; tsMaps: true; tsNodes: 1; element: h1
   ,-[files/App.svelte:1:1]
 1 | <script context="module" lang="ts">
   : ^
 2 |   export const prerender = true;
 3 | </script>
 4 |
 5 | <script lang="ts">
 6 |   import type { User } from "./types";
 7 |
 8 |   export let user: User = { name: "world" };
 9 | </script>
10 |
11 | <style>
12 |   h1 {
13 |     color: red;
14 |   }
15 | </style>
16 |
17 | <h1>Hello {user.name}</h1>
   `----
  x whole-file-svelte-type-aware(options-visible): parser: true; services: true; projectService: true; extraFileExtensions: true; nestedParserFn: true; svelteRunes: true; preprocessFn: true; mergedBaseOption: true; scriptCount: 0; styleCount: 0; tsMaps: false; tsNodes: 0; element: section
   ,-[files/NoScript.svelte:1:1]
 1 | <section>
   : ^
 2 |   <h2>No script</h2>
 3 | </section>
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
```
