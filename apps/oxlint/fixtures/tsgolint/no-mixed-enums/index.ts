// Examples of incorrect code for no-mixed-enums rule

enum Status {
  Open = 1,
  Closed = 'closed',
}

enum Direction {
  Up = 'up',
  Down = 2,
  Left = 'left',
  Right = 4,
}