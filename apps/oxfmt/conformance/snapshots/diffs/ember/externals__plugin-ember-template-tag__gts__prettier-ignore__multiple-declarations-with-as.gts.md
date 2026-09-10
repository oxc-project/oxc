# externals/plugin-ember-template-tag/gts/prettier-ignore/multiple-declarations-with-as.gts

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -22,5 +22,5 @@
   <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
 </template> as TemplateOnlyComponent<Signature>,
 ModVar4 = <template>
   Second module variable template.
-</template> as TemplateOnlyComponent<Signature>
+</template> as TemplateOnlyComponent<Signature>;

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
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar2 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar4 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>;

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
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar2 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar4 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>

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
@@ -22,5 +22,5 @@
   <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
 </template> as TemplateOnlyComponent<Signature>,
 ModVar4 = <template>
   Second module variable template.
-</template> as TemplateOnlyComponent<Signature>
+</template> as TemplateOnlyComponent<Signature>;

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
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar2 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar4 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>;

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
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar2 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template> as TemplateOnlyComponent<Signature>,
ModVar4 = <template>
  Second module variable template.
</template> as TemplateOnlyComponent<Signature>

`````
