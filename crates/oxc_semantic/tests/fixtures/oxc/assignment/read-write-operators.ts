let Foo = 0;
// Only this `Foo` should be considered as a write-only reference
Foo = 1;

// Read | Write
Foo += 1;
Foo -= 1;
Foo *= 1;
Foo /= 1;
Foo %= 1;
Foo **= 1;
Foo <<= 1;
Foo >>= 1;
Foo >>>= 1;
Foo &= 1;
Foo ^= 1;
Foo |= 1;
Foo &&= 1;
Foo ||= 1;
Foo ??= 1;