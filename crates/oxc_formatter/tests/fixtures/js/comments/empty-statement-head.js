// A comment between a head's `)` and its empty-statement body `;` keeps its
// position (the `;` is content, head-body comment policy): an inline block
// comment stays, a line comment keeps its line with the `;` forced onto the
// next line, an own-line comment keeps its own line.
// Known divergences (js/comments/between-head-and-body/empty-statement.js,
// js/for/9812-2.js, js/for-of/comments.js): Prettier pulls the comment
// backward into the parentheses (`while (x /* c */);`) or hoists an own-line
// comment onto the head line (`for (x of y) // c`) -- attachment artifacts of
// the kind prettier is currently fixing elsewhere (prettier#19894 family).
while (a) /* c */;
while (a) // c
;
while (a)
// c
;

for (a;;) /* c */;
for (a; b; c) /* c */;
for (x in y) /* c */;
for (x of y) /* c */;
for (x of y)
// c
;
if (a) /* c */;
with (a) /* c */;
