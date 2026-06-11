// Assignments inside and outside typeof guards.

export function ok_assign_inside_guard(input: number | null): number {
  let ok_target: number = 0;
  if (typeof input === "number") {
    ok_target = input;
  }
  return ok_target;
}

export function bad_assign_outside_guard(input: number | string): number {
  let bad_target: number = 0;
  if (typeof input === "number") {
    bad_target = input;
  }
  bad_target = input;
  return bad_target;
}

export function ok_early_return_guard(input: string | undefined): string {
  if (typeof input !== "string") {
    return "";
  }
  const ok_after_return: string = input;
  return ok_after_return;
}
