# externals/plugin-ember-template-tag/gts/prettier-ignore/one-line.gts

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
-const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>
+const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>;

`````

### Actual (oxfmt)

`````gts
import type { TOC } from "@ember/component/template-only";

export interface Sig {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>;

`````

### Expected (prettier)

`````gts
import type { TOC } from "@ember/component/template-only";

export interface Sig {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>

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
-const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>
+const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>;

`````

### Actual (oxfmt)

`````gts
import type { TOC } from '@ember/component/template-only';

export interface Sig {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>;

`````

### Expected (prettier)

`````gts
import type { TOC } from '@ember/component/template-only';

export interface Sig {
  Element: HTMLElement;
  Args: {};
  Yields: [];
}

// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template> as TOC<Sig>

`````
