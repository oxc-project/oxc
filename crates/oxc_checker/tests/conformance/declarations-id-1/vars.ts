declare const flag: boolean;

function compute(): number {
  return 42;
}

// Clean: literal initializers are trivially inferable.
export const ok_num = 1;
export const ok_str = "hello";
export const ok_bool = true;

// Clean: explicit annotation makes any initializer fine.
export const ok_annotated: number = compute();

// Errors: non-trivial initializers without annotations.
export const bad_call = compute();
export const bad_cond = flag ? 1 : 2;
export const bad_template = `value-${compute()}`;

// Silent: not exported, so isolated declarations does not complain.
const ok_local = compute();
export const ok_uses_local: number = ok_local;
