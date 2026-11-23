# Input

`````html
<div style="/* comment */"></div>
<div style="  /* a */ /* b */  "></div>
<div style="/* a */
/* b */"></div>
<div style="/* c */ color: red"></div>
<div style="/* c */
color: red"></div>
<div style="color: red;
/* c */ display: block"></div>
<div style="color: red; /* c */ display: block"></div>
<div style="/* a very long comment that does not fit on the line at all, really */ color: red; display: block"></div>

`````

# Output

## `{"printWidth":80}`

`````html
<div style="/* comment */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* c */ color: red"></div>
<div style="/* c */ color: red"></div>
<div style="color: red; /* c */ display: block"></div>
<div style="color: red; /* c */ display: block"></div>
<div
  style="
    /* a very long comment that does not fit on the line at all, really */
    color: red;
    display: block;
  "
></div>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<div style="/* comment */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* c */ color: red"></div>
<div style="/* c */ color: red"></div>
<div style="color: red; /* c */ display: block"></div>
<div style="color: red; /* c */ display: block"></div>
<div
  style="
    /* a very long comment that does not fit on the line at all, really */
    color: red;
    display: block;
  "
></div>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<div style="/* comment */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* a */ /* b */"></div>
<div style="/* c */ color: red"></div>
<div style="/* c */ color: red"></div>
<div style="color: red; /* c */ display: block"></div>
<div style="color: red; /* c */ display: block"></div>
<div style="/* a very long comment that does not fit on the line at all, really */ color: red; display: block"></div>

`````
