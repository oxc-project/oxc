<!-- A code span fence never equals a literal backtick run before it.
     Emphasis markers normalize only when the printed text pairs as the source did;
     otherwise they stay as written -->

A stray ` backtick and then ``code`` that must not shrink to one backtick.

A literal *** run and then __strong__ that may become two asterisks (still one strong).

A literal _ underscore and then *emphasis* that may become underscores (still one emphasis).

No stray run here, so ``code`` shrinks and __strong__ and *emphasis* normalize.

**a ***b*** c** keeps its asterisks: with `_` the inner emphasis reads differently.

*a**b* keeps its inner run escaped, and _a\_\_b_ its normalized marker.

A backslash \ before a line break stays on its line, never a hard break \
when the next word arrives.

[label]: /destination followed by text that is not a title keeps its first line
whole (a wrap after the destination would leave a definition behind).

text\
	:::note after a hard break was indented in the source and stays paragraph text
