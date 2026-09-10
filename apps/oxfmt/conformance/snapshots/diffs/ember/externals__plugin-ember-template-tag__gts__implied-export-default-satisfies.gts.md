# externals/plugin-ember-template-tag/gts/implied-export-default-satisfies.gts

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,5 +1,5 @@
 import type { TemplateOnlyComponent } from "@ember/component/template-only";
 
 <template>
   Implied Export Default with Satisfies
-</template> satisfies TemplateOnlyComponent;
+</template> satisfies TemplateOnlyComponent

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

<template>
  Implied Export Default with Satisfies
</template> satisfies TemplateOnlyComponent

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

<template>
  Implied Export Default with Satisfies
</template> satisfies TemplateOnlyComponent;

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
@@ -1,3 +1,3 @@
 import type { TemplateOnlyComponent } from '@ember/component/template-only';
 
-<template>Implied Export Default with Satisfies</template> satisfies TemplateOnlyComponent;
+<template>Implied Export Default with Satisfies</template> satisfies TemplateOnlyComponent

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

<template>Implied Export Default with Satisfies</template> satisfies TemplateOnlyComponent

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

<template>Implied Export Default with Satisfies</template> satisfies TemplateOnlyComponent;

`````
