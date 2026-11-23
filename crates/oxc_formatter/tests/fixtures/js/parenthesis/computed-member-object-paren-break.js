// DIVERGENCES.md#computed-member-object-paren-break
(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb).c;
(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb)[c];
(aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa || bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb)?.[c];
(someCondition ? someVeryLongConsequentValueeeeeeeeeeeeeeeeeeee : someVeryLongAlternateValueeeeeeeeeee).c;
(someCondition ? someVeryLongConsequentValueeeeeeeeeeeeeeeeeeee : someVeryLongAlternateValueeeeeeeeeee)[c];

// The property keeps its own layout
xxxxxxxxxx[aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb];
