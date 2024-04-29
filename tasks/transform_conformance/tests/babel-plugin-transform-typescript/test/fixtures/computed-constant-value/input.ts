enum A {
  a = Infinity,
  b,
  c = Infinity + 1,
  d = Infinity + "test",
  e = -(-(-Infinity)),
}

enum B {
  a = NaN,
  b,
  c = NaN + 1,
  d = "nan" + NaN,
  e = -NaN,
}

enum C {
  a = "test" + 1e20,
  b = 1e30 + "test",
  c = "test" + 1234567890987 + "test",
}

enum D {
  a = +"hello",
  b = -"hello",
  c = ~"hello",
}
