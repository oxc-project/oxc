# Exit code
1

# stdout
```
  x ember(template-no-let-reference): update-able variables are not supported in templates, reference a const variable
   ,-[files/let-reference.gjs:5:8]
 4 | <template>
 5 |   <p>{{message}}</p>
   :        ^^^^^^^
 6 |   <p>{{greeting}}</p>
   `----

  x ember(template-no-let-reference): update-able variables are not supported in templates, reference a const variable
   ,-[files/typescript.gts:4:11]
 3 | <template>
 4 |   <span>{{count}}</span>
   :           ^^^^^
 5 | </template>
   `----

Found 0 warnings and 2 errors.
Finished in Xms on 3 files with 96 rules using X threads.
```

# stderr
```
```
