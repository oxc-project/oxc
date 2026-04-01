# prettier/js/multiparser-html/issue-10691.js

> js-in-html(`<script>`)-in-js needs lot more work; Please see oxc_formatter/src/print/template/embed/html.rs

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,9 @@
 export default function include_photoswipe(gallery_selector = ".my-gallery") {
-  return /* HTML */ ` <script>
-    window.addEventListener("load", () =>
-      initPhotoSwipeFromDOM("${gallery_selector}"),
-    );
-  </script>`;
+  return /* HTML */ `
+    <script>
+      window.addEventListener("load", () =>
+        initPhotoSwipeFromDOM("${gallery_selector}"),
+      );
+    </script>
+  `;
 }

`````

### Actual (oxfmt)

`````js
export default function include_photoswipe(gallery_selector = ".my-gallery") {
  return /* HTML */ `
    <script>
      window.addEventListener("load", () =>
        initPhotoSwipeFromDOM("${gallery_selector}"),
      );
    </script>
  `;
}

`````

### Expected (prettier)

`````js
export default function include_photoswipe(gallery_selector = ".my-gallery") {
  return /* HTML */ ` <script>
    window.addEventListener("load", () =>
      initPhotoSwipeFromDOM("${gallery_selector}"),
    );
  </script>`;
}

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,9 @@
 export default function include_photoswipe(gallery_selector = ".my-gallery") {
   return /* HTML */ `
     <script>
-      window.addEventListener("load", () => initPhotoSwipeFromDOM("${gallery_selector}"));
+      window.addEventListener("load", () =>
+        initPhotoSwipeFromDOM("${gallery_selector}"),
+      );
     </script>
   `;
 }

`````

### Actual (oxfmt)

`````js
export default function include_photoswipe(gallery_selector = ".my-gallery") {
  return /* HTML */ `
    <script>
      window.addEventListener("load", () =>
        initPhotoSwipeFromDOM("${gallery_selector}"),
      );
    </script>
  `;
}

`````

### Expected (prettier)

`````js
export default function include_photoswipe(gallery_selector = ".my-gallery") {
  return /* HTML */ `
    <script>
      window.addEventListener("load", () => initPhotoSwipeFromDOM("${gallery_selector}"));
    </script>
  `;
}

`````
