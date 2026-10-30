# externals/prettier/html/basics/with-colon.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -221,11 +221,9 @@
   </style>
   <html:script
     >const func = function() { console.log('Hello, there');}</html:script
   >
-  <html:style
-    >.a{color:#f00}</html:style
-  >
+  <html:style>.a{color:#f00}</html:style>
   <svg>
     <style>
       .a {
         color: #f00;
@@ -242,7 +240,5 @@
 </with:colon>
 <html:script
   >const func = function() { console.log('Hello, there');}</html:script
 >
-<html:style
-  >.a{color:#f00}</html:style
->
+<html:style>.a{color:#f00}</html:style>

`````

### Actual (oxfmt)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:bar
  >
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:div
  >
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:span
  >
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo-bar
  >
</div>

<!-- without colon -->
<div>
  <div>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
  </div>
</div>

<!-- #7236 -->
<with:colon>
  <div>
    <h1>
      text text text text text text text text text text text text text text
    </h1>
  </div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script
    >const func = function() { console.log('Hello, there');}</html:script
  >
  <html:style>.a{color:#f00}</html:style>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script
  >const func = function() { console.log('Hello, there');}</html:script
>
<html:style>.a{color:#f00}</html:style>

`````

### Expected (prettier)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:bar
  >
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:div
  >
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo:span
  >
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span></foo-bar
  >
</div>

<!-- without colon -->
<div>
  <div>
    <div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </span>
    <span> inline </span><span> inline </span> <span> inline </span
    ><span> inline </span>
    <html:div>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block
    </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV>
    <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span>
  </div>
</div>

<!-- #7236 -->
<with:colon>
  <div>
    <h1>
      text text text text text text text text text text text text text text
    </h1>
  </div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script
    >const func = function() { console.log('Hello, there');}</html:script
  >
  <html:style
    >.a{color:#f00}</html:style
  >
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script
  >const func = function() { console.log('Hello, there');}</html:script
>
<html:style
  >.a{color:#f00}</html:style
>

`````

## Option 2

`````json
{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -214,14 +214,10 @@
     .A {
       color: #f00;
     }
   </style>
-  <html:script>
-    const func = function() { console.log('Hello, there');}
-  </html:script>
-  <html:style>
-    .a{color:#f00}
-  </html:style>
+  <html:script>const func = function() { console.log('Hello, there');}</html:script>
+  <html:style>.a{color:#f00}</html:style>
   <svg>
     <style>
       .a {
         color: #f00;
@@ -235,10 +231,6 @@
       }
     </style>
   </svg>
 </with:colon>
-<html:script>
-  const func = function() { console.log('Hello, there');}
-</html:script>
-<html:style>
-  .a{color:#f00}
-</html:style>
+<html:script>const func = function() { console.log('Hello, there');}</html:script>
+<html:style>.a{color:#f00}</html:style>

`````

### Actual (oxfmt)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:bar>
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:div>
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:span>
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo-bar>
</div>

<!-- without colon -->
<div>
  <div>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </div>
</div>

<!-- #7236 -->
<with:colon>
  <div><h1>text text text text text text text text text text text text text text</h1></div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script>const func = function() { console.log('Hello, there');}</html:script>
  <html:style>.a{color:#f00}</html:style>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script>const func = function() { console.log('Hello, there');}</html:script>
<html:style>.a{color:#f00}</html:style>

`````

### Expected (prettier)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:bar>
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:div>
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo:span>
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </foo-bar>
</div>

<!-- without colon -->
<div>
  <div>
    <div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</div>
    <div>block</div>
    <div>BLOCK</div>
    <div>block</div>
    <div>block</div>
    <div>block</div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <span>inline</span>
    <html:div>looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block</html:div>
    <html:DIV>block</html:DIV>
    <HTML:DIV>BLOCK</HTML:DIV>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:div>block</html:div>
    <html:pre>pre pr e</html:pre>
    <html:textarea>pre-wrap pr e-wrap</html:textarea>
    <html:span>
      looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline
    </html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
    <html:span>inline</html:span>
  </div>
</div>

<!-- #7236 -->
<with:colon>
  <div><h1>text text text text text text text text text text text text text text</h1></div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script>
    const func = function() { console.log('Hello, there');}
  </html:script>
  <html:style>
    .a{color:#f00}
  </html:style>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script>
  const func = function() { console.log('Hello, there');}
</html:script>
<html:style>
  .a{color:#f00}
</html:style>

`````

## Option 3

`````json
{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -144,14 +144,10 @@
     .A {
       color: #f00;
     }
   </style>
-  <html:script
-    >const func = function() { console.log('Hello, there');}</html:script
-  >
-  <html:style
-    >.a{color:#f00}</html:style
-  >
+  <html:script>const func = function() { console.log('Hello, there');}</html:script>
+  <html:style>.a{color:#f00}</html:style>
   <svg>
     <style>
       .a {
         color: #f00;
@@ -165,10 +161,6 @@
       }
     </style>
   </svg>
 </with:colon>
-<html:script
-  >const func = function() { console.log('Hello, there');}</html:script
->
-<html:style
-  >.a{color:#f00}</html:style
->
+<html:script>const func = function() { console.log('Hello, there');}</html:script>
+<html:style>.a{color:#f00}</html:style>

`````

### Actual (oxfmt)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:bar
  >
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:div
  >
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:span
  >
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo-bar
  >
</div>

<!-- without colon -->
<div>
  <div>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></div
  >
</div>

<!-- #7236 -->
<with:colon>
  <div><h1> text text text text text text text text text text text text text text </h1></div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script>const func = function() { console.log('Hello, there');}</html:script>
  <html:style>.a{color:#f00}</html:style>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script>const func = function() { console.log('Hello, there');}</html:script>
<html:style>.a{color:#f00}</html:style>

`````

### Expected (prettier)

`````html
<!-- unknown tag with colon -->
<div>
  <foo:bar>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:bar
  >
</div>

<!-- block tag with colon -->
<div>
  <foo:div>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:div
  >
</div>

<!-- inline tag with colon -->
<div>
  <foo:span>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo:span
  >
</div>

<!-- unknown -->
<div>
  <foo-bar>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></foo-bar
  >
</div>

<!-- without colon -->
<div>
  <div>
    <div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </div>
    <div> block </div><div> BLOCK </div> <div> block </div><div> block </div><div> block </div>
    <pre>
 pre pr
e</pre>
    <textarea>
 pre-wrap pr
e-wrap </textarea>
    <span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </span>
    <span> inline </span><span> inline </span> <span> inline </span><span> inline </span>
    <html:div> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog block </html:div>
    <html:DIV> block </html:DIV><HTML:DIV> BLOCK </HTML:DIV> <html:div> block </html:div><html:div> block </html:div
    ><html:div> block </html:div>
    <html:pre> pre pr e</html:pre>
    <html:textarea> pre-wrap pr e-wrap </html:textarea>
    <html:span> looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooog inline </html:span>
    <html:span> inline </html:span><html:span> inline </html:span> <html:span> inline </html:span
    ><html:span> inline </html:span></div
  >
</div>

<!-- #7236 -->
<with:colon>
  <div><h1> text text text text text text text text text text text text text text </h1></div>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
</with:colon>

<!-- script like -->
<with:colon>
  <style>
    .a {
      color: #f00;
    }
  </style>
  <script>
    const func = function () {
      console.log("Hello, there");
    };
  </script>
  <style>
    .A {
      color: #f00;
    }
  </style>
  <html:script
    >const func = function() { console.log('Hello, there');}</html:script
  >
  <html:style
    >.a{color:#f00}</html:style
  >
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
  <svg>
    <style>
      .a {
        color: #f00;
      }
    </style>
  </svg>
</with:colon>
<html:script
  >const func = function() { console.log('Hello, there');}</html:script
>
<html:style
  >.a{color:#f00}</html:style
>

`````
