function compute(): number {
  return 42;
}

// Clean: fully annotated function and arrow.
export function ok_add(a: number, b: number): number {
  return a + b;
}
export const ok_arrow = (msg: string): string => msg;

// Clean: literal parameter default is trivially inferable.
export function ok_literal_default(scale: number = 2): number {
  return scale;
}

// Error: missing return type annotation on exported function.
export function bad_no_return(a: number) {
  return a * 2;
}

// Error: exported arrow missing return type annotation.
export const bad_arrow = (n: number) => n + 1;

// Error: parameter default is not trivially inferable.
export function bad_param_default(scale = compute()): void {
  void scale;
}

// Silent: unexported helper needs no annotations.
function ok_helper(n: number) {
  return n - 1;
}
export const ok_helper_result: number = ok_helper(3);
