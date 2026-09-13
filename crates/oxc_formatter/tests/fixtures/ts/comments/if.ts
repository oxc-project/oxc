// Known divergence: the blank line between `// comment2` and `else` is
// preserved, as in every other leading-comment position; Prettier collapses
// it only here.
if (true) {}

// comment1
else if (false) {}

// comment2

else {}