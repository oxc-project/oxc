# Exit code
1

# stdout
```
  x vue-scripts-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/define-in-both.vue:5:1]
 4 | };
 5 | debugger;
   : ^^^^^^^^^
 6 | </script>
   `----

  x vue(valid-define-emits): Custom events are defined in both `defineEmits` and `export default {}`.
    ,-[files/define-in-both.vue:10:1]
  9 | // `emits` is declared in the other `<script>` block too, so this is an error.
 10 | defineEmits(["submit"]);
    : ^^^^^^^^^^^^^^^^^^^^^^^
 11 | debugger;
    `----
  help: Remove `export default`.

  x vue-scripts-plugin(no-debugger): Unexpected Debugger Statement
    ,-[files/define-in-both.vue:11:1]
 10 | defineEmits(["submit"]);
 11 | debugger;
    : ^^^^^^^^^
 12 | </script>
    `----

  x vue-scripts-plugin(no-debugger): Unexpected Debugger Statement
   ,-[files/emits-in-other-script.vue:5:1]
 4 | };
 5 | debugger;
   : ^^^^^^^^^
 6 | </script>
   `----

  x vue-scripts-plugin(no-debugger): Unexpected Debugger Statement
    ,-[files/emits-in-other-script.vue:11:1]
 10 | defineEmits();
 11 | debugger;
    : ^^^^^^^^^
 12 | </script>
    `----

Found 0 warnings and 5 errors.
Finished in Xms on 2 files with 2 rules using X threads.
```

# stderr
```
```
