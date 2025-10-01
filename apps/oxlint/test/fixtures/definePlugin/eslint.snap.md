# Exit code
1

# stdout
```
<root>/apps/oxlint/test/fixtures/definePlugin/files/1.js
  0:1  error  create body:
this === rule: true                                                define-plugin-plugin/create
  0:1  error  before hook:
createOnce call count: 1
this === rule: true
filename: files/1.js  define-plugin-plugin/create-once
  0:1  error  before hook:
filename: files/1.js                                               define-plugin-plugin/create-once-before-false
  0:1  error  before hook:
filename: files/1.js                                               define-plugin-plugin/create-once-before-only
  0:1  error  before hook:
filename: files/1.js                                               define-plugin-plugin/create-once-hooks-only
  0:1  error  after hook:
identNum: 2
filename: files/1.js                                    define-plugin-plugin/create-once
  0:1  error  after hook:
filename: files/1.js                                                define-plugin-plugin/create-once-after-only
  0:1  error  after hook:
filename: files/1.js                                                define-plugin-plugin/create-once-hooks-only
  1:5  error  ident visit fn "a":
filename: files/1.js                                        define-plugin-plugin/create
  1:5  error  ident visit fn "a":
identNum: 1
filename: files/1.js                            define-plugin-plugin/create-once
  1:5  error  ident visit fn "a":
filename: files/1.js                                        define-plugin-plugin/create-once-before-only
  1:5  error  ident visit fn "a":
filename: files/1.js                                        define-plugin-plugin/create-once-after-only
  1:5  error  ident visit fn "a":
filename: files/1.js                                        define-plugin-plugin/create-once-no-hooks
  1:8  error  ident visit fn "b":
filename: files/1.js                                        define-plugin-plugin/create
  1:8  error  ident visit fn "b":
identNum: 2
filename: files/1.js                            define-plugin-plugin/create-once
  1:8  error  ident visit fn "b":
filename: files/1.js                                        define-plugin-plugin/create-once-before-only
  1:8  error  ident visit fn "b":
filename: files/1.js                                        define-plugin-plugin/create-once-after-only
  1:8  error  ident visit fn "b":
filename: files/1.js                                        define-plugin-plugin/create-once-no-hooks

<root>/apps/oxlint/test/fixtures/definePlugin/files/2.js
  0:1  error  create body:
this === rule: true                                                define-plugin-plugin/create
  0:1  error  before hook:
createOnce call count: 1
this === rule: true
filename: files/2.js  define-plugin-plugin/create-once
  0:1  error  before hook:
filename: files/2.js                                               define-plugin-plugin/create-once-before-false
  0:1  error  before hook:
filename: files/2.js                                               define-plugin-plugin/create-once-before-only
  0:1  error  before hook:
filename: files/2.js                                               define-plugin-plugin/create-once-hooks-only
  0:1  error  after hook:
identNum: 2
filename: files/2.js                                    define-plugin-plugin/create-once
  0:1  error  after hook:
filename: files/2.js                                                define-plugin-plugin/create-once-before-false
  0:1  error  after hook:
filename: files/2.js                                                define-plugin-plugin/create-once-after-only
  0:1  error  after hook:
filename: files/2.js                                                define-plugin-plugin/create-once-hooks-only
  1:5  error  ident visit fn "c":
filename: files/2.js                                        define-plugin-plugin/create
  1:5  error  ident visit fn "c":
identNum: 1
filename: files/2.js                            define-plugin-plugin/create-once
  1:5  error  ident visit fn "c":
filename: files/2.js                                        define-plugin-plugin/create-once-before-false
  1:5  error  ident visit fn "c":
filename: files/2.js                                        define-plugin-plugin/create-once-before-only
  1:5  error  ident visit fn "c":
filename: files/2.js                                        define-plugin-plugin/create-once-after-only
  1:5  error  ident visit fn "c":
filename: files/2.js                                        define-plugin-plugin/create-once-no-hooks
  1:8  error  ident visit fn "d":
filename: files/2.js                                        define-plugin-plugin/create
  1:8  error  ident visit fn "d":
identNum: 2
filename: files/2.js                            define-plugin-plugin/create-once
  1:8  error  ident visit fn "d":
filename: files/2.js                                        define-plugin-plugin/create-once-before-false
  1:8  error  ident visit fn "d":
filename: files/2.js                                        define-plugin-plugin/create-once-before-only
  1:8  error  ident visit fn "d":
filename: files/2.js                                        define-plugin-plugin/create-once-after-only
  1:8  error  ident visit fn "d":
filename: files/2.js                                        define-plugin-plugin/create-once-no-hooks

✖ 39 problems (39 errors, 0 warnings)
```

# stderr
```
```
