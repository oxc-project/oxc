export function pick(value: string | undefined): string {
  // `value` is already narrowed, so the `?? ''` is unnecessary.
  return value ? value ?? '' : '';
}
