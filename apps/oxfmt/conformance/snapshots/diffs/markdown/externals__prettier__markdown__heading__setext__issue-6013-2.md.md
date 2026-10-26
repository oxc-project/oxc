# externals/prettier/markdown/heading/setext/issue-6013-2.md

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,2 +1,3 @@
-Heading [![Shields.io badge](https://img.shields.io/badge/Shields.io-badge-brightgreen)](https://shields.io)
+Heading
+[![Shields.io badge](https://img.shields.io/badge/Shields.io-badge-brightgreen)](https://shields.io)
 =======

`````

### Actual (oxfmt)

`````md
Heading
[![Shields.io badge](https://img.shields.io/badge/Shields.io-badge-brightgreen)](https://shields.io)
=======

`````

### Expected (prettier)

`````md
Heading [![Shields.io badge](https://img.shields.io/badge/Shields.io-badge-brightgreen)](https://shields.io)
=======

`````
