// Single-line tuples stay flat unless a threshold wraps them
type Flat = [string, number];
type Triple = [string, number, boolean];
// A newline after `[` keeps a tuple expanded under preserve-based modes
type Preserved = [
  string,
  number,
];
// Named, optional, and rest members count as elements
type Named = [first: string, second?: number, ...rest: boolean[]];
// Nested tuples are evaluated independently
type Nested = [[string, number], [boolean]];
