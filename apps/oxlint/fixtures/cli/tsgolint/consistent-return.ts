function maybe(flag: boolean): number {
  if (flag) {
    return 1;
  }
  return;
}

maybe(Math.random() > 0.5);

export {};
