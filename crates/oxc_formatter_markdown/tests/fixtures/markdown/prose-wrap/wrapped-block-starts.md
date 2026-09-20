<!-- A word that would open a block at a line start never begins a wrapped line:
     the parser's `lexical::line_start` decides, not a list of markers -->

padding words to push the risky token over the edge padding words to push <!-- c --> tail words.

padding words to push the risky token over the edge padding words to <div>x</div> tail words.

padding words to push the risky token over the edge padding words to push the *** tail words.

padding words to push the risky token over the edge padding words to push the --- tail words.

padding words to push the risky token over the edge padding words to push the 1. tail words.

padding words to push the risky token over the edge padding words to push the ``` tail words.

padding words to push the risky token over the edge padding words to push the | a | tail words.

padding words to push the risky token over the edge padding words to push [^1]: tail words.

padding words to push the risky token over the edge padding words to push the - item tail words.

padding words to push the risky token over the edge padding words to push the # not a heading tail words.

padding words to push the risky token over the edge padding words to push the > not a quote tail words.

<!-- A kept line break whose next line loses its indentation is dropped the same way: printed at the line start, the row would be a table delimiter -->

| x | y |
        |---|---|

text then
    - item
