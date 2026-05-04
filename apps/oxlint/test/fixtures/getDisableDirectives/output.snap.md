# Exit code
1

# stdout
```
  x get-disable-directives-plugin(get-disable-directives): getDisableDirectives:
  |   total: 12
  |   block: 6
  |   line: 2
  |   next-line: 2
  |   enable: 2
    ,-[files/test.js:2:1]
  1 |     // oxlint-disable no-unused-vars
  2 | ,-> let a;
  3 | |   
  4 | |   // eslint-disable no-unused-vars
  5 | |   let b;
  6 | |   
  7 | |   // oxlint-disable-next-line no-console
  8 | |   console.log("test");
  9 | |   
 10 | |   // eslint-disable-next-line no-console
 11 | |   console.log("test2");
 12 | |   
 13 | |   let c; // oxlint-disable-line no-unused-vars
 14 | |   let d; // eslint-disable-line no-unused-vars
 15 | |   
 16 | |   // eslint-disable no-foo -- justification for disabling no-foo
 17 | |   // oxlint-disable no-bar -- justification for disabling no-bar
 18 | |   let e;
 19 | |   let f;
 20 | |   
 21 | |   /* oxlint-disable no-unused-vars */
 22 | |   /* eslint-disable no-unused-vars */
 23 | |   let g;
 24 | |   let h;
 25 | |   /* oxlint-enable no-unused-vars */
 26 | `-> /* eslint-enable no-unused-vars */
    `----

Found 0 warnings and 1 error.
Finished in Xms on 1 file with 96 rules using X threads.
```

# stderr
```
```
