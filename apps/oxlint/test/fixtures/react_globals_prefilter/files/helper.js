let someGlobal = false;

// Not a component or hook, so the compiler does not lint it.
export function helper() {
  someGlobal = true;
  return someGlobal;
}
