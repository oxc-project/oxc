function foo<T>(y: T) {
  return { y };
}

export type Foo<T> = typeof foo<T>;
