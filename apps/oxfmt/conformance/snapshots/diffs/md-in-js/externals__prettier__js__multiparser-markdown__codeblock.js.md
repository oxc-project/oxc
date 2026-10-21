# externals/prettier/js/multiparser-markdown/codeblock.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,9 @@
 md`
-~~~js
+~~~~js
 markdown\`
   ~~~js
   console.log("hi");
   ~~~
 \`;
-~~~
+~~~~
 `;

`````

### Actual (oxfmt)

`````js
md`
~~~~js
markdown\`
  ~~~js
  console.log("hi");
  ~~~
\`;
~~~~
`;

`````

### Expected (prettier)

`````js
md`
~~~js
markdown\`
  ~~~js
  console.log("hi");
  ~~~
\`;
~~~
`;

`````

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,9 +1,9 @@
 md`
-~~~js
+~~~~js
 markdown\`
   ~~~js
   console.log("hi");
   ~~~
 \`;
-~~~
+~~~~
 `;

`````

### Actual (oxfmt)

`````js
md`
~~~~js
markdown\`
  ~~~js
  console.log("hi");
  ~~~
\`;
~~~~
`;

`````

### Expected (prettier)

`````js
md`
~~~js
markdown\`
  ~~~js
  console.log("hi");
  ~~~
\`;
~~~
`;

`````
