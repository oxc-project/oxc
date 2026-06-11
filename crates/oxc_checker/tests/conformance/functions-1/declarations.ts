export function bad_string_from_number(): string {
  return 42;
}

export function ok_number(): number {
  return 42;
}

export function bad_bare_return(): number {
  return;
}

export function ok_void_bare_return(): void {
  return;
}

export function bad_no_return_at_all(): boolean {
}

export function bad_missing_else_return(flag: boolean): number {
  if (flag) {
    return 1;
  }
}

export function ok_if_else_all_paths(flag: boolean): string {
  if (flag) {
    return "yes";
  } else {
    return "no";
  }
}
