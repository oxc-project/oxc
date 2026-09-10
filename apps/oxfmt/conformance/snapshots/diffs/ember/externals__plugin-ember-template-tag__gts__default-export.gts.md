# externals/plugin-ember-template-tag/gts/default-export.gts

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -5,11 +5,11 @@
   Args: {};
   Yields: [];
 }
 
-<template>
+export default <template>
   Explicit default export module top level component. Explicit default export
   module top level component. Explicit default export module top level
   component. Explicit default export module top level component. Explicit
   default export module top level component.
-</template> as TemplateOnlyComponent<Signature>
+</template> as TemplateOnlyComponent<Signature>;
 /*AMBIGUOUS*/

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

export default <template>
  Explicit default export module top level component. Explicit default export
  module top level component. Explicit default export module top level
  component. Explicit default export module top level component. Explicit
  default export module top level component.
</template> as TemplateOnlyComponent<Signature>;
/*AMBIGUOUS*/

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

<template>
  Explicit default export module top level component. Explicit default export
  module top level component. Explicit default export module top level
  component. Explicit default export module top level component. Explicit
  default export module top level component.
</template> as TemplateOnlyComponent<Signature>
/*AMBIGUOUS*/

`````

## Option 2

`````json
{"printWidth":120,"singleQuote":true,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -5,10 +5,10 @@
   Args: {};
   Yields: [];
 }
 
-<template>
+export default <template>
   Explicit default export module top level component. Explicit default export module top level component. Explicit
   default export module top level component. Explicit default export module top level component. Explicit default export
   module top level component.
-</template> as TemplateOnlyComponent<Signature>
+</template> as TemplateOnlyComponent<Signature>;
 /*AMBIGUOUS*/

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

export default <template>
  Explicit default export module top level component. Explicit default export module top level component. Explicit
  default export module top level component. Explicit default export module top level component. Explicit default export
  module top level component.
</template> as TemplateOnlyComponent<Signature>;
/*AMBIGUOUS*/

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

<template>
  Explicit default export module top level component. Explicit default export module top level component. Explicit
  default export module top level component. Explicit default export module top level component. Explicit default export
  module top level component.
</template> as TemplateOnlyComponent<Signature>
/*AMBIGUOUS*/

`````
