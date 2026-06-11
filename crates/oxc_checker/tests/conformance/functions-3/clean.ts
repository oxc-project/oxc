export function ok_sum(values: number[]): number {
  let total: number = 0;
  for (const value of values) {
    total += value;
  }
  return total;
}

export function ok_early_void_exit(message: string): void {
  if (message === "") {
    return;
  }
}

export const ok_arrow_concat = (left: string, right: string): string =>
  left + right;

export const ok_arrow_block = (flag: boolean): number => {
  if (flag) {
    return 1;
  }
  return 0;
};

export function ok_while_countdown(start: number): number {
  let current: number = start;
  while (current > 0) {
    current -= 1;
  }
  return current;
}

export function ok_optional_fall_through(flag: boolean): number | undefined {
  if (flag) {
    return 5;
  }
}
