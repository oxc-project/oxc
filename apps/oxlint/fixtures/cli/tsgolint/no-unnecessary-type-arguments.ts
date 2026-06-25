function identity<T = string>(arg: T): T {
  return arg;
}
const result = identity<string>('hello');