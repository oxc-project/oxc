function f<T extends U, U>(a: T): U {
//                   ^ resolves to the type parameter `U` declared after it:
//                     all type parameters are declared before signature
//                     references are resolved
  return a;
}
