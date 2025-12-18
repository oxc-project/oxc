# Exit code
1

# stdout
```
  × @tanstack/query(exhaustive-deps): useQuery must have exhaustive dependencies
   ╭─[files/test.tsx:5:20]
 4 │       // This should trigger the exhaustive-deps rule
 5 │ ╭─▶   const { data } = useQuery({
 6 │ │       queryKey: ["todos"],
 7 │ │       queryFn: () => fetch("/api/todos"),
 8 │ ╰─▶   });
 9 │     
   ╰────

Found 0 warnings and 1 error.
Finished in Xms on 1 file using X threads.
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
