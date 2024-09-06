// ES2015
// RegExpSticky
x1 = /./y
// RegExpUnicode
x2 = /./u
// ES2018
// RegExpDotAllFlag
a1 = /a.b/s
// RegExpLookbehindAssertions
b1 = /(?<!x)/
b2 = /(?<=x)/
b3 = /((?<!x)){2}/
b4 = /((?<=x)){3}/
// RegExpNamedCaptureGroups
c1 = /(?<a>b)/
c2 = /((?<c>d)){4}/;
// RegExpUnicodePropertyEscapes
d1 = /\p{Emoji}/u
// ES2022
// RegExpMatchIndices
f1 = /y/d
// ES2024
// RegExpSetNotation
g1 = /[\p{White_Space}&&\p{ASCII}]/v
