enum A {
  X = 1,
}

enum B {
  Y = A.X + 1,
}

B.Y;
