// `start` puts operators at the head of continuation lines; `end` leaves them trailing.
const result = aaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccccccccccc;

const condition = aaaaaaaaaaaaaaaaaaaaaa && bbbbbbbbbbbbbbbbbbbbbbbb && cccccccccccccccccccccccccc;

// Inlined logical right side (non-empty object literal):
// the operator stays trailing in either mode.
const inlined = someLongLongLongLongLongLongCondition && { foo: "bar", baz: "qux", quux: "corge" };

// Binary `&` with an object literal is NOT inlined; it breaks like any binary chain.
const masked = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa & { cause: unknown, foo: barbarbar };
