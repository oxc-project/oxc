# Input

`````js
// Tagged template literals with md and markdown tags
const documentation = md`#Heading
This is **bold**.
-Item 1
-Item 2`;

const readme = markdown`##Installation
\`\`\`bash
npm install package
\`\`\``;

`````

# Output

## `{}`

`````js
// Tagged template literals with md and markdown tags
const documentation = md`
#Heading
This is **bold**.
-Item 1
-Item 2
`;

const readme = markdown`
##Installation

~~~bash
npm install package
~~~
`;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Tagged template literals with md and markdown tags
const documentation = md`#Heading
This is **bold**.
-Item 1
-Item 2`;

const readme = markdown`##Installation
\`\`\`bash
npm install package
\`\`\``;

`````

## `{"endOfLine":"crlf"}`

`````js
// Tagged template literals with md and markdown tags<CRLF>
const documentation = md`<CRLF>
#Heading<CRLF>
This is **bold**.<CRLF>
-Item 1<CRLF>
-Item 2<CRLF>
`;<CRLF>
<CRLF>
const readme = markdown`<CRLF>
##Installation<CRLF>
<CRLF>
~~~bash<CRLF>
npm install package<CRLF>
~~~<CRLF>
`;<CRLF>

`````
