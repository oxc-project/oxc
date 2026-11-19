# Input

`````js
md`
\`\`\`md
~~~js
const  a = 1
~~~
\`\`\`
`

`````

# Output

## `{"printWidth":80}`

`````js
md`
~~~~md
~~~js
const a = 1;
~~~
~~~~
`;

`````

## `{"printWidth":100,"proseWrap":"always"}`

`````js
md`
~~~~md
~~~js
const a = 1;
~~~
~~~~
`;

`````
