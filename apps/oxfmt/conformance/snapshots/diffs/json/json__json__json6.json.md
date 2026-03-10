# json/json/json6.json

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -2,9 +2,8 @@
   {
     "//": "Keyword `undefined`",
     "data": [{ "undefined": undefined }, undefined, [undefined]]
   },
-
   {
     "//": "back-tick quoted strings",
     "data": [
       ``,
@@ -15,11 +14,9 @@
       `\u{1F409}\`'"\${}`,
       { "as-object-value": `foo` }
     ]
   },
-
   { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },
-
   {
     "//": "Numbers",
     "data": [
       0o123,
@@ -43,7 +40,6 @@
       -123_456,
       -0xdeed_beef
     ]
   },
-
   { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
 ]

`````

### Actual (oxfmt)

`````json
[
  {
    "//": "Keyword `undefined`",
    "data": [{ "undefined": undefined }, undefined, [undefined]]
  },
  {
    "//": "back-tick quoted strings",
    "data": [
      ``,
      `foo`,
      `
  multiple-line
`,
      `\u{1F409}\`'"\${}`,
      { "as-object-value": `foo` }
    ]
  },
  { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },
  {
    "//": "Numbers",
    "data": [
      0o123,
      0b101010,
      1e5,
      123_456,
      0xdeed_beef,
      0123,
      -Infinity,
      -NaN,
      +1,
      -1,
      +0o123,
      +0b101010,
      +1e5,
      +123_456,
      +0xdeed_beef,
      -0o123,
      -0b101010,
      -1e5,
      -123_456,
      -0xdeed_beef
    ]
  },
  { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
]

`````

### Expected (prettier)

`````json
[
  {
    "//": "Keyword `undefined`",
    "data": [{ "undefined": undefined }, undefined, [undefined]]
  },

  {
    "//": "back-tick quoted strings",
    "data": [
      ``,
      `foo`,
      `
  multiple-line
`,
      `\u{1F409}\`'"\${}`,
      { "as-object-value": `foo` }
    ]
  },

  { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },

  {
    "//": "Numbers",
    "data": [
      0o123,
      0b101010,
      1e5,
      123_456,
      0xdeed_beef,
      0123,
      -Infinity,
      -NaN,
      +1,
      -1,
      +0o123,
      +0b101010,
      +1e5,
      +123_456,
      +0xdeed_beef,
      -0o123,
      -0b101010,
      -1e5,
      -123_456,
      -0xdeed_beef
    ]
  },

  { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
]

`````

## Option 2

`````json
{"printWidth":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,6 @@
 [
   { "//": "Keyword `undefined`", "data": [{ "undefined": undefined }, undefined, [undefined]] },
-
   {
     "//": "back-tick quoted strings",
     "data": [
       ``,
@@ -12,11 +11,9 @@
       `\u{1F409}\`'"\${}`,
       { "as-object-value": `foo` }
     ]
   },
-
   { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },
-
   {
     "//": "Numbers",
     "data": [
       0o123,
@@ -40,7 +37,6 @@
       -123_456,
       -0xdeed_beef
     ]
   },
-
   { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
 ]

`````

### Actual (oxfmt)

`````json
[
  { "//": "Keyword `undefined`", "data": [{ "undefined": undefined }, undefined, [undefined]] },
  {
    "//": "back-tick quoted strings",
    "data": [
      ``,
      `foo`,
      `
  multiple-line
`,
      `\u{1F409}\`'"\${}`,
      { "as-object-value": `foo` }
    ]
  },
  { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },
  {
    "//": "Numbers",
    "data": [
      0o123,
      0b101010,
      1e5,
      123_456,
      0xdeed_beef,
      0123,
      -Infinity,
      -NaN,
      +1,
      -1,
      +0o123,
      +0b101010,
      +1e5,
      +123_456,
      +0xdeed_beef,
      -0o123,
      -0b101010,
      -1e5,
      -123_456,
      -0xdeed_beef
    ]
  },
  { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
]

`````

### Expected (prettier)

`````json
[
  { "//": "Keyword `undefined`", "data": [{ "undefined": undefined }, undefined, [undefined]] },

  {
    "//": "back-tick quoted strings",
    "data": [
      ``,
      `foo`,
      `
  multiple-line
`,
      `\u{1F409}\`'"\${}`,
      { "as-object-value": `foo` }
    ]
  },

  { "//": "String escapes ", "data": ["\0", "\xFF", "\u00FF", `\u{1F409}`] },

  {
    "//": "Numbers",
    "data": [
      0o123,
      0b101010,
      1e5,
      123_456,
      0xdeed_beef,
      0123,
      -Infinity,
      -NaN,
      +1,
      -1,
      +0o123,
      +0b101010,
      +1e5,
      +123_456,
      +0xdeed_beef,
      -0o123,
      -0b101010,
      -1e5,
      -123_456,
      -0xdeed_beef
    ]
  },

  { "//": "empty members", "data": [[,], [1, , 2, , , ,], [1, , 2], [1, , 2]] }
]

`````
