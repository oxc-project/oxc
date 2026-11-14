# Exit code
1

# stdout
```
  x vue-actual-text-plugin(actual-text): actualOffset: 0, isWholeFile: true, sectionMatches: true
   ,-[files/plain.js:1:1]
 1 | const ok = 1;
   : ^^^^^^^^^^^^^^
   `----

  x vue-actual-text-plugin(actual-text): actualOffset: 0, isWholeFile: false, sectionMatches: true
   ,-[files/template-only.vue:1:1]
 1 | <template>
   : ^
 2 |   <span>template-bad</span>
   `----

  x vue-actual-text-plugin(actual-text): Unexpected template marker
   ,-[files/template-only.vue:2:9]
 1 | <template>
 2 |   <span>template-bad</span>
   :         ^^^^^^^^^^^^
 3 | </template>
   `----

  x vue-actual-text-plugin(actual-text): Unexpected template marker
   ,-[files/with-script.vue:2:13]
 1 | <template>
 2 |   <div>😀 template-bad</div>
   :           ^^^^^^^^^^^^
 3 | </template>
   `----

  x vue-actual-text-plugin(actual-text): actualOffset: 68, isWholeFile: false, sectionMatches: true
   ,-[files/with-script.vue:6:1]
 5 | <script setup>
 6 | const ok = 1;
   : ^^^^^^^^^^^^^^
 7 | </script>
   `----

Found 0 warnings and 5 errors.
Finished in Xms on 3 files with 1 rules using X threads.
```

# stderr
```
```
