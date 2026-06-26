// Enum declarations that are re-exported via `export { ... }` clause
// should NOT be removed, even though they are not directly exported.
enum A { X = 1 }
enum B { Y = "hello" }
export { A, B }

A.X;
B.Y;
