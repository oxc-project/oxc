// Truthiness narrowing under strictNullChecks: every case here is clean.
// This fixture documents where tsgo stays silent; expected.txt is empty.

export function ok_truthy_string(name: string | null): string {
  if (name) {
    return name;
  }
  return "anonymous";
}

export function ok_truthy_array(items: number[] | undefined): number {
  if (items) {
    return items.length;
  }
  return 0;
}

export function ok_logical_and(label: string | undefined): number {
  return label && label.length > 0 ? label.length : 0;
}

export function ok_loose_null_check(count: number | null | undefined): number {
  if (count != null) {
    const ok_value: number = count;
    return ok_value;
  }
  return -1;
}

export function ok_while_truthy(node: { id: number } | null): number {
  let ok_last: number = 0;
  while (node) {
    ok_last = node.id;
    node = null;
  }
  return ok_last;
}
