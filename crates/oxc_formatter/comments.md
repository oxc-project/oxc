# Formatter Comment Handling (Incomplete)

This document outlines how to handle comments in JavaScript code based on [Prettier's design](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/main/comments/attach.js#L205).

## Comment Categories

JavaScript comments fall into three main categories:

## 1. OwnLine Comments

Comments that appear on their own line with a newline character before them.

### Leading OwnLine Comments

Comments that precede a node:

```js
// comment
/* comment */ const A = 1;
```

### Trailing OwnLine Comments

Comments that follow a node with no subsequent node after them. These typically appear at the end of statement lists in `Program` and `BlockStatement` nodes:

```js
const A = 2;
// comment
```

```js
if (true) {
  console.log('test');
  // comment
}
```

### Dangling OwnLine Comments

Comments inside a non-empty block. These can appear in various block-like structures such as `FunctionDeclaration`, `ClassDeclaration`, and `IfStatement`:

```js
class Cls {
  // comment
}

{
  // comment
}

[
  // comment
];

if (true) {
  // comment
}
```

#### Special Case: CallExpression Arguments

There's a special case for comments inside empty call expressions:

Input:

```js
console.log(); /* no arguments */
```

**Prettier** formats this as:

```js
console
  .log /* no arguments */();
```

While **Biome** formats this as:

```js
console.log(/* no arguments */);
```

The Biome approach better preserves the comment's intent and should be considered for our implementation.

## 2. EndOfLine Comments

Comments at the end of a line that aren't followed by a node. Note that some EndOfLine comments may be treated as OwnLine if they have a line break before them.

### Leading EndOfLine Comments

Comments at the start of code or after a semicolon (`;`) and before a node:

```js
// comment
const A = 1;
```

> Note: The `;` represents an EmptyStatement node that won't be printed in the output.

### Trailing EndOfLine Comments

Comments that appear after a node without any subsequent node:

```js
const A = 2; // comment
const B = 2; /* comment */
```

### Dangling EndOfLine Comments

Comments inside a block that appear after a left brace (`{` or `[`) without a following node:

```js
{ // comment
}

[ // comment
];
```

## 3. Remaining Comments

All other comments that don't fit into the OwnLine or EndOfLine categories.

### Leading Remaining Comments

```js
const A = /* comment */ 1;
```

### Trailing Remaining Comments

```js
const A /* comment */ = 1;
```

### Dangling Remaining Comments

```js
{/* comment */}
```

````
### Dangling comments

```js
{/* comment */}
````
