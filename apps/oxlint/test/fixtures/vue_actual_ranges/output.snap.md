# Exit code
1

# stdout
```
  x vue-actual-plugin(report-template): Unexpected template marker
   ,-[files/template-only.vue:2:9]
 1 | <template>
 2 |   <span>template-bad</span>
   :         ^^^^^^^^^^^^
 3 | </template>
   `----

  x vue-actual-plugin(report-template-from-section): Unexpected section marker
   ,-[files/two-scripts-one-reports.vue:2:8]
 1 | <template>
 2 |   <div>section-bad</div>
   :        ^^^^^^^^^^^
 3 | </template>
   `----

  x vue-actual-plugin(report-template): Unexpected template marker
   ,-[files/two-scripts.vue:2:8]
 1 | <template>
 2 |   <div>template-bad</div>
   :        ^^^^^^^^^^^^
 3 | </template>
   `----

  x vue-actual-plugin(report-template): Unexpected template marker
   ,-[files/with-script.vue:2:8]
 1 | <template>
 2 |   <div>template-bad</div>
   :        ^^^^^^^^^^^^
 3 | </template>
   `----

Found 0 warnings and 4 errors.
Finished in Xms on 5 files with 2 rules using X threads.
```

# stderr
```
```
