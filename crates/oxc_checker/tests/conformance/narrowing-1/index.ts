// typeof guards: assignment to narrowed-compatible targets must be silent.

export function ok_typeof_string(value: string | number): string {
  if (typeof value === "string") {
    const ok_narrowed: string = value;
    return ok_narrowed;
  }
  return value.toFixed(2);
}

export function ok_typeof_else_branch(value: string | boolean): boolean {
  if (typeof value === "string") {
    return value.length > 0;
  }
  const ok_bool: boolean = value;
  return ok_bool;
}

export function bad_no_guard(value: string | number): string {
  const bad_direct: string = value;
  return bad_direct;
}
