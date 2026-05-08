// Issue #22043 - line comment followed by block comment after type alias equals
type myType = // Comment
/* Comment */ "VALUE";

// Conditional type aliases keep Prettier's existing trailing placement.
type ConditionalLine = // Comment
Foo<T> extends Bar ? Baz : Qux;
type ConditionalLineBlock = // Comment
/* Block */ Foo<T> extends Bar ? Baz : Qux;

// Value assignments keep Prettier's existing trailing placement.
const value = // Comment
/* Comment */ "VALUE";
