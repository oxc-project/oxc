// Examples of incorrect code for no-unsafe-enum-comparison rule

enum Status {
  Open = 'open',
  Closed = 'closed',
}

enum Color {
  Red = 'red',
  Blue = 'blue',
}

// Comparing different enums
const comparison = Status.Open === Color.Red;