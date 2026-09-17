# externals/plugin-ember-template-tag/gts/prettier-ignore/exported-mod-var-with-as.gts

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
-export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>
+export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>;

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
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>;

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
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>

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
-export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>
+export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>;

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
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>;

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
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template> as TemplateOnlyComponent<Signature>

`````
