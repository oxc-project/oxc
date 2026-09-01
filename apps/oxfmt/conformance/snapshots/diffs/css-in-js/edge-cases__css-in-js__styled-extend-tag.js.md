# edge-cases/css-in-js/styled-extend-tag.js

> `Xxx.extend` not recognized as tag. See apps/oxfmt/DIVERGENCES.md#styled-extend-tag

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
 // DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
 // see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
 const TomatoButton = Button.extend`
-  color: tomato;
+	color  : tomato  ;
 `;
 
 Button.extend.attr({})`
-  border-color: black;
+border-color : black;
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
// see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
const TomatoButton = Button.extend`
	color  : tomato  ;
`;

Button.extend.attr({})`
border-color : black;
`;

`````

### Expected (prettier)

`````js
// DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
// see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
const TomatoButton = Button.extend`
  color: tomato;
`;

Button.extend.attr({})`
  border-color: black;
`;

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
@@ -1,9 +1,9 @@
 // DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
 // see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
 const TomatoButton = Button.extend`
-  color: tomato;
+	color  : tomato  ;
 `;
 
 Button.extend.attr({})`
-  border-color: black;
+border-color : black;
 `;

`````

### Actual (oxfmt)

`````js
// DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
// see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
const TomatoButton = Button.extend`
	color  : tomato  ;
`;

Button.extend.attr({})`
border-color : black;
`;

`````

### Expected (prettier)

`````js
// DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
// see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
const TomatoButton = Button.extend`
  color: tomato;
`;

Button.extend.attr({})`
  border-color: black;
`;

`````
