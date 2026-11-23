# Input

`````html
<script type="text/html">
  <div style="color:red;background:blue" onclick="foo( 1 )" class="  a   b  ">x</div>
</script>

`````

# Output

## `{"printWidth":80}`

`````html
<script type="text/html">
  <div style="color:red;background:blue" onclick="foo( 1 )" class="  a   b  ">
    x
  </div>
</script>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<script type="text/html">
  <div
    style="color:red;background:blue"
    onclick="foo( 1 )"
    class="  a   b  "
  >
    x
  </div>
</script>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<script type="text/html">
  <div style="color:red;background:blue" onclick="foo( 1 )" class="  a   b  ">x</div>
</script>

`````
