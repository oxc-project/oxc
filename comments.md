
Figure out how treat comments in JavaScript code based on [Prettier](https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/main/comments/attach.js#L205) design.


## OwnLine comments

Comments that are on their own line, and there is newline before the comment, are considered as own-line comments.

### Leading comments

```js
// comment
/* comment */ const A = 1;
```

### Trailing comments

When comments are after a node and there is no node after that comment, then it is considered as a trailing comment.
As so far i know, this only happens in comment that is at the end of a vec of statements, such as `Program` and `BlockStatement`.

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

### Dangling comments

When comments are inside a block and that block is not empty, then it is considered as a dangling comment. Such as `FunctionDeclaration`, `FunctionExpression`, `ClassDeclaration`, `ClassExpression`, and `IfStatement`.

```js
class Cls {
    // comment
}

{
    // comment
};

[
    // comment
]

if (true) {
    // comment
}
```

There is a exception case that is happens in CallExpression's arguments.

Input:
```js
console.log(
  /* no arguments */
)
```

Prettier Output:
```js
console
  .log
  /* no arguments */
  ();
```

Biome Output:
```js
console.log(/* no arguments */);
```

In my opinion, obviously Biome's output is better, and it keeps the intention of the comment. So we may consider which we should port to our implementation.

## EndOfLine comments

Comments that are at the end of a line, and not followed by a node, are considered as end-of-line comments.

Some of EndOfLine would be treated as `OwnLine` when there is a line break before the comment.
For example:
```js

// comment
const A = 1;
```

### Leading comments

Only comments that are at the start of a code or after `;` and before a node are considered as leading comments.

```js
// comment
const A = 1;
```

```js
;// comment
const A = 1;
```

> Note: The `;` is a EmptyStatement node, and it won't be printed in the output.

### Trailing comments

When comments are after a node and not followed by a node, then it is considered as a trailing comment.

```js
const A = 2; // comment
const B = 2; /* comment */
```

### Dangling comments

When comments are inside a block and placed after a left brace, and not followed by a node, then it is considered as a dangling comment. Such as `BlockStatement` and `ArrayExpression`.

```js
{ // comment
}

[ // comment
]
```

## Remaining comments

### Leading comments

```js
const A = /* comment */ 1;
```

### Trailing comments

```js
const A /* comment */ = 1 ;
```

### Dangling comments

