function decorator() {}
class A {
  foo(@decorator a, @decorator [b], @decorator { c }, @decorator d = 1) {}
}
