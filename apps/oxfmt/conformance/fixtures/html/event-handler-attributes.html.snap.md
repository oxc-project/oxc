# Input

`````html
<button onclick="a(  ) // c" onfocus="a(); // c" onblur="/* c */" onkeyup="" onkeydown="  ;  " oninput="a();b()" onchange="x = &quot;y&quot;">x</button>
<div onclick="#!foo"></div>
<div onclick="#!foo
a(); b()"></div>

`````

# Output

## `{"printWidth":80}`

`````html
<button
  onclick="
    a() // c
  "
  onfocus="
    a() // c
  "
  onblur="/* c */"
  onkeyup=""
  onkeydown=""
  oninput="
    a();
    b();
  "
  onchange="x = 'y'"
>
  x
</button>
<div onclick="#!foo"></div>
<div
  onclick="
    #!foo
    a();
    b();
  "
></div>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<button
  onclick="
    a() // c
  "
  onfocus="
    a() // c
  "
  onblur="/* c */"
  onkeyup=""
  onkeydown=""
  oninput="
    a();
    b();
  "
  onchange="x = 'y'"
>
  x
</button>
<div onclick="#!foo"></div>
<div
  onclick="
    #!foo
    a();
    b();
  "
></div>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<button
  onclick="
    a() // c
  "
  onfocus="
    a() // c
  "
  onblur="/* c */"
  onkeyup=""
  onkeydown=""
  oninput="
    a();
    b();
  "
  onchange="x = 'y'"
  >x</button
>
<div onclick="#!foo"></div>
<div
  onclick="
    #!foo
    a();
    b();
  "></div>

`````
