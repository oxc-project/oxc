function f(x: T): U {
//            ^   ^ both resolve to the types declared in the body:
//                    neither is visible when the parameters are declared,
//                    so resolution falls back to the end-of-program state
  interface T {}
  type U = number;
  return 0;
}
