// Issue #22043 - line comment followed by block comment after type alias equals
type myType = // Comment
/* Comment */ "VALUE";

// Value assignments keep Prettier's existing trailing placement.
const value = // Comment
/* Comment */ "VALUE";
