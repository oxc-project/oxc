namespace Foo {
  export namespace Bar {
    export const A = () => { };

    function B() { };
    export const B1 = B;
  }

  export const C = () => { };
  export function D() { };

  namespace NotExported {
    export const E = () => { };
  }
}
