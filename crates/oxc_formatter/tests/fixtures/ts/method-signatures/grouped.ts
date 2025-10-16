
type A = {
  new(...args): T<{
    A
  }
  >
};


type A1 = {
  (...args): T<{
    A
  }
  >
};

type A2 = {
  bar(
    ...args
  ): T<{
    A
  }>
}

