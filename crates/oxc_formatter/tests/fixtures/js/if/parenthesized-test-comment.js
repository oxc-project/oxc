// The condition's closing `)` is the LAST `)` before the body: a comment stays
// inside the condition at any paren depth (it used to escape one paren per pass).
// The basic shapes are pinned by the conformance suite
// (js/comments/while-like/{if,while}.js); these paren depths are not.
if ((a, b) /* comment */) {}

while ((a, b) /* comment */) {}

if (((x)) /* comment */) {}

if (((0, 0 /* comment */))) {}
