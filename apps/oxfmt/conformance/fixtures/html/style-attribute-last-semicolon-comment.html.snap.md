# Input

`````html
<div style="color: red;"></div>
<div style="color: red; /* c */"></div>
<div style="color: red /* c */"></div>

`````

# Output

## `{"printWidth":80}`

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````

## `{"printWidth":100,"htmlWhitespaceSensitivity":"ignore","singleAttributePerLine":true}`

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````

## `{"printWidth":120,"htmlWhitespaceSensitivity":"strict","bracketSameLine":true}`

`````html
<div style="color: red"></div>
<div style="color: red /* c */"></div>
<div style="color: red /* c */"></div>

`````
