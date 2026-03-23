# prettier/vue/multiparser/lang-tsx.vue

> `lang=tsx` is not supported

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,13 +1,7 @@
 <script lang="tsx">
-import { VNode } from "vue";
+import {VNode} from "vue"
 export default {
-  computed: {
-    foo(): string {
-      return "foo";
-    },
-  },
-  render(h): VNode {
-    return <div>{this.foo}</div>;
-  },
-};
+  computed: {  foo( ):string { return "foo" }, },
+  render(h):VNode {  return <div>{ this.foo }</div> },
+}
 </script>

`````

### Actual (oxfmt)

`````vue
<script lang="tsx">
import {VNode} from "vue"
export default {
  computed: {  foo( ):string { return "foo" }, },
  render(h):VNode {  return <div>{ this.foo }</div> },
}
</script>

`````

### Expected (prettier)

`````vue
<script lang="tsx">
import { VNode } from "vue";
export default {
  computed: {
    foo(): string {
      return "foo";
    },
  },
  render(h): VNode {
    return <div>{this.foo}</div>;
  },
};
</script>

`````

## Option 2

`````json
{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,13 +1,7 @@
 <script lang="tsx">
-  import { VNode } from 'vue';
+  import {VNode} from "vue"
   export default {
-    computed: {
-      foo(): string {
-        return 'foo';
-      },
-    },
-    render(h): VNode {
-      return <div>{this.foo}</div>;
-    },
-  };
+    computed: {  foo( ):string { return "foo" }, },
+    render(h):VNode {  return <div>{ this.foo }</div> },
+  }
 </script>

`````

### Actual (oxfmt)

`````vue
<script lang="tsx">
  import {VNode} from "vue"
  export default {
    computed: {  foo( ):string { return "foo" }, },
    render(h):VNode {  return <div>{ this.foo }</div> },
  }
</script>

`````

### Expected (prettier)

`````vue
<script lang="tsx">
  import { VNode } from 'vue';
  export default {
    computed: {
      foo(): string {
        return 'foo';
      },
    },
    render(h): VNode {
      return <div>{this.foo}</div>;
    },
  };
</script>

`````
