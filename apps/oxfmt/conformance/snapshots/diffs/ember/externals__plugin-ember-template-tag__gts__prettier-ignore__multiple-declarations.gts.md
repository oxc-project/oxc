# externals/plugin-ember-template-tag/gts/prettier-ignore/multiple-declarations.gts

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
 </template>,
 ModVar4: TemplateOnlyComponent<Signature> = <template>
   Second module variable template.
-</template>
+</template>;

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
const ModVar1: TemplateOnlyComponent<Signature> = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 : TemplateOnlyComponent<Signature>= <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3: TemplateOnlyComponent<Signature>  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4: TemplateOnlyComponent<Signature> = <template>
  Second module variable template.
</template>;

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
const ModVar1: TemplateOnlyComponent<Signature> = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 : TemplateOnlyComponent<Signature>= <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3: TemplateOnlyComponent<Signature>  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4: TemplateOnlyComponent<Signature> = <template>
  Second module variable template.
</template>

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
 </template>,
 ModVar4: TemplateOnlyComponent<Signature> = <template>
   Second module variable template.
-</template>
+</template>;

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
const ModVar1: TemplateOnlyComponent<Signature> = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 : TemplateOnlyComponent<Signature>= <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3: TemplateOnlyComponent<Signature>  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4: TemplateOnlyComponent<Signature> = <template>
  Second module variable template.
</template>;

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
const ModVar1: TemplateOnlyComponent<Signature> = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 : TemplateOnlyComponent<Signature>= <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool: boolean = false, ModVar3: TemplateOnlyComponent<Signature>  = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4: TemplateOnlyComponent<Signature> = <template>
  Second module variable template.
</template>

`````
