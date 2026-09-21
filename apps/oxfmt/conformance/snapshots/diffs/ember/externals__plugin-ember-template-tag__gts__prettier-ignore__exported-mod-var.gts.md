# externals/plugin-ember-template-tag/gts/prettier-ignore/exported-mod-var.gts

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -6,5 +6,5 @@
   Yields: [];
 }
 
 // prettier-ignore
-export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>
+export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from "@ember/component/template-only";

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>

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
@@ -6,5 +6,5 @@
   Yields: [];
 }
 
 // prettier-ignore
-export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>
+export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Actual (oxfmt)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Expected (prettier)

`````gts
import type { TemplateOnlyComponent } from '@ember/component/template-only';

export interface Signature {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
export const Exported: TemplateOnlyComponent<Signature>     = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>

`````
