function f(x: string): void;
function f(x: number): void;
function f(x: unknown) {}
function g(): void;
f(1); g();
namespace N {
  export interface f {}
  export function f(): void;
  export function g(): void;
  export interface g {}
  export const x = 1;
}
