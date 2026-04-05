# Exit code
1

# stdout
```
  x real-svelte-fixes(whole-file-edits): suggest greeting text; parserServices: true
   ,-[files/App.svelte:1:22]
 1 | <h1 class="greeting">Hello{name}</h1>
   :                      ^^^^^
   `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
```

# File altered: files/App.svelte
```
<h1 class="welcome">Hello{name}</h1>

```
