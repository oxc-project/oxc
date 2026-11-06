# Input

`````tsx
<div className={`flex
items-center
bg-blue-500
p-4
text-white
${isActive ? `font-bold
shadow-lg` : "font-normal"}`} />;
`````

# Output

## `{"sortTailwindcss":{"preserveWhitespace":true}}`

`````tsx
<div className={`flex
items-center
bg-blue-500
p-4
text-white
${isActive ? `font-bold
shadow-lg` : "font-normal"}`} />;

`````

### Input variants differ: CRLF, CR

## `{"sortTailwindcss":{}}`

`````tsx
<div
  className={`flex items-center bg-blue-500 p-4 text-white ${
    isActive ? `font-bold shadow-lg` : "font-normal"
  }`}
/>;

`````
