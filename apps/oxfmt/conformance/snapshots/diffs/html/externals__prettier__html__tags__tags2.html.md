# externals/prettier/html/tags/tags2.html

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -12,11 +12,10 @@
   after
 </div>
 
 <div>
-  before
-  <dialog open>dialog long long long long long long long long</dialog>
-  after
+  before<dialog open>dialog long long long long long long long long</dialog
+  >after
 </div>
 
 <div>
   before<object data="horse.wav">

`````

### Actual (oxfmt)

`````html
<div>
  before<noscript>noscript long long long long long long long long</noscript
  >after
</div>

<div>
  before
  <details>
    <summary>summary long long long long</summary>
    details
  </details>
  after
</div>

<div>
  before<dialog open>dialog long long long long long long long long</dialog
  >after
</div>

<div>
  before<object data="horse.wav">
    <param name="autoplay" value="true" />
    <param name="autoplay" value="true" /></object
  >after
</div>

<div>
  before<meter
    min="0"
    max="1"
    low=".4"
    high=".7"
    optimum=".5"
    value=".2"
  ></meter
  >after
</div>

<div>before<progress value=".5" max="1"></progress>after</div>

`````

### Expected (prettier)

`````html
<div>
  before<noscript>noscript long long long long long long long long</noscript
  >after
</div>

<div>
  before
  <details>
    <summary>summary long long long long</summary>
    details
  </details>
  after
</div>

<div>
  before
  <dialog open>dialog long long long long long long long long</dialog>
  after
</div>

<div>
  before<object data="horse.wav">
    <param name="autoplay" value="true" />
    <param name="autoplay" value="true" /></object
  >after
</div>

<div>
  before<meter
    min="0"
    max="1"
    low=".4"
    high=".7"
    optimum=".5"
    value=".2"
  ></meter
  >after
</div>

<div>before<progress value=".5" max="1"></progress>after</div>

`````
