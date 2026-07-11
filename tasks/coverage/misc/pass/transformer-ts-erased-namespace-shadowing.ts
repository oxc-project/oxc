const B = 1;
namespace A {
    declare namespace B {
        export const x: number;
    }
    console.log(B);
}
