// A trailing suppression comment suppresses an enum member like a leading one;
// the `,` is a separator the list prints.
enum E {
  A   = 1, // prettier-ignore
  B   = 2 // prettier-ignore
}

enum F {
  // prettier-ignore
  A   = 1,
  B   = 2, // prettier-ignore
}
